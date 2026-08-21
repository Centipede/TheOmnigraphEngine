use crate::hocr_parser;
use crate::hocr_parser::{HocrBlockKind, HocrPage, HocrPath};
use crate::routes::projects::forms::{AddRequest, ChangeTypeBulkRequest, ChangeTypeRequest, MergeItemsRequest, MergeRequest, SplitRequest, RescanRequest};
use crate::routes::projects::handlers_api::get_hocr_json;
use crate::routes::projects::storage;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::path::PathBuf;
use crate::ocr_poll::ServerStatus;
use serde::Serialize;
// ── HELPERS ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct HocrUpdateResponse<'a> {
    page: &'a HocrPage,
    new_id: Option<String>,
}

fn save_and_report(
    page: &HocrPage,
    projects_dir: &PathBuf,
    machine_name: &str,
    stem: &str,
    new_id: Option<String>,
) -> impl IntoResponse {
    let html = page.to_hocr_html();
    if let Err(_e) = storage::save_hocr_edited(&projects_dir, &machine_name, &stem, &html) {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    return Json(HocrUpdateResponse { page, new_id }).into_response();
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn careas_merge(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<MergeItemsRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let paths = payload.item_ids.iter().map(|id| hocr_parser::find_node(&page, id)).collect::<Vec<_>>();

    if paths.iter().any(|path| !matches!(path, Some(HocrPath::Carea { .. }))) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let mut careas = paths.iter().map(|path| if let Some(HocrPath::Carea { carea }) = path { *carea } else { unreachable!() }).collect::<Vec<usize>>();

    match page.merge_careas(&mut careas) {
        Ok(()) => {}
        Err(err) => return (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn carea_move_up(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap();
    let HocrPath::Carea { carea } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_carea_up(carea);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn carea_move_down(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path = hocr_parser::find_node(&page, &id).unwrap();
    let HocrPath::Carea { carea } = path else {
        return StatusCode::NOT_FOUND.into_response();
    };

    page.move_carea_down(carea);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    let new_id = match page.add_carea(payload.bbox, payload.erase_underneath, payload.erase_overlap) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, new_id).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn carea_rescan(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<RescanRequest>,
) -> impl IntoResponse {
    // 1. Parse current page hOCR
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };

    // 2. Find the selected carea by ID
    let carea_index = match hocr_parser::find_node(&page, &id) {
        Some(HocrPath::Carea { carea }) => carea,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let carea_bbox = page.careas[carea_index].bbox;
    let dx = carea_bbox.left().max(0);
    let dy = carea_bbox.top().max(0);

    // 3. Identify child image blocks within that carea
    let image_blocks: Vec<hocr_parser::HocrBbox> = page.careas[carea_index].blocks.iter()
        .filter(|b| b.kind == HocrBlockKind::Image)
        .map(|b| b.bbox)
        .collect();

    // 4. Load the page image from disk
    let pages_db_path = state.projects_dir.join(&machine_name).join("pages").join("pagedata.json");
    let pages_db = storage::load_page_db(&pages_db_path);
    let page_meta = match pages_db.pages.iter().find(|p| {
        println!("{:?}", std::path::Path::new(&p.scan).file_stem());
        std::path::Path::new(&p.scan).file_stem().and_then(|s| s.to_str()) == Some(&stem)
    }) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let scan_path = state.projects_dir.join(&machine_name).join("pages").join("scans").join(&page_meta.scan);
    let img_bytes = match tokio::fs::read(&scan_path).await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 5. Get page CropEdges
    let crop_edges = page_meta.crop_edges;

    // 6. Call image_utils::extract_and_process_carea_image (with 50px padding)
    let processed_bytes = match crate::image_utils::extract_and_process_carea_image(&img_bytes, carea_bbox, crop_edges, &image_blocks, 50) {
        Ok(b) => b,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 7. Send the resulting image bytes to ocr_client::ocr_image_hocr
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
    let Some(server_config) = server else {
        return (StatusCode::SERVICE_UNAVAILABLE, "OCR server not configured").into_response();
    };

    let ocr_results = match crate::ocr_client::call_ocr_service(&server_config, vec![("rescan.png".to_string(), processed_bytes)], &payload.language).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let hocr_text = match ocr_results.into_iter().next() {
        Some(r) => match r.hocr {
            Some(h) => h,
            None => return (StatusCode::INTERNAL_SERVER_ERROR, r.error.unwrap_or("OCR failed".to_string())).into_response(),
        },
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "OCR failed").into_response(),
    };

    // 8. Parse the returned hOCR as a HocrPage
    let new_page = match tokio::task::spawn_blocking(move || crate::hocr_parser::parse(&hocr_text)).await.unwrap_or(None) {
        Some(p) => p,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse OCR result").into_response(),
    };

    // 9. Shift the coordinates of the new OCR results by (+dx - 50, +dy - 50)
    let mut new_careas = new_page.careas;
    for nc in &mut new_careas {
        nc.shift(dx - 50, dy - 50);
    }

    // 10. Merge the results back into the original page
    page.replace_or_merge_carea(carea_index, new_careas);

    // 11. Save the updated hOCR and return the page JSON
    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn carea_change_flow_bulk(
    State(_state): State<AppState>,
    Path((_machine_name, _stem)): Path<(String, String)>,
    Json(_payload): Json<ChangeTypeBulkRequest>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn carea_change_layout_bulk(
    State(_state): State<AppState>,
    Path((_machine_name, _stem)): Path<(String, String)>,
    Json(_payload): Json<ChangeTypeBulkRequest>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn blocks_merge(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<MergeItemsRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let paths = payload.item_ids.iter().map(|id| hocr_parser::find_node(&page, id)).collect::<Vec<_>>();

    if paths.iter().any(|path| !matches!(path, Some(HocrPath::Block { .. }))) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let mut blocks = paths.iter().map(|path| if let Some(HocrPath::Block { carea,  block }) = path { (*carea, *block) } else { unreachable!() }).collect::<Vec<(usize, usize)>>();

    match page.merge_blocks(&mut blocks) {
        Ok(()) => {}
        Err(err) => return (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    let result = if let Some(to_carea) = payload.to_carea {
        if let Some(HocrPath::Carea { carea }) = hocr_parser::find_node(&page, &to_carea) {
            page.add_block(Some(carea), payload.bbox, payload.block_type, payload.shrink_wrap_carea, payload.erase_underneath, payload.erase_overlap)
        }
        else {
            return (StatusCode::NOT_FOUND, "to_carea not found").into_response();
        }
    }
    else {
        page.add_block(None, payload.bbox, payload.block_type, payload.shrink_wrap_carea, payload.erase_underneath, payload.erase_overlap)
    };

    let new_id = match result {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, new_id).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn block_change_type_bulk(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<ChangeTypeBulkRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };

    let paths = payload
        .item_ids
        .iter()
        .map(|id| hocr_parser::find_node(&page, id))
        .collect::<Vec<_>>();

    if paths
        .iter()
        .any(|path| !matches!(path, Some(HocrPath::Block { .. })))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "All items must be blocks"})),
        )
            .into_response();
    }

    let mut blocks = paths
        .iter()
        .map(|path| {
            if let Some(HocrPath::Block { carea, block }) = path {
                (*carea, *block)
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<(usize, usize)>>();

    let Some(kind) = HocrBlockKind::from_json_name(&payload.kind) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid block kind"})),
        )
            .into_response();
    };

    if let Err(err) = page.merge_blocks(&mut blocks) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response();
    }

    // After merge, they are all in the same carea and have been merged into one big block
    let (carea, block) = blocks[0];
    page.change_block_kind(carea, block, kind.clone());

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

// ── LINE ─────────────────────────────────────────────────────────────

pub async fn line_merge(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<MergeRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path1 = hocr_parser::find_node(&page, &id).unwrap();
    let path2 = hocr_parser::find_node(&page, &payload.other_id).unwrap();
    let HocrPath::Line {
        carea: carea1,
        block: block1,
        line: line1,
    } = path1
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Line {
        carea: carea2,
        block: block2,
        line: line2,
    } = path2
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if carea1 != carea2 || block1 != block2 {
        return StatusCode::BAD_REQUEST.into_response();
    }

    page.merge_line(carea1, block1, line1, line2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn lines_merge(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<MergeItemsRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let paths = payload
        .item_ids
        .iter()
        .map(|id| hocr_parser::find_node(&page, id))
        .collect::<Vec<_>>();

    if paths
        .iter()
        .any(|path| !matches!(path, Some(HocrPath::Line { .. })))
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let mut lines = paths
        .iter()
        .map(|path| {
            if let Some(HocrPath::Line {
                carea,
                block,
                line,
            }) = path
            {
                (*carea, *block, *line)
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<(usize, usize, usize)>>();

    match page.merge_lines(&mut lines) {
        Ok(()) => {}
        Err(err) => return (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

// ── WORD ─────────────────────────────────────────────────────────────

pub async fn word_merge(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<MergeRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let path1 = hocr_parser::find_node(&page, &id).unwrap();
    let path2 = hocr_parser::find_node(&page, &payload.other_id).unwrap();
    let HocrPath::Word {
        carea: carea1,
        block: block1,
        line: line1,
        word: word1,
    } = path1
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let HocrPath::Word {
        carea: carea2,
        block: block2,
        line: line2,
        word: word2,
    } = path2
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if carea1 != carea2 || block1 != block2 || line1 != line2 {
        return StatusCode::BAD_REQUEST.into_response();
    }

    page.merge_word(carea1, block1, line1, word1, word2);

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn words_merge(
    State(state): State<AppState>,
    Path((machine_name, stem)): Path<(String, String)>,
    Json(payload): Json<MergeItemsRequest>,
) -> impl IntoResponse {
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };
    let paths = payload
        .item_ids
        .iter()
        .map(|id| hocr_parser::find_node(&page, id))
        .collect::<Vec<_>>();

    if paths
        .iter()
        .any(|path| !matches!(path, Some(HocrPath::Word { .. })))
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let mut words = paths
        .iter()
        .map(|path| {
            if let Some(HocrPath::Word {
                carea,
                block,
                line,
                word,
            }) = path
            {
                (*carea, *block, *line, *word)
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<(usize, usize, usize, usize)>>();

    match page.merge_words(&mut words) {
        Ok(()) => {}
        Err(err) => return (StatusCode::BAD_REQUEST, Json(json!({"error": err}))).into_response(),
    }

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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

    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
}

pub async fn word_rescan(
    State(state): State<AppState>,
    Path((machine_name, stem, id)): Path<(String, String, String)>,
    Json(payload): Json<RescanRequest>,
) -> impl IntoResponse {
    // 1. Parse current page hOCR
    let mut page = match parse_page(&state.projects_dir, &machine_name, &stem).await {
        Ok(page) => page,
        Err(status_code) => return status_code.into_response(),
    };

    // 2. Find the selected word by ID
    let word_path = match hocr_parser::find_node(&page, &id) {
        Some(HocrPath::Word {
            carea,
            block,
            line,
            word,
        }) => HocrPath::Word {
            carea,
            block,
            line,
            word,
        },
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let HocrPath::Word {
        carea: c_idx,
        block: b_idx,
        line: l_idx,
        word: w_idx,
    } = word_path
    else {
        unreachable!()
    };
    let word_bbox = page.careas[c_idx].blocks[b_idx].lines[l_idx].words[w_idx].bbox;
    let dx = word_bbox.left().max(0);
    let dy = word_bbox.top().max(0);

    // 3. Load the page image from disk
    let pages_db_path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("pagedata.json");
    let pages_db = storage::load_page_db(&pages_db_path);
    let page_meta = match pages_db.pages.iter().find(|p| {
        std::path::Path::new(&p.scan)
            .file_stem()
            .and_then(|s| s.to_str())
            == Some(&stem)
    }) {
        Some(p) => p,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let scan_path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("scans")
        .join(&page_meta.scan);
    let img_bytes = match tokio::fs::read(&scan_path).await {
        Ok(b) => b,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // 4. Get page CropEdges
    let crop_edges = page_meta.crop_edges;

    // 5. Extract image segment using word_bbox (with 50px padding)
    // For word rescan, we don't have child image blocks to mask.
    let processed_bytes = match crate::image_utils::extract_and_process_carea_image(
        &img_bytes,
        word_bbox,
        crop_edges,
        &[],
        50,
    ) {
        Ok(b) => b,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let Ok(_) = tokio::fs::write("/tmp/testimage.jpg", &processed_bytes).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };


    // 6. Call OCR service
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
    let Some(server_config) = server else {
        return (StatusCode::SERVICE_UNAVAILABLE, "OCR server not configured").into_response();
    };

    let ocr_results = match crate::ocr_client::call_ocr_service(
        &server_config,
        vec![("rescan.png".to_string(), processed_bytes)],
        &payload.language,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let hocr_text = match ocr_results.into_iter().next() {
        Some(r) => match r.hocr {
            Some(h) => h,
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    r.error.unwrap_or("OCR failed".to_string()),
                )
                    .into_response()
            }
        },
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "OCR failed").into_response(),
    };

    // 7. Parse the returned hOCR as a HocrPage
    let new_page =
        match tokio::task::spawn_blocking(move || crate::hocr_parser::parse(&hocr_text))
            .await
            .unwrap_or(None)
        {
            Some(p) => p,
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse OCR result",
                )
                    .into_response()
            }
        };

    // 8. Harvest all words from new_page and shift them by (+dx - 50, +dy - 50)
    let mut new_words = new_page.collect_all_words();
    for nw in &mut new_words {
        nw.shift(dx - 50, dy - 50);
    }

    // 9. Replace the original word with the new words
    page.replace_words(&id, new_words);

    // 10. Save the updated hOCR and return the page JSON
    save_and_report(&page, &state.projects_dir, &machine_name, &stem, None).into_response()
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
        Some(mut p) => {
            p.page_id = stem.to_string();
            Ok(p)
        }
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
