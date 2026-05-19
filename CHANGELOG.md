## v1.0.1 - 2026-05-19

### ✨ What's new

- 🚀 Updated the app version to `1.0.1` across manifest files and release metadata.
- 📁 Renamed the frontend folder from `frontend/` to `src/` and updated configuration/docs accordingly.
- 🧾 Fixed the gallery card description rendering so missing or undefined text no longer shows `undefined`.
- 🔍 Aligned the Search page sort bar with the Home page sort bar by adding the `All` sort option.
- 🔒 Added rustls TLS-backed reqwest support for HTTPS nhentai.net API calls so network requests work reliably at runtime.
- 🧠 Updated documentation and agent guidance to reflect the correct nhentai v2 API usage, confirm-before-save policy, and root-cause-first fix approach.
- 🔒 Added rustls TLS support for reqwest so HTTPS API requests to nhentai.net are reliable at runtime.

## v1.0.0 - 2026-05-18

### ✨ What's new

- Initial release of the NHentai Tauri App with a full custom frontend and API-backed browsing experience for nhentai.net.
- Includes live tag filtering, search, reader, favorites, history, settings, downloads, and platform-specific desktop features.

### 📝 Release highlights

- ✅ Live API-backed tag filter
  - Tag Filter page now fetches supported tags directly from nhentai API endpoints instead of relying on local DB population.
  - Supports tabbed type browsing, incremental pagination, and live search via `/api/v2/tags/search`.
  - All current tags are loaded from nhentai API listings so the filter page can only apply what is actually available.

- ✅ Home sort and filter behavior
  - Added an explicit `All` home sort option.
  - Tag filters are only applied when Home is set to `All`; other home sorts ignore active tag filters to preserve expected browse behavior.

- ✅ Robust API layer
  - Backend now includes dedicated `api_search_tags` and improved tag listing commands.
  - Removed the custom internal rate limiter from the Rust API layer so API requests are not artificially delayed.
  - Fixed Tauri invoke compatibility for tag listing and search commands.

- ✅ Search and reader improvements
  - Search view uses active tag chips and live API search.
  - Gallery reader and detail views now correctly build and display page URLs for cover and reader images.

- ✅ Persistence and caching
  - Search/home result caching is keyed by sort and tag filter context.
  - Gallery cache stores detail metadata and pre-computed page URLs for faster reader loads.

- ✅ Desktop polish
  - Secure download interception, filename sanitization, and collision-safe save handling.
  - System tray integration and global shortcuts for desktop convenience.
  - Deep link support via `nhentai://` and universal links.

### 🧩 Technical notes

- Built with Tauri v2 and Rust, using a single-page frontend in `src/index.html`.
- All nhentai API requests go through a Rust reqwest proxy to bypass WebView CORS and support optional API key authentication.
- SQLite is used for persistent storage of tags, history, favorites, settings, gallery cache, and search cache.
