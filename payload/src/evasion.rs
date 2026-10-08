pub fn apply_stealth() {
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::winuser::{ShowWindow, SW_HIDE, GetConsoleWindow};
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    
    #[cfg(target_os = "windows")]
    unsafe {
        use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
        use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    }

    #[cfg(target_os = "android")]
    {
        log::info!("[AEGIS] Android stealth protocols active");
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        log::info!("[AEGIS] Unix stealth protocols active");
    }
}
