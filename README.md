# 🏠 NHentai Tauri App

**🚀 Current release:** [v1.0.1](CHANGELOG.md#v101---2026-05-19) • **Updated:** 2026-05-19

A native desktop client for [nhentai.net](https://nhentai.net) — built with **Tauri v2** and **Rust**.  
Custom API-driven frontend for browsing; WebView for gallery reading.

## 🆕 Release summary

- ✅ Live API-backed tag filtering with tag listings loaded directly from nhentai's tag API
- 🏷️ Tag Filter page now uses live API data instead of local DB tag population
- 🔁 Home `All` sort is the only sort that applies active tag filters
- 🚫 Removed custom backend rate limiting; requests now flow without artificial delay
- 🔒 Added rustls TLS-backed reqwest support for HTTPS API calls to nhentai.net
- 📱 Android users should visit https://github.com/maxwai/NClientV3 for the dedicated Android client.
- 🧠 Updated docs and AI agent guidance to enforce correct nhentai v2 API usage, root-cause fixes, and confirm-before-save behavior
- 📖 Improved search, reader, cache, and API command behavior for a more reliable app experience

---

## 🎯 What is this?

This app replaces the nhentai.net browsing experience with a **custom frontend** powered by the nhentai API. Browse, search, and manage tag blacklists in a native-feeling UI. Gallery pages open on nhentai.net itself within the app's WebView.

- 🖥️ **Custom UI** — API-driven browsing with gallery grid, search, and blacklist manager
- 🏷️ **Tag blacklisting** — API-level exclusion via `-tag:"name"` syntax; galleries are pre-filtered
- 🛡️ **Ad blocking** — JS-level interception on nhentai.net pages (tsyndicate.com)
- 🔗 **Smart navigation** — Gallery clicks open nhentai.net; "← App" button returns to custom UI
- 📥 **Safe downloads** — Filename sanitization, collision handling, native notifications
- ⌨️ **Global shortcut** — `Ctrl+Shift+O` to show the window (desktop-only)
- 🖥️ **System tray** — Background operation with quick restore (desktop-only)
- ☁️ **Anti-bot** — Chrome 126 User-Agent to avoid Cloudflare challenges

---

## 🚀 Quick start

```bash
# 1. Install Tauri CLI
cargo install tauri-cli --version "^2" --locked

# 2. Clone and enter
git clone https://github.com/PhantomNimbi/NHentai-Tauri-App.git
cd NHentai-Tauri-App

# 3. Build for production
cargo tauri build

# 4. Run in dev mode
cargo tauri dev
```

---

## 📚 Documentation

| Page | Description |
|---|---|
| [📖 Introduction](docs/index.md) | Project overview, features, tech stack |
| [📐 Architecture](docs/architecture.md) | Data flow, module layering, platform split |
| [✨ Features](docs/features.md) | Detailed documentation of every feature |
| [🛠️ Development](docs/development.md) | Building, testing, debugging, release process |

---

## 📁 Project structure

```
nhentai-tauri-app/
├── src/
│   └── index.html               # Custom app UI (API-driven)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                # App setup, command registration
│   │   ├── main.rs               # Binary entrypoint
│   │   └── ext/
│   │       ├── adblock.rs        # JS-level ad blocking (tsyndicate.com)
│   │       ├── tag_blacklist.rs  # Tag blacklisting (nhentai.net pages)
│   │       ├── cloudfare.rs      # Cloudflare UA spoofing
│   │       ├── navigation.rs     # Link handling + Back to App button
│   │       ├── context_menu.rs   # Native right-click menu (desktop)
│   │       ├── downloads.rs      # Download interception
│   │       ├── tray.rs           # System tray (desktop)
│   │       ├── global_shortcuts.rs # Global shortcuts (desktop)
│   │       ├── webnotifications.rs # Notification permissions
│   │       └── universal_deep_link.rs # Deep link handler
│   ├── Cargo.toml
│   └── tauri.conf.json
└── docs/
```

---

## ✅ Supported platforms

| Platform | Target triple | Status |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | ✅ Full support |
| macOS | `aarch64-apple-darwin` | ✅ Full support |
| Linux | `x86_64-unknown-linux-gnu` | ✅ Full support |
