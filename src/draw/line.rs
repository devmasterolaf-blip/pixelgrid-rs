//! Lines and rectangles.

use super::span;
use crate::image::Image;
use crate::pixel::Pixel;

/// Walks the pixels of the Bresenham line from `(x0, y0)` to `(x1, y1)`, both ends included.
fn bresenham<F: FnMut(i64, i64)>(x0: i32, y0: i32, x1: i32, y1: i32, mut plot: F) {
    let (mut x, mut y) = (i64::from(x0), i64::from(y0));
    let (x1, y1) = (i64::from(x1), i64::from(y1));
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        plot(x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draws a one-pixel-wide line with Bresenham's algorithm. Both end points are drawn.
pub fn line<P: Pixel>(image: &mut Image<P>, x0: i32, y0: i32, x1: i32, y1: i32, color: P) {
    bresenham(x0, y0, x1, y1, |x, y| span(image, x, x, y, color));
}

/// Draws a line `thickness` pixels wide.
///
/// The line is a Bresenham line widened perpendicular to its major axis: lines that are more
/// horizontal than vertical grow vertically, steep lines grow horizontally. The ends are flat.
/// For an even thickness the extra pixel goes to the right of / below the centre line. A
/// thickness of zero draws nothing.
pub fn thick_line<P: Pixel>(
    image: &mut Image<P>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: u32,
    color: P,
) {
    if thickness == 0 {
        return;
    }
    let before = i64::from((thickness - 1) / 2);
    let after = i64::from(thickness / 2);
    let shallow = (i64::from(x1) - i64::from(x0)).abs() >= (i64::from(y1) - i64::from(y0)).abs();
    bresenham(x0, y0, x1, y1, |x, y| {
        if shallow {
            for yy in y - before..=y + after {
                span(image, x, x, yy, color);
            }
        } else {
            span(image, x - before, x + after, y, color);
        }
    });
}

/// Fills the `width` x `height` rectangle whose top-left corner is `(x, y)`.
pub fn fill_rect<P: Pixel>(image: &mut Image<P>, x: i32, y: i32, width: u32, height: u32, color: P) {
    if width == 0 {
        return;
    }
    let (x, y) = (i64::from(x), i64::from(y));
    let top = y.max(0);
    let bottom = (y + i64::from(height)).min(i64::from(image.height()));
    for row in top..bottom {
        span(image, x, x + i64::from(width) - 1, row, color);
    }
}

/// Draws the one-pixel outline of the `width` x `height` rectangle whose top-left corner is
/// `(x, y)`. The outline lies inside the rectangle.
pub fn rect<P: Pixel>(image: &mut Image<P>, x: i32, y: i32, width: u32, height: u32, color: P) {
    if width == 0 || height == 0 {
        return;
    }
    let (left, top) = (i64::from(x), i64::from(y));
    let right = left + i64::from(width) - 1;
    let bottom = top + i64::from(height) - 1;
    span(image, left, right, top, color);
    span(image, left, right, bottom, color);
    for row in (top + 1).max(0)..bottom.min(i64::from(image.height())) {
        span(image, left, left, row, color);
        span(image, right, right, row, color);
    }
}
