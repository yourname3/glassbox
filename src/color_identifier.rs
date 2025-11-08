use egui::{Color32, ColorImage};

pub fn identify_colors_in(image: &ColorImage) -> (Color32, Color32) {
    let pixels = image.pixels.iter().map(|p| iris_lib::color::Color {
        r: p.r(),
        g: p.g(),
        b: p.b(),
        a: p.a()
    }).collect();
    let mut color_bucket = iris_lib::color_bucket::ColorBucket::from_pixels(pixels).unwrap();
    let palette = color_bucket.make_palette(1);

    let to_egui = |c: iris_lib::color::Color| {
        egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    };

    return (to_egui(palette[0]), to_egui(palette[1])); 
}

pub const FG_DEFAULT: Color32 = Color32::from_rgba_unmultiplied_const(143, 143, 143, 255);
pub const BG_DEFAULT: Color32 = Color32::from_rgba_unmultiplied_const(230, 230, 230, 255);

pub fn identify_colors(image: Option<&ColorImage>) -> (Color32, Color32) {
    if let Some(img) = image {
        return identify_colors_in(img);
    }

    // Defaults
    (FG_DEFAULT, BG_DEFAULT)
}