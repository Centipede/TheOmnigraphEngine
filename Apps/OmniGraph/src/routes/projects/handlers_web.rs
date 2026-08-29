
use crate::state::AppState;
use axum::extract::{Path, State, Query};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse};
use minijinja::context;
use std::fs;
use crate::routes::projects::models::CropEdges;
use crate::routes::projects::storage;

#[derive(serde::Deserialize)]
pub struct ImageQuery {
    pub processed: Option<bool>,
    pub crop: Option<String>,
}

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
    Query(query): Query<ImageQuery>,
) -> impl IntoResponse {
    let path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("thumbs")
        .join(&filename);
    match fs::read(&path) {
        Ok(data) => {
            if query.processed.unwrap_or(false) {
                let crop = parse_crop(&query.crop);
                let project = match storage::read_project(&state.projects_dir, &machine_name) {
                    Ok(p) => p,
                    Err(e) => return e.into_response(),
                };
                let pagedb = storage::load_page_db(&state.project_pagesdb_path(&machine_name));
                let hints = pagedb.pages.iter()
                    .find(|p| p.thumb == filename)
                    .map(|p| p.hints.as_slice())
                    .unwrap_or(&[]);

                match crate::image_utils::apply_image_pipeline(&data, crop, project.processing.as_ref(), hints) {
                    Ok(processed) => {
                        let mime = mime_guess::from_path(&filename).first_or_octet_stream();
                        ([(header::CONTENT_TYPE, mime.as_ref().to_string())], processed).into_response()
                    }
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
                }
            } else {
                let mime = mime_guess::from_path(&filename).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref().to_string())], data).into_response()
            }
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn serve_scan(
    State(state): State<AppState>,
    Path((machine_name, filename)): Path<(String, String)>,
    Query(query): Query<ImageQuery>,
) -> impl IntoResponse {
    let path = state
        .projects_dir
        .join(&machine_name)
        .join("pages")
        .join("scans")
        .join(&filename);
    match fs::read(&path) {
        Ok(data) => {
            if query.processed.unwrap_or(false) {
                let crop = parse_crop(&query.crop);
                let project = match storage::read_project(&state.projects_dir, &machine_name) {
                    Ok(p) => p,
                    Err(e) => return e.into_response(),
                };
                let pagedb = storage::load_page_db(&state.project_pagesdb_path(&machine_name));
                let hints = pagedb.pages.iter()
                    .find(|p| p.scan == filename)
                    .map(|p| p.hints.as_slice())
                    .unwrap_or(&[]);

                match crate::image_utils::apply_image_pipeline(&data, crop, project.processing.as_ref(), hints) {
                    Ok(processed) => {
                        let mime = mime_guess::from_path(&filename).first_or_octet_stream();
                        ([(header::CONTENT_TYPE, mime.as_ref().to_string())], processed).into_response()
                    }
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
                }
            } else {
                let mime = mime_guess::from_path(&filename).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref().to_string())], data).into_response()
            }
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn parse_crop(crop_str: &Option<String>) -> Option<CropEdges> {
    crop_str.as_ref().and_then(|c| {
        let parts: Vec<u32> = c.split(',').filter_map(|s| s.parse().ok()).collect();
        if parts.len() == 4 {
            Some(CropEdges {
                left: parts[0],
                top: parts[1],
                right: parts[2],
                bottom: parts[3],
            })
        } else {
            None
        }
    })
}
