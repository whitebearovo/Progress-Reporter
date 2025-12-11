mod commands;
mod config;
mod media;
mod report;
mod window;

pub use commands::{cmd_get_status, cmd_start_watcher, cmd_stop_watcher};

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use window::SessionKind;

        let changed = process_name != last_process
            || media_metadata != last_media
            || (Utc::now() - last_time).num_seconds() > 20;
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![commands::cmd_start_watcher, commands::cmd_stop_watcher, commands::cmd_get_status])
            let _ = report::send_report(
                &process_name,
                media_metadata.title.as_deref().unwrap_or(""),
                media_metadata.artist.as_deref().unwrap_or(""),
                &cfg.api_key,
                &cfg.api_url,
            )
            .await;

            last_process = process_name.clone();
            last_media = media_metadata.clone();
            last_time = Utc::now();

            let mut snap = state.0.snapshot.lock().await;
            snap.running = true;
            snap.session = session;
            snap.config_path = Some(config_path.clone());
            snap.last_process = Some(process_name);
            snap.last_media_title = last_media.title.clone();
            snap.last_media_artist = last_media.artist.clone();
            snap.last_report_at = Some(last_time.format("%Y-%m-%d %H:%M:%S").to_string());
        }

        tokio::time::sleep(Duration::from_secs(cfg.watch_time.max(3))).await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![cmd_start_watcher, cmd_stop_watcher, cmd_get_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
