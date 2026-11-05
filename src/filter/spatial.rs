//! Neighbourhood filters: blurs, sharpening, edge detection and the median filter.

use super::kernel::{convolve_2d, convolve_separable, BorderMode, Kernel};
use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::{Gray, Pixel};

/// Averages every pixel with its `(2 * radius + 1)` square neighbourhood (clamp-to-edge).
/// A radius of zero returns a copy.
pub fn box_blur<P: Pixel>(image: &Image<P>, radius: u32) -> Image<P> {
    let size = 2 * radius as usize + 1;
    let line = vec![1.0 / size as f32; size];
    convolve_separable(image, &line, &line, BorderMode::Clamp).expect("box kernel has odd length")
}

/// The normalized 1-D Gaussian with standard deviation `sigma`, cut off at `ceil(3 * sigma)`
/// samples on either side of the centre.
///
/// # Errors
/// [`ImageError::InvalidParameter`] unless `sigma` is finite and greater than zero.
pub fn gaussian_kernel(sigma: f32) -> Result<Vec<f32>, ImageError> {
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err(ImageError::InvalidParameter {
            name: "sigma",
            reason: format!("must be a positive number, got {sigma}"),
        });
    }
    let radius = (3.0 * sigma).ceil() as i32;
    let weights: Vec<f32> =
        (-radius..=radius).map(|i| (-((i * i) as f32) / (2.0 * sigma * sigma)).exp()).collect();
    let sum: f32 = weights.iter().sum();
    Ok(weights.into_iter().map(|w| w / sum).collect())
}

/// Blurs with a Gaussian of standard deviation `sigma` pixels (clamp-to-edge).
///
/// # Errors
/// [`ImageError::InvalidParameter`] unless `sigma` is finite and greater than zero.
pub fn gaussian_blur<P: Pixel>(image: &Image<P>, sigma: f32) -> Result<Image<P>, ImageError> {
    let line = gaussian_kernel(sigma)?;
    convolve_separable(image, &line, &line, BorderMode::Clamp)
}

/// Sharpens with the 3x3 kernel `[0 -1 0; -1 5 -1; 0 -1 0]` (clamp-to-edge).
pub fn sharpen<P: Pixel>(image: &Image<P>) -> Image<P> {
    let kernel = Kernel::from_rows(&[[0.0, -1.0, 0.0], [-1.0, 5.0, -1.0], [0.0, -1.0, 0.0]])
        .expect("3x3 kernel is valid");
    convolve_2d(image, &kernel, BorderMode::Clamp)
}

/// Sobel edge strength of the image's luma.
///
/// Computes the horizontal and vertical Sobel derivatives `gx` and `gy` (clamp-to-edge) and
/// returns `sqrt(gx^2 + gy^2)` rounded to nearest and clamped to 255.
pub fn sobel<P: Pixel>(image: &Image<P>) -> Image<Gray> {
    let gray: Image<Gray> = image.convert();
    let at = |x: i64, y: i64| i32::from(gray.get_clamped(x, y).0);
    Image::from_fn(gray.width(), gray.height(), |x, y| {
        let (x, y) = (i64::from(x), i64::from(y));
        let gx = (at(x + 1, y - 1) + 2 * at(x + 1, y) + at(x + 1, y + 1))
            - (at(x - 1, y - 1) + 2 * at(x - 1, y) + at(x - 1, y + 1));
        let gy = (at(x - 1, y + 1) + 2 * at(x, y + 1) + at(x + 1, y + 1))
            - (at(x - 1, y - 1) + 2 * at(x, y - 1) + at(x + 1, y - 1));
        let magnitude = f64::from(gx * gx + gy * gy).sqrt().round();
        Gray(magnitude.min(255.0) as u8)
    })
}

/// Replaces every channel of every pixel by the median of its `(2 * radius + 1)` square
/// neighbourhood (clamp-to-edge). Channels are processed independently, alpha included. A
/// radius of zero returns a copy.
pub fn median<P: Pixel>(image: &Image<P>, radius: u32) -> Image<P> {
    if radius == 0 || image.is_empty() {
        return image.clone();
    }
    let r = i64::from(radius);
    let side = 2 * radius as usize + 1;
    let mut window: Vec<[u8; 4]> = Vec::with_capacity(side * side);
    let mut values: Vec<u8> = Vec::with_capacity(side * side);
    Image::from_fn(image.width(), image.height(), |x, y| {
        window.clear();
        for dy in -r..=r {
            for dx in -r..=r {
                window.push(image.get_clamped(i64::from(x) + dx, i64::from(y) + dy).channels());
            }
        }
        let mut out = [0u8; 4];
        for (channel, slot) in out.iter_mut().enumerate().take(P::CHANNELS) {
            values.clear();
            values.extend(window.iter().map(|c| c[channel]));
            let middle = values.len() / 2;
            *slot = *values.select_nth_unstable(middle).1;
        }
        P::from_channels(out)
    })
}
