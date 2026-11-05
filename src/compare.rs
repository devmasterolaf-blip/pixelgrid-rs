//! Comparing two images pixel by pixel.

use crate::dynimage::DynImage;
use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::{Gray, Pixel};

/// Summary of a pixel-by-pixel comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diff {
    /// Number of pixels compared.
    pub total: usize,
    /// Number of pixels with a channel difference above the tolerance.
    pub differing: usize,
    /// Largest channel difference found anywhere, whether or not it is within the tolerance.
    pub max_delta: u8,
    /// Position of the first differing pixel in row order.
    pub first: Option<(u32, u32)>,
}

impl Diff {
    /// Whether no pixel differs by more than the tolerance.
    pub fn is_match(&self) -> bool {
        self.differing == 0
    }
}

fn check_sizes<P: Pixel>(a: &Image<P>, b: &Image<P>) -> Result<(), ImageError> {
    if a.dimensions() == b.dimensions() {
        Ok(())
    } else {
        Err(ImageError::DimensionMismatch { left: a.dimensions(), right: b.dimensions() })
    }
}

/// Compares two images of the same size.
///
/// A pixel counts as differing when any of its channels (alpha included) differs by more than
/// `tolerance`.
///
/// # Errors
/// [`ImageError::DimensionMismatch`] when the sizes are not equal.
pub fn compare<P: Pixel>(a: &Image<P>, b: &Image<P>, tolerance: u8) -> Result<Diff, ImageError> {
    check_sizes(a, b)?;
    let mut diff = Diff { total: a.as_slice().len(), differing: 0, max_delta: 0, first: None };
    for ((x, y, pa), pb) in a.enumerate_pixels().zip(b.pixels()) {
        let delta = pa.max_delta(*pb);
        diff.max_delta = diff.max_delta.max(delta);
        if delta > tolerance {
            diff.differing += 1;
            diff.first.get_or_insert((x, y));
        }
    }
    Ok(diff)
}

/// A mask that is 255 where the two images differ by more than `tolerance` and 0 elsewhere.
///
/// # Errors
/// [`ImageError::DimensionMismatch`] when the sizes are not equal.
pub fn diff_mask<P: Pixel>(a: &Image<P>, b: &Image<P>, tolerance: u8) -> Result<Image<Gray>, ImageError> {
    check_sizes(a, b)?;
    let data = a
        .pixels()
        .zip(b.pixels())
        .map(|(pa, pb)| Gray(if pa.max_delta(*pb) > tolerance { 255 } else { 0 }))
        .collect();
    Image::from_vec(a.width(), a.height(), data)
}

/// Compares two images whose pixel types are only known at run time.
///
/// Images of the same pixel type are compared directly. Otherwise both are widened to RGBA
/// first, so a grey image equals an RGB image whose channels all hold the same grey values.
///
/// # Errors
/// [`ImageError::DimensionMismatch`] when the sizes are not equal.
pub fn compare_dyn(a: &DynImage, b: &DynImage, tolerance: u8) -> Result<Diff, ImageError> {
    match (a, b) {
        (DynImage::Gray(a), DynImage::Gray(b)) => compare(a, b, tolerance),
        (DynImage::Rgb(a), DynImage::Rgb(b)) => compare(a, b, tolerance),
        _ => compare(&a.to_rgba(), &b.to_rgba(), tolerance),
    }
}
