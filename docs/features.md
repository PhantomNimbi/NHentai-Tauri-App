# Features

> Detailed documentation of every feature in the NHentai Tauri App.

---

## Custom App UI

The app serves a **custom frontend** (`src/index.html`) that replaces the nhentai.net website for browsing, searching, and reading.

| Aspect | Detail |
|---|---|
| **Technology** | Single HTML file with vanilla JS |
| **Data source** | nhentai.net v2 API via Rust reqwest proxy |
| **Gallery reading** | In-app reader (no WebView navigation) |
| **Layout** | Sidebar drawer, responsive gallery grid |

### Views

- **Home** — Gallery grid with sort selector (recent, popular today/week/month/all-time), pagination; tag filters are only applied when `All` is selected
- **Search** — Text query + active tag chips (with three-state status toggles) + sort + paginated results
- **Tag Filter** — Tabbed tag manager per type with three-state toggles and tag search via API autocomplete; uses live nhentai API tag listings instead of local DB-only data
- **Random** — Load and display a random gallery
- **Favorites** — Locally stored favorite galleries (star toggles)
- **History** — Recently viewed galleries (last 200)
- **Settings** — Default sort, language filter, grid columns, remove avoided toggle, API key input
- **Reader** — Page-by-page gallery reading with click areas and keyboard shortcuts

---

## Three-State Tag System (Matching NClientV3)

Tags are managed with a three-state system:

| State | Visual | Query Effect | Meaning |
|---|---|---|---|
| DEFAULT (0) | Gray circle | Not included | Neutral — no filtering |
| ACCEPTED (2) | Green checkmark | `tag:"name"` appended | Only show galleries containing this tag |
| AVOIDED (1) | Red X | `-tag:"name"` appended | Exclude galleries containing this tag |

Tags cycle: DEFAULT → ACCEPTED → AVOIDED → DEFAULT on click.

### How it works

1. **On nhentai.net pages**: Click any `a.tag` element to toggle between DEFAULT and AVOIDED (quick blacklist)
2. **In Search view**: Active tags appear as chips below the search bar — click to cycle status, × to remove
3. **In Tag Filter view**: Browse all known tags grouped by type, search for new tags via API autocomplete, cycle status with a click
4. **In search queries**: Every API call includes all ACCEPTED tags (required) and all AVOIDED tags (exclusions). Language filter is also appended when set.

### Storage

- `nhentai_tag_state_v4` (localStorage) — tag_id → {status, name, type} for non-DEFAULT tags
- `nhentai_tag_db_v1` (localStorage) — tag_id → {name, type, count} for all known tags
- SQLite `tags` table — full tag state persisted via DB snapshot/bulk upsert

---

## In-App Reader

Gallery reading happens entirely within the app — no WebView navigation to nhentai.net.

| Aspect | Detail |
|---|---|
| **Navigation** | Click left half → prev page, click right half → next page |
| **Keyboard** | ← / → arrow keys, Esc to close |
| **Page input** | Type page number and press Enter to jump |
| **Tag display** | Gallery tags shown below reader |
| **Back** | "← Back" button returns to previous view |
| **Data source** | `gallery_cache` (DB) with pre-computed page image URLs on every gallery item |

### Loading

`loadReader(id)`: checks `gallery_cache` in DB first. If cached, renders from cache. If not, fetches gallery detail from API, pre-computes all page image URLs (`page.url`), caches the result in DB, then renders.

### Page image URL caching

Page image URLs are pre-computed from the CDN config (`imageServers`) and `media_id` + page number. Once computed, they're stored in the cached JSON so subsequent loads don't need network access to construct URLs.

---

## API Layer

All nhentai.net API calls go through a Rust reqwest HTTP client, bypassing WebView CORS entirely.

| Aspect | Detail |
|---|---|
| **Client** | `reqwest::Client` (singleton, lazy-initialized; rustls TLS-enabled) |
| **Base URL** | `https://nhentai.net/api/v2` |
| **User-Agent** | `NClient/1.0.1` |
| **Cookie jar** | Shared in-memory `HashMap`; forwarded from WebView via JS injection |
| **API key** | Optional; sent as `Authorization: Key <api_key>` header |

### API key

- Set in Settings view (password field)
- Persisted to DB settings (`api_key` key)
- Loaded from DB on startup and set in the global `API_KEY` mutex
- Sent as `Authorization: Key <api_key>` on every API request
- Clear button removes from memory and DB

### Endpoints

| Endpoint | Command | Purpose |
|---|---|---|
| `/api/v2/galleries` | `api_all` | Browse all |
| `/api/v2/search` | `api_search` | Search + tag filters |
| `/api/v2/galleries/{id}` | `api_gallery` | Gallery detail |
| `/api/v2/galleries/random` | `api_random` | Random gallery |
| `/api/v2/favorites` | `api_favorites` | User favorites |
| `/api/v2/user` | `api_user` | Login check |
| `/api/v2/blacklist` | `api_blacklist` | Import blacklist |
| `/api/v2/config` | `api_get_config` | CDN server config |

---

## Caching

### Gallery cache (`gallery_cache`)
- Keyed by `gallery_id`
- Stores full gallery detail JSON with pre-computed page image URLs (`page.url`)
- Used by reader; checked before API call

### Search cache (`search_cache`)
- Keyed by `search:{query}:{page}:{sort}` or `home:{sort}:1`
- Stores full API response JSON with pre-computed cover URLs (`g.coverUrl`)
- Used as fallback when API call fails
- Fresh API data is always fetched on view switch; cache is fallback only

### Cover URL caching
- `thumbUrl(g)` computes CDN URL from `g.thumbnail` + `g.media_id` and stores as `g.coverUrl`
- `ensureGalleryUrls(data)` pre-computes `coverUrl` for all galleries in a result set
- Called before every `db_save_search_cache` so cached JSON has stable URLs

### CDN config cache
- `/api/v2/config` response cached in DB settings (`cdn_config`)
- Loaded on startup; background refresh on every `loadConfig()` call
- Provides `thumb_servers` and `image_servers` arrays

---

## Smart Navigation

Internal links (nhentai.net domain) open in the main webview window for login flows. External links open in system browser.

- `window.open()` overridden to intercept popups
- Anchor clicks with `target="_blank"` or modifier keys are captured
- `/login/` URLs routed to the main webview
- Non-nhentai URLs open in system browser via `tauri-plugin-opener`
- Login auto-redirect: after successful login, page redirects back to app via `sessionStorage` tracking
- Cookie forwarding: JS polls `document.cookie` for 5s after page load, sends cookies to `api_set_cookie` command

---

## Download Handling

All file downloads are intercepted and managed:

1. **Filename sanitization** — strips path separators, control characters, leading dots/spaces
2. **Length limiting** — filenames capped at 200 characters
3. **Collision handling** — appends `(1)`, `(2)`, etc. on duplicates
4. **Native notifications** — on completion or failure

---

## System Tray (Desktop-only)

- Tray icon with "Show" and "Quit" context menu
- Left-click shows/focuses main window

---

## Global Shortcut (Desktop-only)

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+O` (Windows/Linux) | Show and focus main window |
| `Cmd+Shift+O` (macOS) | Show and focus main window |

---

## Deep Linking

| Scheme | Example |
|---|---|
| `nhentai://` | `nhentai://g/12345` |
| HTTPS universal | `https://nhentai.net/g/12345` |

Handled by `universal_deep_link.rs` — navigates main webview to the gallery page.

---

## Cloudflare Anti-Spam Challenge Support

| Aspect | Detail |
|---|---|
| **User-Agent** | Chrome 126 Windows UA via `WebviewWindowBuilder::user_agent()` |
| **Header augmentation** | Windows: WebView2 `WebResourceRequested` COM event adds browser-like headers |
| **API key alternative** | API key auth bypasses Cloudflare entirely for API calls |
| **Login routing** | `/login/` routed to main webview |
| **Platform** | UA set on all platforms; COM injection Windows-only |
