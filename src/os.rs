use std::{mem::MaybeUninit, os::raw::c_void};

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
use wayland_backend::client::ObjectId;
use wayland_client::{Dispatch, EventQueue, protocol::{wl_compositor::WlCompositor, wl_region::WlRegion, wl_registry::{self, WlRegistry}, wl_surface}};
#[cfg(all(unix, not(target_os = "macos")))]
use wayland_client::{Connection, Proxy, protocol::wl_surface::WlSurface};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1}, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1};

/// State object used to implement the "set region to null" operation for Wayland.
struct WaylandRegionSettingState {
    initialized: bool,
    compositor_name: u32,
    compositor_version: u32,

    has_layer_shell: bool,
    zwlr_layer_shell_name: u32,
    zwlr_layer_shell_version: u32,
}

impl Dispatch<WlRegion, ()> for WaylandRegionSettingState {
    fn event(
        _: &mut Self,
        _: &WlRegion,
        _: <WlRegion as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) { }
}

impl Dispatch<WlCompositor, ()> for WaylandRegionSettingState {
    fn event(
        _: &mut Self,
        _: &WlCompositor,
        _: <WlCompositor as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) { }
}

impl Dispatch<WlRegistry, ()> for WaylandRegionSettingState {
    fn event(
        state: &mut Self,
        _proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            eprintln!("{} {} {}", name, interface, version);
            if interface == "wl_compositor" {
                state.compositor_name = name;
                state.compositor_version = version;
                state.initialized = true;
            }

            if interface == "zwlr_layer_shell_v1" {
                state.zwlr_layer_shell_name = name;
                state.zwlr_layer_shell_version = version;
                state.has_layer_shell = true;
            }
        }
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for WaylandRegionSettingState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerShellV1,
        event: <ZwlrLayerShellV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandRegionSettingState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: <ZwlrLayerSurfaceV1 as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        
    }
}

/// On Wayland, makes a window transparent to mouse input by setting an empty input region.
/// This allows mouse events to pass through to windows beneath it.
/// 
/// For this to work properly, you need to call this once per window after creation,
/// providing the raw Wayland display and surface pointers.
pub fn make_wayland_window_input_transparent(surface_ptr: *mut c_void, display_ptr: *mut c_void) -> Option<()> {
    let conn = unsafe {
        Connection::from_backend(wayland_backend::sys::client::Backend::from_foreign_display(display_ptr as _))
    };

    let surface = unsafe {
        WlSurface::from_id(&conn, ObjectId::from_ptr(
            WlSurface::interface(),
            surface_ptr as _).ok()?
        ).ok()?
    };

    let mut queue: EventQueue<WaylandRegionSettingState> = conn.new_event_queue();
    let mut state = WaylandRegionSettingState {
        initialized: false,
        compositor_name: 0, 
        compositor_version: 0,

        has_layer_shell: false,
        zwlr_layer_shell_name: 0,
        zwlr_layer_shell_version: 0,
    };

    let qh = queue.handle();
    let display = conn.display();
    let registry = display.get_registry(&qh, ());

    let _ = queue.roundtrip(&mut state);

    if state.initialized {
        let compositor = registry.bind::<WlCompositor, _, _>(state.compositor_name, state.compositor_version, &qh, ());
        let region = compositor.create_region(&qh, ());
        surface.set_input_region(Some(&region));
    }

    if state.has_layer_shell {
        let layer_shell = registry.bind::<ZwlrLayerShellV1, _, _>(
            state.zwlr_layer_shell_name, state.zwlr_layer_shell_version, &qh, ()
        );

        layer_shell.get_layer_surface(&surface, None, Layer::Overlay, "glassbox".into(), &qh, ());
    }

    Some(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn apply_window_transparency<T: HasWindowHandle + HasDisplayHandle>(has_handle: T) {
    if let Ok(handle) = has_handle.window_handle() {
        use raw_window_handle::{RawWindowHandle, WindowHandle};

        if let RawWindowHandle::Wayland(window) = handle.as_raw() {
            if let Ok(display) = has_handle.display_handle() {
                if let RawDisplayHandle::Wayland(display) = display.as_raw() {
                    make_wayland_window_input_transparent(window.surface.as_ptr(), display.display.as_ptr());
                }
            }
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_global_mouse_position(ctx: &egui::Context) -> egui::Pos2 {
    egui::pos2(0.0, 100000.0)
}