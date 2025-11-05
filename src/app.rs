use egui::{Color32, ViewportId};

pub struct App {

}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // NOTE: I think this stuff requires serde? But it is very cool.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        App {
        }
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(target_os = "windows")]
        crate::os::apply_window_transparency(_frame);

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                .fill(Color32::from_rgba_unmultiplied(176, 134, 189, 127)))
            .show(ctx, |ui| {

        });

        ctx.show_viewport_deferred(ViewportId::from_hash_of("glassbox_controls"),
            egui::ViewportBuilder::default()
                .with_inner_size((400.0, 300.0)),
            |ctx, class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Control Window");
                });
            });
    }
}
