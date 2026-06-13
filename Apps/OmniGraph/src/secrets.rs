use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

/// In-memory secrets — fields wrapped in Secret to prevent accidental log leakage.
#[derive(Default)]
pub struct AppSecrets {
    pub openai_api_key: Option<Secret<String>>,
    pub perplexity_api_key: Option<Secret<String>>,
}

impl AppSecrets {
    pub fn openai_is_set(&self) -> bool {
        self.openai_api_key.is_some()
    }
    pub fn perplexity_is_set(&self) -> bool {
        self.perplexity_api_key.is_some()
    }
}

/// Plain struct used only for TOML serialisation — never stored in AppState.
#[derive(Deserialize, Serialize, Default)]
struct SecretsFile {
    #[serde(default)]
    openai_api_key: Option<String>,
    #[serde(default)]
    perplexity_api_key: Option<String>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omnigraph")
        .join("secrets.toml")
}

pub fn load() -> AppSecrets {
    let Ok(contents) = std::fs::read_to_string(config_path()) else {
        return AppSecrets::default();
    };
    let file: SecretsFile = toml::from_str(&contents).unwrap_or_default();
    AppSecrets {
        openai_api_key: non_empty(file.openai_api_key),
        perplexity_api_key: non_empty(file.perplexity_api_key),
    }
}

pub fn save(secrets: &AppSecrets) -> io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = SecretsFile {
        openai_api_key: secrets.openai_api_key.as_ref().map(|s| s.expose_secret().clone()),
        perplexity_api_key: secrets.perplexity_api_key.as_ref().map(|s| s.expose_secret().clone()),
    };
    let contents = toml::to_string(&file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    write_restricted(&path, &contents)
}

fn non_empty(v: Option<String>) -> Option<Secret<String>> {
    v.filter(|s| !s.trim().is_empty()).map(Secret::new)
}

#[cfg(unix)]
fn write_restricted(path: &PathBuf, contents: &str) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true).create(true).truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(contents.as_bytes())
}

#[cfg(not(unix))]
fn write_restricted(path: &PathBuf, contents: &str) -> io::Result<()> {
    std::fs::write(path, contents)
}
