use tauri::{AppHandle, Manager};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Global shortcuts handling.
///
/// This module registers desktop-only hotkeys and routes them to the
/// main application window.
pub fn init_global_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let app_handle = app.clone();

        app.global_shortcut().on_shortcut(
            "CmdOrControl+Shift+O",
            move |_app, _shortcut, _event| {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            },
        )?;
    }

    Ok(())
}
