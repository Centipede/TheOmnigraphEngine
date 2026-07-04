use axum::{
    extract::{Multipart, State},
    routing::post,
    extract::DefaultBodyLimit,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::{fs::{self, File}, io::AsyncWriteExt, process::Command};

#[derive(Clone)]
struct AppState {
    tesseract_config: Arc<TesseractCommandConfig>,
}

#[derive(Deserialize)]
struct AppConfig {
    tesseract: TesseractCommandConfig,
}

#[derive(Deserialize)]
struct TesseractCommandConfig {
    command: String,
    args: Vec<String>,
}

#[derive(Serialize)]
struct OcrResponse {
    results: Vec<ScannedImage>,
}

#[derive(Serialize)]
pub struct ScannedImage {
    pub upload_name: String,
    pub temp_path: String,
    pub hocr: Option<String>,
    pub text: Option<String>,
    pub error: Option<String>,
}

#[tokio::main]
async fn main() {
    let config_path = get_config_path()
        .expect("failed to read command line arguments");

    let tesseract_config = load_tesseract_config(config_path)
        .await
        .expect("failed to load tesseract configuration");

    let state = AppState {
        tesseract_config: Arc::new(tesseract_config),
    };

    let app = Router::new()
        .route("/ocr/tesseract", post(ocr_tesseract))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn load_tesseract_config(config_path: PathBuf) -> Result<TesseractCommandConfig, String> {
    let config_contents = fs::read_to_string(&config_path)
        .await
        .map_err(|err| {
            format!(
                "failed to read config file {}: {err}",
                config_path.display()
            )
        })?;

    let app_config: AppConfig = toml::from_str(&config_contents)
        .map_err(|err| {
            format!(
                "failed to parse config file {}: {err}",
                config_path.display()
            )
        })?;

    Ok(app_config.tesseract)
}

fn get_config_path() -> Result<PathBuf, String> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--config" {
            let config_path = args
                .next()
                .ok_or_else(|| "--config requires a path argument".to_string())?;

            return Ok(PathBuf::from(config_path));
        }
    }

    let executable_path = std::env::current_exe()
        .map_err(|err| format!("failed to get executable path: {err}"))?;

    let executable_dir = executable_path
        .parent()
        .ok_or_else(|| "failed to get executable directory".to_string())?;

    Ok(executable_dir.join("tesseract.toml"))
}

async fn ocr_tesseract(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Json<OcrResponse> {
    let mut language: Option<String> = None;
    let mut config: Option<String> = None;
    let mut results: Vec<ScannedImage> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "language" => {
                language = field.text().await.ok();
            }
            "config" => {
                config = field.text().await.ok();
            }
            "files" => {
                let upload_name = field
                    .file_name()
                    .unwrap_or("upload")
                    .to_string();

                let bytes = field.bytes().await.unwrap();

                let temp_path = format!("/tmp/{}", &upload_name);

                let mut file = File::create(&temp_path).await.unwrap();
                file.write_all(&bytes).await.unwrap();

                results.push(ScannedImage {
                    upload_name,
                    temp_path,
                    hocr: None,
                    text: None,
                    error: None,
                });
            }
            _ => {}
        }
    }

    let config = config.unwrap_or_else(|| "text".to_string());
    let language = language.unwrap_or_else(|| "eng".to_string());

    for scanned_image in &mut results {
        match run_tesseract(
            &state.tesseract_config,
            &scanned_image.temp_path,
            &language,
            &config,
        ).await {
            Ok(output) if config == "text" => scanned_image.text = Some(output),
            Ok(output) if config == "hocr" => scanned_image.hocr = Some(output),
            Ok(_) => scanned_image.error = Some(format!("unsupported config: {config}")),
            Err(error) => scanned_image.error = Some(error),
        }
    }

    Json(OcrResponse {
        results,
    })
}

async fn run_tesseract(
    tesseract_config: &TesseractCommandConfig,
    image_path: &str,
    language: &str,
    config: &str,
) -> Result<String, String> {
    let mut temp_config_path: Option<String> = None;

    let config_path = if config == "hocr" {
        let path = format!(
            "/tmp/tesseract_hocr_{}_{}.conf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("failed to create config timestamp: {err}"))?
                .as_nanos()
        );

        let config_contents = "\
tessedit_create_hocr 1
hocr_font_info 0
user_defined_dpi 320
";

        fs::write(&path, config_contents)
            .await
            .map_err(|err| format!("failed to write tesseract config: {err}"))?;

        temp_config_path = Some(path.clone());
        path
    } else {
        let path = format!(
            "/tmp/tesseract_text_{}_{}.conf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("failed to create config timestamp: {err}"))?
                .as_nanos()
        );

        let config_contents = "\
hocr_font_info 0
user_defined_dpi 320
";

        fs::write(&path, config_contents)
            .await
            .map_err(|err| format!("failed to write tesseract config: {err}"))?;

        temp_config_path = Some(path.clone());
        path
    };

    let mut command = Command::new(&tesseract_config.command);

    for arg in &tesseract_config.args {
        let arg = arg
            .replace("{image_path}", image_path)
            .replace("{language}", language)
            .replace("{config_path}", &config_path);

        if !arg.is_empty() {
            command.arg(arg);
        }
    }

    println!("Running command: {:?}", command);

    let output = command
        .output()
        .await
        .map_err(|err| format!("failed to run tesseract: {err}"));

    if let Some(path) = temp_config_path {
        let _ = fs::remove_file(path).await;
    }

    let output = output?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}