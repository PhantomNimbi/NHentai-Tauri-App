use std::error::Error;

use serde_json;
use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use url::Url;

/// Initializes universal deep-link handling for both desktop and mobile.
///
/// Supports both the custom scheme `nhentai://...` and HTTPS Universal
/// Links targeting `https://nhentai.net//...`.
pub fn init_universal_deep_link(app: AppHandle) -> Result<(), Box<dyn Error>> {
    let handle = app.clone();
    let listener_handle = handle.clone();

    app.listen("deep-link://new-url", move |event| {
        if let Ok(urls) = serde_json::from_str::<Vec<Url>>(event.payload()) {
            for url in urls {
                handle_deep_link(&listener_handle, &url);
            }
        }
    });

    if let Ok(Some(urls)) = app.deep_link().get_current() {
        for url in &urls {
            handle_deep_link(&handle, url);
        }
    }

    Ok(())
}

fn handle_deep_link(app: &AppHandle, url: &Url) {
    let target = match url.scheme() {
        "nhentai" => map_nhentai_scheme(url),
        "https" if url.host_str() == Some("nhentai.net") => url.clone(),
        _ => return,
    };

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.navigate(target);
    }
}

fn map_nhentai_scheme(url: &Url) -> Url {
    let mut mapped = Url::parse("https://nhentai.net/").expect("hardcoded URL is valid");
    mapped.set_path(url.path());
    mapped.set_query(url.query());
    mapped.set_fragment(url.fragment());
    mapped
}
