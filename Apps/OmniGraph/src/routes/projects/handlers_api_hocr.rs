use crate::hocr_parser;
use crate::hocr_parser::{HocrBlockKind, HocrPage, HocrPath};
use crate::routes::projects::forms::{AddRequest, ChangeTypeRequest, MergeRequest, SplitRequest};
use crate::routes::projects::handlers_api::get_hocr_json;
use crate::routes::projects::storage;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::path::PathBuf;

// ── HELPERS ──────────────────────────────────────────────────────────

fn save_and_report(
    page: &HocrPage,
    projects_dir: &PathBuf,
    machine_name: &str,
    stem: &str,
) -> impl IntoResponse {
    let html = page.to_hocr_html();
    if let Err(_e) = storage::save_hocr_edited(&projects_dir, &machine_name, &stem, &html) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    return Json(page).into_response();
}

// ── CAREA ────────────────────────────────────────────────────────────

pub async fn carea_merge(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<MergeRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path1 = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let path2 = hocr_parser::find_node(&page, &payload.other_id).unwrap(); // TODO: No unwrap!
    let HocrPath::Carea { carea: carea1 } = path1 else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Carea { carea: carea2 } = path2 else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.merge_carea(carea1, carea2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn carea_split(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<SplitRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let before_id = payload.before_id.clone();
    let after_id = payload.after_id.clone();
    let carea_path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let before = hocr_parser::find_node(&page, &before_id).unwrap(); // TODO: No unwrap!
    let after = hocr_parser::find_node(&page, &after_id).unwrap(); // TODO: No unwrap!

    let HocrPath::Carea { carea } = carea_path else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Block {
        carea: carea1,
        block: block1,
    } = before
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Block {
        carea: carea2,
        block: block2,
    } = after
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if !(carea == carea1 && carea1 == carea2) {
        return (
            StatusCode::BAD_REQUEST,
            "blocks must belong to the same carea",
        )
            .into_response();
    }

    if !block1 == block2 - 1 {
        return (
            StatusCode::BAD_REQUEST,
            "Can only split blocks that are adjacent to each other.",
        )
            .into_response();
    }

    page.split_carea(carea, block1, block2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn carea_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn carea_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn carea_add(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<AddRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };

    page.add_carea(payload.bbox);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn carea_remove(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Carea { carea } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.remove_carea(carea);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

// ── BLOCK ────────────────────────────────────────────────────────────

pub async fn block_merge(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<MergeRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path1 = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let path2 = hocr_parser::find_node(&page, &payload.other_id).unwrap(); // TODO: No unwrap!
    let HocrPath::Block {
        carea: carea1,
        block: block1,
    } = path1
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Block {
        carea: carea2,
        block: block2,
    } = path2
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if carea1 != carea2 {
        return StatusCode::BAD_REQUEST.into_response();
    }

    page.merge_block(carea1, block1, block2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_split(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<SplitRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let before_id = payload.before_id.clone();
    let after_id = payload.after_id.clone();
    let block_path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let before = hocr_parser::find_node(&page, &before_id).unwrap(); // TODO: No unwrap!
    let after = hocr_parser::find_node(&page, &after_id).unwrap(); // TODO: No unwrap!

    let HocrPath::Block { carea, block } = block_path else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Line {
        carea: carea1,
        block: block1,
        line: line1,
    } = before
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Line {
        carea: carea2,
        block: block2,
        line: line2,
    } = after
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !(carea == carea1 && carea1 == carea2 && block == block1 && block1 == block2) {
        return (
            StatusCode::BAD_REQUEST,
            "lines must belong to the same block",
        )
            .into_response();
    }

    if !line1 == line2 - 1 {
        return (
            StatusCode::BAD_REQUEST,
            "Can only split lines that are adjacent to each other.",
        )
            .into_response();
    }

    page.split_block(carea1, block1, line1, line2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_move_up(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap();
    let HocrPath::Block { carea, block } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_block_up(carea, block);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_move_down(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Block { carea, block } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_block_down(carea, block);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_add(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<AddRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let Some(to_carea) = payload.to_carea else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(HocrPath::Carea { carea }) = hocr_parser::find_node(&page, &to_carea) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.add_block(carea, payload.bbox);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_remove(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Block { carea, block } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.remove_block(carea, block);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn block_change_type(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<ChangeTypeRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };

    let Some(HocrPath::Block { carea, block }) = hocr_parser::find_node(&page, &id) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some(kind) = HocrBlockKind::from_json_name(&payload.kind) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    page.change_block_kind(carea, block, kind);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

// ── LINE ─────────────────────────────────────────────────────────────

pub async fn line_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn line_move_up(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Line { carea, block, line } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_line_up(carea, block, line);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn line_move_down(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap();
    let HocrPath::Line { carea, block, line } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_line_down(carea, block, line);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn line_add(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<AddRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let Some(to_block) = payload.to_block else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(HocrPath::Block { carea, block }) = hocr_parser::find_node(&page, &to_block) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.add_line(carea, block, payload.bbox);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn line_remove(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Line { carea, block, line } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.remove_line(carea, block, line);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

// ── WORD ─────────────────────────────────────────────────────────────

pub async fn word_merge(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
    Json(_payload): Json<MergeRequest>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn word_move_up(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn word_move_down(
    State(_state): State<AppState>,
    Path((_machine_name, _stem, _id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn word_add(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<AddRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let Some(to_line) = payload.to_block else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(HocrPath::Line { carea, block, line }) = hocr_parser::find_node(&page, &to_line)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.add_word(carea, block, line, payload.bbox, payload.text);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

pub async fn word_remove(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap(); // TODO: No unwrap!
    let HocrPath::Word {
        carea,
        block,
        line,
        word,
    } = path
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.remove_word(carea, block, line, word);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem).into_response()
}

// ── TOOLS ────────────────────────────────────────────────────────────

pub async fn parse_page(
    projects_dir: &PathBuf,
    machine_name: &str,
    stem: &str,
) -> Result<HocrPage, StatusCode> {
    let Some(hocr_path) = storage::hocr_active_path(&projects_dir, &machine_name, &stem) else {
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

pub async fn restore_from_original(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
) -> impl IntoResponse {
    let edited_path = storage::hocr_edited_path(&state.projects_dir, &machine_name, &stem);
    let original_path = storage::hocr_original_path(&state.projects_dir, &machine_name, &stem);

    if !edited_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Err(_e) = std::fs::copy(&original_path, &edited_path) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    get_hocr_json(State(state), Path((machine_name, stem)))
        .await
        .into_response()
}
