//! A built-in 3x5 pixel font and text drawing.

use super::line::fill_rect;
use crate::image::Image;
use crate::pixel::Pixel;

/// Width of a glyph in font pixels.
pub const GLYPH_WIDTH: u32 = 3;
/// Height of a glyph in font pixels.
pub const GLYPH_HEIGHT: u32 = 5;

/// Glyphs as five octal digits, one per row from top to bottom; within a row the 4-bit is the
/// left pixel. `0o75557` is the digit zero: `###`, `#.#`, `#.#`, `#.#`, `###`.
const GLYPHS: &[(char, u16)] = &[
    (' ', 0o00000),
    ('0', 0o75557),
    ('1', 0o26227),
    ('2', 0o71747),
    ('3', 0o71717),
    ('4', 0o55711),
    ('5', 0o74717),
    ('6', 0o74757),
    ('7', 0o71111),
    ('8', 0o75757),
    ('9', 0o75717),
    ('A', 0o25755),
    ('B', 0o65656),
    ('C', 0o34443),
    ('D', 0o65556),
    ('E', 0o74647),
    ('F', 0o74644),
    ('G', 0o34553),
    ('H', 0o55755),
    ('I', 0o72227),
    ('J', 0o11152),
    ('K', 0o55655),
    ('L', 0o44447),
    ('M', 0o57755),
    ('N', 0o65555),
    ('O', 0o25552),
    ('P', 0o65644),
    ('Q', 0o25563),
    ('R', 0o65655),
    ('S', 0o34216),
    ('T', 0o72222),
    ('U', 0o55557),
    ('V', 0o55552),
    ('W', 0o55775),
    ('X', 0o55255),
    ('Y', 0o55222),
    ('Z', 0o71247),
    ('.', 0o00002),
    (',', 0o00024),
    (':', 0o02020),
    ('-', 0o00700),
    ('+', 0o02720),
    ('=', 0o07070),
    ('!', 0o22202),
    ('?', 0o71202),
    ('/', 0o11244),
    ('(', 0o12221),
    (')', 0o42224),
    ('%', 0o51245),
    ('#', 0o57575),
    ('_', 0o00007),
    ('\'', 0o22000),
];

/// What characters without a glyph are drawn as: a solid block.
const UNKNOWN: u16 = 0o77777;

/// Looks up the glyph for `c` as five rows of three bits (bit 2 is the left pixel).
///
/// The font has digits, upper-case letters and a little punctuation; lower-case letters use
/// the upper-case shapes. Returns `None` for characters the font does not cover.
pub fn glyph(c: char) -> Option<[u8; 5]> {
    let c = c.to_ascii_uppercase();
    let bits = GLYPHS.iter().find(|(g, _)| *g == c)?.1;
    Some(rows(bits))
}

fn rows(bits: u16) -> [u8; 5] {
    let mut out = [0u8; 5];
    for (i, row) in out.iter_mut().enumerate() {
        *row = ((bits >> (3 * (4 - i))) & 0o7) as u8;
    }
    out
}

/// The size in pixels that [`draw_text`] needs for `text` at `scale`.
///
/// Characters advance by `GLYPH_WIDTH + 1` and lines by `GLYPH_HEIGHT + 1` font pixels; the
/// trailing gap after the last column and row is not counted. Empty text measures `(0, 0)`.
pub fn text_size(text: &str, scale: u32) -> (u32, u32) {
    if text.is_empty() || scale == 0 {
        return (0, 0);
    }
    let lines = text.split('\n').count() as u32;
    let longest = text.split('\n').map(|l| l.chars().count()).max().unwrap_or(0) as u32;
    let width = (longest * (GLYPH_WIDTH + 1)).saturating_sub(1) * scale;
    let height = (lines * (GLYPH_HEIGHT + 1) - 1) * scale;
    (width, height)
}

/// Draws `text` with its top-left corner at `(x, y)`, clipped to the image.
///
/// Every font pixel becomes a `scale` x `scale` block. `\n` starts a new line at the original
/// `x`. Characters the font does not have are drawn as a solid block. Only the glyph pixels
/// are written, so the background shows through.
pub fn draw_text<P: Pixel>(image: &mut Image<P>, x: i32, y: i32, text: &str, color: P, scale: u32) {
    let advance = i64::from((GLYPH_WIDTH + 1) * scale);
    let line_height = i64::from((GLYPH_HEIGHT + 1) * scale);
    let clamp = |v: i64| v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
    for (line_index, line) in text.split('\n').enumerate() {
        let top = i64::from(y) + line_index as i64 * line_height;
        for (column, c) in line.chars().enumerate() {
            let left = i64::from(x) + column as i64 * advance;
            let glyph_rows = glyph(c).unwrap_or_else(|| rows(UNKNOWN));
            for (gy, bits) in glyph_rows.iter().enumerate() {
                for gx in 0..GLYPH_WIDTH {
                    if bits & (0b100 >> gx) != 0 {
                        let px = left + i64::from(gx * scale);
                        let py = top + gy as i64 * i64::from(scale);
                        fill_rect(image, clamp(px), clamp(py), scale, scale, color);
                    }
                }
            }
        }
    }
}
