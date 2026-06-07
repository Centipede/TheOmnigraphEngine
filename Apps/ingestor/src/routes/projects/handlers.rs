use std::fs;
use axum::extract::{Multipart, Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect};
use minijinja::context;
use axum_extra::extract::Form;
use axum::http::{header, StatusCode};
use axum::Json;
use crate::routes::projects;
use crate::routes::projects::forms::{CreateProject, IngestQuery, MetadataForm, RemoveForm, RemoveQuery};
use crate::routes::projects::models::{Author, Page, Project, IMPORT_ORDER_GAP};
use crate::routes::projects::{images, storage};
use crate::state::AppState;

pub async fn projects_page(State(state): State<AppState>) -> impl IntoResponse {
    let projects = storage::read_projects(&state);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/index.html").unwrap()
        .render(context! { projects }).unwrap();
    Html(html)
}

pub async fn create_project_form(
    State(state): State<AppState>,
    Form(payload): Form<CreateProject>,
) -> impl IntoResponse {
    let machine_name = payload.machine_name.trim().to_string();
    let name = payload.name.trim().to_string();

    if !storage::is_valid_machine_name(&machine_name) || name.is_empty() {
        return Redirect::to("/projects").into_response();
    }

    let project_dir = state.projects_dir.join(&machine_name);
    if project_dir.exists() {
        return Redirect::to("/projects").into_response();
    }

    if let Err(_) = storage::create_project_on_disk(&state, &name, &machine_name) {
        return Redirect::to("/projects").into_response();
    }

    Redirect::to("/projects").into_response()
}

pub async fn project_overview_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    println!("Published: {:?} --- ", project.published);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/show.html").unwrap()
        .render(context! { project }).unwrap();
    Html(html).into_response()
}

pub async fn project_metadata_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let published_str = project.published.map(|d| {
        format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
    });
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/metadata.html").unwrap()
        .render(context! { project, published_str }).unwrap();
    Html(html).into_response()
}

pub async fn project_metadata_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Form(form): Form<MetadataForm>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    if !toml_path.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let name = form.name.trim().to_string();
    if name.is_empty() {
        return Redirect::to(&format!("/projects/{}/metadata", machine_name)).into_response();
    }

    let authors: Vec<Author> = form.author_names.iter()
        .zip(form.author_abbrevs.iter())
        .filter(|(n, _)| !n.trim().is_empty())
        .map(|(n, a)| Author {
            full_name: n.trim().to_string(),
            abbrev: a.trim().to_string(),
        })
        .collect();

    let opt = |s: String| -> Option<String> {
        let s = s.trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    };

    let project = Project {
        name,
        machine_name: machine_name.clone(),
        abbrev: opt(form.abbrev),
        description: opt(form.description),
        authors,
        published: opt(form.published).and_then(|s| projects::parse_date(&s)),
    };

    let Ok(toml_str) = toml::to_string(&project) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if fs::write(&toml_path, toml_str).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to(&format!("/projects/{}/metadata", machine_name)).into_response()
}

pub async fn project_pages_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let pagedb_path = state.projects_dir
        .join(&machine_name)
        .join("pages")
        .join("pagedata.json");

    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let pagedb = storage::load_page_db(&pagedb_path);
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/pages.html").unwrap()
        .render(context! { project, pagedb }).unwrap();
    Html(html).into_response()
}

pub async fn ingest_images_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Query(query): Query<IngestQuery>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir
        .join(&machine_name)
        .join("metadata")
        .join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let is_insert = query.after.is_some() || query.before.is_some();
    let anchor_page = query.after.or(query.before);
    let anchor_page = anchor_page.and_then(|idx| {
        let db_path = state.projects_dir.join(&machine_name).join("pages").join("pagedata.json");
        storage::load_page_db(&db_path).pages.into_iter().find(|p| p.index == idx)
    });

    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/ingest.html").unwrap()
        .render(context! { project, is_insert, after_index => query.after, before_index => query.before, anchor_page }).unwrap();
    Html(html).into_response()
}

pub async fn ingest_images_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Query(query): Query<IngestQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let pages_dir = state.projects_dir.join(&machine_name).join("pages");
    if !pages_dir.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let scans_dir = pages_dir.join("scans");
    let thumbs_dir = pages_dir.join("thumbs");
    if fs::create_dir_all(&scans_dir).is_err() || fs::create_dir_all(&thumbs_dir).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let pagedb_path = pages_dir.join("pagedata.json");
    let mut pagedb = storage::load_page_db(&pagedb_path);

    // Collect and validate all files before writing anything, so we can sort
    // by filename and assign contiguous order keys within the batch.
    let mut incoming: Vec<(String, axum::body::Bytes)> = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = match field.file_name() {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => continue,
        };
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if !matches!(ext.as_deref(), Some("jpg") | Some("jpeg") | Some("png") | Some("tif") | Some("tiff") | Some("webp")) {
            continue;
        }
        let Ok(data) = field.bytes().await else { continue; };
        incoming.push((filename, data));
    }
    incoming.sort_by(|(a, _), (b, _)| a.cmp(b));

    let batch = pagedb.next_batch;
    pagedb.next_batch += 1;
    let base_import_order = pagedb.pages.iter().map(|p| p.import_order).max()
        .map_or(0, |max| max + IMPORT_ORDER_GAP);

    let mut new_pages: Vec<Page> = Vec::new();
    for (i, (filename, data)) in incoming.into_iter().enumerate() {
        let final_name = images::resolve_scan_filename(&scans_dir, &filename);
        if fs::write(scans_dir.join(&final_name), &data).is_err() {
            continue;
        }
        let stem = std::path::Path::new(&final_name)
            .file_stem().and_then(|s| s.to_str()).unwrap_or(&final_name);
        let thumb_name = format!("{stem}.jpg");
        let (sw, sh, tw, th) = images::generate_thumb(&data, &thumbs_dir.join(&thumb_name))
            .unwrap_or((0, 0, 0, 0));
        let import_order = base_import_order + (i as u32) * IMPORT_ORDER_GAP;
        new_pages.push(Page { index: 0, name: String::new(), scan: final_name, scan_width: sw, scan_height: sh, thumb: thumb_name, thumb_width: tw, thumb_height: th, batch, import_order });
    }

    match (query.after, query.before) {
        (Some(_), Some(_)) => {
            return StatusCode::BAD_REQUEST.into_response();
        }
        (None, None) => {
            pagedb.pages.extend(new_pages);
        }
        (Some(after), None) => {
            let insert_pos = (after + 1).min(pagedb.pages.len());
            for (i, page) in new_pages.into_iter().enumerate() {
                pagedb.pages.insert(insert_pos + i, page);
            }
        }
        (None, Some(before)) => {
            let insert_pos = before.min(pagedb.pages.len());
            for (i, page) in new_pages.into_iter().enumerate() {
                pagedb.pages.insert(insert_pos + i, page);
            }
        }
    }

    storage::reindex(&mut pagedb);
    let _ = storage::save_page_db(&pagedb_path, &pagedb);

    Redirect::to(&format!("/projects/{}", machine_name)).into_response()
}

pub async fn remove_images_get(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Query(query): Query<RemoveQuery>,
) -> impl IntoResponse {
    let toml_path = state.projects_dir.join(&machine_name).join("metadata").join("project.toml");
    let Ok(contents) = fs::read_to_string(&toml_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(project) = toml::from_str::<Project>(&contents) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let indices_str = query.indices.unwrap_or_default();
    if indices_str.is_empty() {
        return Redirect::to(&format!("/projects/{}/pages", machine_name)).into_response();
    }
    let indices: Vec<usize> = indices_str.split(',').filter_map(|s| s.trim().parse().ok()).collect();
    let db_path = state.projects_dir.join(&machine_name).join("pages").join("pagedata.json");
    let db = storage::load_page_db(&db_path);
    let pages_to_remove: Vec<Page> = indices.iter()
        .filter_map(|&i| db.pages.iter().find(|p| p.index == i).cloned())
        .collect();
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("projects/remove.html").unwrap()
        .render(context! { project, pages_to_remove, indices_str }).unwrap();
    Html(html).into_response()
}

pub async fn remove_images_post(
    State(state): State<AppState>,
    Path(machine_name): Path<String>,
    Form(form): Form<RemoveForm>,
) -> impl IntoResponse {
    let db_path = state.projects_dir.join(&machine_name).join("pages").join("pagedata.json");
    let mut db = storage::load_page_db(&db_path);
    let to_remove: std::collections::HashSet<usize> = form.indices.split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    db.pages.retain(|p| !to_remove.contains(&p.index));
    storage::reindex(&mut db);
    let _ = storage::save_page_db(&db_path, &db);
    Redirect::to(&format!("/projects/{}/pages", machine_name)).into_response()
}

pub async fn serve_thumb(
    State(state): State<AppState>,
    Path((machine_name, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    let path = state.projects_dir
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
    let path = state.projects_dir
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

pub async fn list_projects(State(state): State<AppState>) -> impl IntoResponse {
    Json(storage::read_projects(&state))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProject>,
) -> impl IntoResponse {
    let machine_name = payload.machine_name.trim().to_string();
    let name = payload.name.trim().to_string();

    if !storage::is_valid_machine_name(&machine_name) || name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "invalid name"}))).into_response();
    }

    let project_dir = state.projects_dir.join(&machine_name);
    if project_dir.exists() {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "project already exists"}))).into_response();
    }

    match storage::create_project_on_disk(&state, &name, &machine_name) {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"machine_name": machine_name}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}