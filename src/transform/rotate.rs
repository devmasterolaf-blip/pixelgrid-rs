//! Rotation by an arbitrary angle.

use super::geometry::{rotate180, rotate270, rotate90};
use super::resample::{sample, Interpolation};
use crate::image::Image;
use crate::pixel::Pixel;

/// Rotates the image clockwise by `degrees` around its centre.
///
/// With `expand` the result is enlarged to the bounding box of the rotated image; without it
/// the result keeps the source size and the corners that turn out of the frame are cut off.
/// Destination pixels whose centre maps outside the source rectangle get `background`; the
/// others are read with `interpolation` (clamp-to-edge at the border, so `background` never
/// bleeds into the picture).
///
/// Angles are taken modulo 360. Multiples of 90 degrees are exact pixel permutations whenever
/// the result size allows it (always with `expand`, and for square images or half turns
/// without).
///
/// # Panics
/// Panics when `degrees` is not finite.
pub fn rotate<P: Pixel>(
    image: &Image<P>,
    degrees: f64,
    interpolation: Interpolation,
    expand: bool,
    background: P,
) -> Image<P> {
    assert!(degrees.is_finite(), "rotation angle must be finite");
    let (w, h) = image.dimensions();
    if image.is_empty() {
        return image.clone();
    }
    let angle = degrees.rem_euclid(360.0);
    if angle == 0.0 {
        return image.clone();
    }
    if angle == 180.0 {
        return rotate180(image);
    }
    if (expand || w == h) && angle == 90.0 {
        return rotate90(image);
    }
    if (expand || w == h) && angle == 270.0 {
        return rotate270(image);
    }
    let (sin, cos) = angle.to_radians().sin_cos();
    let (fw, fh) = (f64::from(w), f64::from(h));
    let (out_w, out_h) = if expand {
        // The tiny slack keeps rounding noise from adding a row or column.
        let bw = (fw * cos.abs() + fh * sin.abs() - 1e-9).ceil().max(1.0);
        let bh = (fw * sin.abs() + fh * cos.abs() - 1e-9).ceil().max(1.0);
        (bw as u32, bh as u32)
    } else {
        (w, h)
    };
    let (src_cx, src_cy) = ((fw - 1.0) / 2.0, (fh - 1.0) / 2.0);
    let (dst_cx, dst_cy) = ((f64::from(out_w) - 1.0) / 2.0, (f64::from(out_h) - 1.0) / 2.0);
    Image::from_fn(out_w, out_h, |x, y| {
        let (dx, dy) = (f64::from(x) - dst_cx, f64::from(y) - dst_cy);
        // Inverse of the clockwise rotation (y grows downwards).
        let sx = dx * cos + dy * sin + src_cx;
        let sy = -dx * sin + dy * cos + src_cy;
        if sx < -0.5 || sy < -0.5 || sx >= fw - 0.5 || sy >= fh - 0.5 {
            background
        } else {
            sample(image, sx, sy, interpolation)
        }
    })
}
