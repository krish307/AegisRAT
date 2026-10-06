/// Applies platform-specific stealth measures.
pub fn apply_stealth() {
    // 1. Windows: Hide Console Window
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::winuser::{ShowWindow, SW_HIDE, GetConsoleWindow};
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    
    // 2. Windows: Set Process Priority to Below Normal
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
        use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    }

    // 3. Android: Set Process Title (if supported)
    #[cfg(target_os = "android")]
    {
        // On Android, the process name is determined by the package name.
        // We can't easily change it, but we can ensure our threads 
        // have benign names (handled in main.rs).
        
        // Disable debuggable flag (handled in AndroidManifest.xml)
        
        log::info!("[AEGIS] Android stealth protocols active");
    }
    
    // 4. Linux/macOS: Set Process Title
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Use prctl on Linux or setproctitle on macOS
        // This is optional for now
        log::info!("[AEGIS] Unix stealth protocols active");
    }
}
