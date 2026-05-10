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

#[derive(Serialize)]
pub struct Project {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
}

// HTML handlers

pub async fn projects_page(State(state): State<AppState>) -> impl IntoResponse {
    let projects = read_projects(&state);
    let html = state.templates
        .get_template("projects/index.html").unwrap()
        .render(context! { projects }).unwrap();
    Html(html)
}

pub async fn create_project_form(
    State(state): State<AppState>,
    Form(payload): Form<CreateProject>,
) -> impl IntoResponse {
    let name = payload.name.trim().to_string();
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Redirect::to("/projects").into_response();
    }
    let _ = fs::create_dir_all(state.projects_dir.join(&name));
    Redirect::to("/projects").into_response()
}

// JSON API handlers

pub async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    Json(read_projects(&state))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProject>,
) -> impl IntoResponse {
    let name = payload.name.trim().to_string();

    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid project name"}))).into_response();
    }

    let path = state.projects_dir.join(&name);
    if path.exists() {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "project already exists"}))).into_response();
    }

    match fs::create_dir_all(&path) {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"name": name}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// Shared helper

fn read_projects(state: &AppState) -> Vec<Project> {
    fs::read_dir(state.projects_dir.as_ref())
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| Project { name: e.file_name().to_string_lossy().into_owned() })
                .collect()
        })
        .unwrap_or_default()
}
