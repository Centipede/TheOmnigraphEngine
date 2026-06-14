pub mod settings;
pub mod projects;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post, put},
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
        .route("/projects", get(handlers_api::list_projects).post(handlers_web::create_project))
        .route("/projects/{machine_name}", get(handlers_api::get_project_metadata))
        .route("/projects/{machine_name}/pages", get(handlers_api::get_project_pagesdb).put(handlers_api::put_project_pagesdb))
    ;

    Router::new()
        .route("/", get(|| async { Redirect::to("/projects") }))
        .route("/projects", get(handlers_web::projects_page).post(handlers_web::create_project_form))
        .route("/projects/{machine_name}", get(handlers_web::project_overview_get))
        .route("/projects/{machine_name}/folios", get(handlers_web::folios_get))
        .route("/projects/{machine_name}/folios/crop", get(handlers_web::folios_crop_get))
        .route("/projects/{machine_name}/metadata", get(handlers_web::project_metadata_get).post(handlers_web::project_metadata_post))
        .route("/projects/{machine_name}/pages/thumbs/{filename}", get(handlers_web::serve_thumb))
        .route("/projects/{machine_name}/pages/scans/{filename}", get(handlers_web::serve_scan))
        .route("/projects/{machine_name}/ingestor", get(handlers_web::project_pages_get))
        .route("/projects/{machine_name}/ingestor/append", get(handlers_web::ingest_images_get).post(handlers_web::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/ingestor/insert", get(handlers_web::ingest_images_get).post(handlers_web::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/ingestor/remove", get(handlers_web::remove_images_get).post(handlers_web::remove_images_post))
        .route("/projects/{machine_name}/ingestor/rename", post(handlers_web::rename_pages_post))
        .route("/settings", get(settings::settings_get).post(settings::settings_post))
        .nest("/api", api)
        .fallback(static_handler)
        .with_state(state)
}
