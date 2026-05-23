use axum::{
    Form, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::state::AppState;

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
    pub published: Option<time::Date>,
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub machine_name: String,
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
    #[serde(default, deserialize_with = "str_or_vec")]
    pub author_names: Vec<String>,
    #[serde(default, deserialize_with = "str_or_vec")]
    pub author_abbrevs: Vec<String>,
}

// serde_urlencoded passes a bare string for a single occurrence of a key
// and a sequence for multiple occurrences. This deserializer handles both.
fn str_or_vec<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("string or sequence of strings")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(vec![v.to_owned()])
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = seq.next_element()? { out.push(v); }
            Ok(out)
        }
    }
    d.deserialize_any(V)
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

pub async fn project_page(
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
