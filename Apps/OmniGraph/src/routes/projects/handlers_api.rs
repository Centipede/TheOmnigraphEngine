use crate::routes::projects::storage;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    Json(storage::read_projects(&state))
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

pub async fn get_project_pagesdb(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let pagedb_path = state.project_pagesdb_path(&machine_name);
    let pagedb = storage::load_page_db(&pagedb_path);

    Json(pagedb).into_response()
}
