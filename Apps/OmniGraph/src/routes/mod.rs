pub mod settings;
pub mod projects;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use projects::{handlers_api, handlers_web};
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
        .route("/projects", get(handlers_api::list_projects).post(handlers_api::create_project))
        .route("/projects/{machine_name}", get(handlers_api::get_project_metadata).put(handlers_api::put_project_metadata))
        .route("/projects/{machine_name}/pages", get(handlers_api::get_project_pagesdb).put(handlers_api::put_project_pagesdb))
        .route("/projects/{machine_name}/pages/hocr-status", get(handlers_api::get_hocr_status))
        .route("/projects/{machine_name}/pages/{stem}/hocr-json", get(handlers_api::get_hocr_json))
        .route("/projects/{machine_name}/pages/scan", post(handlers_api::scan_pages_post))
        ;

    let projects = Router::new()
        .route("/", get(handlers_web::vue_app))
        .route("/{machine_name}/ingestor", get(handlers_web::project_pages_get))
        .route("/{machine_name}/ingestor/append", get(handlers_web::ingest_images_get).post(handlers_web::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/{machine_name}/ingestor/insert", get(handlers_web::ingest_images_get).post(handlers_web::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/{machine_name}/ingestor/remove", get(handlers_web::remove_images_get).post(handlers_web::remove_images_post))
        .route("/{machine_name}/ingestor/rename", post(handlers_web::rename_pages_post))
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
