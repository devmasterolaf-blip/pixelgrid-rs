//! Flood fill.

use crate::error::ImageError;
use crate::image::{Connectivity, Image};
use crate::pixel::Pixel;

/// Replaces the connected region around `(x, y)` with `color` and returns how many pixels were
/// filled.
///
/// A pixel belongs to the region when it can be reached from the seed through neighbours (as
/// defined by `connectivity`) that all differ from the *seed's original colour* by at most
/// `tolerance` in every channel. Comparing against the seed rather than the neighbour keeps a
/// smooth gradient from leaking arbitrarily far. With a tolerance of zero only pixels equal to
/// the seed are filled.
///
/// # Errors
/// [`ImageError::OutOfBounds`] when the seed lies outside the image.
pub fn flood_fill<P: Pixel>(
    image: &mut Image<P>,
    x: u32,
    y: u32,
    color: P,
    tolerance: u8,
    connectivity: Connectivity,
) -> Result<usize, ImageError> {
    let (width, height) = image.dimensions();
    let seed = image.get(x, y).ok_or(ImageError::OutOfBounds { x, y, width, height })?;
    let index = |x: u32, y: u32| y as usize * width as usize + x as usize;
    let mut visited = vec![false; width as usize * height as usize];
    let mut stack = vec![(x, y)];
    visited[index(x, y)] = true;
    let mut filled = 0;
    while let Some((px, py)) = stack.pop() {
        image[(px, py)] = color;
        filled += 1;
        for &(dx, dy) in connectivity.offsets() {
            let (nx, ny) = (i64::from(px) + i64::from(dx), i64::from(py) + i64::from(dy));
            if nx < 0 || ny < 0 || nx >= i64::from(width) || ny >= i64::from(height) {
                continue;
            }
            let (nx, ny) = (nx as u32, ny as u32);
            if !visited[index(nx, ny)] && image[(nx, ny)].max_delta(seed) <= tolerance {
                visited[index(nx, ny)] = true;
                stack.push((nx, ny));
            }
        }
    }
    Ok(filled)
}
