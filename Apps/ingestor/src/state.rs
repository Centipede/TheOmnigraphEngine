use crate::secrets::AppSecrets;
use minijinja::{Environment, Value};
use minijinja_autoreload::AutoReloader;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct AppState {
    pub projects_dir: Arc<PathBuf>,
    pub templates: Arc<AutoReloader>,
    pub secrets: Arc<RwLock<AppSecrets>>,
}

fn date_filter(value: Value, format: &str) -> Result<String, minijinja::Error> {
    let input = value.to_string();

    let input_format = time::macros::format_description!("[year]-[month]-[day]");
    let date = time::Date::parse(&input, input_format).map_err(|err| {
        minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            format!("failed to parse date '{input}': {err}"),
        )
    })?;

    let output_format = match format {
        "Y-m-d" => time::macros::format_description!("[year]-[month]-[day]"),
        _ => {
            return Err(minijinja::Error::new(
                minijinja::ErrorKind::InvalidOperation,
                format!("unsupported date format '{format}'"),
            ));
        }
    };

    date.format(output_format).map_err(|err| {
        minijinja::Error::new(
            minijinja::ErrorKind::InvalidOperation,
            format!("failed to format date: {err}"),
        )
    })
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
            env.add_filter("date", date_filter);
            Ok(env)
        });

        Self {
            projects_dir: Arc::new(projects_dir),
            templates: Arc::new(reloader),
            secrets: Arc::new(RwLock::new(crate::secrets::load())),
        }
    }
}
