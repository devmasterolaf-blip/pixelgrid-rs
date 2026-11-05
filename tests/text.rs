mod common;

use common::picture;
use pixelgrid::draw::{draw_text, glyph, text_size, GLYPH_HEIGHT, GLYPH_WIDTH};
use pixelgrid::{Gray, Image};

const ON: Gray = Gray(255);

fn render(text: &str, scale: u32) -> Vec<String> {
    let (w, h) = text_size(text, scale);
    let mut img = Image::new(w, h);
    draw_text(&mut img, 0, 0, text, ON, scale);
    picture(&img)
}

#[test]
fn glyph_rows_are_three_bits_wide() {
    assert_eq!(glyph('0'), Some([0b111, 0b101, 0b101, 0b101, 0b111]));
    assert_eq!(glyph('1'), Some([0b010, 0b110, 0b010, 0b010, 0b111]));
    assert_eq!(glyph('T'), Some([0b111, 0b010, 0b010, 0b010, 0b010]));
    assert_eq!(glyph(' '), Some([0; 5]));
    assert_eq!(glyph('~'), None);
    assert_eq!(glyph('é'), None);
    assert_eq!((GLYPH_WIDTH, GLYPH_HEIGHT), (3, 5));
}

#[test]
fn lower_case_uses_upper_case_shapes() {
    for (lower, upper) in ('a'..='z').zip('A'..='Z') {
        assert!(glyph(upper).is_some(), "{upper}");
        assert_eq!(glyph(lower), glyph(upper));
    }
}

#[test]
fn glyphs_are_distinct() {
    let chars: Vec<char> = ('0'..='9').chain('A'..='Z').chain(".,:-+=!?/()%#_'".chars()).collect();
    for (i, a) in chars.iter().enumerate() {
        let ga = glyph(*a).unwrap_or_else(|| panic!("no glyph for {a}"));
        assert_ne!(ga, [0; 5], "{a} is blank");
        for b in &chars[i + 1..] {
            assert_ne!(Some(ga), glyph(*b), "{a} and {b} look the same");
        }
    }
}

#[test]
fn draws_a_word() {
    let expected = [
        "#.#.###",
        "#.#..#.",
        "###..#.",
        "#.#..#.",
        "#.#.###",
    ];
    assert_eq!(render("HI", 1), expected);
    assert_eq!(render("hi", 1), expected);
}

#[test]
fn scale_enlarges_every_font_pixel() {
    let expected = [
        "..##..",
        "..##..",
        "####..",
        "####..",
        "..##..",
        "..##..",
        "..##..",
        "..##..",
        "######",
        "######",
    ];
    assert_eq!(render("1", 2), expected);
}

#[test]
fn newline_starts_a_new_row_of_text() {
    let expected = [
        "###....",
        "..#....",
        "..#....",
        "..#....",
        "..#....",
        ".......",
        "#.#.###",
        "#.#.#.#",
        "###.###",
        "..#.#.#",
        "..#.###",
    ];
    assert_eq!(render("7\n48", 1), expected);
}

#[test]
fn text_size_counts_gaps_between_but_not_after() {
    assert_eq!(text_size("", 1), (0, 0));
    assert_eq!(text_size("A", 1), (3, 5));
    assert_eq!(text_size("AB", 1), (7, 5));
    assert_eq!(text_size("ABC", 2), (22, 10));
    assert_eq!(text_size("A\nBCD\nE", 1), (11, 17));
    assert_eq!(text_size("A\n", 1), (3, 11));
    assert_eq!(text_size("ABC", 0), (0, 0));
}

#[test]
fn unknown_characters_are_solid_blocks() {
    assert_eq!(render("~", 1), ["###", "###", "###", "###", "###"]);
}

#[test]
fn text_is_clipped_and_leaves_the_background_alone() {
    let mut img: Image<Gray> = Image::filled(4, 4, Gray(7));
    draw_text(&mut img, -1, -1, "8", ON, 1);
    // The glyph 8 is ###, #.#, ###, #.#, ###; its first column and row are cut off.
    let rows: Vec<Vec<u8>> = img.rows().map(|r| r.iter().map(|p| p.0).collect()).collect();
    assert_eq!(rows, [[7, 255, 7, 7], [255, 255, 7, 7], [7, 255, 7, 7], [255, 255, 7, 7]]);
    let mut img: Image<Gray> = Image::new(3, 3);
    draw_text(&mut img, 50, 0, "X", ON, 1);
    draw_text(&mut img, 0, -50, "X", ON, 3);
    draw_text(&mut img, i32::MAX, i32::MAX, "XYZ\nXYZ", ON, 4);
    draw_text(&mut img, 0, 0, "X", ON, 0);
    assert!(img.pixels().all(|p| p.0 == 0));
}

#[test]
fn huge_scales_saturate_instead_of_overflowing() {
    assert_eq!(text_size("AB", u32::MAX), (u32::MAX, u32::MAX));
    assert_eq!(text_size("A", u32::MAX / 3), (u32::MAX / 3 * 3, u32::MAX));
    assert_eq!(text_size("A\nB", 1 << 30), (3 << 30, u32::MAX));

    // At a scale of 2^31 one font pixel is far larger than the image. The top-left font pixel
    // of T is set and covers everything; the top-left font pixel of A is not.
    let scale = 1 << 31;
    let mut img: Image<Gray> = Image::new(4, 4);
    draw_text(&mut img, 0, 0, "A", ON, scale);
    assert!(img.pixels().all(|p| p.0 == 0));
    draw_text(&mut img, 0, 0, "T", ON, scale);
    assert!(img.pixels().all(|p| p.0 == 255));
    // Shifted left by one font pixel, the middle column of A's top row lands on the image.
    let mut img: Image<Gray> = Image::new(4, 4);
    draw_text(&mut img, i32::MIN, 0, "A", ON, scale);
    assert!(img.pixels().all(|p| p.0 == 255));
    draw_text(&mut img, i32::MAX, i32::MAX, "XYZ\nXYZ", Gray(1), u32::MAX);
    assert!(img.pixels().all(|p| p.0 == 255));
}
