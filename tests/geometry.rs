mod common;

use common::{gray, gray_rows, rgb_pattern};
use pixelgrid::transform::{crop, flip_horizontal, flip_vertical, rotate180, rotate270, rotate90};
use pixelgrid::{Gray, Image, ImageError};

fn sample() -> Image<Gray> {
    gray(&[&[1, 2, 3], &[4, 5, 6]])
}

#[test]
fn crop_copies_a_rectangle() {
    let img = gray(&[&[1, 2, 3, 4], &[5, 6, 7, 8], &[9, 10, 11, 12]]);
    assert_eq!(gray_rows(&crop(&img, 1, 1, 2, 2).unwrap()), [[6, 7], [10, 11]]);
    assert_eq!(gray_rows(&crop(&img, 3, 0, 1, 3).unwrap()), [[4], [8], [12]]);
    assert_eq!(crop(&img, 0, 0, 4, 3).unwrap(), img);
}

#[test]
fn crop_may_be_empty_but_not_out_of_bounds() {
    let img = sample();
    assert_eq!(crop(&img, 3, 2, 0, 0).unwrap().dimensions(), (0, 0));
    assert_eq!(crop(&img, 1, 0, 0, 2).unwrap().dimensions(), (0, 2));
    let err = crop(&img, 2, 0, 2, 1).unwrap_err();
    let expected = ImageError::RegionOutOfBounds {
        x: 2,
        y: 0,
        width: 2,
        height: 1,
        image_width: 3,
        image_height: 2,
    };
    assert_eq!(err, expected);
    assert!(crop(&img, 0, 0, u32::MAX, 1).is_err());
    assert!(crop(&img, u32::MAX, u32::MAX, 2, 2).is_err());
}

#[test]
fn flips() {
    assert_eq!(gray_rows(&flip_horizontal(&sample())), [[3, 2, 1], [6, 5, 4]]);
    assert_eq!(gray_rows(&flip_vertical(&sample())), [[4, 5, 6], [1, 2, 3]]);
}

#[test]
fn quarter_turns() {
    // Clockwise: the left column becomes the top row, read bottom to top.
    assert_eq!(gray_rows(&rotate90(&sample())), [[4, 1], [5, 2], [6, 3]]);
    assert_eq!(gray_rows(&rotate180(&sample())), [[6, 5, 4], [3, 2, 1]]);
    assert_eq!(gray_rows(&rotate270(&sample())), [[3, 6], [2, 5], [1, 4]]);
}

#[test]
fn turns_compose() {
    let img = rgb_pattern(5, 3);
    assert_eq!(rotate90(&rotate90(&img)), rotate180(&img));
    assert_eq!(rotate90(&rotate180(&img)), rotate270(&img));
    assert_eq!(rotate90(&rotate270(&img)), img);
    assert_eq!(rotate270(&rotate90(&img)), img);
    assert_eq!(flip_horizontal(&flip_vertical(&img)), rotate180(&img));
    assert_eq!(flip_horizontal(&flip_horizontal(&img)), img);
    assert_eq!(flip_vertical(&flip_vertical(&img)), img);
    // A transpose is a quarter turn followed by a mirror.
    let transposed = flip_horizontal(&rotate90(&img));
    for (x, y, p) in img.enumerate_pixels() {
        assert_eq!(transposed[(y, x)], p);
    }
}

#[test]
fn empty_and_single_line_images() {
    let empty: Image<Gray> = Image::new(0, 3);
    assert_eq!(flip_horizontal(&empty).dimensions(), (0, 3));
    assert_eq!(flip_vertical(&empty).dimensions(), (0, 3));
    assert_eq!(rotate90(&empty).dimensions(), (3, 0));
    assert_eq!(rotate180(&empty).dimensions(), (0, 3));
    assert_eq!(rotate270(&empty).dimensions(), (3, 0));
    let row = gray(&[&[1, 2, 3]]);
    assert_eq!(gray_rows(&rotate90(&row)), [[1], [2], [3]]);
    assert_eq!(gray_rows(&rotate270(&row)), [[3], [2], [1]]);
}
