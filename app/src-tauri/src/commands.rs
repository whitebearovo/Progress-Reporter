use crate::config::Config;
use crate::media::{self, MediaMetadata};
use crate::report;
use crate::window::{active_window_process, detect_session, SessionKind};
use crate::{AppState, Snapshot};
use chrono::Utc;
use std::time::Duration;
use tauri::State;

#[tauri::command]
pub async fn cmd_start_watcher(state: State<'_, AppState>, config_path: Option<String>) -> Result<Snapshot, String> {
    let path = config_path.unwrap_or_else(|| ".env.process".to_string());
    let cfg = crate::config::load_config(&path).map_err(|e| e.to_string())?;

    let session = detect_session();
    let mut handle = state.0.handle.lock().await;
    if let Some(h) = handle.take() {
        h.abort();
    }

    let app_state = state.inner().clone();
    let cfg_clone = cfg.clone();
    let config_path_for_task = path.clone();
    *handle = Some(tokio::spawn(async move {
        run_loop(cfg_clone, app_state, config_path_for_task, session).await;
    }));

    let mut snap = state.0.snapshot.lock().await;
    snap.running = true;
    snap.session = session;
    snap.config_path = Some(path);
    Ok(snap.clone())
}

#[tauri::command]
pub async fn cmd_stop_watcher(state: State<'_, AppState>) -> Result<(), String> {
    let mut handle = state.0.handle.lock().await;
    if let Some(h) = handle.take() {
        h.abort();
    }
    let mut snap = state.0.snapshot.lock().await;
    snap.running = false;
    Ok(())
}

#[tauri::command]
pub async fn cmd_get_status(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let handle = state.0.handle.lock().await;
    let running = handle.is_some();
    drop(handle);

    let mut snap = state.0.snapshot.lock().await;
    snap.running = running;
    Ok(snap.clone())
}

#[tauri::command]
pub async fn cmd_read_config(config_path: Option<String>) -> Result<Config, String> {
    let path = config_path.unwrap_or_else(|| ".env.process".to_string());
    crate::config::load_config(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_write_config(config_path: Option<String>, cfg: Config) -> Result<(), String> {
    let path = config_path.unwrap_or_else(|| ".env.process".to_string());
    crate::config::save_config(&path, &cfg).map_err(|e| e.to_string())
}

async fn run_loop(cfg: Config, state: AppState, config_path: String, session: SessionKind) {
    let mut last_process = String::new();
    let mut last_media = MediaMetadata::default();
    let mut last_time = Utc::now();

    loop {
        let process_name = active_window_process().unwrap_or_else(|_| "None".to_string());
        let media_metadata = if cfg.media_enable {
            media::get_media_metadata().unwrap_or_default()
        } else {
            MediaMetadata::default()
        };

        let changed = process_name != last_process
            || media_metadata != last_media
            || (Utc::now() - last_time).num_seconds() > 20;

        if changed {
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
