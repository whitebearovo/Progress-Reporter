use chrono::Utc;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize)]
struct MediaPayload<'a> {
    title: &'a str,
    artist: &'a str,
}

#[derive(Serialize)]
struct ReportPayload<'a> {
    key: &'a str,
    process: &'a str,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<MediaPayload<'a>>,
}

pub async fn send_report(
    process_name: &str,
    media_title: &str,
    media_artist: &str,
    api_key: &str,
    api_url: &str,
) -> Result<(), Box<dyn Error>> {
    let payload = ReportPayload {
        key: api_key,
        process: process_name,
        timestamp: Utc::now().timestamp(),
        media: if media_title.is_empty() && media_artist.is_empty() {
            None
        } else {
            Some(MediaPayload {
                title: media_title,
                artist: media_artist,
            })
        },
    };

    let client = reqwest::Client::builder().build()?;
    let _ = client.post(api_url).json(&payload).send().await?.text().await?;

    Ok(())
}
