use std::{path::PathBuf, sync::{Arc, RwLock}};
use minijinja::Environment;
use minijinja_autoreload::AutoReloader;
use crate::secrets::AppSecrets;

#[derive(Clone)]
pub struct AppState {
    pub projects_dir: Arc<PathBuf>,
    pub templates: Arc<AutoReloader>,
    pub secrets: Arc<RwLock<AppSecrets>>,
}

impl AppState {
    pub fn from_env() -> Self {
        let projects_dir = std::env::var("PROJECTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./projects"));

        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("templates"));
        let reloader = AutoReloader::new(|notifier| {
            let mut env = Environment::new();
            notifier.watch_path("templates", true);
            env.set_loader(minijinja::path_loader("templates"));
            Ok(env)
        });

        Self {
            projects_dir: Arc::new(projects_dir),
            templates: Arc::new(reloader),
            secrets: Arc::new(RwLock::new(crate::secrets::load())),
        }
    }
}
