use egui::{Color32, ColorImage};

#[derive(Clone, Copy)]
pub struct Palette {
    pub fg_a: Color32,
    pub fg_b: Color32,

    pub bg_a: Color32,
    pub bg_b: Color32,
}

pub fn identify_colors_in(image: &ColorImage) -> Palette {
    let pixels = image.pixels.iter().map(|p| iris_lib::color::Color {
        r: p.r(),
        g: p.g(),
        b: p.b(),
        a: p.a()
    }).collect();
    let mut color_bucket = iris_lib::color_bucket::ColorBucket::from_pixels(pixels).unwrap();
    let palette = color_bucket.make_palette(2);

    let to_egui = |c: iris_lib::color::Color| {
        egui::Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
    };

    let get_palette = |idx: usize| {
        *palette.get(idx).or(palette.get(0)).unwrap_or(&iris_lib::color::Color { r: 0, g: 0, b: 0, a: 255 })
    };

    return Palette {
        fg_a: to_egui(get_palette(0)),
        fg_b: to_egui(get_palette(1)),

        bg_a: to_egui(get_palette(2)),
        bg_b: to_egui(get_palette(3)),
    }
}

pub const FG_DEFAULT: Color32 = Color32::from_rgba_unmultiplied_const(143, 143, 143, 255);
pub const BG_DEFAULT: Color32 = Color32::from_rgba_unmultiplied_const(230, 230, 230, 255);

pub fn identify_colors(image: Option<&ColorImage>) -> Palette {
    if let Some(img) = image {
        return identify_colors_in(img);
    }

    // Defaults
    // TODO: Cool gradient
    Palette {
        fg_a: FG_DEFAULT,
        fg_b: FG_DEFAULT,

        bg_a: BG_DEFAULT,
        bg_b: BG_DEFAULT,
    }
}

pub fn default() -> Palette {
    Palette {
        fg_a: FG_DEFAULT,
        fg_b: FG_DEFAULT,

        bg_a: BG_DEFAULT,
        bg_b: BG_DEFAULT,
    }
}