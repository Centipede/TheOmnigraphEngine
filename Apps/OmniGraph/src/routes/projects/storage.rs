use std::fs;
use std::path::PathBuf;
use axum::http::StatusCode;
use axum::Json;
use secrecy::Secret;
use crate::app_settings::OcrServerConfig;
use crate::routes::projects::forms::{OcrServerData, SettingsResponse, SettingsUpdate};
use crate::routes::projects::models::{Page, PageDb, Project};
use crate::state::AppState;

pub fn is_valid_machine_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

pub fn create_project_on_disk(state: &AppState, name: &str, machine_name: &str) -> std::io::Result<()> {
    let project_dir = state.projects_dir.join(machine_name);
    fs::create_dir_all(project_dir.join("metadata"))?;
    fs::create_dir_all(project_dir.join("pages"))?;

    let project = Project {
        name: name.to_string(),
        machine_name: machine_name.to_string(),
        abbrev: None,
        description: None,
        authors: vec![],
        published: None,
    };

    let toml_str = toml::to_string(&project)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(project_dir.join("metadata").join("project.toml"), toml_str)
}

pub fn load_page_db(path: &std::path::Path) -> PageDb {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_page_db(path: &std::path::Path, db: &PageDb) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(db)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(path, json)
}

pub fn add_page(db: &mut PageDb, scan: String, scan_width: u32, scan_height: u32, thumb: String, thumb_width: u32, thumb_height: u32, batch: u32, import_order: u32) {
    db.pages.push(Page { index: 0, name: String::new(), scan, scan_width, scan_height, thumb, thumb_width, thumb_height, batch, import_order, ..Page::default() });
}

pub fn remove_page(db: &mut PageDb, index: usize) {
    db.pages.retain(|p| p.index != index);
    reindex(db);
}

pub fn assign_name(db: &mut PageDb, index: usize, name: String) {
    if let Some(page) = db.pages.iter_mut().find(|p| p.index == index) {
        page.name = name;
    }
}

// Reassign consecutive indices from current Vec order. Call after any mutation.
pub fn reindex(db: &mut PageDb) {
    for (i, page) in db.pages.iter_mut().enumerate() {
        page.index = i;
    }
}

// Reset display order to the original import sequence, then reindex.
pub fn sort_by_import_order(db: &mut PageDb) {
    db.pages.sort_by(|a, b| a.import_order.cmp(&b.import_order).then_with(|| a.scan.cmp(&b.scan)));
    reindex(db);
}

pub fn read_settings(state: &AppState) -> SettingsResponse {
    let secrets = state.secrets.read().unwrap();
    let settings = state.settings.read().unwrap();
    let ocr_status = state.ocr_status.read().unwrap();

    SettingsResponse {
        openai_api_key_set: secrets.openai_is_set(),
        perplexity_api_key_set: secrets.perplexity_is_set(),
        ocr_server_1: settings.ocr_server_1.as_ref().map(|s| OcrServerData { host: s.host.clone(), port: s.port }),
        ocr_server_2: settings.ocr_server_2.as_ref().map(|s| OcrServerData { host: s.host.clone(), port: s.port }),
        ocr_command_format: settings.ocr_command_format.clone(),
        ocr_server_1_status: ocr_status.server_1.clone(),
        ocr_server_2_status: ocr_status.server_2.clone(),
    }
}

pub fn write_settings(state: &AppState, payload: &SettingsUpdate) -> std::io::Result<()> {
    {
        let mut secrets = state.secrets.write().unwrap();

        if let Some(openai_api_key) = payload.openai_api_key.as_deref() {
            let openai_api_key = openai_api_key.trim();
            if !openai_api_key.is_empty() {
                secrets.openai_api_key = Some(Secret::new(openai_api_key.to_string()));
            }
        }

        if let Some(perplexity_api_key) = payload.perplexity_api_key.as_deref() {
            let perplexity_api_key = perplexity_api_key.trim();
            if !perplexity_api_key.is_empty() {
                secrets.perplexity_api_key = Some(Secret::new(perplexity_api_key.to_string()));
            }
        }

        crate::secrets::save(&secrets)?;
    }

    if let Some(ocr) = &payload.ocr {
        let mut settings = state.settings.write().unwrap();
        settings.ocr_server_1 = ocr.server_1.as_ref().map(|s| OcrServerConfig { host: s.host.clone(), port: s.port });
        settings.ocr_server_2 = ocr.server_2.as_ref().map(|s| OcrServerConfig { host: s.host.clone(), port: s.port });
        settings.ocr_command_format = ocr.command_format.clone();
        crate::app_settings::save(&settings)?;
    }

    Ok(())
}

pub fn read_projects(state: &AppState) -> Vec<Project> {
    let Ok(entries) = fs::read_dir(state.projects_dir.as_ref()) else {
        return vec![];
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let toml_path = e.path().join("metadata").join("project.toml");
            let contents = fs::read_to_string(toml_path).ok()?;
            toml::from_str(&contents).ok()
        })
        .collect()
}

pub fn read_project(projects_dir: &PathBuf, machine_name: &str) -> Result<Project, StatusCode> {
    let toml_path = projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    Ok(project)
}

pub fn write_project(projects_dir: &PathBuf, machine_name: &str, project: &Project) -> Result<(), StatusCode> {
    let toml_path = projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");

    fs::create_dir_all(toml_path.parent().unwrap())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let toml_str = toml::to_string(&project)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    fs::write(&toml_path, toml_str)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}