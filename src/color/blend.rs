//! Alpha compositing with straight and premultiplied alpha, and separable blend modes.
//!
//! All arithmetic is done in integers with round-to-nearest, so results are exact and
//! reproducible.

use crate::image::Image;
use crate::pixel::{Rgb, Rgba};

/// `a * b / 255`, rounded to nearest.
pub fn mul_div255(a: u8, b: u8) -> u8 {
    ((u32::from(a) * u32::from(b) + 127) / 255) as u8
}

/// Converts straight alpha to premultiplied alpha.
pub fn premultiply(c: Rgba) -> Rgba {
    Rgba::new(mul_div255(c.r, c.a), mul_div255(c.g, c.a), mul_div255(c.b, c.a), c.a)
}

/// Converts premultiplied alpha back to straight alpha. Fully transparent input gives
/// transparent black.
pub fn unpremultiply(c: Rgba) -> Rgba {
    if c.a == 0 {
        return Rgba::TRANSPARENT;
    }
    let a = u32::from(c.a);
    let un = |v: u8| ((u32::from(v) * 255 + a / 2) / a).min(255) as u8;
    Rgba::new(un(c.r), un(c.g), un(c.b), c.a)
}

/// Porter-Duff "source over" for straight-alpha colours.
pub fn over(src: Rgba, dst: Rgba) -> Rgba {
    let (sa, da) = (u32::from(src.a), u32::from(dst.a));
    // Output alpha scaled by 255: sa + da * (1 - sa).
    let alpha = sa * 255 + da * (255 - sa);
    if alpha == 0 {
        return Rgba::TRANSPARENT;
    }
    let channel = |s: u8, d: u8| {
        let num = u32::from(s) * sa * 255 + u32::from(d) * da * (255 - sa);
        ((num + alpha / 2) / alpha) as u8
    };
    Rgba::new(
        channel(src.r, dst.r),
        channel(src.g, dst.g),
        channel(src.b, dst.b),
        ((alpha + 127) / 255) as u8,
    )
}

/// Porter-Duff "source over" for premultiplied colours: `src + dst * (1 - src.a)`.
pub fn over_premultiplied(src: Rgba, dst: Rgba) -> Rgba {
    let inv = 255 - src.a;
    let channel = |s: u8, d: u8| s.saturating_add(mul_div255(d, inv));
    Rgba::new(
        channel(src.r, dst.r),
        channel(src.g, dst.g),
        channel(src.b, dst.b),
        channel(src.a, dst.a),
    )
}

/// How a source pixel is combined with the destination pixel under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// The source replaces the destination, alpha included.
    Replace,
    /// Plain alpha compositing, see [`over`].
    Over,
    Multiply,
    Screen,
    /// Channel-wise saturating addition.
    Add,
    Darken,
    Lighten,
    Difference,
}

impl BlendMode {
    /// Parses a lower-case mode name such as `"multiply"`.
    pub fn from_name(name: &str) -> Option<BlendMode> {
        Some(match name {
            "replace" => BlendMode::Replace,
            "over" => BlendMode::Over,
            "multiply" => BlendMode::Multiply,
            "screen" => BlendMode::Screen,
            "add" => BlendMode::Add,
            "darken" => BlendMode::Darken,
            "lighten" => BlendMode::Lighten,
            "difference" => BlendMode::Difference,
            _ => return None,
        })
    }

    fn mix(self, s: u8, d: u8) -> u8 {
        match self {
            BlendMode::Replace | BlendMode::Over => s,
            BlendMode::Multiply => mul_div255(s, d),
            BlendMode::Screen => 255 - mul_div255(255 - s, 255 - d),
            BlendMode::Add => s.saturating_add(d),
            BlendMode::Darken => s.min(d),
            BlendMode::Lighten => s.max(d),
            BlendMode::Difference => s.abs_diff(d),
        }
    }
}

/// Blends `src` onto `dst` with straight alpha.
///
/// For the separable modes the blended colour only applies where the destination has coverage:
/// the source colour becomes `(1 - dst.a) * src + dst.a * mix(src, dst)` and is then composited
/// with [`over`].
pub fn blend(mode: BlendMode, src: Rgba, dst: Rgba) -> Rgba {
    match mode {
        BlendMode::Replace => src,
        BlendMode::Over => over(src, dst),
        _ => {
            let da = u32::from(dst.a);
            let channel = |s: u8, d: u8| {
                let mixed = u32::from(mode.mix(s, d));
                (((255 - da) * u32::from(s) + da * mixed + 127) / 255) as u8
            };
            let mixed = Rgba::new(
                channel(src.r, dst.r),
                channel(src.g, dst.g),
                channel(src.b, dst.b),
                src.a,
            );
            over(mixed, dst)
        }
    }
}

/// Composites an RGBA image over a solid background colour, giving an opaque RGB image.
pub fn flatten(image: &Image<Rgba>, background: Rgb) -> Image<Rgb> {
    image.map(|p| over(p, background.with_alpha(255)).rgb())
}
