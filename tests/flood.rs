mod common;

use common::{gray, gray_rows, mask, picture};
use pixelgrid::draw::flood_fill;
use pixelgrid::{Connectivity, Gray, Image, ImageError, Rgb, Rgba};

#[test]
fn fills_a_closed_region_only() {
    let mut img = mask(&["#####", "#...#", "#.#.#", "#...#", "#####"]);
    let n = flood_fill(&mut img, 1, 1, Gray(7), 0, Connectivity::Four).unwrap();
    assert_eq!(n, 8);
    let expected: Vec<Vec<u8>> = vec![
        vec![255, 255, 255, 255, 255],
        vec![255, 7, 7, 7, 255],
        vec![255, 7, 255, 7, 255],
        vec![255, 7, 7, 7, 255],
        vec![255, 255, 255, 255, 255],
    ];
    assert_eq!(gray_rows(&img), expected);
}

#[test]
fn diagonal_gaps_leak_only_with_eight_connectivity() {
    let rows = ["..#.", ".#..", "#...", "...."];
    let mut four = mask(&rows);
    assert_eq!(flood_fill(&mut four, 0, 0, Gray(255), 0, Connectivity::Four).unwrap(), 3);
    assert_eq!(picture(&four), ["###.", "##..", "#...", "...."]);
    let mut eight = mask(&rows);
    assert_eq!(flood_fill(&mut eight, 0, 0, Gray(255), 0, Connectivity::Eight).unwrap(), 13);
    assert!(eight.pixels().all(|p| p.0 == 255));
}

#[test]
fn eight_connectivity_follows_a_diagonal_line() {
    let mut img = mask(&["#...", ".#..", "..#.", "...#"]);
    assert_eq!(flood_fill(&mut img, 0, 0, Gray(9), 0, Connectivity::Four).unwrap(), 1);
    let mut img = mask(&["#...", ".#..", "..#.", "...#"]);
    assert_eq!(flood_fill(&mut img, 0, 0, Gray(9), 0, Connectivity::Eight).unwrap(), 4);
    assert_eq!(img[(3, 3)], Gray(9));
    assert_eq!(img[(1, 0)], Gray(0));
}

#[test]
fn tolerance_is_measured_against_the_seed_colour() {
    // A ramp in steps of 10: each neighbour is close to the previous pixel, but the fill stops
    // once the difference to the seed exceeds the tolerance.
    let mut img = gray(&[&[100, 110, 120, 130, 140, 150]]);
    assert_eq!(flood_fill(&mut img, 0, 0, Gray(0), 25, Connectivity::Four).unwrap(), 3);
    assert_eq!(gray_rows(&img), [[0, 0, 0, 130, 140, 150]]);
    // The tolerance is inclusive and works in both directions.
    let mut img = gray(&[&[100, 110, 120, 130, 140, 150]]);
    assert_eq!(flood_fill(&mut img, 2, 0, Gray(1), 20, Connectivity::Four).unwrap(), 5);
    assert_eq!(gray_rows(&img), [[1, 1, 1, 1, 1, 150]]);
}

#[test]
fn tolerance_uses_the_largest_channel_difference() {
    let near = Rgb::new(10, 10, 14);
    let far = Rgb::new(10, 10, 16);
    let mut img = Image::from_vec(3, 1, vec![Rgb::new(10, 10, 10), near, far]).unwrap();
    let n = flood_fill(&mut img, 0, 0, Rgb::WHITE, 5, Connectivity::Four).unwrap();
    assert_eq!(n, 2);
    assert_eq!(img.as_slice(), [Rgb::WHITE, Rgb::WHITE, far]);
}

#[test]
fn alpha_counts_as_a_channel() {
    let mut img = Image::from_vec(2, 1, vec![Rgba::new(5, 5, 5, 255), Rgba::new(5, 5, 5, 0)]).unwrap();
    let n = flood_fill(&mut img, 0, 0, Rgba::WHITE, 0, Connectivity::Four).unwrap();
    assert_eq!(n, 1);
    assert_eq!(img[(1, 0)], Rgba::new(5, 5, 5, 0));
}

#[test]
fn filling_with_the_same_colour_terminates() {
    let mut img: Image<Gray> = Image::filled(4, 3, Gray(8));
    assert_eq!(flood_fill(&mut img, 2, 1, Gray(8), 0, Connectivity::Eight).unwrap(), 12);
    assert!(img.pixels().all(|p| p.0 == 8));
}

#[test]
fn full_tolerance_fills_everything() {
    let mut img = gray(&[&[0, 255], &[13, 200]]);
    assert_eq!(flood_fill(&mut img, 1, 1, Gray(3), 255, Connectivity::Four).unwrap(), 4);
    assert_eq!(gray_rows(&img), [[3, 3], [3, 3]]);
}

#[test]
fn seed_outside_the_image_is_an_error() {
    let mut img: Image<Gray> = Image::new(2, 2);
    let err = flood_fill(&mut img, 2, 0, Gray(1), 0, Connectivity::Four).unwrap_err();
    assert_eq!(err, ImageError::OutOfBounds { x: 2, y: 0, width: 2, height: 2 });
    let mut empty: Image<Gray> = Image::new(0, 0);
    assert!(flood_fill(&mut empty, 0, 0, Gray(1), 0, Connectivity::Four).is_err());
}

#[test]
fn connectivity_offsets_and_names() {
    assert_eq!(Connectivity::Four.offsets().len(), 4);
    assert_eq!(Connectivity::Eight.offsets().len(), 8);
    assert!(Connectivity::Four.offsets().iter().all(|(dx, dy)| dx.abs() + dy.abs() == 1));
    assert_eq!(Connectivity::from_name("4"), Some(Connectivity::Four));
    assert_eq!(Connectivity::from_name("8"), Some(Connectivity::Eight));
    assert_eq!(Connectivity::from_name("6"), None);
}
