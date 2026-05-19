use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::Duration;

const API_BASE: &str = "https://nhentai.net/api/v2";
const UA: &str = "NHentaiTauriApp/1.0.0 (github.com/PhantomNimbi/NHentai-Tauri-App)";
const MAX_RETRIES: u32 = 3;
const BASE_BACKOFF: Duration = Duration::from_secs(2);

static CLIENT: OnceLock<Client> = OnceLock::new();
static API_KEY: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));
static COOKIES: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(UA)
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .build()
            .expect("Failed to build reqwest Client")
    })
}

#[tauri::command]
pub fn api_set_cookie(cookie_str: String) -> Result<(), String> {
    let parts: Vec<&str> = cookie_str.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Invalid cookie format".into());
    }
    let mut c = COOKIES.lock().map_err(|e| e.to_string())?;
    c.insert(parts[0].trim().to_string(), parts[1].to_string());
    Ok(())
}

#[tauri::command]
pub fn api_set_api_key(key: String) -> Result<(), String> {
    let mut k = API_KEY.lock().map_err(|e| e.to_string())?;
    *k = Some(key.clone());
    let _ = crate::ext::database::db_set_setting("api_key".into(), key);
    Ok(())
}

#[tauri::command]
pub fn api_clear_api_key() -> Result<(), String> {
    let mut k = API_KEY.lock().map_err(|e| e.to_string())?;
    *k = None;
    let _ = crate::ext::database::db_set_setting("api_key".into(), String::new());
    Ok(())
}

#[tauri::command]
pub fn api_key_is_set() -> Result<bool, String> {
    let k = API_KEY.lock().map_err(|e| e.to_string())?;
    Ok(k.is_some())
}

fn build_cookie_header() -> Option<String> {
    let cookies = COOKIES.lock().ok()?;
    if cookies.is_empty() {
        return None;
    }
    Some(
        cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

fn build_auth_header() -> Option<String> {
    let k = API_KEY.lock().ok()?;
    k.as_ref().map(|key| format!("Key {}", key))
}

fn build_req(method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
    let url = format!("{}/{}", API_BASE, path);
    let cookie_hdr = build_cookie_header();
    let auth_hdr = build_auth_header();
    let mut req = client().request(method, &url);
    if let Some(c) = &cookie_hdr {
        req = req.header("Cookie", c);
    }
    if let Some(a) = &auth_hdr {
        req = req.header("Authorization", a);
    }
    req
}

async fn api_get(path: &str) -> Result<Value, String> {
    let mut retries = 0u32;
    loop {
        let resp = match build_req(reqwest::Method::GET, path)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => return Err(format!("HTTP error: {}", e)),
        };
        let status = resp.status();
        if status.as_u16() == 429 && retries < MAX_RETRIES {
            let backoff = BASE_BACKOFF * 2u32.pow(retries);
            retries += 1;
            tokio::time::sleep(backoff).await;
            continue;
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status.as_u16(), body));
        }
        return resp.json::<Value>().await.map_err(|e| format!("JSON error: {}", e));
    }
}

async fn api_post(path: &str) -> Result<Value, String> {
    let resp = build_req(reqwest::Method::POST, path)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }
    resp.json::<Value>().await.map_err(|e| format!("JSON error: {}", e))
}

async fn api_post_json(path: &str, body: Value) -> Result<Value, String> {
    let resp = build_req(reqwest::Method::POST, path)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }
    resp.json::<Value>().await.map_err(|e| format!("JSON error: {}", e))
}

async fn api_delete(path: &str) -> Result<Value, String> {
    let resp = build_req(reqwest::Method::DELETE, path)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), body));
    }
    resp.json::<Value>().await.map_err(|e| format!("JSON error: {}", e))
}

// ---------------------------------------------------------------------------
//  Commands — nhentai.net API v2 endpoints
// ---------------------------------------------------------------------------

// -- Galleries -----------------------------------------------------------------

/// GET /api/v2/galleries — paginated galleries
/// sort values: popular, popular-week, popular-today, popular-month
#[tauri::command]
pub async fn api_all(page: u32, sort: String) -> Result<Value, String> {
    let mut path = format!("galleries?page={}", page);
    if !sort.is_empty() && sort != "newest" && sort != "date" {
        path.push_str(&format!("&sort={}", sort));
    }
    api_get(&path).await
}

/// GET /api/v2/galleries/popular — today's popular galleries
/// Response is a raw array; wrap it like the other list endpoints for the frontend.
#[tauri::command]
pub async fn api_galleries_popular() -> Result<Value, String> {
    let resp = api_get("galleries/popular").await?;
    Ok(serde_json::Value::Object(serde_json::Map::from_iter([("result".into(), resp)])))
}

/// GET /api/v2/galleries/random — random gallery, returns { id }
#[tauri::command]
pub async fn api_random() -> Result<Value, String> {
    api_get("galleries/random").await
}

/// GET /api/v2/galleries/tagged — galleries by tag
#[tauri::command]
pub async fn api_galleries_tagged(tag_id: i64, page: u32, sort: String) -> Result<Value, String> {
    let mut path = format!("galleries/tagged?tag_id={}&page={}", tag_id, page);
    if !sort.is_empty() && sort != "date" {
        path.push_str(&format!("&sort={}", sort));
    }
    api_get(&path).await
}

/// GET /api/v2/galleries/{id} — single gallery detail
#[tauri::command]
pub async fn api_gallery(id: u32) -> Result<Value, String> {
    api_get(&format!("galleries/{}?include=related,favorite", id)).await
}

/// GET /api/v2/galleries/{id}/comments — gallery comments
#[tauri::command]
pub async fn api_gallery_comments(id: u32) -> Result<Value, String> {
    api_get(&format!("galleries/{}/comments", id)).await
}

/// GET /api/v2/galleries/{id}/comments/count — comment count
#[tauri::command]
pub async fn api_gallery_comment_count(id: u32) -> Result<Value, String> {
    api_get(&format!("galleries/{}/comments/count", id)).await
}

/// GET /api/v2/galleries/{id}/related — related galleries
#[tauri::command]
pub async fn api_gallery_related(id: u32) -> Result<Value, String> {
    api_get(&format!("galleries/{}/related", id)).await
}

/// GET /api/v2/galleries/{id}/favorite — check if favorited
#[tauri::command]
pub async fn api_check_favorite(id: u32) -> Result<Value, String> {
    api_get(&format!("galleries/{}/favorite", id)).await
}

/// POST /api/v2/galleries/{id}/favorite — add to favorites
#[tauri::command]
pub async fn api_add_favorite(id: u32) -> Result<Value, String> {
    api_post(&format!("galleries/{}/favorite", id)).await
}

/// DELETE /api/v2/galleries/{id}/favorite — remove from favorites
#[tauri::command]
pub async fn api_remove_favorite(id: u32) -> Result<Value, String> {
    api_delete(&format!("galleries/{}/favorite", id)).await
}

/// POST /api/v2/galleries/{id}/download — get download URL
#[tauri::command]
pub async fn api_get_download_url(id: u32) -> Result<Value, String> {
    api_post(&format!("galleries/{}/download", id)).await
}

// -- Search --------------------------------------------------------------------

/// GET /api/v2/search — search galleries with optional sort
/// Valid sort values (per v2 spec): date, popular, popular-today, popular-week, popular-month
/// When query is empty, the query parameter is omitted (for sort-only browsing).
#[tauri::command]
pub async fn api_search(query: String, page: u32, sort: String) -> Result<Value, String> {
    let mut path = if query.trim().is_empty() {
        format!("search?page={}", page)
    } else {
        format!("search?query={}&page={}", urlenc(&query), page)
    };
    // Only pass sort when it's a non-default value (date is the default per spec)
    if !sort.is_empty() && sort != "date" {
        path.push_str(&format!("&sort={}", sort));
    }
    api_get(&path).await
}

// -- Tags ----------------------------------------------------------------------

/// GET /api/v2/tags/ids?ids=1,2,3 — resolve tag IDs to names
#[tauri::command]
pub async fn api_tags_by_ids(ids: String) -> Result<Value, String> {
    api_get(&format!("tags/ids?ids={}", ids)).await
}

#[tauri::command]
pub async fn api_search_tags(query: String, tag_type: Option<String>) -> Result<Value, String> {
    if query.trim().is_empty() {
        return Err("Query is required".to_string());
    }
    let mut payload = json!({"query": query});
    if let Some(tt) = tag_type {
        if !tt.trim().is_empty() {
            payload["type"] = Value::String(tt.trim().to_string());
        }
    }
    api_post_json("tags/search", payload).await
}

/// GET /api/v2/tags/{tag_type} — browse tags by type
#[tauri::command]
pub async fn api_tags_by_type(tag_type: String, page: u32) -> Result<Value, String> {
    api_get(&format!("tags/{}?page={}", tag_type, page)).await
}

#[tauri::command]
pub async fn api_populate_all_tags() -> Result<Value, String> {
    let tag_types = ["tag", "artist", "character", "parody", "group", "category", "language"];
    let mut all_tags: Vec<crate::ext::database::TagEntry> = Vec::new();

    for tag_type in &tag_types {
        let mut page = 1;
        loop {
            let response = api_get(&format!("tags/{}?page={}", tag_type, page)).await?;
            let tags = response
                .get("result")
                .or_else(|| response.get("tags"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Invalid tag list response".to_string())?;

            if tags.is_empty() {
                break;
            }

            for tag in tags {
                let id = tag
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| "Tag missing id".to_string())? as i64;
                let name = tag
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let count = tag
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as i64;

                all_tags.push(crate::ext::database::TagEntry {
                    id,
                    name,
                    tag_type: tag_type.to_string(),
                    count,
                    status: 0,
                });
            }

            let total_pages = response
                .get("num_pages")
                .and_then(|v| v.as_u64())
                .or_else(|| response.get("pages").and_then(|v| v.as_u64()))
                .unwrap_or(page as u64) as u32;
            if page >= total_pages {
                break;
            }
            page += 1;
        }
    }

    crate::ext::database::db_upsert_tags_bulk(all_tags)
        .map_err(|e| e.to_string())?;

    Ok(Value::String("ok".to_string()))
}

// -- Favorites -----------------------------------------------------------------

/// GET /api/v2/favorites?page=N — user favorites list
#[tauri::command]
pub async fn api_favorites(page: u32) -> Result<Value, String> {
    api_get(&format!("favorites?page={}", page)).await
}

/// GET /api/v2/favorites/random — random favorite
#[tauri::command]
pub async fn api_favorites_random() -> Result<Value, String> {
    api_get("favorites/random").await
}

// -- User / Account ------------------------------------------------------------

/// GET /api/v2/user — current user info
#[tauri::command]
pub async fn api_user() -> Result<Value, String> {
    api_get("user").await
}

/// GET /api/v2/user/keys — list API keys
#[tauri::command]
pub async fn api_user_keys() -> Result<Value, String> {
    api_get("user/keys").await
}

// -- Blacklist -----------------------------------------------------------------

/// GET /api/v2/blacklist — get blacklist
#[tauri::command]
pub async fn api_blacklist() -> Result<Value, String> {
    api_get("blacklist").await
}

/// GET /api/v2/blacklist/ids — get blacklist IDs
#[tauri::command]
pub async fn api_blacklist_ids() -> Result<Value, String> {
    api_get("blacklist/ids").await
}

// -- Config --------------------------------------------------------------------

/// GET /api/v2/cdn — CDN server config
#[tauri::command]
pub async fn api_get_cdn_config() -> Result<Value, String> {
    api_get("cdn").await
}



fn urlenc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push_str("+"),
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// Fetch an image via reqwest (bypassing WebView CORS/CDN restrictions)
/// Returns base64-encoded image data with MIME type.
#[tauri::command]
pub async fn api_fetch_image_base64(url: String) -> Result<serde_json::Value, String> {
    let cookie_hdr = build_cookie_header();
    let mut req = client()
        .get(&url)
        .header("Referer", "https://nhentai.net/")
        .header("User-Agent", UA);
    if let Some(c) = &cookie_hdr {
        req = req.header("Cookie", c);
    }
    let resp = req.send().await.map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}: Failed to fetch image", status.as_u16()));
    }
    let mime = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();
    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(serde_json::json!({ "mime": mime, "data": b64 }))
}
