use crate::ocr_poll::ServerStatus;
use crate::routes::projects::forms::{CreateProject, ScanConflict, ScanPageResult, ScanRequest, ScanResponse, SettingsUpdate};
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
            Json(storage::read_project(&state.projects_dir, &payload.machine_name).unwrap()),
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

pub async fn put_project_metadata(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Json(project): Json<crate::routes::projects::models::Project>,
) -> impl IntoResponse {
    match storage::write_project(&state.projects_dir, &machine_name, &project) {
        Ok(_) => (
            StatusCode::OK,
            Json(project),
        ).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn scan_pages_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    // Load pagedb and resolve the requested pages by index
    let pagedb_path = state.project_pagesdb_path(&machine_name);
    let pagedb = storage::load_page_db(&pagedb_path);

    let pages: Vec<_> = payload
        .indices
        .iter()
        .filter_map(|&idx| pagedb.pages.iter().find(|p| p.index == idx))
        .collect();

    if pages.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "no valid pages"}))).into_response();
    }

    // Conflict check: pages with unsaved edits block a forced re-scan
    if !payload.force {
        let conflicts: Vec<String> = pages
            .iter()
            .filter(|p| storage::has_unsaved_edits(&state.projects_dir, &machine_name, &p.scan))
            .map(|p| if p.name.is_empty() { p.scan.clone() } else { p.name.clone() })
            .collect();

        if !conflicts.is_empty() {
            return (StatusCode::CONFLICT, Json(ScanConflict { pages: conflicts })).into_response();
        }
    }

    // Pick the first online OCR server (priority 1 → 2)
    let server = {
        let settings = state.settings.read().unwrap();
        let ocr_status = state.ocr_status.read().unwrap();
        if matches!(ocr_status.server_1, ServerStatus::Online) {
            settings.ocr_server_1.clone()
        } else if matches!(ocr_status.server_2, ServerStatus::Online) {
            settings.ocr_server_2.clone()
        } else {
            None
        }
    };

    let Some(server) = server else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "no OCR server available"})),
        )
            .into_response();
    };

    // Read scan files from disk
    let scans_dir = state.projects_dir.join(&machine_name).join("pages").join("scans");
    let mut scan_files: Vec<(String, Vec<u8>)> = Vec::new();

    for page in &pages {
        match tokio::fs::read(scans_dir.join(&page.scan)).await {
            Ok(bytes) => scan_files.push((page.scan.clone(), bytes)),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("failed to read {}: {}", page.scan, e)})),
                )
                    .into_response();
            }
        }
    }

    // Call OCR service
    let ocr_results = match crate::ocr_client::call_ocr_service(&server, scan_files).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };

    // Save each hOCR result and build the response
    let mut results: Vec<ScanPageResult> = Vec::new();

    for ocr in ocr_results {
        let page = pages.iter().find(|p| p.scan == ocr.upload_name);
        let Some(page) = page else { continue };

        if let Some(hocr) = ocr.hocr {
            match storage::save_hocr_original(&state.projects_dir, &machine_name, &page.scan, &hocr) {
                Ok(()) => results.push(ScanPageResult { scan: page.scan.clone(), success: true, error: None }),
                Err(e) => results.push(ScanPageResult { scan: page.scan.clone(), success: false, error: Some(e.to_string()) }),
            }
        } else {
            results.push(ScanPageResult { scan: page.scan.clone(), success: false, error: ocr.error });
        }
    }

    Json(ScanResponse { results }).into_response()
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
