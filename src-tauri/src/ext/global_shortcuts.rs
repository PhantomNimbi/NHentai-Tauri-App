use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Global shortcuts handling.
///
/// This module registers desktop hotkeys and routes them to the
/// main application window.
pub fn init_global_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
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

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(
        "CmdOrControl+Shift+H",
        move |_app, _shortcut, _event| {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.hide();
            }
        },
    )?;

    Ok(())
}
