mod commands;
mod config;
mod media;
mod report;
mod window;

pub use commands::{cmd_get_status, cmd_start_watcher, cmd_stop_watcher};

use serde::Serialize;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use window::SessionKind;

#[derive(Clone, Default, Serialize)]
pub struct Snapshot {
    pub running: bool,
    pub session: SessionKind,
    pub last_process: Option<String>,
    pub last_media_title: Option<String>,
    pub last_media_artist: Option<String>,
    pub last_report_at: Option<String>,
    pub config_path: Option<String>,
}

pub struct AppStateInner {
    pub handle: Mutex<Option<JoinHandle<()>>>,
    pub snapshot: Mutex<Snapshot>,
}

impl Default for AppStateInner {
    fn default() -> Self {
        Self {
            handle: Mutex::new(None),
            snapshot: Mutex::new(Snapshot::default()),
        }
    }
}

#[derive(Default)]
pub struct AppState(pub Arc<AppStateInner>);

impl AppState {
    pub fn new() -> Self {
        Self(Arc::new(AppStateInner::default()))
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::cmd_start_watcher,
            commands::cmd_stop_watcher,
            commands::cmd_get_status,
            commands::cmd_read_config,
            commands::cmd_write_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
