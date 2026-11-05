//! Circles and ellipses drawn with the integer midpoint algorithms.

use super::span;
use crate::image::Image;
use crate::pixel::Pixel;

/// Calls `visit(x, y)` for the first-octant points (`x >= y >= 0`) of a circle of radius `r`
/// centred on the origin.
fn circle_octant<F: FnMut(i64, i64)>(radius: u32, mut visit: F) {
    let mut x = i64::from(radius);
    let mut y = 0i64;
    let mut d = 1 - x;
    while x >= y {
        visit(x, y);
        y += 1;
        if d < 0 {
            d += 2 * y + 1;
        } else {
            x -= 1;
            d += 2 * (y - x) + 1;
        }
    }
}

/// Draws the outline of a circle. A radius of zero draws the centre pixel.
pub fn circle<P: Pixel>(image: &mut Image<P>, cx: i32, cy: i32, radius: u32, color: P) {
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    circle_octant(radius, |x, y| {
        for (dx, dy) in [(x, y), (y, x)] {
            span(image, cx + dx, cx + dx, cy + dy, color);
            span(image, cx - dx, cx - dx, cy + dy, color);
            span(image, cx + dx, cx + dx, cy - dy, color);
            span(image, cx - dx, cx - dx, cy - dy, color);
        }
    });
}

/// Draws a filled circle covering the same pixels as [`circle`] plus its interior.
pub fn fill_circle<P: Pixel>(image: &mut Image<P>, cx: i32, cy: i32, radius: u32, color: P) {
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    circle_octant(radius, |x, y| {
        span(image, cx - x, cx + x, cy + y, color);
        span(image, cx - x, cx + x, cy - y, color);
        span(image, cx - y, cx + y, cy + x, color);
        span(image, cx - y, cx + y, cy - x, color);
    });
}

/// Calls `visit(x, y)` for the first-quadrant points of an axis-aligned ellipse centred on the
/// origin, walking from `(0, ry)` to `(rx, 0)`.
fn ellipse_quadrant<F: FnMut(i64, i64)>(rx: u32, ry: u32, mut visit: F) {
    let (rx, ry) = (i64::from(rx), i64::from(ry));
    if ry == 0 {
        // A flat ellipse is a horizontal segment; the general loop would only see its centre.
        for x in 0..=rx {
            visit(x, 0);
        }
        return;
    }
    let (a2, b2) = (rx * rx, ry * ry);
    let (mut x, mut y) = (0i64, ry);
    let (mut px, mut py) = (0i64, 2 * a2 * y);
    visit(x, y);
    // Region 1: the slope is shallower than -1, step in x. Decision values are scaled by 4 so
    // the quarter terms of the midpoint test stay integral.
    let mut p = 4 * b2 - 4 * a2 * ry + a2;
    while px < py {
        x += 1;
        px += 2 * b2;
        if p < 0 {
            p += 4 * (b2 + px);
        } else {
            y -= 1;
            py -= 2 * a2;
            p += 4 * (b2 + px - py);
        }
        visit(x, y);
    }
    // Region 2: step in y.
    p = b2 * (2 * x + 1) * (2 * x + 1) + 4 * a2 * (y - 1) * (y - 1) - 4 * a2 * b2;
    while y > 0 {
        y -= 1;
        py -= 2 * a2;
        if p > 0 {
            p += 4 * (a2 - py);
        } else {
            x += 1;
            px += 2 * b2;
            p += 4 * (a2 - py + px);
        }
        visit(x, y);
    }
}

/// Draws the outline of an axis-aligned ellipse with horizontal radius `rx` and vertical radius
/// `ry`. A zero radius collapses the ellipse to a line or a point.
pub fn ellipse<P: Pixel>(image: &mut Image<P>, cx: i32, cy: i32, rx: u32, ry: u32, color: P) {
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    ellipse_quadrant(rx, ry, |x, y| {
        span(image, cx + x, cx + x, cy + y, color);
        span(image, cx - x, cx - x, cy + y, color);
        span(image, cx + x, cx + x, cy - y, color);
        span(image, cx - x, cx - x, cy - y, color);
    });
}

/// Draws a filled ellipse covering the same pixels as [`ellipse`] plus its interior.
pub fn fill_ellipse<P: Pixel>(image: &mut Image<P>, cx: i32, cy: i32, rx: u32, ry: u32, color: P) {
    let (cx, cy) = (i64::from(cx), i64::from(cy));
    ellipse_quadrant(rx, ry, |x, y| {
        span(image, cx - x, cx + x, cy + y, color);
        span(image, cx - x, cx + x, cy - y, color);
    });
}
