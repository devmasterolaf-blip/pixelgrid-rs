//! Conversion between RGB and the cylindrical HSV and HSL models.

use crate::pixel::Rgb;

/// Hue, saturation, value. `h` is in degrees `[0, 360)`, `s` and `v` are in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

/// Hue, saturation, lightness. `h` is in degrees `[0, 360)`, `s` and `l` are in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

fn unit(v: u8) -> f32 {
    f32::from(v) / 255.0
}

fn to_byte(v: f32) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Returns `(hue in degrees, max, min)` of the normalised channels. Greys get hue 0.
fn hue_max_min(c: Rgb) -> (f32, f32, f32) {
    let (r, g, b) = (unit(c.r), unit(c.g), unit(c.b));
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    (hue, max, min)
}

/// Maps a hue, a chroma and an offset to RGB. Shared by the HSV and HSL conversions.
fn from_hue_chroma(h: f32, chroma: f32, m: f32) -> Rgb {
    let sector = h.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    Rgb::new(to_byte(r + m), to_byte(g + m), to_byte(b + m))
}

/// Converts an RGB colour to HSV.
pub fn rgb_to_hsv(c: Rgb) -> Hsv {
    let (h, max, min) = hue_max_min(c);
    let s = if max == 0.0 { 0.0 } else { (max - min) / max };
    Hsv { h, s, v: max }
}

/// Converts HSV to RGB. The hue wraps around; saturation and value are clamped to `[0, 1]`.
pub fn hsv_to_rgb(c: Hsv) -> Rgb {
    let (s, v) = (c.s.clamp(0.0, 1.0), c.v.clamp(0.0, 1.0));
    let chroma = v * s;
    from_hue_chroma(c.h, chroma, v - chroma)
}

/// Converts an RGB colour to HSL.
pub fn rgb_to_hsl(c: Rgb) -> Hsl {
    let (h, max, min) = hue_max_min(c);
    let l = (max + min) / 2.0;
    let delta = max - min;
    let s = if delta == 0.0 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()) };
    Hsl { h, s, l }
}

/// Converts HSL to RGB. The hue wraps around; saturation and lightness are clamped to `[0, 1]`.
pub fn hsl_to_rgb(c: Hsl) -> Rgb {
    let (s, l) = (c.s.clamp(0.0, 1.0), c.l.clamp(0.0, 1.0));
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    from_hue_chroma(c.h, chroma, l - chroma / 2.0)
}

impl From<Rgb> for Hsv {
    fn from(c: Rgb) -> Self {
        rgb_to_hsv(c)
    }
}

impl From<Hsv> for Rgb {
    fn from(c: Hsv) -> Self {
        hsv_to_rgb(c)
    }
}

impl From<Rgb> for Hsl {
    fn from(c: Rgb) -> Self {
        rgb_to_hsl(c)
    }
}

impl From<Hsl> for Rgb {
    fn from(c: Hsl) -> Self {
        hsl_to_rgb(c)
    }
}
