mod common;

use common::picture;
use pixelgrid::draw::{fill_polygon, polygon, FillRule};
use pixelgrid::{Gray, Image};

const ON: Gray = Gray(255);

fn filled(width: u32, height: u32, points: &[(i32, i32)], rule: FillRule) -> Vec<String> {
    let mut img = Image::new(width, height);
    fill_polygon(&mut img, points, rule, ON);
    picture(&img)
}

#[test]
fn square_covers_exactly_its_pixels() {
    let square = [(1, 1), (3, 1), (3, 3), (1, 3)];
    let expected = ["....", ".##.", ".##.", "...."];
    assert_eq!(filled(4, 4, &square, FillRule::EvenOdd), expected);
    assert_eq!(filled(4, 4, &square, FillRule::NonZero), expected);
}

#[test]
fn winding_direction_does_not_matter() {
    let clockwise = [(0, 0), (3, 0), (3, 2), (0, 2)];
    let counter: Vec<(i32, i32)> = clockwise.iter().rev().copied().collect();
    for rule in [FillRule::EvenOdd, FillRule::NonZero] {
        assert_eq!(filled(4, 3, &clockwise, rule), ["###.", "###.", "...."]);
        assert_eq!(filled(4, 3, &counter, rule), ["###.", "###.", "...."]);
    }
}

#[test]
fn right_triangle_fills_pixels_whose_centre_is_inside() {
    // The hypotenuse x + y = 4 passes through the centres of the anti-diagonal pixels, which
    // lie on a right edge and therefore stay empty.
    let triangle = [(0, 0), (4, 0), (0, 4)];
    assert_eq!(filled(4, 4, &triangle, FillRule::EvenOdd), ["###.", "##..", "#...", "...."]);
    // The mirrored triangle has the hypotenuse as its left edge and takes those pixels.
    let mirrored = [(4, 0), (4, 4), (0, 4)];
    assert_eq!(filled(4, 4, &mirrored, FillRule::EvenOdd), ["...#", "..##", ".###", "####"]);
}

#[test]
fn triangles_sharing_an_edge_tile_without_gap_or_overlap() {
    let mut img: Image<Gray> = Image::new(5, 5);
    for triangle in [[(0, 0), (5, 0), (0, 5)], [(5, 0), (5, 5), (0, 5)]] {
        let mut layer: Image<Gray> = Image::new(5, 5);
        fill_polygon(&mut layer, &triangle, FillRule::NonZero, Gray(1));
        for (total, part) in img.pixels_mut().zip(layer.pixels()) {
            total.0 += part.0;
        }
    }
    assert!(img.pixels().all(|p| p.0 == 1), "{:?}", common::gray_rows(&img));
}

#[test]
fn doubly_wound_square_depends_on_the_rule() {
    let twice = [(0, 0), (3, 0), (3, 3), (0, 3), (0, 0), (3, 0), (3, 3), (0, 3)];
    assert_eq!(filled(3, 3, &twice, FillRule::EvenOdd), ["...", "...", "..."]);
    assert_eq!(filled(3, 3, &twice, FillRule::NonZero), ["###", "###", "###"]);
}

#[test]
fn self_overlapping_path_depends_on_the_rule() {
    // An outline that runs once around the L-shaped region and a second time around the
    // 2x2 block at (2..4, 2..4).
    let path = [(0, 0), (6, 0), (6, 4), (2, 4), (2, 2), (4, 2), (4, 6), (0, 6)];
    let nonzero = ["######", "######", "######", "######", "####..", "####.."];
    let evenodd = ["######", "######", "##..##", "##..##", "####..", "####.."];
    assert_eq!(filled(6, 6, &path, FillRule::NonZero), nonzero);
    assert_eq!(filled(6, 6, &path, FillRule::EvenOdd), evenodd);
}

#[test]
fn concave_polygon() {
    // A "U" shape: the notch between the arms stays empty.
    let u = [(0, 0), (2, 0), (2, 3), (4, 3), (4, 0), (6, 0), (6, 5), (0, 5)];
    let expected = ["##..##", "##..##", "##..##", "######", "######"];
    assert_eq!(filled(6, 5, &u, FillRule::EvenOdd), expected);
    assert_eq!(filled(6, 5, &u, FillRule::NonZero), expected);
}

#[test]
fn diamond_with_sloped_edges() {
    // Every edge passes through pixel centres: the ones on the two left edges are filled, the
    // ones on the two right edges are not.
    let diamond = [(3, 0), (6, 3), (3, 6), (0, 3)];
    let expected = ["..#...", ".###..", "#####.", "#####.", ".###..", "..#..."];
    assert_eq!(filled(6, 6, &diamond, FillRule::EvenOdd), expected);
}

#[test]
fn fill_is_clipped_to_the_image() {
    let big = [(-5, -5), (2, -5), (2, 2), (-5, 2)];
    assert_eq!(filled(3, 3, &big, FillRule::NonZero), ["##.", "##.", "..."]);
    let outside = [(10, 10), (20, 10), (20, 20), (10, 20)];
    assert_eq!(filled(3, 3, &outside, FillRule::NonZero), ["...", "...", "..."]);
    let huge = [(i32::MIN, i32::MIN), (i32::MAX, i32::MIN), (i32::MAX, i32::MAX), (i32::MIN, i32::MAX)];
    assert_eq!(filled(2, 2, &huge, FillRule::EvenOdd), ["##", "##"]);
}

#[test]
fn degenerate_polygons_fill_nothing() {
    assert_eq!(filled(3, 3, &[], FillRule::EvenOdd), ["...", "...", "..."]);
    assert_eq!(filled(3, 3, &[(0, 0), (3, 3)], FillRule::EvenOdd), ["...", "...", "..."]);
    assert_eq!(filled(3, 3, &[(0, 1), (1, 1), (3, 1)], FillRule::NonZero), ["...", "...", "..."]);
    let mut empty: Image<Gray> = Image::new(0, 0);
    fill_polygon(&mut empty, &[(0, 0), (3, 0), (0, 3)], FillRule::EvenOdd, ON);
}

#[test]
fn outline_closes_the_path() {
    let mut img = Image::new(5, 4);
    polygon(&mut img, &[(0, 0), (4, 0), (4, 3), (0, 3)], ON);
    assert_eq!(picture(&img), ["#####", "#...#", "#...#", "#####"]);
    let mut img = Image::new(3, 3);
    polygon(&mut img, &[(1, 1)], ON);
    assert_eq!(picture(&img), ["...", ".#.", "..."]);
    polygon(&mut img, &[], ON);
    polygon(&mut img, &[(0, 0), (2, 2)], ON);
    assert_eq!(picture(&img), ["#..", ".#.", "..#"]);
}

#[test]
fn fill_rule_names() {
    assert_eq!(FillRule::from_name("evenodd"), Some(FillRule::EvenOdd));
    assert_eq!(FillRule::from_name("nonzero"), Some(FillRule::NonZero));
    assert_eq!(FillRule::from_name("winding"), None);
}
