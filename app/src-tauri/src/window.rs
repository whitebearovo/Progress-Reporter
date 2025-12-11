use std::env;
use std::process::Command;

use thiserror::Error;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::AtomEnum;
use x11rb::rust_connection::RustConnection;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum SessionKind {
    X11,
    WaylandKde,
    Unknown,
}

impl Default for SessionKind {
    fn default() -> Self {
        SessionKind::Unknown
    }
}

#[derive(Debug, Error)]
pub enum WindowError {
    #[error("X11 query failed: {0}")]
    X11(String),
    #[error("KDE Wayland query failed: {0}")]
    Wayland(String),
    #[error("No active window detected")]
    None,
}

pub fn detect_session() -> SessionKind {
    let session = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let is_kde = env::var("KDE_FULL_SESSION").is_ok() || env::var("DESKTOP_SESSION").unwrap_or_default().to_lowercase().contains("kde");

    match session.as_str() {
        "x11" | "xorg" => SessionKind::X11,
        "wayland" if is_kde => SessionKind::WaylandKde,
        _ => SessionKind::Unknown,
    }
}

pub fn active_window_process() -> Result<String, WindowError> {
    match detect_session() {
        SessionKind::X11 => x11_active_window().map_err(WindowError::X11).and_then(|opt| opt.ok_or(WindowError::None)),
        SessionKind::WaylandKde => kde_wayland_active_window().map_err(WindowError::Wayland).and_then(|opt| opt.ok_or(WindowError::None)),
        SessionKind::Unknown => Err(WindowError::None),
    }
}

fn x11_active_window() -> Result<Option<String>, String> {
    let (conn, screen_num) = RustConnection::connect(None).map_err(|e| e.to_string())?;
    let root = conn.setup().roots.get(screen_num).ok_or("missing root")?.root;

    let net_active = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .atom;

    let wm_class = conn
        .intern_atom(false, b"WM_CLASS")
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .atom;

    let prop = conn
        .get_property(false, root, net_active, AtomEnum::WINDOW.into(), 0, 1024)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;

    let window = match prop.value32().and_then(|mut v| v.next()) {
        Some(id) if id != 0 => id,
        _ => return Ok(None),
    };

    let wm_class_reply = conn
        .get_property(false, window, wm_class, AtomEnum::STRING.into(), 0, 1024)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;

    if wm_class_reply.value.is_empty() {
        return Ok(None);
    }

    let class = String::from_utf8_lossy(&wm_class_reply.value)
        .split('\0')
        .filter(|s| !s.is_empty())
        .last()
        .map(|s| s.to_string());

    Ok(class)
}

fn kde_wayland_active_window() -> Result<Option<String>, String> {
    let output = Command::new("qdbus")
        .arg("org.kde.KWin")
        .arg("/KWin")
        .arg("supportInformation")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("qdbus returned status {}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Active window class:") {
            return Ok(Some(rest.trim().to_string()));
        }
        if let Some(rest) = trimmed.strip_prefix("Active window resource class:") {
            return Ok(Some(rest.trim().to_string()));
        }
        if let Some(rest) = trimmed.strip_prefix("Active window resource name:") {
            return Ok(Some(rest.trim().to_string()));
        }
    }

    Ok(None)
}
