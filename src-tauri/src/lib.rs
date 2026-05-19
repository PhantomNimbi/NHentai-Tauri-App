mod ext;

use ext::database;
use ext::navigation;
use ext::api;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use ext::webnotifications;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(any(target_os = "android", target_os = "ios"))]
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(any(target_os = "android", target_os = "ios"))]
use url::Url;

pub fn build_init_script() -> String {
    let origin = "https://tauri.localhost";
    let nav = navigation::build_navigation_init(origin);
    let tag_bl = ext::tag_blacklist::build_tag_blacklist_script();
    let ctx = ext::context_menu::build_context_menu_script();
    format!("{}\n{}\n{}", nav, tag_bl, ctx)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
fn open_in_main_window_cmd(app: tauri::AppHandle, url: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let parsed = url::Url::parse(&url).map_err(|e| e.to_string())?;
        window.navigate(parsed).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
fn show_native_context_menu_cmd(
    app: tauri::AppHandle,
    payload: crate::ext::context_menu::ContextMenuPayload,
) -> Result<(), String> {
    crate::ext::context_menu::show_native_context_menu(&app, payload).map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn run() {
    let builder = tauri::Builder::default()
         .invoke_handler(tauri::generate_handler![
             open_in_main_window_cmd,
             show_native_context_menu_cmd,
             database::db_get_tags,
             database::db_upsert_tag,
             database::db_delete_tag,
             database::db_clear_tags,
             database::db_upsert_tags_bulk,
             database::db_get_history,
             database::db_add_history,
             database::db_clear_history,
             database::db_get_favorites,
             database::db_toggle_favorite,
             database::db_clear_favorites,
             database::db_get_setting,
             database::db_set_setting,
             database::db_get_snapshot,
             database::db_get_gallery_cache,
             database::db_get_gallery_cache_bulk,
             database::db_save_gallery_cache,
             database::db_get_search_cache,
             database::db_save_search_cache,
             api::api_all,
             api::api_search,
             api::api_gallery,
             api::api_random,
             api::api_favorites,
             api::api_user,
             api::api_blacklist,
             api::api_get_cdn_config,
             api::api_set_cookie,
             api::api_set_api_key,
             api::api_clear_api_key,
             api::api_key_is_set,
             api::api_populate_all_tags,
             api::api_galleries_tagged,
             api::api_galleries_popular,
             api::api_check_favorite,
             api::api_add_favorite,
             api::api_remove_favorite,
             api::api_get_download_url,
             api::api_download_gallery,
             api::api_tags_by_ids,
             api::api_search_tags,
             api::api_tags_by_type,
             api::api_gallery_comments,
             api::api_gallery_comment_count,
             api::api_gallery_related,
             api::api_favorites_random,
             api::api_user_keys,
             api::api_blacklist_ids,
            api::api_fetch_image_base64,
            webnotifications::native_notify,
        ])
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            database::init_database(app.handle())?;

            crate::ext::universal_deep_link::init_universal_deep_link(app.handle().clone())?;

            let window_builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App("index.html".into()),
            )
            .title("NHentai")
            .inner_size(1280.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .center();

            let window_builder = ext::cloudfare::configure_window(window_builder);

            let _window = window_builder
                .initialization_script(build_init_script())
                .on_download(|_window, event| {
                    crate::ext::downloads::handle_download_event(
                        _window.app_handle(),
                        "main",
                        event,
                    )
                })
                .build()?;

            ext::cloudfare::init(app.handle());

            let _ = crate::ext::context_menu::init_context_menu(&app.handle());
            let _ = crate::ext::downloads::init_downloads(&app.handle());
            let _ = crate::ext::global_shortcuts::init_global_shortcuts(&app.handle());
            let _ = crate::ext::webnotifications::init_webnotifications(&app.handle());
            crate::ext::tray::init_tray(&app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[tauri::mobile_entry_point]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            crate::ext::universal_deep_link::init_universal_deep_link(app.handle().clone())?;

            let site_url: Url = "https://nhentai.net/"
                .parse()
                .expect("hardcoded URL is valid");

            let window_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(site_url))
                .title("NHentai");
            let window_builder = ext::cloudfare::configure_window(window_builder);
            window_builder.build()?;

            ext::cloudfare::init(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
