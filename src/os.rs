#[cfg(target_os = "windows")]
use {
    raw_window_handle::HasWindowHandle,
    raw_window_handle::RawWindowHandle,
    
    windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW,
            GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE,

            SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE
        }
    }
};

#[cfg(target_os = "windows")]
fn window_to_hwnd<T: HasWindowHandle>(has_handle: T) -> Option<HWND> {
    if let Ok(handle) = has_handle.window_handle() {
        if let RawWindowHandle::Win32(win) = handle.as_raw() {
            use std::ffi::c_void;

            let hwnd = HWND(win.hwnd.get() as *mut c_void);
            return Some(hwnd);
        }
    }

    return None;
}

#[cfg(target_os = "windows")]
pub fn apply_window_transparency<T: HasWindowHandle>(has_handle: T) {
    let Some(hwnd) = window_to_hwnd(has_handle) else { return; };
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let style = style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32 | WS_EX_NOACTIVATE.0 as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, style);

        let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}