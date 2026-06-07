use serde::Deserialize;

#[derive(Deserialize)]
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