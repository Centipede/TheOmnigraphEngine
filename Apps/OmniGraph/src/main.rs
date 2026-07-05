mod app_settings;
mod hocr_parser;
mod image_utils;
mod ocr_client;
mod ocr_poll;
mod routes;
mod secrets;
mod state;

use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let state = AppState::from_env();
    let app = routes::build_router(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}
