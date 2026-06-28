use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub ocr_server_1: Option<OcrServerConfig>,
    #[serde(default)]
    pub ocr_server_2: Option<OcrServerConfig>,
    #[serde(default)]
    pub ocr_command_format: OcrCommandFormat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OcrServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OcrCommandFormat {
    #[default]
    Native,
    Docker,
}

pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omnigraph")
        .join("settings.toml")
}

pub fn load() -> AppSettings {
    let Ok(contents) = std::fs::read_to_string(settings_path()) else {
        return AppSettings::default();
    };
    toml::from_str(&contents).unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string(settings)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(path, contents)
}
