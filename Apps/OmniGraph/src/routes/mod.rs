pub mod projects;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::{IntoResponse},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use projects::{handlers_api, handlers_web};
use crate::routes::projects::handlers_api_hocr;
use crate::state::AppState;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref().to_owned())], content.data).into_response()
        }
        None => match Assets::get("index.html") {
            Some(content) => {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref().to_owned())], content.data).into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/settings", get(handlers_api::settings_get).post(handlers_api::settings_post))
        .route("/settings/service/status", get(handlers_api::settings_service_status_get))
        .route("/projects", get(handlers_api::list_projects).post(handlers_api::create_project))
        .route("/projects/{machine_name}", get(handlers_api::get_project_metadata).put(handlers_api::put_project_metadata))
        .route("/projects/{machine_name}/structure", get(handlers_api::get_project_structure).put(handlers_api::put_project_structure))
        .route("/projects/{machine_name}/pages", get(handlers_api::get_project_pagesdb).put(handlers_api::put_project_pagesdb))
        .route("/projects/{machine_name}/pages/hocr-status", get(handlers_api::get_hocr_status))
        .route("/projects/{machine_name}/pages/{stem}/hocr-json", get(handlers_api::get_hocr_json))
        .route("/projects/{machine_name}/pages/{stem}/restore-original", post(handlers_api_hocr::restore_from_original))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/merge",           post(handlers_api_hocr::careas_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/change-flow",     post(handlers_api_hocr::carea_change_flow_bulk))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/change-layout",   post(handlers_api_hocr::carea_change_layout_bulk))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/add",             post(handlers_api_hocr::carea_add))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/merge",      post(handlers_api_hocr::carea_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/split",      post(handlers_api_hocr::carea_split))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/move-up",    post(handlers_api_hocr::carea_move_up))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/move-down",  post(handlers_api_hocr::carea_move_down))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/remove",     post(handlers_api_hocr::carea_remove))
        .route("/projects/{machine_name}/pages/{stem}/hocr/careas/{id}/rescan",     post(handlers_api_hocr::carea_rescan))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/add",             post(handlers_api_hocr::block_add))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/merge",           post(handlers_api_hocr::blocks_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/change-type",     post(handlers_api_hocr::block_change_type_bulk))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/merge",      post(handlers_api_hocr::block_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/split",      post(handlers_api_hocr::block_split))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/move-up",    post(handlers_api_hocr::block_move_up))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/move-down",  post(handlers_api_hocr::block_move_down))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/remove",     post(handlers_api_hocr::block_remove))
        .route("/projects/{machine_name}/pages/{stem}/hocr/blocks/{id}/change-type",post(handlers_api_hocr::block_change_type))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/add",              post(handlers_api_hocr::line_add))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/merge",            post(handlers_api_hocr::lines_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/{id}/merge",       post(handlers_api_hocr::line_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/{id}/move-up",     post(handlers_api_hocr::line_move_up))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/{id}/move-down",   post(handlers_api_hocr::line_move_down))
        .route("/projects/{machine_name}/pages/{stem}/hocr/lines/{id}/remove",      post(handlers_api_hocr::line_remove))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/add",              post(handlers_api_hocr::word_add))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/merge",            post(handlers_api_hocr::words_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/{id}/merge",       post(handlers_api_hocr::word_merge))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/{id}/move-up",     post(handlers_api_hocr::word_move_up))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/{id}/move-down",   post(handlers_api_hocr::word_move_down))
        .route("/projects/{machine_name}/pages/{stem}/hocr/words/{id}/remove",      post(handlers_api_hocr::word_remove))
        .route("/projects/{machine_name}/pages/scan", post(handlers_api::scan_pages_post))
        .route("/projects/{machine_name}/pages/append", post(handlers_api::post_append_images).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/pages/insert", post(handlers_api::post_append_images).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/pages/remove", post(handlers_api::post_remove_images))
        ;

    let projects = Router::new()
        .route("/", get(handlers_web::vue_app))
        .fallback(handlers_web::vue_app);

    Router::new()
        .route("/", get(handlers_web::vue_app))
        .route("/media/projects/{machine_name}/pages/thumbs/{filename}", get(handlers_web::serve_thumb))
        .route("/media/projects/{machine_name}/pages/scans/{filename}", get(handlers_web::serve_scan))
        .nest("/projects", projects)
        .nest("/api", api)
        .fallback(static_handler)
        .with_state(state)
}
