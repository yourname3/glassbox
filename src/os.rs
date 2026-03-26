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

#[cfg(all(unix, not(target_os = "macos")))]
use raw_window_handle::HasWindowHandle;
use wayland_client::{Dispatch, globals::GlobalListContents, protocol::{wl_compositor::WlCompositor, wl_region::WlRegion, wl_registry::WlRegistry}};
#[cfg(all(unix, not(target_os = "macos")))]
use wayland_client::{Connection, Proxy};

struct MyState {

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
#[cfg(all(unix, not(target_os = "macos")))]
pub fn make_wayland_window_input_transparent(
    //display_ptr: *mut std::ffi::c_void,
    surface_ptr: *mut std::ffi::c_void,
) -> bool {
    use std::ffi::c_void;
    use std::ptr::NonNull;

    unsafe {
        // Get the wl_display and wl_surface objects

        let Ok(meow) = wayland_client::backend::ObjectId::from_ptr(
            WlSurface::interface(),
            surface_ptr.cast()
        ) else {
            eprintln!("can't get object id");
            return false;
        };
        

        use wayland_client::{EventQueue, protocol::{wl_display::WlDisplay, wl_surface::WlSurface}};
        let Ok(conn) = wayland_client::Connection::connect_to_env() else {
            eprintln!("no connection :(");
            return false;
        };

        let mut event_queue = conn.new_event_queue::<MyState>();
        let qh = event_queue.handle();

        // Initialize globals
        let (globals, _) = match wayland_client::globals::registry_queue_init::<MyState>(&conn) {
            Ok(res) => res,
            Err(_) => { eprintln!("no registry queue"); return false; }
        };

        // Bind to wl_compositor
        let compositor: wayland_client::protocol::wl_compositor::WlCompositor = 
            match globals.bind(&qh, 1..=6, ()) {
                Ok(c) => c,
                Err(_) => { eprintln!("no compositor"); return false; }
            };

        // Create an empty region (with no rectangles added to it)
        let qh = event_queue.handle();
        let empty_region = compositor.create_region(&qh, ());

        // Get the surface object
        //let surface = &*(surface_ptr as *mut WlSurface);
        
        let Ok(surface) = WlSurface::from_id(&conn, meow) else {
            eprintln!("couldn't get wl surface");
            return false;
        };

        // Set the empty input region - this makes the surface transparent to pointer events
        surface.set_input_region(Some(&empty_region));
        surface.commit();

        // Process events to ensure changes are sent to the server
        let mut state = MyState {};
        //let _ = event_queue.roundtrip(&mut state);

        true
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn apply_window_transparency<T: HasWindowHandle>(has_handle: T) {
    if let Ok(handle) = has_handle.window_handle() {
        use raw_window_handle::{RawWindowHandle, WindowHandle};

        if let RawWindowHandle::Wayland(wayland) = handle.as_raw() {
            make_wayland_window_input_transparent(wayland.surface.as_ptr());
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_global_mouse_position(ctx: &egui::Context) -> egui::Pos2 {
    egui::pos2(0.0, 100000.0)
}