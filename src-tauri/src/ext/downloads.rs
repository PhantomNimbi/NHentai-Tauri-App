use std::path::{Path, PathBuf};

use tauri::webview::DownloadEvent;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Downloads handling (filename sanitization + collision handling + notifications).
///
/// The webview `on_download` hook lets us sanitize the destination path and
/// send a native notification when the download completes.
pub fn init_downloads(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn handle_download_event(app: &AppHandle, _window_label: &str, event: DownloadEvent) -> bool {
    match event {
        DownloadEvent::Requested { destination, .. } => {
            let filename = destination
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("download");
            let safe_filename = sanitise_filename(filename);
            let directory = destination.parent().unwrap_or_else(|| Path::new("."));
            let new_destination = unique_download_path(directory, &safe_filename);
            if let Some(parent) = new_destination.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            *destination = new_destination;
            true
        }
        DownloadEvent::Finished { path, success, .. } => {
            if success {
                if let Some(file_path) = path {
                    let name = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    let _ = app
                        .notification()
                        .builder()
                        .title("Download Complete")
                        .body(format!("{name} has been saved to your Downloads folder."))
                        .show();
                }
            } else if let Some(file_path) = path {
                let name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                let _ = app
                    .notification()
                    .builder()
                    .title("Download Failed")
                    .body(format!("{name} could not be saved."))
                    .show();
            }
            true
        }
        _ => true,
    }
}

fn sanitise_filename(raw: &str) -> String {
    let sanitised: String = raw
        .chars()
        .filter(|&c| c != '/' && c != '\\' && !c.is_ascii_control())
        .collect();
    let trimmed = sanitised.trim_start_matches(['.', ' ']);
    let capped = if trimmed.len() > 200 {
        &trimmed[..200]
    } else {
        trimmed
    };
    if capped.is_empty() {
        "download".to_string()
    } else {
        capped.to_string()
    }
}

fn unique_download_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    let ext = path.extension().and_then(|e| e.to_str());

    for n in 1u32..=999 {
        let new_name = match ext {
            Some(e) => format!("{stem} ({n}).{e}"),
            None => format!("{stem} ({n})"),
        };
        let new_path = dir.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
    }

    candidate
}
