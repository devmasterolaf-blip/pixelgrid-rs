//! Polygon outlines and scanline fill.

use super::line::line;
use super::span;
use crate::image::Image;
use crate::pixel::Pixel;

/// Decides which regions of a self-intersecting or multiply wound polygon are inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FillRule {
    /// A point is inside when a ray from it crosses the outline an odd number of times.
    EvenOdd,
    /// A point is inside when the outline winds around it a non-zero number of times.
    NonZero,
}

impl FillRule {
    /// Parses `"evenodd"` or `"nonzero"`.
    pub fn from_name(name: &str) -> Option<FillRule> {
        match name {
            "evenodd" => Some(FillRule::EvenOdd),
            "nonzero" => Some(FillRule::NonZero),
            _ => None,
        }
    }
}

/// Draws the closed outline through `points` with one-pixel Bresenham lines.
///
/// A single point draws one pixel; an empty slice draws nothing.
pub fn polygon<P: Pixel>(image: &mut Image<P>, points: &[(i32, i32)], color: P) {
    for (i, &(x0, y0)) in points.iter().enumerate() {
        let (x1, y1) = points[(i + 1) % points.len()];
        line(image, x0, y0, x1, y1, color);
    }
}

/// The first pixel column whose centre is at or right of the point where the edge
/// `(x0, y0) -> (x1, y1)` crosses the centre line of pixel row `y`.
///
/// The crossing is `x0 + (2y + 1 - 2 * y0) * dx / (2 * dy)`; the result is
/// `ceil(crossing - 1/2)`, computed exactly in integers.
fn crossing_column(x0: i64, y0: i64, x1: i64, y1: i64, y: i64) -> i64 {
    let mut den = i128::from(2 * (y1 - y0));
    let mut num = i128::from(2 * y + 1 - 2 * y0) * i128::from(x1 - x0);
    if den < 0 {
        den = -den;
        num = -num;
    }
    // crossing - 1/2 = (2 * x0 * den + 2 * num - den) / (2 * den)
    let top = 2 * i128::from(x0) * den + 2 * num - den;
    let bottom = 2 * den;
    (-((-top).div_euclid(bottom))) as i64
}

/// Fills the polygon through `points`, which is closed implicitly.
///
/// Vertices sit on pixel corners: the polygon `(0,0) (2,0) (2,2) (0,2)` covers exactly the four
/// pixels of a 2x2 square. A pixel is filled when its centre is inside the polygon according to
/// `rule`; centres exactly on a left or top edge count as inside, those on a right or bottom edge
/// as outside, so polygons sharing an edge never fill a pixel twice or leave a gap. Fewer than
/// three points fill nothing.
pub fn fill_polygon<P: Pixel>(
    image: &mut Image<P>,
    points: &[(i32, i32)],
    rule: FillRule,
    color: P,
) {
    if points.len() < 3 || image.is_empty() {
        return;
    }
    let top = points.iter().map(|p| i64::from(p.1)).min().unwrap_or(0).max(0);
    let bottom = points.iter().map(|p| i64::from(p.1)).max().unwrap_or(0).min(i64::from(image.height()));
    let mut crossings: Vec<(i64, i32)> = Vec::new();
    for y in top..bottom {
        crossings.clear();
        for (i, &(x0, y0)) in points.iter().enumerate() {
            let (x1, y1) = points[(i + 1) % points.len()];
            let (x0, y0, x1, y1) = (i64::from(x0), i64::from(y0), i64::from(x1), i64::from(y1));
            if y0.min(y1) <= y && y < y0.max(y1) {
                let direction = if y1 > y0 { 1 } else { -1 };
                crossings.push((crossing_column(x0, y0, x1, y1, y), direction));
            }
        }
        crossings.sort_unstable_by_key(|c| c.0);
        let mut winding = 0;
        for pair in crossings.windows(2) {
            winding += pair[0].1;
            let inside = match rule {
                FillRule::EvenOdd => winding % 2 != 0,
                FillRule::NonZero => winding != 0,
            };
            if inside {
                span(image, pair[0].0, pair[1].0 - 1, y, color);
            }
        }
    }
}
