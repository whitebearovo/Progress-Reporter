use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
    pub watch_time: u64,
    pub media_enable: bool,
    pub log_enable: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            api_key: String::new(),
            watch_time: 5,
            media_enable: true,
            log_enable: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found at {0}")]
    NotFound(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Failed to parse config: {0}")]
    Parse(String),
}

pub fn load_config(config_path: impl AsRef<Path>) -> Result<Config, Box<dyn Error>> {
    let path: PathBuf = config_path.as_ref().to_path_buf();
    if !path.exists() {
        return Err(Box::new(ConfigError::NotFound(path.display().to_string())));
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut cfg = Config::default();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            match k {
                "API_URL" => cfg.api_url = v.trim().to_string(),
                "API_KEY" => cfg.api_key = v.trim().to_string(),
                "WATCH_TIME" => cfg.watch_time = v.trim().parse::<u64>().unwrap_or(cfg.watch_time),
                "MEDIA_ENABLE" => cfg.media_enable = v.trim().parse::<bool>().unwrap_or(cfg.media_enable),
                "LOG_ENABLE" => cfg.log_enable = v.trim().parse::<bool>().unwrap_or(cfg.log_enable),
                _ => {}
            }
        }
    }

    if cfg.api_url.is_empty() {
        return Err(Box::new(ConfigError::MissingField("API_URL".into())));
    }
    if cfg.api_key.is_empty() {
        return Err(Box::new(ConfigError::MissingField("API_KEY".into())));
    }

    Ok(cfg)
}

pub fn save_config(config_path: impl AsRef<Path>, cfg: &Config) -> Result<(), Box<dyn Error>> {
    let path: PathBuf = config_path.as_ref().to_path_buf();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(&path)?;
    writeln!(file, "API_URL={}", cfg.api_url.trim())?;
    writeln!(file, "API_KEY={}", cfg.api_key.trim())?;
    writeln!(file, "WATCH_TIME={}", cfg.watch_time)?;
    writeln!(file, "MEDIA_ENABLE={}", cfg.media_enable)?;
    writeln!(file, "LOG_ENABLE={}", cfg.log_enable)?;
    Ok(())
}
