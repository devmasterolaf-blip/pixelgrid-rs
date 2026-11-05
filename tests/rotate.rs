mod common;

use common::{gray, gray_rows, rgb_pattern};
use pixelgrid::transform::{rotate, rotate180, rotate270, rotate90, Interpolation};
use pixelgrid::{Gray, Image, Rgb};

const BG: Gray = Gray(9);

#[test]
fn right_angles_with_expand_are_exact() {
    let img = rgb_pattern(4, 3);
    for interpolation in [Interpolation::Nearest, Interpolation::Bilinear, Interpolation::Bicubic] {
        assert_eq!(rotate(&img, 0.0, interpolation, true, Rgb::BLACK), img);
        assert_eq!(rotate(&img, 90.0, interpolation, true, Rgb::BLACK), rotate90(&img));
        assert_eq!(rotate(&img, 180.0, interpolation, true, Rgb::BLACK), rotate180(&img));
        assert_eq!(rotate(&img, 270.0, interpolation, true, Rgb::BLACK), rotate270(&img));
    }
}

#[test]
fn angles_wrap_around() {
    let img = rgb_pattern(4, 3);
    let turn = |degrees| rotate(&img, degrees, Interpolation::Nearest, true, Rgb::BLACK);
    assert_eq!(turn(-90.0), rotate270(&img));
    assert_eq!(turn(450.0), rotate90(&img));
    assert_eq!(turn(-180.0), rotate180(&img));
    assert_eq!(turn(720.0), img);
}

#[test]
fn quarter_turn_without_expand_keeps_the_frame() {
    // A 3x1 strip turned upright inside its own 3x1 frame: only the centre pixel remains.
    let strip = gray(&[&[1, 2, 3]]);
    let out = rotate(&strip, 90.0, Interpolation::Nearest, false, BG);
    assert_eq!(gray_rows(&out), [[9, 2, 9]]);
    // Square images turn exactly even without expanding.
    let square = gray(&[&[1, 2], &[3, 4]]);
    assert_eq!(rotate(&square, 90.0, Interpolation::Bilinear, false, BG), rotate90(&square));
    assert_eq!(rotate(&strip, 180.0, Interpolation::Bilinear, false, BG), rotate180(&strip));
}

#[test]
fn quarter_turn_of_a_cross_inside_a_wider_frame() {
    let img = gray(&[&[0, 0, 1, 0, 0], &[5, 5, 2, 6, 6], &[0, 0, 3, 0, 0]]);
    let out = rotate(&img, 90.0, Interpolation::Nearest, false, BG);
    // The vertical bar 1,2,3 becomes horizontal reading 3,2,1; the horizontal bar becomes
    // vertical and only its middle three pixels fit: 5 on top, 6 at the bottom.
    assert_eq!(gray_rows(&out), [[9, 0, 5, 0, 9], [9, 3, 2, 1, 9], [9, 0, 6, 0, 9]]);
}

#[test]
fn forty_five_degrees_moves_corners_to_edge_midpoints() {
    let img = gray(&[&[1, 0, 2], &[0, 5, 0], &[4, 0, 3]]);
    let out = rotate(&img, 45.0, Interpolation::Nearest, true, BG);
    // 3 * sqrt(2) = 4.24, so the bounding box is 5x5.
    assert_eq!(out.dimensions(), (5, 5));
    assert_eq!(out[(2, 2)], Gray(5));
    // Clockwise: top-left goes to the top, top-right to the right, and so on.
    assert_eq!(out[(2, 0)], Gray(1));
    assert_eq!(out[(4, 2)], Gray(2));
    assert_eq!(out[(2, 4)], Gray(3));
    assert_eq!(out[(0, 2)], Gray(4));
    for corner in [(0, 0), (4, 0), (0, 4), (4, 4)] {
        assert_eq!(out[corner], BG);
    }
}

#[test]
fn expand_computes_the_bounding_box() {
    let img: Image<Gray> = Image::filled(4, 2, Gray(1));
    // 30 degrees: 4 * 0.866 + 2 * 0.5 = 4.46 wide, 4 * 0.5 + 2 * 0.866 = 3.73 high.
    assert_eq!(rotate(&img, 30.0, Interpolation::Bilinear, true, BG).dimensions(), (5, 4));
    assert_eq!(rotate(&img, 30.0, Interpolation::Bilinear, false, BG).dimensions(), (4, 2));
    // A constant image only ever produces its own value or the background.
    let out = rotate(&img, 30.0, Interpolation::Bicubic, true, BG);
    assert!(out.pixels().all(|p| *p == Gray(1) || *p == BG));
    assert_eq!(out[(0, 0)], BG);
    assert_eq!(out[(2, 2)], Gray(1));
}

#[test]
fn small_angle_keeps_the_centre_region() {
    let img = gray(&[&[10, 20, 30], &[40, 50, 60], &[70, 80, 90]]);
    let out = rotate(&img, 1.0, Interpolation::Nearest, false, BG);
    assert_eq!(out, img);
    let out = rotate(&img, 1.0, Interpolation::Bilinear, false, BG);
    assert_eq!(out[(1, 1)], Gray(50));
}

#[test]
fn empty_image_is_returned_unchanged() {
    let empty: Image<Gray> = Image::new(0, 2);
    assert_eq!(rotate(&empty, 33.0, Interpolation::Nearest, true, BG).dimensions(), (0, 2));
}

#[test]
#[should_panic(expected = "rotation angle must be finite")]
fn non_finite_angle_panics() {
    let img = gray(&[&[1]]);
    rotate(&img, f64::NAN, Interpolation::Nearest, true, BG);
}
