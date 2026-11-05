mod common;

use common::picture;
use pixelgrid::draw::{circle, ellipse, fill_circle, fill_ellipse};
use pixelgrid::{Gray, Image};

const ON: Gray = Gray(255);

fn canvas(width: u32, height: u32) -> Image<Gray> {
    Image::new(width, height)
}

/// Every row of `filled` must cover exactly the columns between the leftmost and rightmost
/// outline pixel of that row.
fn assert_fill_matches_outline(outline: &Image<Gray>, filled: &Image<Gray>) {
    for (y, (o, f)) in outline.rows().zip(filled.rows()).enumerate() {
        let first = o.iter().position(|p| p.0 != 0);
        let last = o.iter().rposition(|p| p.0 != 0);
        for (x, p) in f.iter().enumerate() {
            let inside = matches!((first, last), (Some(a), Some(b)) if a <= x && x <= b);
            assert_eq!(p.0 != 0, inside, "pixel ({x}, {y})");
        }
    }
}

#[test]
fn circle_radius_zero_and_one() {
    let mut img = canvas(3, 3);
    circle(&mut img, 1, 1, 0, ON);
    assert_eq!(picture(&img), ["...", ".#.", "..."]);
    circle(&mut img, 1, 1, 1, ON);
    assert_eq!(picture(&img), [".#.", "###", ".#."]);
}

#[test]
fn circle_radius_two_and_three() {
    let mut img = canvas(5, 5);
    circle(&mut img, 2, 2, 2, ON);
    assert_eq!(picture(&img), [".###.", "#...#", "#...#", "#...#", ".###."]);
    let mut img = canvas(7, 7);
    circle(&mut img, 3, 3, 3, ON);
    assert_eq!(picture(&img), ["..###..", ".#...#.", "#.....#", "#.....#", "#.....#", ".#...#.", "..###.."]);
}

#[test]
fn circle_radius_four() {
    let mut img = canvas(9, 9);
    circle(&mut img, 4, 4, 4, ON);
    let expected = [
        "...###...",
        ".##...##.",
        ".#.....#.",
        "#.......#",
        "#.......#",
        "#.......#",
        ".#.....#.",
        ".##...##.",
        "...###...",
    ];
    assert_eq!(picture(&img), expected);
}

#[test]
fn filled_circles() {
    let mut img = canvas(5, 5);
    fill_circle(&mut img, 2, 2, 2, ON);
    assert_eq!(picture(&img), [".###.", "#####", "#####", "#####", ".###."]);
    let mut img = canvas(3, 3);
    fill_circle(&mut img, 1, 1, 0, ON);
    assert_eq!(picture(&img), ["...", ".#.", "..."]);
}

#[test]
fn filled_circle_covers_outline_and_interior() {
    for r in 0..=9 {
        let size = 2 * r + 3;
        let c = r as i32 + 1;
        let (mut outline, mut filled) = (canvas(size, size), canvas(size, size));
        circle(&mut outline, c, c, r, ON);
        fill_circle(&mut filled, c, c, r, ON);
        assert_fill_matches_outline(&outline, &filled);
    }
}

#[test]
fn circles_touch_their_radius_and_stay_inside_it() {
    for r in 1..=12u32 {
        let size = 2 * r + 3;
        let c = r + 1;
        let mut img = canvas(size, size);
        circle(&mut img, c as i32, c as i32, r, ON);
        for (x, y) in [(c - r, c), (c + r, c), (c, c - r), (c, c + r)] {
            assert_eq!(img[(x, y)], ON, "radius {r}");
        }
        assert!(img.row(0).iter().all(|p| p.0 == 0) && img.row(size - 1).iter().all(|p| p.0 == 0));
        // Symmetric under a horizontal and a vertical flip.
        for (x, y, p) in img.enumerate_pixels() {
            assert_eq!(p, img[(size - 1 - x, y)]);
            assert_eq!(p, img[(x, size - 1 - y)]);
        }
    }
}

#[test]
fn circles_are_clipped() {
    let mut img = canvas(3, 3);
    circle(&mut img, 0, 0, 2, ON);
    assert_eq!(picture(&img), ["..#", "..#", "##."]);
    let mut img = canvas(3, 3);
    fill_circle(&mut img, 0, 0, 2, ON);
    assert_eq!(picture(&img), ["###", "###", "##."]);
    let mut img = canvas(3, 3);
    circle(&mut img, -10, 1, 3, ON);
    fill_circle(&mut img, 1, 20, 5, ON);
    assert!(img.pixels().all(|p| p.0 == 0));
}

#[test]
fn ellipse_five_by_three() {
    let mut img = canvas(11, 7);
    ellipse(&mut img, 5, 3, 5, 3, ON);
    let expected = [
        "...#####...",
        ".##.....##.",
        "#.........#",
        "#.........#",
        "#.........#",
        ".##.....##.",
        "...#####...",
    ];
    assert_eq!(picture(&img), expected);
    let mut filled = canvas(11, 7);
    fill_ellipse(&mut filled, 5, 3, 5, 3, ON);
    let expected = [
        "...#####...",
        ".#########.",
        "###########",
        "###########",
        "###########",
        ".#########.",
        "...#####...",
    ];
    assert_eq!(picture(&filled), expected);
}

#[test]
fn tall_ellipse() {
    let mut img = canvas(3, 7);
    ellipse(&mut img, 1, 3, 1, 3, ON);
    assert_eq!(picture(&img), [".#.", "#.#", "#.#", "#.#", "#.#", "#.#", ".#."]);
}

#[test]
fn degenerate_ellipses_are_lines_and_points() {
    let mut img = canvas(5, 5);
    ellipse(&mut img, 2, 2, 2, 0, ON);
    assert_eq!(picture(&img), [".....", ".....", "#####", ".....", "....."]);
    let mut img = canvas(5, 5);
    ellipse(&mut img, 2, 2, 0, 2, ON);
    assert_eq!(picture(&img), ["..#..", "..#..", "..#..", "..#..", "..#.."]);
    let mut img = canvas(5, 5);
    fill_ellipse(&mut img, 2, 2, 0, 0, ON);
    assert_eq!(picture(&img), [".....", ".....", "..#..", ".....", "....."]);
}

#[test]
fn filled_ellipse_covers_outline_and_interior() {
    for (rx, ry) in [(1, 1), (4, 2), (2, 4), (7, 3), (3, 8), (6, 1), (9, 9), (0, 3), (3, 0)] {
        let (w, h) = (2 * rx + 3, 2 * ry + 3);
        let (cx, cy) = (rx as i32 + 1, ry as i32 + 1);
        let (mut outline, mut filled) = (canvas(w, h), canvas(w, h));
        ellipse(&mut outline, cx, cy, rx, ry, ON);
        fill_ellipse(&mut filled, cx, cy, rx, ry, ON);
        assert_fill_matches_outline(&outline, &filled);
        // The outline reaches the four extreme points and nothing beyond them.
        assert_eq!(outline[(1, ry + 1)], ON);
        assert_eq!(outline[(2 * rx + 1, ry + 1)], ON);
        assert_eq!(outline[(rx + 1, 1)], ON);
        assert_eq!(outline[(rx + 1, 2 * ry + 1)], ON);
        assert!(outline.row(0).iter().all(|p| p.0 == 0));
        assert!(outline.rows().all(|r| r[0].0 == 0 && r[w as usize - 1].0 == 0));
    }
}

#[test]
fn ellipses_are_clipped() {
    let mut img = canvas(4, 3);
    fill_ellipse(&mut img, 0, 0, 5, 3, ON);
    // The lower-right quadrant of the 5x3 ellipse above, cut to 4x3.
    assert_eq!(picture(&img), ["####", "####", "####"]);
    let mut img = canvas(4, 3);
    ellipse(&mut img, 5, 3, 5, 3, ON);
    assert_eq!(picture(&img), ["...#", ".##.", "#..."]);
    let mut img = canvas(4, 3);
    ellipse(&mut img, 100, 100, 5, 3, ON);
    assert!(img.pixels().all(|p| p.0 == 0));
}
