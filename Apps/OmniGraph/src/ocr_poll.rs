use crate::app_settings::{AppSettings, OcrServerConfig};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, interval};

pub const POLL_INTERVAL_SECS: u64 = 30;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    #[default]
    Unconfigured,
    Online,
    Offline,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct OcrServerStatus {
    pub server_1: ServerStatus,
    pub server_2: ServerStatus,
}

pub fn spawn_poller(settings: Arc<RwLock<AppSettings>>, status: Arc<RwLock<OcrServerStatus>>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(POLL_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            let (s1, s2) = {
                let s = settings.read().unwrap();
                (s.ocr_server_1.clone(), s.ocr_server_2.clone())
            };
            let new_status = OcrServerStatus {
                server_1: probe(s1).await,
                server_2: probe(s2).await,
            };
            *status.write().unwrap() = new_status;
        }
    });
}

async fn probe(server: Option<OcrServerConfig>) -> ServerStatus {
    let Some(server) = server else {
        return ServerStatus::Unconfigured;
    };
    let addr = format!("{}:{}", server.host, server.port);
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_)) => ServerStatus::Online,
        _ => ServerStatus::Offline,
    }
}
