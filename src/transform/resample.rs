//! Resampling: reading an image at fractional positions, and resizing built on top of it.

use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::Pixel;

/// How a value between pixel centres is reconstructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interpolation {
    /// The value of the closest pixel.
    Nearest,
    /// Linear blend of the 2x2 surrounding pixels.
    Bilinear,
    /// Catmull-Rom cubic over the 4x4 surrounding pixels. Sharper than bilinear; results are
    /// clamped to `0..=255`, so overshoot at hard edges is cut off rather than wrapped.
    Bicubic,
}

impl Interpolation {
    /// Parses `"nearest"`, `"bilinear"` or `"bicubic"`.
    pub fn from_name(name: &str) -> Option<Interpolation> {
        match name {
            "nearest" => Some(Interpolation::Nearest),
            "bilinear" => Some(Interpolation::Bilinear),
            "bicubic" => Some(Interpolation::Bicubic),
            _ => None,
        }
    }
}

/// Weights of the two taps at offsets `0, 1` for a sample `t` past the first one.
fn linear_weights(t: f64) -> [f64; 4] {
    [1.0 - t, t, 0.0, 0.0]
}

/// Catmull-Rom weights of the four taps at offsets `-1, 0, 1, 2`.
fn cubic_weights(t: f64) -> [f64; 4] {
    let (t2, t3) = (t * t, t * t * t);
    [
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    ]
}

/// Reads the image at the continuous position `(x, y)`.
///
/// Integer positions are pixel centres, so `sample(img, 2.0, 1.0, _)` is exactly pixel
/// `(2, 1)`. Taps that fall outside the image use the nearest edge pixel (clamp-to-edge). Every
/// channel, alpha included, is interpolated independently and rounded to nearest.
///
/// # Panics
/// Panics when the image is empty.
pub fn sample<P: Pixel>(image: &Image<P>, x: f64, y: f64, interpolation: Interpolation) -> P {
    let (weights, first, taps): (fn(f64) -> [f64; 4], i64, usize) = match interpolation {
        Interpolation::Nearest => {
            return image.get_clamped((x + 0.5).floor() as i64, (y + 0.5).floor() as i64);
        }
        Interpolation::Bilinear => (linear_weights, 0, 2),
        Interpolation::Bicubic => (cubic_weights, -1, 4),
    };
    let (fx, fy) = (x.floor(), y.floor());
    let (wx, wy) = (weights(x - fx), weights(y - fy));
    let mut acc = [0.0f64; 4];
    for (j, wy) in wy.iter().enumerate().take(taps) {
        for (i, wx) in wx.iter().enumerate().take(taps) {
            let p = image.get_clamped(fx as i64 + first + i as i64, fy as i64 + first + j as i64);
            let c = p.channels();
            for (sum, v) in acc.iter_mut().zip(c).take(P::CHANNELS) {
                *sum += wx * wy * f64::from(v);
            }
        }
    }
    P::from_channels(acc.map(|v| v.round().clamp(0.0, 255.0) as u8))
}

/// Scales the image to `width` x `height`.
///
/// Destination pixel `d` covers the source interval `[d * s, (d + 1) * s)` with
/// `s = source_size / destination_size`, and its centre is read at `(d + 0.5) * s - 0.5`. With
/// [`Interpolation::Nearest`] that picks source pixel `floor((d + 0.5) * s)`, computed in exact
/// integer arithmetic. Positions beyond the border use clamp-to-edge. No low-pass filter is
/// applied when shrinking, so strong downscaling aliases; blur first if that matters.
///
/// # Errors
/// [`ImageError::EmptyImage`] when the source has no pixels and
/// [`ImageError::InvalidParameter`] when the requested size has a zero dimension.
pub fn resize<P: Pixel>(
    image: &Image<P>,
    width: u32,
    height: u32,
    interpolation: Interpolation,
) -> Result<Image<P>, ImageError> {
    let (sw, sh) = image.dimensions();
    if image.is_empty() {
        return Err(ImageError::EmptyImage { width: sw, height: sh });
    }
    if width == 0 || height == 0 {
        return Err(ImageError::InvalidParameter {
            name: "size",
            reason: format!("cannot resize to {width}x{height}"),
        });
    }
    if interpolation == Interpolation::Nearest {
        let pick = |d: u32, src: u32, dst: u32| {
            ((2 * u64::from(d) + 1) * u64::from(src) / (2 * u64::from(dst))) as u32
        };
        return Ok(Image::from_fn(width, height, |x, y| image[(pick(x, sw, width), pick(y, sh, height))]));
    }
    let (scale_x, scale_y) = (f64::from(sw) / f64::from(width), f64::from(sh) / f64::from(height));
    Ok(Image::from_fn(width, height, |x, y| {
        let sx = (f64::from(x) + 0.5) * scale_x - 0.5;
        let sy = (f64::from(y) + 0.5) * scale_y - 0.5;
        sample(image, sx, sy, interpolation)
    }))
}
