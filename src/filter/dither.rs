//! Colour reduction to a fixed palette, with and without dithering.

use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::{Pixel, Rgb, Rgba};

/// A list of 1 to 256 colours that an image can be reduced to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: Vec<Rgb>,
}

/// `levels` evenly spaced values from 0 to 255 inclusive.
fn ramp(levels: u32) -> impl Iterator<Item = u8> {
    (0..levels).map(move |i| ((i * 255 + (levels - 1) / 2) / (levels - 1)) as u8)
}

impl Palette {
    /// Builds a palette from explicit colours.
    ///
    /// # Errors
    /// [`ImageError::InvalidParameter`] when `colors` is empty or has more than 256 entries.
    pub fn new(colors: Vec<Rgb>) -> Result<Palette, ImageError> {
        if colors.is_empty() || colors.len() > 256 {
            return Err(ImageError::InvalidParameter {
                name: "palette",
                reason: format!("needs 1 to 256 colours, got {}", colors.len()),
            });
        }
        Ok(Palette { colors })
    }

    /// Black and white.
    pub fn black_white() -> Palette {
        Palette { colors: vec![Rgb::BLACK, Rgb::WHITE] }
    }

    /// `levels` evenly spaced greys from black to white.
    ///
    /// # Errors
    /// [`ImageError::InvalidParameter`] unless `levels` is in `2..=256`.
    pub fn grays(levels: u32) -> Result<Palette, ImageError> {
        if !(2..=256).contains(&levels) {
            return Err(ImageError::InvalidParameter {
                name: "palette",
                reason: format!("grey levels must be between 2 and 256, got {levels}"),
            });
        }
        Ok(Palette { colors: ramp(levels).map(|v| Rgb::new(v, v, v)).collect() })
    }

    /// The `levels`^3 colours with `levels` evenly spaced values per channel, red varying
    /// slowest.
    ///
    /// # Errors
    /// [`ImageError::InvalidParameter`] unless `levels` is in `2..=6` (6^3 = 216 colours).
    pub fn rgb_cube(levels: u32) -> Result<Palette, ImageError> {
        if !(2..=6).contains(&levels) {
            return Err(ImageError::InvalidParameter {
                name: "palette",
                reason: format!("cube levels must be between 2 and 6, got {levels}"),
            });
        }
        let mut colors = Vec::with_capacity((levels * levels * levels) as usize);
        for r in ramp(levels) {
            for g in ramp(levels) {
                for b in ramp(levels) {
                    colors.push(Rgb::new(r, g, b));
                }
            }
        }
        Ok(Palette { colors })
    }

    pub fn colors(&self) -> &[Rgb] {
        &self.colors
    }

    /// Number of colours.
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Always false: a palette holds at least one colour.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Index of the palette colour closest to `color` by squared Euclidean distance in RGB.
    /// The first of several equally close colours wins.
    pub fn nearest_index(&self, color: Rgb) -> usize {
        let distance = |c: &Rgb| {
            let d = |a: u8, b: u8| (i32::from(a) - i32::from(b)).pow(2);
            d(c.r, color.r) + d(c.g, color.g) + d(c.b, color.b)
        };
        let mut best = 0;
        for (i, c) in self.colors.iter().enumerate() {
            if distance(c) < distance(&self.colors[best]) {
                best = i;
            }
        }
        best
    }

    /// The palette colour closest to `color`.
    pub fn nearest(&self, color: Rgb) -> Rgb {
        self.colors[self.nearest_index(color)]
    }
}

fn clamp_channel(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// Rebuilds a pixel of type `P` from a palette colour, keeping the alpha of `original`.
fn with_color<P: Pixel>(original: P, color: Rgb) -> P {
    P::from_rgba(Rgba::new(color.r, color.g, color.b, original.to_rgba().a))
}

/// Maps every pixel to its nearest palette colour without dithering. Alpha is kept.
pub fn quantize<P: Pixel>(image: &Image<P>, palette: &Palette) -> Image<P> {
    image.map(|p| with_color(p, palette.nearest(p.to_rgba().rgb())))
}

/// Reduces the image to `palette` with Floyd-Steinberg error diffusion.
///
/// Pixels are visited row by row, left to right. The difference between the wanted colour
/// (the pixel plus the error diffused onto it, clamped to `0..=255`) and the palette colour
/// chosen for it is passed on: 7/16 to the right, 3/16 below-left, 5/16 below and 1/16
/// below-right. Error that would leave the image is dropped. Alpha is kept and not dithered.
pub fn dither_floyd_steinberg<P: Pixel>(image: &Image<P>, palette: &Palette) -> Image<P> {
    let width = image.width() as usize;
    // Error rows are padded by one entry on each side so the neighbours need no bounds checks.
    let mut current = vec![[0.0f32; 3]; width + 2];
    let mut next = vec![[0.0f32; 3]; width + 2];
    let mut out = image.clone();
    for y in 0..image.height() {
        for (x, pixel) in out.row_mut(y).iter_mut().enumerate() {
            let source = pixel.to_rgba();
            let wanted = [source.r, source.g, source.b];
            let mut value = [0.0f32; 3];
            for c in 0..3 {
                value[c] = (f32::from(wanted[c]) + current[x + 1][c]).clamp(0.0, 255.0);
            }
            let target = Rgb::new(clamp_channel(value[0]), clamp_channel(value[1]), clamp_channel(value[2]));
            let chosen = palette.nearest(target);
            let chosen_channels = [chosen.r, chosen.g, chosen.b];
            for c in 0..3 {
                let error = value[c] - f32::from(chosen_channels[c]);
                current[x + 2][c] += error * 7.0 / 16.0;
                next[x][c] += error * 3.0 / 16.0;
                next[x + 1][c] += error * 5.0 / 16.0;
                next[x + 2][c] += error * 1.0 / 16.0;
            }
            *pixel = with_color(*pixel, chosen);
        }
        std::mem::swap(&mut current, &mut next);
        next.fill([0.0; 3]);
    }
    out
}

/// Size of the Bayer threshold matrix used by [`dither_ordered`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BayerMatrix {
    B2,
    B4,
    B8,
}

impl BayerMatrix {
    /// Side length of the matrix: 2, 4 or 8.
    pub fn size(self) -> u32 {
        match self {
            BayerMatrix::B2 => 2,
            BayerMatrix::B4 => 4,
            BayerMatrix::B8 => 8,
        }
    }

    /// Parses `"2"`, `"4"` or `"8"`.
    pub fn from_name(name: &str) -> Option<BayerMatrix> {
        match name {
            "2" => Some(BayerMatrix::B2),
            "4" => Some(BayerMatrix::B4),
            "8" => Some(BayerMatrix::B8),
            _ => None,
        }
    }

    /// The matrix entry for pixel `(x, y)`, a rank in `0..size * size`. The matrix repeats
    /// every `size` pixels in both directions.
    ///
    /// Entries follow the usual recursive construction `M(2n) = [4M, 4M + 2; 4M + 3, 4M + 1]`
    /// starting from `M(1) = [0]`.
    pub fn rank(self, x: u32, y: u32) -> u32 {
        let mut size = self.size();
        let (mut x, mut y) = (x % size, y % size);
        let mut rank = 0;
        let mut scale = 1;
        // Peel off quadrants from the outermost level inwards; the outermost level contributes
        // the least significant base-4 digit.
        while size > 1 {
            size /= 2;
            let quadrant = match (x >= size, y >= size) {
                (false, false) => 0,
                (true, false) => 2,
                (false, true) => 3,
                (true, true) => 1,
            };
            rank += quadrant * scale;
            scale *= 4;
            x %= size;
            y %= size;
        }
        rank
    }
}

/// Reduces the image to `palette` with ordered (Bayer) dithering.
///
/// Every colour channel of pixel `(x, y)` is offset by
/// `((rank(x, y) + 0.5) / size^2 - 0.5) * spread` before the nearest palette colour is looked
/// up. `spread` should be about the distance between neighbouring palette levels: 255 for
/// black and white, `255 / (levels - 1)` for evenly spaced levels. Alpha is kept.
pub fn dither_ordered<P: Pixel>(
    image: &Image<P>,
    palette: &Palette,
    matrix: BayerMatrix,
    spread: f32,
) -> Image<P> {
    let cells = (matrix.size() * matrix.size()) as f32;
    let mut out = image.clone();
    for y in 0..image.height() {
        for (x, pixel) in out.row_mut(y).iter_mut().enumerate() {
            let offset = ((matrix.rank(x as u32, y) as f32 + 0.5) / cells - 0.5) * spread;
            let source = pixel.to_rgba();
            let shifted = |v: u8| clamp_channel(f32::from(v) + offset);
            let target = Rgb::new(shifted(source.r), shifted(source.g), shifted(source.b));
            *pixel = with_color(*pixel, palette.nearest(target));
        }
    }
    out
}
