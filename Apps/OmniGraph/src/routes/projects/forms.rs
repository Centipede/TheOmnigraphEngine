use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SettingsForm {
    pub openai_api_key:     String,
    pub perplexity_api_key: String,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub perplexity_api_key: Option<String>,
}

#[derive(Serialize)]
pub struct SettingsResponse {
    pub openai_api_key_set: bool,
    pub perplexity_api_key_set: bool,
}


#[derive(Deserialize, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub machine_name: String,
}

#[derive(Deserialize)]
pub struct MetadataForm {
    pub name: String,
    #[serde(default)]
    pub abbrev: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub published: String,
    #[serde(default)]
    pub author_names: Vec<String>,
    #[serde(default)]
    pub author_abbrevs: Vec<String>,
    #[serde(default)]
    pub action: String,
}

#[derive(Deserialize)]
pub struct IngestQuery {
    pub after: Option<usize>,
    pub before: Option<usize>,
}

#[derive(Deserialize)]
pub struct RemoveQuery {
    pub indices: Option<String>,
}

#[derive(Deserialize)]
pub struct RemoveForm {
    pub indices: String,
}

#[derive(Deserialize)]
pub struct RenameForm {
    pub indices: String,
    pub scheme: Option<String>,
    pub first_page: Option<String>,
}
