use tauri::AppHandle;
use tauri_plugin_notification::{NotificationExt, PermissionState};

/// Web notifications handling.
///
/// The notification plugin already injects `window.Notification` into the webview,
/// but we also ensure the native permission flow is initialized on startup.
pub fn init_webnotifications(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let permission = app.notification().permission_state()?;

    if matches!(
        permission,
        PermissionState::Prompt | PermissionState::PromptWithRationale
    ) {
        let _ = app.notification().request_permission()?;
    }

    Ok(())
}

#[tauri::command]
pub fn native_notify(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}
