# 📐 Architecture

> How the NHentai Tauri App is structured under the hood.

---

## 🧱 Layered architecture

The app uses a **custom frontend** for browsing, searching, tag management, and gallery reading, with nhentai.net loaded in the WebView only for login pages.

```sql
┌─────────────────────────────────────────────────┐
│  Custom Frontend (src/index.html)               │
│  Home · Search · Tags · Reader · Favorites      │
│  History · Settings (API key, language, etc.)   │
├─────────────────────────────────────────────────┤
│          Tauri IPC bridge (invoke)              │
├─────────────────────────────────────────────────┤
│              Rust Backend (src-tauri/)          │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐     │
│  │api.rs    │ │database  │ │navigation.rs │     │
│  │(reqwest) │ │(SQLite)  │ │(link handle) │     │
│  ├──────────┤ ├──────────┤ ├──────────────┤     │
│  │cloudfare │ │tray.rs   │ │downloads.rs  │     │
│  └──────────┘ └──────────┘ └──────────────┘     │
├─────────────────────────────────────────────────┤
│  Native Platform Layer                          │
│  WebView2 · WKWebView · GTK                     │
└─────────────────────────────────────────────────┘
```

**Data flow:**

1. App starts → loads `src/index.html` (custom SPA)
2. Frontend calls Tauri `invoke('api_search', ...)` etc.
3. Rust backend makes HTTPS requests to `nhentai.net/api/v2/...` via reqwest with rustls TLS support
4. Responses are cached in SQLite (`gallery_cache`, `search_cache`) with pre-computed image URLs
5. Frontend renders gallery grids and the in-app reader from DB-backed data
6. Login flows through nhentai.net `/login/` in the WebView
7. Init scripts (navigation, tag blacklist, context menu) inject into all pages

---

## 🪟 Desktop-only architecture

### 🖥️ Desktop path (`cfg(desktop)`)
- Full feature set: custom frontend, tray icon, global shortcuts, context menu, notifications
- Windows-only: WebView2 COM header injection via `cloudfare.rs`
- `build_init_script()` combines navigation + tag blacklist + context menu

This app is designed as a desktop-only client for nhentai.net and does not include mobile-specific wrappers or mobile-only builds.
---

## 🧩 Module architecture

### 📁 `src/` — Complete single-page app

| Aspect | Detail |
|---|---|
| **File** | `index.html` — all HTML, CSS, and JS in one file |
| **Views** | Home, Search, Tag Filter, Random, Favorites, History, Settings, Reader |
| **State** | tags (`tagState`/`tagDB`), settings, history, favorites in memory + DB sync |
| **API calls** | All through `invoke('api_*')` — no direct `fetch()` to nhentai |

### 📁 `src-tauri/src/ext/` — Rust feature modules

| Module | Platform | Purpose |
|---|---|---|
| `api.rs` | All | reqwest HTTP client; all v2 API endpoints; API key support; cookie jar |
| `database.rs` | All | SQLite schema + CRUD for tags, history, favorites, settings, gallery_cache, search_cache |
| `navigation.rs` | All | Link handling, login auto-redirect, cookie forwarding |
| `tag_blacklist.rs` | All | Three-state tag toggles on nhentai.net pages |
| `cloudfare.rs` | All | Chrome 126 User-Agent + Windows COM header injection |
| `context_menu.rs` | Desktop | Native right-click context menus |
| `downloads.rs` | All | Download interception + sanitization |
| `tray.rs` | Desktop | System tray icon + menu |
| `global_shortcuts.rs` | Desktop | Global keyboard shortcuts |
| `webnotifications.rs` | All | Notification permission handling |
| `universal_deep_link.rs` | All | Deep link scheme (`nhentai://`) |

### 📁 `src-tauri/src/lib.rs` — Application wiring

The `run()` function (desktop):

1. Registers all Tauri commands (API, database, navigation, context menu)
2. Installs plugins (opener, notification, deep-link, shell, global-shortcut)
3. **Setup closure**:
   - Initializes SQLite database
   - Loads persisted API key from DB into global
   - Configures WebView with Chrome 126 User-Agent
   - Builds window with init scripts (nav + tag blacklist + context menu)
   - Initializes tray, shortcuts, Cloudflare COM, etc.

---

## 🔌 Tauri commands (IPC bridge)

### API commands (api.rs)
| Command | Path | Description |
|---|---|---|
| `api_all` | `/api/v2/galleries` | Browse all galleries |
| `api_search` | `/api/v2/search` | Search with query |
| `api_gallery` | `/api/v2/galleries/{id}` | Gallery detail (includes related, favorite) |
| `api_random` | `/api/v2/galleries/random` | Random gallery |
| `api_favorites` | `/api/v2/favorites` | User favorites |
| `api_user` | `/api/v2/user` | User info (login check) |
| `api_blacklist` | `/api/v2/blacklist` | User blacklist |
| `api_get_config` | `/api/v2/config` | CDN server config |
| `api_set_cookie` | — | Forward cookie from WebView to reqwest |
| `api_set_api_key` | — | Set API key (persisted to DB) |
| `api_clear_api_key` | — | Clear API key |

### Database commands (database.rs)
| Command | Description |
|---|---|
| `db_get_tags` / `db_upsert_tags_bulk` | Tag CRUD + bulk sync |
| `db_add_history` / `db_get_history` / `db_clear_history` | History ops |
| `db_toggle_favorite` / `db_get_favorites` / `db_clear_favorites` | Favorite ops |
| `db_get_setting` / `db_set_setting` | Settings ops |
| `db_get_gallery_cache` / `db_save_gallery_cache` | Gallery detail caching |
| `db_get_search_cache` / `db_save_search_cache` | Search result caching |
| `db_get_snapshot` | Full state dump (tags + history + favorites + settings) |

---

## 💾 SQLite Schema

```sql
tags (id, name, type, count, status)          — Three-state tag system
history (id, gallery_id, title, time)         — Browsing history
favorites (gallery_id, time)                  — Starred galleries
settings (key, value)                         — Key-value config store
gallery_cache (gallery_id, data, updated_at)  — Full gallery detail JSON
search_cache (cache_key, data, updated_at)    — Search/home result JSON
```

All computed image URLs (cover, pages) are pre-computed and stored in the `data` JSON before caching.

---

## 🔒 Security model

| Concern | Mitigation |
|---|---|
| **API key storage** | Stored in SQLite (local file); never exposed to WebView |
| **Download path traversal** | Filenames sanitized, path separators stripped |
| **Deep link injection** | Only `nhentai://` and nhentai.net universal links accepted |
| **Script injection** | Controlled init scripts only; website JS in normal sandbox |
| **Cloudflare detection** | Chrome 126 UA + browser-like headers to avoid challenges |
