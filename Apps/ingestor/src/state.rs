use std::{path::PathBuf, sync::Arc};
use minijinja::Environment;

#[derive(Clone)]
pub struct AppState {
    pub projects_dir: Arc<PathBuf>,
    pub templates: Arc<Environment<'static>>,
}

impl AppState {
    pub fn from_env() -> Self {
        let projects_dir = std::env::var("PROJECTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./projects"));

        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("templates"));

        Self {
            projects_dir: Arc::new(projects_dir),
            templates: Arc::new(env),
        }
    }
}
