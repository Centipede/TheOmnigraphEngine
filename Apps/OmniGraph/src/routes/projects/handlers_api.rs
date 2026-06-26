use crate::routes::projects::forms::{CreateProject, SettingsUpdate};
use crate::routes::projects::models::PageDb;
use crate::routes::projects::{storage};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;



pub async fn settings_get(State(state): State<AppState>) -> impl IntoResponse {
    Json(storage::read_settings(&state))
}

pub async fn settings_post(
    State(state): State<AppState>,
    Json(payload): Json<SettingsUpdate>,
) -> impl IntoResponse {
    match storage::write_settings(&state, &payload) {
        Ok(()) => Json(storage::read_settings(&state)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

pub async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    Json(storage::read_projects(&state))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(mut payload): Json<CreateProject>,
) -> impl IntoResponse {
    payload.machine_name = payload.machine_name.trim().to_string();
    payload.name = payload.name.trim().to_string();

    if !storage::is_valid_machine_name(&payload.machine_name) || payload.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid name"})),
        )
            .into_response();
    }

    let project_dir = state.projects_dir.join(&payload.machine_name);
    if project_dir.exists() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "project already exists"})),
        )
            .into_response();
    }

    match storage::create_project_on_disk(&state, &payload.name, &payload.machine_name) {
        Ok(_) => (
            StatusCode::CREATED,
            Json(payload),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
pub async fn get_project_metadata(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let project = match storage::read_project(&state.projects_dir, &machine_name) {
        Ok(project) => project,
        Err(status) => return status.into_response(),
    };

    Json(project).into_response()
}

pub async fn post_project_metadata(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Json(project): Json<crate::routes::projects::models::Project>,
) -> impl IntoResponse {
    match storage::write_project(&state.projects_dir, &machine_name, &project) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_project_pagesdb(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let pagedb_path = state.project_pagesdb_path(&machine_name);
    let pagedb = storage::load_page_db(&pagedb_path);
    Json(pagedb).into_response()
}

pub async fn put_project_pagesdb(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Json(pagedb): Json<PageDb>,
) -> impl IntoResponse {
    let pagedb_path = state.project_pagesdb_path(&machine_name);
    if !pagedb_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    match storage::save_page_db(&pagedb_path, &pagedb) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
