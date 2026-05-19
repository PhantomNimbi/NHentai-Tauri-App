use crate::show_main_window;
use tauri::{AppHandle, Manager};

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn init_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )) {
        let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
        let hide_item = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

        let icon = app
            .default_window_icon()
            .ok_or("no default window icon")?
            .clone();

        let _ = TrayIconBuilder::new()
            .icon(icon)
            .tooltip("NHentai")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(move |app, event| match event.id.as_ref() {
                "show" => crate::show_main_window(app),
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            })
            .on_tray_icon_event(move |tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    show_main_window(tray.app_handle());
                }
            })
            .build(app)?;

        Ok(())
    } else {
        Ok(())
    }
}
