# 🏠 NHentai Tauri App

> **Version:** v1.0.1 · **Updated:** 2026-05-19

A native desktop client for [nhentai.net](https://nhentai.net) — built with **Tauri v2** and **Rust** — featuring a custom API-driven frontend for browsing, searching, and reading galleries.

---

## 🎯 What is this?

This app replaces nhentai.net with a **custom frontend** that uses the nhentai API for browsing, searching, tag management, and gallery reading. nhentai.net is only loaded in the WebView for login pages.

- 🖥️ **Custom UI** — API-driven browsing with gallery grid, search, and tag management
- 🏷️ **Three-state tag system** — DEFAULT (neutral), ACCEPTED (required), AVOIDED (excluded)
- 🔁 **Home sort behavior** — tag filters only apply when Home is set to `All`, preventing unintended filtering on other sorts
- 📖 **In-app reader** — Read galleries entirely within the app with keyboard shortcuts
- 🔌 **Rust-proxied API** — All API calls go through reqwest with rustls TLS support (bypasses WebView CORS)
- 🔑 **API key support** — Optional `Authorization: Key` header for authenticated access
- 📱 **Android users** — See https://github.com/maxwai/NClientV3 for a dedicated Android client.
- 💾 **SQLite persistence** — Tags, history, favorites, settings, gallery cache, search cache
- 📄 **Gallery & search caching** — Full API response caching with pre-computed image URLs
- 🔗 **Smart navigation** — External links open in system browser; internal handled in-app
- 📥 **Secure downloads** — filename sanitization and collision handling
- 🖥️ **System tray** + **global shortcuts** — desktop convenience features

---

## ✨ Quick features

| Feature | Description |
|---|---|
| 🖥️ **Custom frontend** | API-driven browsing with home, search, tag filter, favorites, history, settings |
| 🏷️ **Three-state tags** | ACCEPTED (`tag:"name"`) / AVOIDED (`-tag:"name"`) in API queries |
| 📖 **In-app reader** | Page-by-page gallery reading with click areas and keyboard shortcuts |
| 💾 **SQLite database** | All data persisted — tags, history, favorites, settings, gallery + search cache |
| 🔑 **API key** | Optional nhentai API key for authenticated requests, persisted in DB settings |
| 🔌 **Rust API proxy** | reqwest HTTP client with rustls TLS, bypasses WebView CORS; shared cookie jar + API key header |
| 📦 **Gallery cache** | Full gallery detail + pre-computed page image URLs cached in DB |
| 🔍 **Search cache** | Search/home results + pre-computed cover URLs cached in DB |
| 🛡️ **Anti-bot** | Chrome 126 User-Agent for Cloudflare challenge avoidance |
| 📥 **Safe downloads** | Filename sanitization, collision handling, native notifications |
| 🖥️ **System tray** | Background operation with quick restore (desktop-only) |
| ⌨️ **Global shortcut** | `Ctrl+Shift+O` to show the window (desktop-only) |
| 🔗 **Deep linking** | `nhentai://` scheme and universal links |

---

## 🏗️ Tech stack

| Layer | Technology |
|---|---|
| **Shell** | Tauri v2 |
| **Webview** | WebView2 (Windows), WKWebView (macOS), WebKitGTK (Linux) |
| **Frontend** | Vanilla HTML/CSS/JS (single page, no framework) |
| **API proxy** | Rust reqwest (bypasses CORS, supports API key auth) |
| **Database** | SQLite via rusqlite (bundled) |
| **Data source** | nhentai.net REST API v2 |
| **Language** | Rust (edition 2021) |

---

## 📂 Project layout

```sql
nhentai-tauri-app/
├── src/
│   └── index.html                      # Complete SPA (reader, tags, settings, etc.)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                     # Binary entrypoint
│   │   ├── lib.rs                      # App setup, command registration, init script
│   │   └── ext/
│   │       ├── mod.rs                  # Module declarations
│   │       ├── api.rs                  # reqwest HTTP client + all v2 API commands
│   │       ├── database.rs             # SQLite schema + all CRUD commands
│   │       ├── navigation.rs           # Link handling + login auto-redirect
│   │       ├── tag_blacklist.rs        # Three-state tag toggle on nhentai.net pages
│   │       ├── cloudfare.rs            # Cloudflare UA spoofing + COM headers
│   │       ├── context_menu.rs         # Native right-click menu
│   │       ├── downloads.rs            # Download interception + sanitization
│   │       ├── tray.rs                 # System tray (desktop)
│   │       ├── global_shortcuts.rs     # Global shortcuts (desktop)
│   │       ├── webnotifications.rs     # Notification permissions
│   │       └── universal_deep_link.rs  # Deep link handler
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/
│   ├── index.md                # You are here
│   ├── architecture.md         # Architecture deep-dive
│   ├── features.md             # Feature documentation
│   └── development.md          # Building and contributing
├── AGENTS.md
├── CHANGELOG.md
└── README.md
```

---

## 📖 Continue reading

| Page | What you'll find |
|---|---|
| [📐 Architecture](architecture.md) | How the app is structured, data flow, platform split |
| [✨ Features](features.md) | Detailed documentation of every feature |
| [🛠️ Development](development.md) | Building, testing, and contributing |
