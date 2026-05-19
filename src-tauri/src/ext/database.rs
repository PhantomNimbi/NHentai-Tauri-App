use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn init_database(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("nhentai.db");
    let conn = Connection::open(&path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL DEFAULT 'tag',
            count INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            gallery_id INTEGER NOT NULL,
            title TEXT NOT NULL DEFAULT '',
            time INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS favorites (
            gallery_id INTEGER PRIMARY KEY,
            time INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS gallery_cache (
            gallery_id INTEGER PRIMARY KEY,
            data TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS search_cache (
            cache_key TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );
        ",
    )?;

    DB.set(Mutex::new(conn))
        .map_err(|_| "database already initialized")?;

    Ok(())
}

fn db() -> &'static Mutex<Connection> {
    DB.get().expect("database not initialized")
}

// ---------------------------------------------------------------------------
// Tag operations
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagEntry {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub tag_type: String,
    pub count: i64,
    pub status: i32,
}

#[tauri::command]
pub fn db_get_tags() -> Result<Vec<TagEntry>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, type, count, status FROM tags ORDER BY count DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TagEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                tag_type: row.get(2)?,
                count: row.get(3)?,
                status: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    for row in rows {
        tags.push(row.map_err(|e| e.to_string())?);
    }
    Ok(tags)
}

#[tauri::command]
pub fn db_upsert_tag(
    id: i64,
    name: String,
    tag_type: String,
    count: i64,
    status: i32,
) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO tags (id, name, type, count, status)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           type = excluded.type,
           count = excluded.count,
           status = excluded.status",
        params![id, name, tag_type, count, status],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn db_delete_tag(id: i64) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn db_clear_tags() -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tags", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn db_upsert_tags_bulk(tags: Vec<TagEntry>) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;
    for t in &tags {
        conn.execute(
            "INSERT INTO tags (id, name, type, count, status)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               type = excluded.type,
               count = excluded.count,
               status = excluded.status",
            params![t.id, t.name, t.tag_type, t.count, t.status],
        )
        .map_err(|e| e.to_string())?;
    }
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// History operations
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub gallery_id: i64,
    pub title: String,
    pub time: i64,
}

#[tauri::command]
pub fn db_get_history() -> Result<Vec<HistoryEntry>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, gallery_id, title, time FROM history ORDER BY time DESC LIMIT 200")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                gallery_id: row.get(1)?,
                title: row.get(2)?,
                time: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }
    Ok(entries)
}

#[tauri::command]
pub fn db_add_history(gallery_id: i64, title: String) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    // Remove duplicate
    conn.execute(
        "DELETE FROM history WHERE gallery_id = ?1",
        params![gallery_id],
    )
    .map_err(|e| e.to_string())?;
    // Keep only 200 entries
    conn.execute(
        "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT 199)",
        [],
    )
    .map_err(|e| e.to_string())?;
    // Insert new
    conn.execute(
        "INSERT INTO history (gallery_id, title, time) VALUES (?1, ?2, ?3)",
        params![gallery_id, title, chrono_now()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn db_clear_history() -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM history", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Favorites operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn db_get_favorites() -> Result<Vec<i64>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT gallery_id FROM favorites ORDER BY time DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(row.map_err(|e| e.to_string())?);
    }
    Ok(ids)
}

#[tauri::command]
pub fn db_toggle_favorite(gallery_id: i64) -> Result<bool, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM favorites WHERE gallery_id = ?1",
            params![gallery_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?
        > 0;
    if exists {
        conn.execute(
            "DELETE FROM favorites WHERE gallery_id = ?1",
            params![gallery_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        conn.execute(
            "INSERT INTO favorites (gallery_id, time) VALUES (?1, ?2)",
            params![gallery_id, chrono_now()],
        )
        .map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn db_clear_favorites() -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM favorites", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Settings operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn db_get_setting(key: String) -> Result<Option<String>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let result = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .ok();
    Ok(result)
}

#[tauri::command]
pub fn db_set_setting(key: String, value: String) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Gallery cache operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn db_get_gallery_cache(gallery_id: i64) -> Result<Option<String>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let result = conn
        .query_row(
            "SELECT data FROM gallery_cache WHERE gallery_id = ?1",
            params![gallery_id],
            |row| row.get(0),
        )
        .ok();
    Ok(result)
}

#[tauri::command]
pub fn db_save_gallery_cache(gallery_id: i64, data: String) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO gallery_cache (gallery_id, data, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(gallery_id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
        params![gallery_id, data, chrono_now()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Search cache operations
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn db_get_search_cache(cache_key: String) -> Result<Option<String>, String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    let result = conn
        .query_row(
            "SELECT data FROM search_cache WHERE cache_key = ?1",
            params![cache_key],
            |row| row.get(0),
        )
        .ok();
    Ok(result)
}

#[tauri::command]
pub fn db_save_search_cache(cache_key: String, data: String) -> Result<(), String> {
    let conn = db().lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO search_cache (cache_key, data, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(cache_key) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
        params![cache_key, data, chrono_now()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Bulk sync: dump all DB data to JSON so the frontend can build its state
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DbSnapshot {
    pub tags: Vec<TagEntry>,
    pub history: Vec<HistoryEntry>,
    pub favorites: Vec<i64>,
    pub settings: HashMap<String, String>,
}

#[tauri::command]
pub fn db_get_snapshot() -> Result<DbSnapshot, String> {
    let tags = db_get_tags()?;
    let history = db_get_history()?;
    let favorites = db_get_favorites()?;

    let conn = db().lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    let mut settings = std::collections::HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| e.to_string())?;
        settings.insert(k, v);
    }

    Ok(DbSnapshot {
        tags,
        history,
        favorites,
        settings,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn chrono_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
