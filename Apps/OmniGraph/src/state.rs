use crate::app_settings::AppSettings;
use crate::ocr_poll::OcrServerStatus;
use crate::secrets::AppSecrets;
use minijinja::{Environment, Value};
use minijinja_autoreload::AutoReloader;
use rust_embed::RustEmbed;
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(RustEmbed)]
#[folder = "templates/"]
struct EmbeddedTemplates;

#[derive(Clone)]
pub struct AppState {
    pub projects_dir: Arc<PathBuf>,
    pub templates: Arc<AutoReloader>,
    pub secrets: Arc<RwLock<AppSecrets>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub ocr_status: Arc<RwLock<OcrServerStatus>>,
}

impl AppState {
    #[allow(dead_code)]
    pub fn projects_dir(&self) -> PathBuf {
        self.projects_dir.as_ref().clone()
    }

    #[allow(dead_code)]
    pub fn projects_toml_path(&self, machine_name: &str) -> PathBuf {
        self.projects_dir
            .join(machine_name)
            .join("metadata")
            .join("project.toml")
    }

    pub fn project_pagesdb_path(&self, machine_name: &str) -> PathBuf {
        self.projects_dir
            .join(machine_name)
            .join("pages")
            .join("pagedata.json")
    }

    pub fn project_structuredb_path(&self, machine_name: &str) -> PathBuf {
        self.projects_dir
            .join(machine_name)
            .join("sections")
            .join("sectiondata.json")
    }
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

#[cfg(debug_assertions)]
fn build_template_reloader() -> AutoReloader {
    AutoReloader::new(|notifier| {
        let mut env = Environment::new();
        notifier.watch_path("templates", true);
        env.set_loader(minijinja::path_loader("templates"));
        env.add_filter("date", date_filter);
        Ok(env)
    })
}

#[cfg(not(debug_assertions))]
fn build_template_reloader() -> AutoReloader {
    AutoReloader::new(|_notifier| {
        let mut env = Environment::new();

        for path in EmbeddedTemplates::iter() {
            let path = path.to_string();

            if !path.ends_with(".html") {
                continue;
            }

            let file = EmbeddedTemplates::get(&path).ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::TemplateNotFound,
                    format!("embedded template not found: {path}"),
                )
            })?;

            let source = std::str::from_utf8(file.data.as_ref()).map_err(|err| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("embedded template is not valid UTF-8: {path}: {err}"),
                )
            })?;

            env.add_template_owned(path, source.to_owned())?;
        }

        env.add_filter("date", date_filter);
        Ok(env)
    })
}

impl AppState {
    pub fn from_env() -> Self {
        let projects_dir = std::env::var("PROJECTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./projects"));

        let reloader = build_template_reloader();

        let settings = Arc::new(RwLock::new(crate::app_settings::load()));
        let ocr_status = Arc::new(RwLock::new(OcrServerStatus::default()));
        crate::ocr_poll::spawn_poller(Arc::clone(&settings), Arc::clone(&ocr_status));

        Self {
            projects_dir: Arc::new(projects_dir),
            templates: Arc::new(reloader),
            secrets: Arc::new(RwLock::new(crate::secrets::load())),
            settings,
            ocr_status,
        }
    }
}
