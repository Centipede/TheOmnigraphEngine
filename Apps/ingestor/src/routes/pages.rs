use axum::{extract::State, response::{Html, IntoResponse}};
use minijinja::context;
use crate::state::AppState;

pub async fn settings(State(state): State<AppState>) -> impl IntoResponse {
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("settings/index.html").unwrap()
        .render(context! {}).unwrap();
    Html(html)
}
