//! Drawing primitives. Coordinates are signed and everything is clipped to the image, so shapes
//! may lie partly or completely outside it.

pub mod flood;
pub mod line;
pub mod polygon;
pub mod shapes;

pub use flood::flood_fill;
pub use line::{fill_rect, line, rect, thick_line};
pub use polygon::{fill_polygon, polygon, FillRule};
pub use shapes::{circle, ellipse, fill_circle, fill_ellipse};

use crate::image::Image;
use crate::pixel::Pixel;

/// Fills the pixels `x0..=x1` of row `y`, clipped to the image.
pub(crate) fn span<P: Pixel>(image: &mut Image<P>, x0: i64, x1: i64, y: i64, color: P) {
    if y < 0 || y >= i64::from(image.height()) {
        return;
    }
    let start = x0.max(0);
    let end = x1.min(i64::from(image.width()) - 1);
    if start <= end {
        image.row_mut(y as u32)[start as usize..=end as usize].fill(color);
    }
}
