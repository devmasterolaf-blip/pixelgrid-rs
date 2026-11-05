//! Copying one image onto another.

use crate::color::{blend, BlendMode};
use crate::image::Image;
use crate::pixel::Pixel;

/// Draws `src` onto `dst` with its top-left corner at `(x, y)`, combining pixels with `mode`.
///
/// The part of `src` that falls outside `dst` is skipped. The two images may have different
/// pixel types: every pair of pixels is widened to RGBA, blended and narrowed back to the
/// destination type, so an RGBA sprite can be composited onto an RGB or grey canvas. With
/// [`BlendMode::Replace`] onto a destination without alpha the source alpha is simply dropped.
pub fn blit<P: Pixel, Q: Pixel>(dst: &mut Image<P>, src: &Image<Q>, x: i32, y: i32, mode: BlendMode) {
    let (x, y) = (i64::from(x), i64::from(y));
    // The source columns and rows that land inside the destination.
    let sx0 = (-x).max(0);
    let sy0 = (-y).max(0);
    let sx1 = i64::from(src.width()).min(i64::from(dst.width()) - x);
    let sy1 = i64::from(src.height()).min(i64::from(dst.height()) - y);
    if sx0 >= sx1 || sy0 >= sy1 {
        return;
    }
    for sy in sy0..sy1 {
        let src_row = &src.row(sy as u32)[sx0 as usize..sx1 as usize];
        let dst_row = &mut dst.row_mut((sy + y) as u32)[(sx0 + x) as usize..(sx1 + x) as usize];
        for (d, s) in dst_row.iter_mut().zip(src_row) {
            *d = P::from_rgba(blend(mode, s.to_rgba(), d.to_rgba()));
        }
    }
}
