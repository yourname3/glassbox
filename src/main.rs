#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod os;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use crate::app::App;

    env_logger::init();

    log::info!("logging ready!");

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

            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            Ok(app)
        }),
    )
}