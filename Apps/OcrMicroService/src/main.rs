use axum::{
    extract::Multipart,
    routing::post,
    Json, Router,
};
use serde::Serialize;
use tokio::{fs::{self, File}, io::AsyncWriteExt, process::Command};

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
    let app = Router::new()
        .route("/ocr", post(ocr));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn ocr(mut multipart: Multipart) -> Json<OcrResponse> {
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
        match run_tesseract(&scanned_image.temp_path, &language, &config).await {
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
    image_path: &str,
    language: &str,
    config: &str,
) -> Result<String, String> {
    let mut command = Command::new("tesseract");
    let mut temp_config_path: Option<String> = None;

    command
        .arg(image_path)
        .arg("stdout")
        .arg("-l")
        .arg(language);

    if config == "hocr" {
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

        command.arg(&path);
        temp_config_path = Some(path);
    }

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