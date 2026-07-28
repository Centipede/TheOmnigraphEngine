use crate::app_settings::OcrCommandFormat;
use crate::ocr_poll::ServerStatus;
use serde::{Deserialize, Serialize};
use crate::hocr_parser::HocrBbox;

#[derive(Deserialize, Serialize, Clone)]
pub struct OcrServerData {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize)]
pub struct OcrSettingsUpdate {
    #[serde(default)]
    pub server_1: Option<OcrServerData>,
    #[serde(default)]
    pub server_2: Option<OcrServerData>,
    pub command_format: OcrCommandFormat,
}

#[derive(Deserialize, Serialize)]
pub struct SettingsUpdate {
    #[serde(default)]
    pub openai_api_key: Option<String>,
    #[serde(default)]
    pub perplexity_api_key: Option<String>,
    #[serde(default)]
    pub ocr: Option<OcrSettingsUpdate>,
}

#[derive(Deserialize, Serialize)]
pub struct SettingsResponse {
    pub openai_api_key_set: bool,
    pub perplexity_api_key_set: bool,
    pub ocr_server_1: Option<OcrServerData>,
    pub ocr_server_2: Option<OcrServerData>,
    pub ocr_command_format: OcrCommandFormat,
    pub ocr_server_1_status: ServerStatus,
    pub ocr_server_2_status: ServerStatus,
}


#[derive(Deserialize, Serialize)]
pub struct ScanRequest {
    pub indices: Vec<usize>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize, Serialize)]
pub struct ScanPageResult {
    pub scan: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct ScanResponse {
    pub results: Vec<ScanPageResult>,
}

#[derive(Deserialize, Serialize)]
pub struct ScanConflict {
    pub pages: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub machine_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct IngestQuery {
    pub after: Option<usize>,
    pub before: Option<usize>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddBlockType {
    Text,
    Image,
}

#[derive(Deserialize, Serialize)]
pub struct AddRequest {
    pub to_carea: Option<String>,
    pub to_block: Option<String>,
    pub to_line: Option<String>,
    pub bbox: HocrBbox,
    pub block_type: Option<AddBlockType>,
    pub text: Option<String>,
    pub erase_underneath: Option<bool>,
    pub erase_overlap: Option<u8>
}

#[derive(Deserialize, Serialize)]
pub struct RemoveRequest {
    pub indices: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct MergeRequest {
    pub other_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct SplitRequest {
    pub before_id: String,
    pub after_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChangeTypeRequest {
    pub kind: String,
}