/// Cloudflare anti-spam challenge support for nhentai.net.
///
/// nhentai.net uses Cloudflare JS Detection (JSD) and Turnstile challenges
/// on `/login/`. This module configures the WebView to appear as a standard
/// Chrome browser so the challenge JavaScript executes and completes
/// without interference.
///
/// ## Approach
///
/// 1. **User-Agent spoofing** – Set a Chrome 126 Windows User-Agent via
///    Tauri's `WebviewWindowBuilder::user_agent()` so Cloudflare receives a
///    realistic browser fingerprint on every navigation.
/// 2. **Header augmentation (Windows only)** – Hook into WebView2's
///    `WebResourceRequested` COM event to add browser-like request headers
///    (`Accept-Language`) that real Chrome sends automatically.
/// 3. **No request blocking** – Unlike the old adblock interception layer,
///    this module never denies or modifies a response. It only *adds* headers
///    to the outgoing request, so Cloudflare challenges and CDN resources
///    always load normally.

use tauri::{Manager, Runtime, WebviewWindowBuilder};

/// Chrome 126 Windows User-Agent string.
///
/// Matches what a real Chrome 126 on Windows 10 sends, so Cloudflare's
/// JS Detection fingerprinting sees a standard browser.
pub const CHROME_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// Apply Cloudflare-friendly settings to a `WebviewWindowBuilder`.
///
/// Must be called **before** `.build()`.
pub fn configure_window<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    builder.user_agent(CHROME_UA)
}

/// Post-build Cloudflare initialisation.
///
/// On **Windows** this uses `with_webview()` to access the raw
/// `ICoreWebView2` COM interface and registers a
/// `WebResourceRequested` event handler that adds browser-like request
/// headers (`Accept-Language`) to every web resource request.
///
/// On other platforms this is a no-op.
///
/// Must be called **after** the main window has been built.
pub fn init(app: &tauri::AppHandle) {
    #[cfg(windows)]
    if let Err(e) = platform::init(app) {
        eprintln!("[cloudfare] init warning: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Windows-only COM-level header augmentation
// ---------------------------------------------------------------------------
#[cfg(windows)]
mod platform {
    use tauri::{AppHandle, Manager};
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL, ICoreWebView2,
        ICoreWebView2WebResourceRequestedEventArgs,
    };
    use webview2_com::WebResourceRequestedEventHandler;

    /// Register a `WebResourceRequested` handler that adds browser-like
    /// HTTP headers to every outgoing request.
    ///
    /// This is purely additive – no request is ever blocked.
    pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let window = app
            .get_webview_window("main")
            .ok_or("main window not found")?;

        window.with_webview(move |platform_webview| {
            let controller = platform_webview.controller();

            unsafe {
                // Obtain the ICoreWebView2 from the controller.
                let core_webview = match controller.CoreWebView2() {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("[cloudfare] CoreWebView2 failed: {}", e);
                        return;
                    }
                };

                // Intercept every URL and every resource type.
                if let Err(e) = core_webview.AddWebResourceRequestedFilter(
                    &windows_core::HSTRING::from("*"),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                ) {
                    eprintln!("[cloudfare] AddWebResourceRequestedFilter failed: {}", e);
                    return;
                }

                // Event handler: add browser-typical headers to every request.
                let handler = WebResourceRequestedEventHandler::create(Box::new(
                    move |_sender: Option<ICoreWebView2>,
                          args: Option<ICoreWebView2WebResourceRequestedEventArgs>| {
                        if let Some(args) = args {
                            if let Ok(request) = args.Request() {
                                if let Ok(headers) = request.Headers() {
                                    // Accept-Language – sent by every real browser.
                                    let _ = headers.SetHeader(
                                        &windows_core::HSTRING::from("Accept-Language"),
                                        &windows_core::HSTRING::from("en-US,en;q=0.9"),
                                    );
                                }
                            }
                        }
                        Ok(())
                    },
                ));

                let mut token: i64 = 0;
                if let Err(e) = core_webview.add_WebResourceRequested(&handler, &mut token) {
                    eprintln!("[cloudfare] add_WebResourceRequested failed: {}", e);
                }
            }
        })?;

        Ok(())
    }
}
