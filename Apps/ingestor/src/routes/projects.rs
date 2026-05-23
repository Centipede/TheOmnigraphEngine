use axum::{
    Form, Json,
    extract::State,
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

pub async fn project_page(State(_state): State<AppState>) -> impl IntoResponse {
    StatusCode::OK
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
