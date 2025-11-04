use egui::{Color32, ViewportId};
use raw_window_handle::HasWindowHandle;

pub struct App {
    pub label: String,
    pub value: f32,
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
            label: "label".to_string(),
            value: 20.0,
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
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            use windows::Win32::Foundation::HWND;
            if let Ok(handle) = _frame.window_handle() {
                if let RawWindowHandle::Win32(win) = handle.as_raw() {
                    use std::ffi::c_void;

                    let hwnd = HWND(win.hwnd.get() as *mut c_void);
                    use windows::Win32::UI::WindowsAndMessaging::{
                        GetWindowLongW, SetWindowLongW,
                        GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE,

                        SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE
                    };

                    use crate::apply_window_transparency;
                    unsafe {
                        let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        eprintln!("style: {:b}", style);
                    }

                    // TODO: Do we have to set this every frame..?
                    apply_window_transparency(hwnd);
                }
            }
        }

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

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}