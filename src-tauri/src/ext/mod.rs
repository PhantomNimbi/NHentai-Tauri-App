pub mod api;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod context_menu;
pub mod cloudfare;
pub mod database;
pub mod tag_blacklist;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod downloads;
pub mod navigation;
pub mod universal_deep_link;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod webnotifications;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod global_shortcuts;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod tray;
