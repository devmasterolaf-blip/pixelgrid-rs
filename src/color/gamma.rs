//! The sRGB transfer function and helpers that work in linear light.

use crate::pixel::Rgb;

/// Decodes an 8-bit sRGB channel value to linear light in `[0, 1]`.
pub fn srgb_to_linear(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Encodes a linear-light value to an 8-bit sRGB channel. Input is clamped to `[0, 1]`.
pub fn linear_to_srgb(l: f32) -> u8 {
    let l = l.clamp(0.0, 1.0);
    let c = if l <= 0.003_130_8 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
    (c * 255.0).round() as u8
}

/// Relative luminance (Rec. 709 weights) of an sRGB colour, computed in linear light.
pub fn relative_luminance(c: Rgb) -> f32 {
    0.2126 * srgb_to_linear(c.r) + 0.7152 * srgb_to_linear(c.g) + 0.0722 * srgb_to_linear(c.b)
}

/// Mixes two sRGB colours in linear light: `t = 0` gives `a`, `t = 1` gives `b`.
pub fn mix_linear(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| {
        let (lx, ly) = (srgb_to_linear(x), srgb_to_linear(y));
        linear_to_srgb(lx + (ly - lx) * t)
    };
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}
