use std::error::Error;
use std::sync::{Mutex, OnceLock};

use serde::Deserialize;
use tauri::menu::{IsMenuItem, Menu, MenuItem};
use tauri::{AppHandle, LogicalPosition, Manager};
use tauri_plugin_opener::OpenerExt;

static LAST_CONTEXT_MENU_PAYLOAD: OnceLock<Mutex<Option<ContextMenuPayload>>> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuPayload {
    pub x: i32,
    pub y: i32,
    pub href: Option<String>,
    #[allow(dead_code)]
    pub selection: Option<String>,
}

fn payload_state() -> &'static Mutex<Option<ContextMenuPayload>> {
    LAST_CONTEXT_MENU_PAYLOAD.get_or_init(|| Mutex::new(None))
}

fn set_last_payload(payload: ContextMenuPayload) {
    if let Ok(mut guard) = payload_state().lock() {
        *guard = Some(payload);
    }
}

fn take_last_payload() -> Option<ContextMenuPayload> {
    payload_state()
        .lock()
        .ok()
        .and_then(|mut guard| guard.take())
}

pub fn build_context_menu_script() -> String {
    r#"
(function () {
    'use strict';

    const COMMAND = 'show_native_context_menu';

    function isEditable(element) {
        while (element) {
            if (element instanceof Element) {
                const name = element.nodeName.toUpperCase();
                if (element.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT', 'OPTION', 'BUTTON'].includes(name)) {
                    return true;
                }
            }
            element = element.parentElement;
        }
        return false;
    }

    window.addEventListener('contextmenu', function (e) {
        if (e.defaultPrevented) {
            return;
        }

        if (isEditable(e.target)) {
            return;
        }

        let anchor = null;
        try {
            anchor = e.composedPath().find(function (el) {
                return el instanceof HTMLAnchorElement && el.href;
            });
        } catch (_) {
            anchor = null;
        }

        const href = anchor ? anchor.href : null;
        const selection = window.getSelection ? window.getSelection().toString().trim() : '';

        function tauriInvoke(command, payload) {
            try {
                if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
                    return window.__TAURI_INTERNALS__.invoke(command, payload || {});
                }
                if (window.__TAURI__ && typeof window.__TAURI__.invoke === 'function') {
                    return window.__TAURI__.invoke(command, payload || {});
                }
                if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === 'function') {
                    return window.__TAURI__.tauri.invoke(command, payload || {});
                }
                if (typeof window.__TAURI_INVOKE__ === 'function') {
                    return window.__TAURI_INVOKE__(command, payload || {});
                }
            } catch (err) {
                console.warn('Tauri invoke failed:', command, err);
            }
            console.warn('Tauri invoke unavailable:', command);
            return Promise.reject(new Error('not in Tauri'));
        }

        try {
            tauriInvoke(COMMAND, {
                x: Math.floor(e.clientX),
                y: Math.floor(e.clientY),
                href: href,
                selection: selection || null,
            });
        } catch (_) {
            // If the native command is unavailable, allow the page to handle the event.
        }
    }, true);
}());
"#
        .to_string()
}

pub fn init_context_menu(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    if let Some(window) = app.get_webview_window("main") {
        window.on_menu_event(move |window, event| match event.id.as_ref() {
            "context-menu-open-link" => {
                if let Some(payload) = take_last_payload() {
                    if let Some(href) = payload.href {
                        let _ = window.app_handle().opener().open_url(href, None::<String>);
                    }
                }
            }
            _ => {}
        });
    }
    Ok(())
}

pub fn show_native_context_menu(
    app: &AppHandle,
    payload: ContextMenuPayload,
) -> Result<(), Box<dyn Error>> {
    set_last_payload(payload.clone());

    let window = app
        .get_webview_window("main")
        .ok_or("main window not available")?;

    let mut menu_items = Vec::new();

    if payload.href.is_some() {
        menu_items.push(MenuItem::with_id(
            app,
            "context-menu-open-link",
            "Open Link in Browser",
            true,
            None::<&str>,
        )?);
    }

    if menu_items.is_empty() {
        return Ok(());
    }

    let menu_refs: Vec<&dyn IsMenuItem<_>> = menu_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<_>)
        .collect();
    let menu = Menu::with_items(app, &menu_refs)?;
    window.popup_menu_at(&menu, LogicalPosition::new(payload.x, payload.y))?;
    Ok(())
}
