use axum::{Form, extract::State, response::{Html, IntoResponse, Redirect}};
use minijinja::context;
use secrecy::Secret;
use serde::Deserialize;
use crate::state::AppState;
use crate::routes::projects::forms::SettingsForm;


pub async fn settings_get(State(state): State<AppState>) -> impl IntoResponse {
    let secrets = state.secrets.read().unwrap();
    let env = state.templates.acquire_env().unwrap();
    let html = env.get_template("settings/index.html").unwrap()
        .render(context! {
            openai_key_set      => secrets.openai_is_set(),
            perplexity_key_set  => secrets.perplexity_is_set(),
        }).unwrap();
    Html(html)
}

pub async fn settings_post(
    State(state): State<AppState>,
    Form(form): Form<SettingsForm>,
) -> impl IntoResponse {
    let mut secrets = state.secrets.write().unwrap();

    if !form.openai_api_key.trim().is_empty() {
        secrets.openai_api_key = Some(Secret::new(form.openai_api_key.trim().to_string()));
    }
    if !form.perplexity_api_key.trim().is_empty() {
        secrets.perplexity_api_key = Some(Secret::new(form.perplexity_api_key.trim().to_string()));
    }

    let _ = crate::secrets::save(&secrets);
    Redirect::to("/settings")
}
