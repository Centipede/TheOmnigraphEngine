use crate::hocr_parser::HocrPage;
use crate::routes::projects::storage;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use crate::hocr_editor::prepare_hocr_for_edit;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MergeRequest {
    pub other_id: String,
}

#[derive(Deserialize)]
pub struct SplitRequest {
    pub before_id: String,
    pub after_id: String,
}

// ── CAREA ────────────────────────────────────────────────────────────

pub async fn carea_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn carea_split(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<SplitRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn carea_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn carea_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn carea_remove(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

// ── BLOCK ────────────────────────────────────────────────────────────

pub async fn block_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn block_split(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<SplitRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn block_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn block_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn block_remove(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

// ── LINE ─────────────────────────────────────────────────────────────

pub async fn line_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn line_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn line_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn line_remove(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

// ── WORD ─────────────────────────────────────────────────────────────

pub async fn word_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn word_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn word_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn word_remove(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse { StatusCode::NOT_IMPLEMENTED }

pub async fn parse_page(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
) -> Result<HocrPage, StatusCode> {
    let Some(hocr_path) = storage::hocr_active_path(&state.projects_dir, &machine_name, &stem)
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let html = match tokio::fs::read_to_string(&hocr_path).await {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let page = tokio::task::spawn_blocking(move || crate::hocr_parser::parse(&html))
        .await
        .unwrap_or(None);

    match page {
        Some(p) => Ok(p),
        None => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}

pub async fn test_edit_page(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
) -> impl IntoResponse {


    let hocr_path = storage::hocr_active_path(&state.projects_dir, &machine_name, &stem).unwrap();
    prepare_hocr_for_edit(&hocr_path);



    let page = match parse_page(
        State(state.clone()),
        Path((machine_name.clone(), stem.clone())),
    )
    .await
    {
        Ok(page) => page,
        Err(status) => return status.into_response(),
    };

    let mut page = page.clone();
    let html = page.to_hocr_html();
    match storage::save_hocr_edited(&state.projects_dir, &machine_name, &stem, &html) {
        Ok(_) => (),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

    Json(page).into_response()
}
