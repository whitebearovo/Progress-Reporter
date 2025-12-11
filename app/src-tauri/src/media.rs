use dbus::arg::RefArg;
use dbus::blocking::stdintf::org_freedesktop_dbus::{DBus, Properties};
use dbus::blocking::Connection;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
}

pub fn get_media_metadata() -> Option<MediaMetadata> {
    let connection = Connection::new_session().ok()?;
    let dbus_proxy = connection.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(1500),
    );
    let names: Vec<String> = dbus_proxy.list_names().ok()?;

    for name in names {
        if !name.starts_with("org.mpris.MediaPlayer2.") {
            continue;
        }

        let proxy_result = connection.with_proxy(name, "/org/mpris/MediaPlayer2", Duration::from_millis(1500));

        let metadata: std::collections::HashMap<String, dbus::arg::Variant<Box<dyn RefArg>>> =
            match proxy_result.get("org.mpris.MediaPlayer2.Player", "Metadata") {
                Ok(data) => data,
                Err(_) => continue,
            };

        let title = metadata
            .get("xesam:title")
            .and_then(|val| val.as_str())
            .map(|s| s.to_string());

        let artist = if let Some(artist_variant) = metadata.get("xesam:artist") {
            match artist_variant {
                dbus::arg::Variant(boxed_value) => {
                    if let Some(artist_str) = boxed_value.as_str() {
                        Some(artist_str.to_string())
                    } else if let Some(artist_array) = boxed_value.as_iter() {
                        let artists: Vec<String> = artist_array.filter_map(|a| a.as_str().map(String::from)).collect();
                        if !artists.is_empty() {
                            Some(artists.join(", "))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };

        if title.is_some() || artist.is_some() {
            return Some(MediaMetadata { title, artist });
        }
    }

    None
}
