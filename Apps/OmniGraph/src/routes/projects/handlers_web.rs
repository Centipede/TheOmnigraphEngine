
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};
use minijinja::context;
use std::fs;

pub async fn vue_app(State(state): State<AppState>) -> impl IntoResponse {
    let env = state.templates.acquire_env().unwrap();
    let html = env
        .get_template("base-ui.html")
        .unwrap()
        .render(context! {})
        .unwrap();

    Html(html)
}

pub async fn serve_thumb(
    State(state): State<AppState>,
    Path((machine_name, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    let path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("thumbs")
        .join(&filename);
    match fs::read(&path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&filename).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref().to_string())], data).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn serve_scan(
    State(state): State<AppState>,
    Path((machine_name, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    let path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("scans")
        .join(&filename);
    match fs::read(&path) {
        Ok(data) => {
            let mime = mime_guess::from_path(&filename).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref().to_string())], data).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
