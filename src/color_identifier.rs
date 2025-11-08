use egui::{Color32, ColorImage};

type Vec3 = (f32, f32, f32);

fn min(a: f32, b: f32, c: f32) -> f32 {
    if a < b {
        if a < c { return a; }
        return c;
    }
    if b < c { return b; }
    return c;
}

fn max(a: f32, b: f32, c: f32) -> f32 {
    if a > b {
        if a > c { return a; }
        return c;
    }
    if b > c { return b; }
    return c;
}

fn rgb_to_hsv(rgb: Vec3) -> Vec3 {
    let r = rgb.0; let g = rgb.1; let b = rgb.2;

    let min = min(rgb.0, rgb.1, rgb.2);
    let max = max(rgb.0, rgb.1, rgb.2);

    let v = max;
    let delta = max - min;
    let mut s = 0.0;
    let mut h = 0.0;

    if max != 0.0 {
        s = delta / max;
    }
    else { return (h, s, v); }

    if delta == 0.0 {
        return (h, s, v);
    }
    if      r == max { h = (g - b) / delta; }
    else if g == max { h = 2.0 + (b - r) / delta; }
    else             { h = 4.0 + (r - g) / delta; }

    h = h / 6.0;
    h = h.rem_euclid(1.0);

    (h, s, v)
}

fn hsv_to_rgb(hsv: Vec3) -> Vec3 {
    let mut h = hsv.0;
    let s = hsv.1;
    let v = hsv.2;

    if s == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    h *= 6.0;
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    match i {
        0.0 => (v, t, p),
        1.0 => (q, v, p),
        2.0 => (p, v, t),
        3.0 => (p, q, v),
        4.0 => (t, p, v),
        5.0 => (v, p, q),
        _ => unreachable!()
    }
}

fn adjust_hsv_fn(hsv: Vec3) -> Vec3 {
    (hsv.0, hsv.1, 0.3 * hsv.2 + 0.7)
}

fn adjust_hsv(color: Color32) -> Color32 {
    let as_f32 = (color.r() as f32 / 255.0, color.g() as f32 / 255.0, color.b() as f32 / 255.0);
    let hsv = rgb_to_hsv(as_f32);
    let hsv = adjust_hsv_fn(hsv);
    let rgb = hsv_to_rgb(hsv);

    Color32::from_rgba_unmultiplied(
        (rgb.0 * 255.0) as u8,
        (rgb.1 * 255.0) as u8,
        (rgb.2 * 255.0) as u8,
        255
    )
}

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

    let fg = Color32::WHITE;
    let bg = Color32::WHITE;

    // First-first pass: Identify the most common value/saturation amount in
    // the image.
    let mut avg_hsv = (0.0, 0.0, 0.0);
    for pixel in &image.pixels {
        let as_f32 = (pixel.r() as f32 / 255.0, pixel.g() as f32 / 255.0, pixel.b() as f32 / 255.0);
        let hsv = rgb_to_hsv(as_f32);

        avg_hsv.0 += hsv.0;
        avg_hsv.1 += hsv.1;
        avg_hsv.2 += hsv.2;
    }
    let denom = image.pixels.len() as f32;
    avg_hsv.0 /= denom;
    avg_hsv.1 /= denom;
    avg_hsv.2 /= denom;
    log::info!("avg_hsv: {:?}", avg_hsv);

    let mut best_hsv = (0.0, 0.0, 0.0);
    let mut best = Color32::GRAY;
    let mut best_score = -2.0;

    // First pass: identify the pixel with best (?) saturation/value match (?)
    for pixel in &image.pixels {
        let as_f32 = (pixel.r() as f32 / 255.0, pixel.g() as f32 / 255.0, pixel.b() as f32 / 255.0);
        let hsv = rgb_to_hsv(as_f32);

        // Score: we want close to the average sat/val. But, we want to bias
        // towards higher sat and val as well.
        let score = hsv.1 * 0.4 + hsv.2 * 0.4
            - (hsv.1 - avg_hsv.1).abs() - (hsv.2 - avg_hsv.2).abs();
        if score > best_score {
            best = *pixel;
            best_score = score;
            best_hsv = hsv;
        }

        // if hsv.1 > best_hsv.1 {
        //     best_hsv = hsv;
        //     best = *pixel;
        // }   
        // else if hsv.1 == best_hsv.1 && hsv.2 > best_hsv.2 {
        //     best_hsv = hsv;
        //     best = *pixel;
        // }
    }

    let mut complement = Color32::GRAY;
    let mut best_score = -2.0;

    // Second pass: Identify the pixel with best other-score and
    // a significant change in hue.
    // for pixel in &image.pixels {
    //     let as_f32 = (pixel.r() as f32 / 255.0, pixel.g() as f32 / 255.0, pixel.b() as f32 / 255.0);
    //     let hsv = rgb_to_hsv(as_f32);

    //     let hue_dif = (hsv.0 - best_hsv.0).rem_euclid(1.0);
    //     // Hue is circular, so differences of ~1 are actually small.
    //     let hue_dif = 0.5 - (hue_dif - 0.5).abs();

    //     //let score = hue_dif * 2.0 + hsv.1 + hsv.2 * 0.5;
    //     //let score = hue_dif * 2.0 - (hsv.1 - avg_hsv.1).abs() - (hsv.2 - avg_hsv.2).abs();
    //     let score = hue_dif * 1.0 + hsv.1 * 0.2 + hsv.2 * 0.2
    //         - (hsv.1 - avg_hsv.1).abs() - (hsv.2 - avg_hsv.2).abs();
    //     if score > best_score {
    //         complement = *pixel;
    //         best_score = score;
    //     }
    // }

    return (adjust_hsv(best), best);
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