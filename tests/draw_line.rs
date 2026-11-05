mod common;

use common::picture;
use pixelgrid::draw::{fill_rect, line, rect, thick_line};
use pixelgrid::{Gray, Image};

const ON: Gray = Gray(255);

fn canvas(width: u32, height: u32) -> Image<Gray> {
    Image::new(width, height)
}

#[test]
fn horizontal_vertical_and_single_point_lines() {
    let mut img = canvas(5, 4);
    line(&mut img, 1, 0, 3, 0, ON);
    line(&mut img, 0, 1, 0, 3, ON);
    line(&mut img, 4, 3, 4, 3, ON);
    assert_eq!(picture(&img), [".###.", "#....", "#....", "#...#"]);
}

#[test]
fn diagonal_line_includes_both_end_points() {
    let mut img = canvas(4, 4);
    line(&mut img, 0, 0, 3, 3, ON);
    assert_eq!(picture(&img), ["#...", ".#..", "..#.", "...#"]);
}

#[test]
fn shallow_line_steps_match_bresenham() {
    // (0,0) -> (4,2): the error term gives y = 0, 1, 1, 2, 2
    let mut img = canvas(5, 3);
    line(&mut img, 0, 0, 4, 2, ON);
    assert_eq!(picture(&img), ["#....", ".##..", "...##"]);
}

#[test]
fn steep_line_steps_match_bresenham() {
    let mut img = canvas(3, 5);
    line(&mut img, 0, 0, 2, 4, ON);
    assert_eq!(picture(&img), ["#..", ".#.", ".#.", "..#", "..#"]);
}

#[test]
fn reversed_end_points_cover_the_same_pixel_count() {
    for (x0, y0, x1, y1) in [(0, 0, 6, 2), (6, 0, 0, 3), (1, 6, 4, 0), (0, 3, 6, 3)] {
        let mut forward = canvas(7, 7);
        let mut backward = canvas(7, 7);
        line(&mut forward, x0, y0, x1, y1, ON);
        line(&mut backward, x1, y1, x0, y0, ON);
        let count = |img: &Image<Gray>| img.pixels().filter(|p| p.0 != 0).count();
        let expected = ((x1 - x0).abs().max((y1 - y0).abs()) + 1) as usize;
        assert_eq!(count(&forward), expected);
        assert_eq!(count(&backward), expected);
        assert_eq!(forward[(x0 as u32, y0 as u32)], ON);
        assert_eq!(backward[(x0 as u32, y0 as u32)], ON);
    }
}

#[test]
fn lines_are_clipped_to_the_image() {
    let mut img = canvas(3, 3);
    line(&mut img, -2, -2, 5, 5, ON);
    assert_eq!(picture(&img), ["#..", ".#.", "..#"]);
    let mut img = canvas(3, 3);
    line(&mut img, -5, 1, 10, 1, ON);
    assert_eq!(picture(&img), ["...", "###", "..."]);
    let mut img = canvas(3, 3);
    line(&mut img, 5, -3, 9, 7, ON);
    assert_eq!(picture(&img), ["...", "...", "..."], "a line completely outside draws nothing");
}

#[test]
fn thick_horizontal_line_grows_vertically() {
    let mut img = canvas(5, 5);
    thick_line(&mut img, 1, 2, 3, 2, 3, ON);
    assert_eq!(picture(&img), [".....", ".###.", ".###.", ".###.", "....."]);
}

#[test]
fn thick_vertical_line_grows_horizontally() {
    let mut img = canvas(5, 4);
    thick_line(&mut img, 2, 0, 2, 3, 2, ON);
    // Even thickness: the extra column goes to the right of the centre line.
    assert_eq!(picture(&img), ["..##.", "..##.", "..##.", "..##."]);
}

#[test]
fn thick_diagonal_line() {
    let mut img = canvas(5, 5);
    thick_line(&mut img, 0, 0, 4, 4, 3, ON);
    // Slope 1 counts as shallow, so every column gets a three-pixel vertical brush.
    assert_eq!(picture(&img), ["##...", "###..", ".###.", "..###", "...##"]);
}

#[test]
fn thickness_zero_and_one() {
    let mut img = canvas(4, 2);
    thick_line(&mut img, 0, 0, 3, 1, 0, ON);
    assert!(img.pixels().all(|p| p.0 == 0));
    let mut thin = canvas(4, 2);
    line(&mut thin, 0, 0, 3, 1, ON);
    thick_line(&mut img, 0, 0, 3, 1, 1, ON);
    assert_eq!(img, thin);
}

#[test]
fn fill_rect_covers_exactly_width_by_height() {
    let mut img = canvas(5, 4);
    fill_rect(&mut img, 1, 1, 3, 2, ON);
    assert_eq!(picture(&img), [".....", ".###.", ".###.", "....."]);
}

#[test]
fn fill_rect_is_clipped_on_every_side() {
    let mut img = canvas(3, 3);
    fill_rect(&mut img, -1, -1, 2, 2, ON);
    assert_eq!(picture(&img), ["#..", "...", "..."]);
    fill_rect(&mut img, 2, 2, 10, 10, ON);
    assert_eq!(picture(&img), ["#..", "...", "..#"]);
    fill_rect(&mut img, -10, -10, 100, 100, Gray(0));
    assert!(img.pixels().all(|p| p.0 == 0));
    fill_rect(&mut img, 1, 1, 0, 5, ON);
    fill_rect(&mut img, 1, 1, 5, 0, ON);
    fill_rect(&mut img, 3, 0, 5, 5, ON);
    fill_rect(&mut img, i32::MAX, i32::MAX, u32::MAX, u32::MAX, ON);
    assert!(img.pixels().all(|p| p.0 == 0), "empty and outside rectangles draw nothing");
}

#[test]
fn rect_outline_lies_inside_the_rectangle() {
    let mut img = canvas(6, 5);
    rect(&mut img, 1, 1, 4, 3, ON);
    assert_eq!(picture(&img), ["......", ".####.", ".#..#.", ".####.", "......"]);
}

#[test]
fn thin_and_clipped_rect_outlines() {
    let mut img = canvas(4, 3);
    rect(&mut img, 0, 0, 1, 3, ON);
    rect(&mut img, 2, 1, 2, 1, ON);
    assert_eq!(picture(&img), ["#...", "#.##", "#..."]);
    let mut img = canvas(3, 3);
    rect(&mut img, -1, -1, 3, 3, ON);
    assert_eq!(picture(&img), [".#.", "##.", "..."]);
    rect(&mut img, 0, 0, 0, 3, Gray(0));
    assert_eq!(picture(&img), [".#.", "##.", "..."], "zero-sized outlines draw nothing");
}
