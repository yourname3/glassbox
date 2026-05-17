use std::mem::MaybeUninit;

#[cfg(target_os = "windows")]
use {
    raw_window_handle::HasWindowHandle,
    raw_window_handle::RawWindowHandle,
    
    windows::Win32::{
        Foundation::{HWND, POINT},
        UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW,
            GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE,

            SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE,

            GetCursorPos,
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

#[cfg(target_os = "windows")]
pub fn get_global_mouse_position(ctx: &egui::Context) -> egui::Pos2 {
    let mut point: POINT = POINT { x: 0, y: 0 };

    // Ignore the error
    unsafe { let _ = GetCursorPos(&mut point); }

    egui::pos2(point.x as f32, point.y as f32) / ctx.pixels_per_point()
}

use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
#[cfg(all(unix, not(target_os = "macos")))]
use raw_window_handle::HasWindowHandle;
use wayland_client::{Dispatch, EventQueue, globals::GlobalListContents, protocol::{wl_compositor::WlCompositor, wl_display::WlDisplay, wl_region::WlRegion, wl_registry::WlRegistry}};
#[cfg(all(unix, not(target_os = "macos")))]
use wayland_client::{Connection, Proxy, protocol::wl_surface::WlSurface};

struct MyState {
    region: MaybeUninit<WlRegion>,
    initialized: bool,
}

impl Dispatch<WlRegion, ()> for MyState {
    fn event(
        state: &mut Self,
        proxy: &WlRegion,
        event: <WlRegion as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        
    }
}

impl Dispatch<WlCompositor, ()> for MyState {
    fn event(
        state: &mut Self,
        proxy: &WlCompositor,
        event: <WlCompositor as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        let region = proxy.create_region(qhandle, ());
        state.region = MaybeUninit::new(region);
        state.initialized = true;
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for MyState {
    fn event(
        state: &mut Self,
        proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        todo!()
    }
}

/// On Wayland, makes a window transparent to mouse input by setting an empty input region.
/// This allows mouse events to pass through to windows beneath it.
/// 
/// For this to work properly, you need to call this once per window after creation,
/// providing the raw Wayland display and surface pointers.
pub fn make_wayland_window_input_transparent<T: HasWindowHandle + HasDisplayHandle>(has_handle: T) {
    // let surface = {
    //     if let Ok(handle) = has_handle.window_handle() {
    //         use raw_window_handle::{RawWindowHandle, WindowHandle};

    //         if let RawWindowHandle::Wayland(wayland) = handle.as_raw() {
    //             let as_surface = wayland.surface.as_ptr() as *mut WlSurface;
                
    //             unsafe { &mut *as_surface }
    //         } else { return }
    //     } else { return; }
    // };

    if let Ok(handle) = has_handle.display_handle() {
        if let RawDisplayHandle::Wayland(wayland) = handle.as_raw() {
            let conn = unsafe { Connection::from_backend(wayland_backend::sys::client::Backend::from_foreign_display(wayland.display.as_ptr() as _)) };

            let mut queue: EventQueue<MyState> = conn.new_event_queue();
            let mut state = MyState { region: MaybeUninit::uninit(), initialized: false };

            queue.blocking_dispatch(&mut state);

            //queue.roundtrip(&mut state);
            eprintln!("did it work? {}", state.initialized);
            //let init = unsafe { state.region.assume_init() };

            //surface.set_input_region(Some(&init));
            //let qh = queue.handle();
        
            //let display = conn.display();
            //let registry = display.get_registry(&qh, ());
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn apply_window_transparency<T: HasWindowHandle + HasDisplayHandle>(has_handle: T) {
    if let Ok(handle) = has_handle.window_handle() {
        use raw_window_handle::{RawWindowHandle, WindowHandle};

        if let RawWindowHandle::Wayland(wayland) = handle.as_raw() {
            make_wayland_window_input_transparent(has_handle);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_global_mouse_position(ctx: &egui::Context) -> egui::Pos2 {
    egui::pos2(0.0, 100000.0)
}