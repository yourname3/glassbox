#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;


#[cfg(target_os = "windows")]
fn apply_window_transparency(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW,
        GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE,

        SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE
    };
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let style = style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32 | WS_EX_NOACTIVATE.0 as i32;
        SetWindowLongW(hwnd, GWL_EXSTYLE, style);
        log::info!("set style: {:b}", style);
        eprintln!("set style: {:b}", style);

        let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use egui::ViewportBuilder;

    use crate::app::App;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    log::info!("logging ready!");
    eprintln!("meow!");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)
            .with_inner_size((800.0, 100.0))
            .with_position((0.0, 0.0))
            .with_transparent(true)
        ,
            //.with_inner_size([400.0, 300.0])
            //.with_min_inner_size([300.0, 220.0]),
            // .with_icon(
            //     // NOTE: Adding an icon is optional
            //     eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
            //         .expect("Failed to load icon"),
            // ),
        ..Default::default()
    };

    eframe::run_native(
        "glassbox",
        native_options,
        Box::new(|cc| {
            let app = Box::new(App::new(cc));

            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                use windows::Win32::Foundation::HWND;
                if let Ok(handle) = cc.window_handle() {
                    if let RawWindowHandle::Win32(win) = handle.as_raw() {
                        use std::ffi::c_void;

                        let hwnd = HWND(win.hwnd.get() as *mut c_void);
                        apply_window_transparency(hwnd);
                    }
                }
            }
            
            Ok(app)
        }),
    )
}