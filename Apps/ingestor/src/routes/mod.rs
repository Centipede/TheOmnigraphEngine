pub mod settings;
pub mod projects;
pub mod pages;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use projects::handlers;
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
        .route("/projects", get(handlers::list_projects).post(handlers::create_project));

    Router::new()
        .route("/", get(|| async { Redirect::to("/projects") }))
        .route("/projects", get(handlers::projects_page).post(handlers::create_project_form))
        .route("/projects/{machine_name}", get(handlers::project_overview_get))
        .route("/projects/{machine_name}/metadata", get(handlers::project_metadata_get).post(handlers::project_metadata_post))
        .route("/projects/{machine_name}/pages", get(handlers::project_pages_get))
        .route("/projects/{machine_name}/pages/append", get(handlers::ingest_images_get).post(handlers::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/pages/insert", get(handlers::ingest_images_get).post(handlers::ingest_images_post).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/pages/remove", get(handlers::remove_images_get).post(handlers::remove_images_post).layer(DefaultBodyLimit::disable()))
        .route("/projects/{machine_name}/pages/thumbs/{filename}", get(handlers::serve_thumb))
        .route("/projects/{machine_name}/pages/scans/{filename}", get(handlers::serve_scan))
        .route("/settings", get(settings::settings_get).post(settings::settings_post))
        .nest("/api", api)
        .fallback(static_handler)
        .with_state(state)
}
