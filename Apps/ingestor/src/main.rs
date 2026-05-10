use axum::{Router, http::{StatusCode, header}, response::IntoResponse};
use rust_embed::RustEmbed;
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() {
    let app = Router::new().fallback(static_handler);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}
