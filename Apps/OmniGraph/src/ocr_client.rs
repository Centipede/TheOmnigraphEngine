use crate::app_settings::OcrServerConfig;
use serde::Deserialize;

const OCR_UPLOAD_LIMIT_BYTES: usize = 100 * 1024 * 1024;
const OCR_UPLOAD_BATCH_TARGET_BYTES: usize = 95 * 1024 * 1024;

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
    let mut batches: Vec<Vec<(String, Vec<u8>)>> = Vec::new();
    let mut current_batch: Vec<(String, Vec<u8>)> = Vec::new();
    let mut current_batch_bytes = 0usize;

    for (filename, bytes) in pages {
        if bytes.len() >= OCR_UPLOAD_LIMIT_BYTES {
            return Err(format!(
                "{} is too large for OCR upload: {} bytes, limit is {} bytes",
                filename,
                bytes.len(),
                OCR_UPLOAD_LIMIT_BYTES
            ));
        }

        if !current_batch.is_empty()
            && current_batch_bytes + bytes.len() > OCR_UPLOAD_BATCH_TARGET_BYTES
        {
            batches.push(current_batch);
            current_batch = Vec::new();
            current_batch_bytes = 0;
        }

        current_batch_bytes += bytes.len();
        current_batch.push((filename, bytes));
    }

    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    let mut all_results = Vec::new();

    for batch in batches {
        let mut batch_results = call_ocr_service_batch(server, batch).await?;
        all_results.append(&mut batch_results);
    }

    Ok(all_results)
}

async fn call_ocr_service_batch(
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