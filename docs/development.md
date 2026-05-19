# 🛠️ Development

> How to build, test, and contribute to the NHentai Tauri App.

---

## 📋 Prerequisites

| Tool | Version | Purpose |
|---|---|---|
| **Rust** | 1.85+ (edition 2021) | Backend compilation |
| **Tauri CLI** | ^2 | Build, dev, and package commands |

### Platform-specific requirements

- **Windows**: WebView2 Runtime (included in Windows 11)
- **macOS**: Xcode Command Line Tools
- **Linux**: WebKitGTK, libsoup, and other Tauri v2 Linux dependencies

---

## 🚀 Quick start

```bash
# Clone the repository
git clone https://github.com/PhantomNimbi/NHentai-Tauri-App.git
cd nhentai-tauri-app

# Install Tauri CLI
cargo install tauri-cli --version "^2" --locked

# Desktop development mode
cargo tauri dev

# Production build (desktop)
cargo tauri build
```

---

## 📁 Project structure

```
nhentai-tauri-app/
├── src/
│   └── index.html              # Complete SPA (reader, tags, settings, etc.)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # Binary entry point
│   │   ├── lib.rs              # App setup, run() for desktop only
│   │   └── ext/
│   │       ├── mod.rs          # Module declarations with platform #[cfg]
│   │       ├── api.rs          # reqwest HTTP client + all v2 API commands
│   │       ├── database.rs     # SQLite schema + all CRUD commands
│   │       ├── navigation.rs   # Link handling + login auto-redirect
│   │       ├── tag_blacklist.rs# Three-state tag toggle on nhentai.net pages
│   │       ├── cloudfare.rs    # Cloudflare UA spoofing + COM headers
│   │       ├── context_menu.rs # Native right-click (desktop-only)
│   │       ├── downloads.rs    # Download interception + sanitization
│   │       ├── tray.rs         # System tray (desktop-only)
│   │       ├── global_shortcuts.rs # Global hotkeys (desktop-only)
│   │       ├── webnotifications.rs # Notification permissions
│   │       └── universal_deep_link.rs # nhentai:// handler
│   ├── Cargo.toml
│   └── tauri.conf.json
└── docs/
```

---

## 🧪 Testing

```bash
# Run all Rust unit tests
cd src-tauri
cargo test

# Run with output
cargo test -- --nocapture
```

---

## 🔨 Making changes

### Adding a new feature to the frontend

1. Edit `src/index.html` — all JS/CSS/HTML in one file
2. Test by running `cargo tauri dev`
3. If adding a new IPC call, register the command in `lib.rs`

### Adding a new Rust feature module

1. Create `src-tauri/src/ext/your_feature.rs`
2. Add `pub mod your_feature;` to `src-tauri/src/ext/mod.rs`
3. Implement Tauri commands with `#[tauri::command]`
4. Register commands in `lib.rs::run()` via `generate_handler![]`
5. Add any setup logic in the `setup()` closure
6. Update documentation in `docs/`

### Modifying the API layer

The API client lives in:
- **Frontend**: `src/index.html` — `invoke('api_*')` calls
- **Backend**: `src-tauri/src/ext/api.rs` — reqwest commands + auth headers (rustls TLS-enabled)
- Tag Filter now fetches tags via API listings and does not rely on local DB tag population
- Home sort `All` is treated as the only sort where tag filters are applied; other sorts ignore active tag filters
- To add a new endpoint: add a `#[tauri::command]` function in api.rs, call `api_get(&path)` inside

### Modifying the database

All SQLite operations are in `src-tauri/src/ext/database.rs`:
- Schema defined in `init_database()`
- Add new tables in the `CREATE TABLE IF NOT EXISTS` batch
- Add `#[tauri::command]` functions for CRUD
- Register in `lib.rs`

### Three-state tag system

- Frontend: `cycleTagStatus(id)` / `setTagStatus(id, status)` in `index.html`
- Backend: `tag_blacklist.rs` injects JS into nhentai.net pages for click-to-toggle
- Storage: `localStorage` (`nhentai_tag_state_v4`, `nhentai_tag_db_v1`) + SQLite `tags` table
- Query building: `buildTagQuery(base)` in frontend appends `tag:"name"` / `-tag:"name"`

---

## 🏗️ Building platform-specific releases

```bash
# Windows
cargo tauri build

# macOS
cargo tauri build --target aarch64-apple-darwin

# Linux
cargo tauri build --target x86_64-unknown-linux-gnu
```

---

## 🐛 Debugging

### Enable logging

The app uses `println!` for debug output (visible in the terminal running `cargo tauri dev`). Key debug points:
- API requests/responses
- Navigation URL checks
- Download events
- Database operations

### API debugging

1. Set your API key in Settings (password field)
2. Check terminal for HTTP response codes
3. Verify `Authorization: Key <api_key>` header is sent

### Tag system debugging

1. **Custom UI**: Open the Tag Filter tab — browse/search/cycle tags
2. **API exclusion**: Search with avoided tags — galleries should be pre-filtered
3. **nhentai.net pages**: Visit a gallery and click tags to toggle — red outline indicates avoided
4. **localStorage**: `localStorage.getItem('nhentai_tag_state_v4')` shows current state

### Gallery caching

- `gallery_cache` table stores full gallery detail JSON
- `search_cache` table stores search/home results
- Both cache pre-computed image URLs — stable across page switches
- Cache is fallback-only on search views; gallery detail cache is primary source for reader

### Known pitfalls

| Pitfall | Solution |
|---|---|
| WebView2 COM calls fail if webview isn't initialized | Always handle `Result` errors, never `.unwrap()` |
| API returns 403 | Ensure API key is set in Settings; the app sends `Authorization: Key` header |
| Cover images broken on tab switch | `ensureGalleryUrls` pre-computes URLs before caching; try clearing search cache |
| Frontend not loading | Check `frontendDist` in `tauri.conf.json` points to `../src` |

---

## 📦 Release process

1. Run `.github/scripts/auto-version.sh --commit --push` to bump version
2. Run `cargo test` to verify everything passes
3. Run `cargo build` to verify compilation
4. Commit with Conventional Commits format
5. The GitHub workflow publishes the release on push to `main`
