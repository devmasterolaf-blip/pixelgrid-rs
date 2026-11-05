//! Lossless transforms: cropping, mirroring and quarter turns.

use crate::error::ImageError;
use crate::image::Image;
use crate::pixel::Pixel;

/// Copies the `width` x `height` rectangle whose top-left corner is `(x, y)`.
///
/// # Errors
/// [`ImageError::RegionOutOfBounds`] when the rectangle does not fit inside the image.
pub fn crop<P: Pixel>(
    image: &Image<P>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Image<P>, ImageError> {
    Ok(image.view(x, y, width, height)?.to_image())
}

/// Mirrors the image left to right.
pub fn flip_horizontal<P: Pixel>(image: &Image<P>) -> Image<P> {
    let (w, h) = image.dimensions();
    Image::from_fn(w, h, |x, y| image[(w - 1 - x, y)])
}

/// Mirrors the image top to bottom.
pub fn flip_vertical<P: Pixel>(image: &Image<P>) -> Image<P> {
    let (w, h) = image.dimensions();
    Image::from_fn(w, h, |x, y| image[(x, h - 1 - y)])
}

/// Rotates by a quarter turn clockwise. The result is `height` x `width`.
pub fn rotate90<P: Pixel>(image: &Image<P>) -> Image<P> {
    let (w, h) = image.dimensions();
    Image::from_fn(h, w, |x, y| image[(y, h - 1 - x)])
}

/// Rotates by a half turn.
pub fn rotate180<P: Pixel>(image: &Image<P>) -> Image<P> {
    let (w, h) = image.dimensions();
    Image::from_fn(w, h, |x, y| image[(w - 1 - x, h - 1 - y)])
}

/// Rotates by a quarter turn counter-clockwise (three quarter turns clockwise).
pub fn rotate270<P: Pixel>(image: &Image<P>) -> Image<P> {
    let (w, h) = image.dimensions();
    Image::from_fn(h, w, |x, y| image[(w - 1 - y, x)])
}
