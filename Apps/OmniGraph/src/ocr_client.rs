use crate::app_settings::OcrServerConfig;
use serde::Deserialize;

#[derive(Deserialize)]
struct OcrResponse {
    results: Vec<RemoteScannedImage>,
}

#[derive(Deserialize)]
struct RemoteScannedImage {
    upload_name: String,
    hocr: Option<String>,
    error: Option<String>,
}

pub struct OcrPageResult {
    pub upload_name: String,
    pub hocr: Option<String>,
    pub error: Option<String>,
}

pub async fn call_ocr_service(
    server: &OcrServerConfig,
    pages: Vec<(String, Vec<u8>)>,
) -> Result<Vec<OcrPageResult>, String> {
    let url = format!("http://{}:{}/ocr/tesseract", server.host, server.port);
    let client = reqwest::Client::new();

    let mut form = reqwest::multipart::Form::new()
        .text("language", "eng")
        .text("config", "hocr");

    for (filename, bytes) in pages {
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename)
            .mime_str("application/octet-stream")
            .map_err(|e| e.to_string())?;
        form = form.part("files", part);
    }

    let resp = client
        .post(&url)
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("OCR service returned {}", resp.status()));
    }

    let body: OcrResponse = resp.json().await.map_err(|e| e.to_string())?;

    Ok(body
        .results
        .into_iter()
        .map(|r| OcrPageResult {
            upload_name: r.upload_name,
            hocr: r.hocr,
            error: r.error,
        })
        .collect())
}
