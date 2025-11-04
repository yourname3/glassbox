#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use egui::ViewportBuilder;

    use crate::app::App;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).


    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(false)
            .with_inner_size((800.0, 100.0))
            .with_position((0.0, 0.0)),
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
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}