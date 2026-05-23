pub mod pages;
pub mod projects;

use axum::{
    Router,
    routing::get,
    http::{StatusCode, header},
    response::{IntoResponse, Redirect},
};
use rust_embed::RustEmbed;
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
        .route("/projects", get(projects::list_projects).post(projects::create_project));

    Router::new()
        .route("/", get(|| async { Redirect::to("/projects") }))
        .route("/projects", get(projects::projects_page).post(projects::create_project_form))
        .route("/projects/:machine_name", get(projects::project_page))
        .route("/settings", get(pages::settings_get).post(pages::settings_post))
        .nest("/api", api)
        .fallback(static_handler)
        .with_state(state)
}
