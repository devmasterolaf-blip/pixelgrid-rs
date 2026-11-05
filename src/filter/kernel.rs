//! Convolution kernels and the convolution routines themselves.

use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::Pixel;

/// What a filter reads when its window reaches past the image border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorderMode {
    /// The nearest edge pixel: `aaa|abcd|ddd`.
    Clamp,
    /// The image mirrored at its border, edge pixel repeated: `cba|abcd|dcb`.
    Mirror,
    /// The image repeated like a tile: `bcd|abcd|abc`.
    Wrap,
    /// Zero in every channel, alpha included.
    Zero,
}

impl BorderMode {
    /// Maps the possibly out-of-range index `i` into `0..n`, or `None` for [`BorderMode::Zero`].
    fn resolve(self, i: i64, n: i64) -> Option<usize> {
        if (0..n).contains(&i) {
            return Some(i as usize);
        }
        match self {
            BorderMode::Clamp => Some(i.clamp(0, n - 1) as usize),
            BorderMode::Mirror => {
                let m = i.rem_euclid(2 * n);
                Some(if m < n { m } else { 2 * n - 1 - m } as usize)
            }
            BorderMode::Wrap => Some(i.rem_euclid(n) as usize),
            BorderMode::Zero => None,
        }
    }
}

/// A rectangular grid of filter weights with odd width and height, anchored at its centre.
#[derive(Debug, Clone, PartialEq)]
pub struct Kernel {
    width: usize,
    height: usize,
    weights: Vec<f32>,
}

impl Kernel {
    /// Builds a kernel from weights in row order.
    ///
    /// # Errors
    /// [`ImageError::InvalidParameter`] when a dimension is even or zero, when the number of
    /// weights is not `width * height`, or when a weight is not finite.
    pub fn new(width: usize, height: usize, weights: Vec<f32>) -> Result<Kernel, ImageError> {
        let invalid = |reason: String| ImageError::InvalidParameter { name: "kernel", reason };
        if width % 2 == 0 || height % 2 == 0 {
            return Err(invalid(format!("dimensions must be odd, got {width}x{height}")));
        }
        if weights.len() != width * height {
            return Err(invalid(format!(
                "a {width}x{height} kernel needs {} weights, got {}",
                width * height,
                weights.len()
            )));
        }
        if weights.iter().any(|w| !w.is_finite()) {
            return Err(invalid("weights must be finite".to_string()));
        }
        Ok(Kernel { width, height, weights })
    }

    /// Builds a kernel from rows of equal length.
    pub fn from_rows<const N: usize>(rows: &[[f32; N]]) -> Result<Kernel, ImageError> {
        Kernel::new(N, rows.len(), rows.iter().flatten().copied().collect())
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// The weights in row order.
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Sum of all weights. A kernel that sums to one preserves overall brightness.
    pub fn sum(&self) -> f32 {
        self.weights.iter().sum()
    }

    /// Scales the weights so that they sum to one.
    ///
    /// # Errors
    /// [`ImageError::InvalidParameter`] when the weights sum to (nearly) zero.
    pub fn normalized(&self) -> Result<Kernel, ImageError> {
        let sum = self.sum();
        if sum.abs() < 1e-6 {
            return Err(ImageError::InvalidParameter {
                name: "kernel",
                reason: "weights sum to zero and cannot be normalized".to_string(),
            });
        }
        let weights = self.weights.iter().map(|w| w / sum).collect();
        Ok(Kernel { width: self.width, height: self.height, weights })
    }

    /// Splits the kernel into a horizontal and a vertical 1-D kernel whose outer product is
    /// this kernel, when such a pair exists (the kernel has rank one).
    ///
    /// Returns `(horizontal, vertical)` with `width` and `height` entries, or `None` for
    /// kernels that are not separable, including the all-zero kernel.
    pub fn separate(&self) -> Option<(Vec<f32>, Vec<f32>)> {
        let (pivot_index, pivot) = self
            .weights
            .iter()
            .copied()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))?;
        if pivot == 0.0 {
            return None;
        }
        let (px, py) = (pivot_index % self.width, pivot_index / self.width);
        let horizontal: Vec<f32> = self.weights[py * self.width..(py + 1) * self.width].to_vec();
        let vertical: Vec<f32> =
            (0..self.height).map(|y| self.weights[y * self.width + px] / pivot).collect();
        let tolerance = pivot.abs() * 1e-5;
        for (y, v) in vertical.iter().enumerate() {
            for (x, h) in horizontal.iter().enumerate() {
                if (self.weights[y * self.width + x] - v * h).abs() > tolerance {
                    return None;
                }
            }
        }
        Some((horizontal, vertical))
    }
}

fn to_pixel<P: Pixel>(acc: [f32; 4]) -> P {
    P::from_channels(acc.map(|v| v.round().clamp(0.0, 255.0) as u8))
}

/// Applies `kernel` to every channel of the image, alpha included.
///
/// The kernel is applied as written, without flipping (cross-correlation): the weight at the
/// kernel's top-left multiplies the pixel up and to the left of the centre. Results are rounded
/// to nearest and clamped to `0..=255`. Separable kernels automatically take the two-pass route
/// of [`convolve_separable`], which costs `width + height` instead of `width * height`
/// multiplications per pixel; because the passes run in `f32`, its results can differ from
/// [`convolve_2d`] by one in rare rounding ties.
pub fn convolve<P: Pixel>(image: &Image<P>, kernel: &Kernel, border: BorderMode) -> Image<P> {
    match kernel.separate() {
        Some((horizontal, vertical)) => separable_passes(image, &horizontal, &vertical, border),
        None => convolve_2d(image, kernel, border),
    }
}

/// Applies `kernel` with the direct two-dimensional loop, never the separable shortcut.
pub fn convolve_2d<P: Pixel>(image: &Image<P>, kernel: &Kernel, border: BorderMode) -> Image<P> {
    let (w, h) = (i64::from(image.width()), i64::from(image.height()));
    let (rx, ry) = ((kernel.width / 2) as i64, (kernel.height / 2) as i64);
    Image::from_fn(image.width(), image.height(), |x, y| {
        let mut acc = [0.0f32; 4];
        for (ky, row) in kernel.weights.chunks_exact(kernel.width).enumerate() {
            let Some(sy) = border.resolve(i64::from(y) + ky as i64 - ry, h) else { continue };
            for (kx, weight) in row.iter().enumerate() {
                let Some(sx) = border.resolve(i64::from(x) + kx as i64 - rx, w) else { continue };
                let c = image.row(sy as u32)[sx].channels();
                for (sum, v) in acc.iter_mut().zip(c).take(P::CHANNELS) {
                    *sum += weight * f32::from(v);
                }
            }
        }
        to_pixel(acc)
    })
}

/// Applies a horizontal 1-D kernel followed by a vertical one.
///
/// This equals [`convolve_2d`] with the outer product of the two kernels. The intermediate
/// image is kept in floating point, so nothing is rounded or clamped between the passes.
///
/// # Errors
/// [`ImageError::InvalidParameter`] when either kernel has an even length or is empty.
pub fn convolve_separable<P: Pixel>(
    image: &Image<P>,
    horizontal: &[f32],
    vertical: &[f32],
    border: BorderMode,
) -> Result<Image<P>, ImageError> {
    for (name, kernel) in [("horizontal kernel", horizontal), ("vertical kernel", vertical)] {
        if kernel.len() % 2 == 0 {
            return Err(ImageError::InvalidParameter {
                name,
                reason: format!("length must be odd, got {}", kernel.len()),
            });
        }
    }
    Ok(separable_passes(image, horizontal, vertical, border))
}

fn separable_passes<P: Pixel>(
    image: &Image<P>,
    horizontal: &[f32],
    vertical: &[f32],
    border: BorderMode,
) -> Image<P> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let (rx, ry) = ((horizontal.len() / 2) as i64, (vertical.len() / 2) as i64);
    let mut rows: Vec<[f32; 4]> = Vec::with_capacity(w * h);
    for row in image.rows() {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (k, weight) in horizontal.iter().enumerate() {
                let Some(sx) = border.resolve(x as i64 + k as i64 - rx, w as i64) else { continue };
                let c = row[sx].channels();
                for (sum, v) in acc.iter_mut().zip(c).take(P::CHANNELS) {
                    *sum += weight * f32::from(v);
                }
            }
            rows.push(acc);
        }
    }
    Image::from_fn(image.width(), image.height(), |x, y| {
        let mut acc = [0.0f32; 4];
        for (k, weight) in vertical.iter().enumerate() {
            let Some(sy) = border.resolve(i64::from(y) + k as i64 - ry, h as i64) else { continue };
            let c = rows[sy * w + x as usize];
            for (sum, v) in acc.iter_mut().zip(c).take(P::CHANNELS) {
                *sum += weight * v;
            }
        }
        to_pixel(acc)
    })
}
