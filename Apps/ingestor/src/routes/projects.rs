use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::state::AppState;

mod optional_date {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::{Date, Month};

    pub fn serialize<S>(date: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&format!(
                "{:04}-{:02}-{:02}",
                date.year(),
                date.month() as u8,
                date.day()
            )),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(s) = Option::<String>::deserialize(deserializer)? else {
            return Ok(None);
        };

        let mut parts = s.splitn(3, '-');
        let year: i32 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing year"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let month: u8 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing month"))?
            .parse()
            .map_err(serde::de::Error::custom)?;
        let day: u8 = parts
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing day"))?
            .parse()
            .map_err(serde::de::Error::custom)?;

        let month = Month::try_from(month).map_err(serde::de::Error::custom)?;
        let date = Date::from_calendar_date(year, month, day).map_err(serde::de::Error::custom)?;

        Ok(Some(date))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub full_name: String,
    pub abbrev: String,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub machine_name: String,
    pub abbrev: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default, with = "optional_date")]
    pub published: Option<time::Date>,
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub machine_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Page {
    pub index: usize,
    pub name: String,
    pub scan: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PageDb {
    pub pages: Vec<Page>,
}

#[derive(Deserialize)]
pub struct MetadataForm {
    pub name: String,
    #[serde(default)]
    pub abbrev: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub published: String,
    #[serde(default)]
    pub author_names: Vec<String>,
    #[serde(default)]
    pub author_abbrevs: Vec<String>,
}


// HTML handlers

pub async fn projects_page(State(state): State<AppState>) -> impl IntoResponse {
    let projects = read_projects(&state);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/index.html").unwrap()
        .render(context! { projects }).unwrap();
    Html(html)
}

pub async fn create_project_form(
    State(state): State<AppState>,
    Form(payload): Form<CreateProject>,
) -> impl IntoResponse {
    let machine_name = payload.machine_name.trim().to_string();
    let name = payload.name.trim().to_string();

    if !is_valid_machine_name(&machine_name) || name.is_empty() {
        return Redirect::to("/projects").into_response();
    }

    let project_dir = state.projects_dir.join(&machine_name);
    if project_dir.exists() {
        return Redirect::to("/projects").into_response();
    }

    if let Err(_) = create_project_on_disk(&state, &name, &machine_name) {
        return Redirect::to("/projects").into_response();
    }

    Redirect::to("/projects").into_response()
}

pub async fn project_overview_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    println!("Published: {:?} --- ", project.published);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/show.html").unwrap()
        .render(context! { project }).unwrap();
    Html(html).into_response()
}

pub async fn project_metadata_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let published_str = project.published.map(|d| {
        format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
    });
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/metadata.html").unwrap()
        .render(context! { project, published_str }).unwrap();
    Html(html).into_response()
}

pub async fn project_metadata_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Form(form): Form<MetadataForm>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    if !toml_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let name = form.name.trim().to_string();
    if name.is_empty() {
        return Redirect::to(&format!("/projects/{}/metadata", machine_name)).into_response();
    }

    let authors: Vec<Author> = form.author_names.iter()
        .zip(form.author_abbrevs.iter())
        .filter(|(n, _)| !n.trim().is_empty())
        .map(|(n, a)| Author {
            full_name: n.trim().to_string(),
            abbrev: a.trim().to_string(),
        })
        .collect();

    let opt = |s: String| -> Option<String> {
        let s = s.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    };

    let project = Project {
        name,
        machine_name: machine_name.clone(),
        abbrev: opt(form.abbrev),
        description: opt(form.description),
        authors,
        published: opt(form.published).and_then(|s| parse_date(&s)),
    };

    let Ok(toml_str) = toml::to_string(&project) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if fs::write(&toml_path, toml_str).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to(&format!("/projects/{}/metadata", machine_name)).into_response()
}


pub async fn project_pages_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let pagedb_path = state.projects_dir
        .join(&machine_name)
        .join("pages")
        .join("pagedata.json");

    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let mut pagedb = load_page_db(&pagedb_path);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/pages.html").unwrap()
        .render(context! { project, pagedb }).unwrap();
    Html(html).into_response()
}


pub async fn ingest_images_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/ingest.html").unwrap()
        .render(context! { project }).unwrap();
    Html(html).into_response()
}

pub async fn ingest_images_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let pages_dir = state.projects_dir.join(&machine_name).join("pages");
    if !pages_dir.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let scans_dir = pages_dir.join("scans");
    if fs::create_dir_all(&scans_dir).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let pagedb_path = pages_dir.join("pagedata.json");
    let mut pagedb = load_page_db(&pagedb_path);

    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = match field.file_name() {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => continue,
        };
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if !matches!(ext.as_deref(), Some("jpg") | Some("jpeg") | Some("png") | Some("tif") | Some("tiff") | Some("webp")) {
            continue;
        }
        let Ok(data) = field.bytes().await else { continue; };
        let final_name = resolve_scan_filename(&scans_dir, &filename);
        if fs::write(scans_dir.join(&final_name), data).is_ok() {
            add_page(&mut pagedb, final_name);
        }
    }

    sort_and_reindex(&mut pagedb);
    let _ = save_page_db(&pagedb_path, &pagedb);

    Redirect::to(&format!("/projects/{}", machine_name)).into_response()
}

// JSON API handlers

pub async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    Json(read_projects(&state))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProject>,
) -> impl IntoResponse {
    let machine_name = payload.machine_name.trim().to_string();
    let name = payload.name.trim().to_string();

    if !is_valid_machine_name(&machine_name) || name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid name"}))).into_response();
    }

    let project_dir = state.projects_dir.join(&machine_name);
    if project_dir.exists() {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "project already exists"}))).into_response();
    }

    match create_project_on_disk(&state, &name, &machine_name) {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"machine_name": machine_name}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// Shared helpers

fn parse_date(s: &str) -> Option<time::Date> {
    let mut parts = s.splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    let month = time::Month::try_from(month).ok()?;
    time::Date::from_calendar_date(year, month, day).ok()
}

fn is_valid_machine_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

fn create_project_on_disk(state: &AppState, name: &str, machine_name: &str) -> std::io::Result<()> {
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

// Page database helpers

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

pub fn add_page(db: &mut PageDb, scan: String) {
    db.pages.push(Page { index: 0, name: String::new(), scan });
}

pub fn remove_page(db: &mut PageDb, index: usize) {
    db.pages.retain(|p| p.index != index);
    sort_and_reindex(db);
}

pub fn assign_name(db: &mut PageDb, index: usize, name: String) {
    if let Some(page) = db.pages.iter_mut().find(|p| p.index == index) {
        page.name = name;
    }
}

pub fn sort_and_reindex(db: &mut PageDb) {
    db.pages.sort_by(|a, b| a.scan.cmp(&b.scan));
    for (i, page) in db.pages.iter_mut().enumerate() {
        page.index = i;
    }
}

// If `filename` already exists in `scans_dir`, generate a new name that sorts
// immediately after it by appending 'b'..'z' before the extension.
fn resolve_scan_filename(scans_dir: &std::path::Path, filename: &str) -> String {
    if !scans_dir.join(filename).exists() {
        return filename.to_string();
    }
    let p = std::path::Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    let dot_ext = if ext.is_empty() { String::new() } else { format!(".{ext}") };
    for c in b'b'..=b'z' {
        let candidate = format!("{stem}{}{dot_ext}", c as char);
        if !scans_dir.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{stem}_dup{dot_ext}")
}

fn read_projects(state: &AppState) -> Vec<Project> {
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
