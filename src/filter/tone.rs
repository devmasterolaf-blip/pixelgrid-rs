//! Point operations and histogram-based tools: every output pixel depends only on the input
//! pixel at the same position (and on global statistics of the image).

use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::{Gray, Pixel};

/// Counts how often each value occurs in one channel of the image.
///
/// # Errors
/// [`ImageError::InvalidParameter`] when `channel` is not below `P::CHANNELS`.
pub fn histogram<P: Pixel>(image: &Image<P>, channel: usize) -> Result<[u32; 256], ImageError> {
    if channel >= P::CHANNELS {
        return Err(ImageError::InvalidParameter {
            name: "channel",
            reason: format!("{} pixels have {} channels, got index {channel}", P::NAME, P::CHANNELS),
        });
    }
    let mut counts = [0u32; 256];
    for p in image.pixels() {
        counts[usize::from(p.channels()[channel])] += 1;
    }
    Ok(counts)
}

/// Histogram of the image's luma (see [`crate::pixel::luma`]); alpha is ignored.
pub fn luma_histogram<P: Pixel>(image: &Image<P>) -> [u32; 256] {
    let mut counts = [0u32; 256];
    for p in image.pixels() {
        counts[usize::from(p.convert::<Gray>().0)] += 1;
    }
    counts
}

/// Applies a lookup table to every colour channel; alpha is left alone.
pub fn apply_lut<P: Pixel>(image: &Image<P>, lut: &[u8; 256]) -> Image<P> {
    image.map(|p| p.map_color(|v| lut[usize::from(v)]))
}

/// Histogram equalization.
///
/// Builds the classic lookup table `(cdf(v) - cdf_min) * 255 / (total - cdf_min)`, rounded to
/// nearest, from the luma histogram and applies it to every colour channel, so the darkest
/// occurring level maps to 0 and the brightest to 255. For grey images this is textbook
/// equalization; for colour images using one table for all channels keeps hues from drifting
/// the way per-channel equalization would. Images with a single luma level are returned
/// unchanged.
pub fn equalize<P: Pixel>(image: &Image<P>) -> Image<P> {
    let counts = luma_histogram(image);
    let total: u64 = counts.iter().map(|c| u64::from(*c)).sum();
    let cdf_min = counts.iter().copied().find(|c| *c > 0).map_or(0, u64::from);
    if total == cdf_min {
        return image.clone();
    }
    let mut lut = [0u8; 256];
    let mut cdf = 0u64;
    for (value, count) in counts.iter().enumerate() {
        cdf += u64::from(*count);
        let above = cdf.saturating_sub(cdf_min);
        lut[value] = ((above * 255 + (total - cdf_min) / 2) / (total - cdf_min)) as u8;
    }
    apply_lut(image, &lut)
}

/// Binarizes the image: pixels whose luma is greater than `level` become 255, the rest 0.
pub fn threshold<P: Pixel>(image: &Image<P>, level: u8) -> Image<Gray> {
    image.map(|p| Gray(if p.convert::<Gray>().0 > level { 255 } else { 0 }))
}

/// Otsu's threshold for a histogram: the level `t` that maximizes the between-class variance
/// of the two classes `v <= t` and `v > t`.
///
/// When several levels tie (for example everywhere in the gap between two clusters) the
/// smallest is returned. A histogram with fewer than two occupied levels gives 0.
pub fn otsu_level(counts: &[u32; 256]) -> u8 {
    let total: f64 = counts.iter().map(|c| f64::from(*c)).sum();
    let weighted_total: f64 = counts.iter().enumerate().map(|(v, c)| v as f64 * f64::from(*c)).sum();
    let (mut best_level, mut best_variance) = (0u8, 0.0f64);
    let (mut below, mut weighted_below) = (0.0f64, 0.0f64);
    for (value, count) in counts.iter().enumerate() {
        below += f64::from(*count);
        weighted_below += value as f64 * f64::from(*count);
        let above = total - below;
        if below == 0.0 || above == 0.0 {
            continue;
        }
        let difference = weighted_below / below - (weighted_total - weighted_below) / above;
        let variance = below * above * difference * difference;
        if variance > best_variance {
            best_variance = variance;
            best_level = value as u8;
        }
    }
    best_level
}

/// Binarizes the image at its Otsu level and returns the result together with the level used.
pub fn otsu_threshold<P: Pixel>(image: &Image<P>) -> (Image<Gray>, u8) {
    let level = otsu_level(&luma_histogram(image));
    (threshold(image, level), level)
}

/// Adds `delta` to every colour channel, saturating at 0 and 255.
pub fn brightness<P: Pixel>(image: &Image<P>, delta: i32) -> Image<P> {
    let mut lut = [0u8; 256];
    for (value, slot) in lut.iter_mut().enumerate() {
        *slot = (value as i64 + i64::from(delta)).clamp(0, 255) as u8;
    }
    apply_lut(image, &lut)
}

/// Scales the distance of every colour channel from mid-grey (128) by `factor`.
///
/// A factor of 1 changes nothing, 0 gives flat grey, values above 1 increase contrast.
///
/// # Errors
/// [`ImageError::InvalidParameter`] unless `factor` is finite and not negative.
pub fn contrast<P: Pixel>(image: &Image<P>, factor: f32) -> Result<Image<P>, ImageError> {
    if !factor.is_finite() || factor < 0.0 {
        return Err(ImageError::InvalidParameter {
            name: "contrast factor",
            reason: format!("must be a non-negative number, got {factor}"),
        });
    }
    let mut lut = [0u8; 256];
    for (value, slot) in lut.iter_mut().enumerate() {
        *slot = ((value as f32 - 128.0) * factor + 128.0).round().clamp(0.0, 255.0) as u8;
    }
    Ok(apply_lut(image, &lut))
}

/// Gamma correction: `out = 255 * (in / 255) ^ (1 / gamma)`. Values above 1 brighten the
/// mid-tones, values below 1 darken them; black and white stay fixed.
///
/// # Errors
/// [`ImageError::InvalidParameter`] unless `gamma` is finite and greater than zero.
pub fn gamma<P: Pixel>(image: &Image<P>, gamma: f32) -> Result<Image<P>, ImageError> {
    if !gamma.is_finite() || gamma <= 0.0 {
        return Err(ImageError::InvalidParameter {
            name: "gamma",
            reason: format!("must be a positive number, got {gamma}"),
        });
    }
    let mut lut = [0u8; 256];
    for (value, slot) in lut.iter_mut().enumerate() {
        *slot = (255.0 * (value as f32 / 255.0).powf(1.0 / gamma)).round() as u8;
    }
    Ok(apply_lut(image, &lut))
}

/// Replaces every colour channel `v` by `255 - v`.
pub fn invert<P: Pixel>(image: &Image<P>) -> Image<P> {
    image.map(|p| p.map_color(|v| 255 - v))
}
