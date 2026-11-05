mod common;

use common::{gray, picture};
use pixelgrid::compare::{compare, compare_dyn, diff_mask, Diff};
use pixelgrid::{DynImage, Gray, Image, ImageError, Rgb, Rgba};

#[test]
fn identical_images_match() {
    let img = common::rgb_pattern(4, 3);
    let diff = compare(&img, &img, 0).unwrap();
    assert_eq!(diff, Diff { total: 12, differing: 0, max_delta: 0, first: None });
    assert!(diff.is_match());
}

#[test]
fn counts_differences_above_the_tolerance() {
    let a = gray(&[&[10, 20, 30], &[40, 50, 60]]);
    let b = gray(&[&[10, 22, 30], &[45, 50, 50]]);
    let exact = compare(&a, &b, 0).unwrap();
    assert_eq!(exact, Diff { total: 6, differing: 3, max_delta: 10, first: Some((1, 0)) });
    assert!(!exact.is_match());
    // The tolerance is inclusive: a difference of exactly 2 passes at tolerance 2.
    assert_eq!(compare(&a, &b, 2).unwrap(), Diff { total: 6, differing: 2, max_delta: 10, first: Some((0, 1)) });
    assert_eq!(compare(&a, &b, 5).unwrap().first, Some((2, 1)));
    let loose = compare(&a, &b, 10).unwrap();
    assert_eq!(loose, Diff { total: 6, differing: 0, max_delta: 10, first: None });
    assert!(loose.is_match());
}

#[test]
fn any_channel_can_make_a_pixel_differ() {
    let a = Image::from_vec(2, 1, vec![Rgba::new(1, 2, 3, 4), Rgba::new(1, 2, 3, 4)]).unwrap();
    let b = Image::from_vec(2, 1, vec![Rgba::new(1, 2, 3, 9), Rgba::new(1, 5, 3, 4)]).unwrap();
    let diff = compare(&a, &b, 3).unwrap();
    assert_eq!((diff.differing, diff.max_delta, diff.first), (1, 5, Some((0, 0))));
}

#[test]
fn sizes_must_agree() {
    let a = gray(&[&[1, 2, 3]]);
    let b = gray(&[&[1], &[2], &[3]]);
    let expected = ImageError::DimensionMismatch { left: (3, 1), right: (1, 3) };
    assert_eq!(compare(&a, &b, 255).unwrap_err(), expected);
    assert_eq!(diff_mask(&a, &b, 0).unwrap_err(), expected);
    assert_eq!(compare_dyn(&a.into(), &b.into(), 0).unwrap_err(), expected);
}

#[test]
fn diff_mask_marks_the_differing_pixels() {
    let a = gray(&[&[10, 20, 30], &[40, 50, 60]]);
    let b = gray(&[&[10, 22, 30], &[45, 50, 50]]);
    assert_eq!(picture(&diff_mask(&a, &b, 0).unwrap()), [".#.", "#.#"]);
    assert_eq!(picture(&diff_mask(&a, &b, 5).unwrap()), ["...", "..#"]);
    assert_eq!(picture(&diff_mask(&a, &a, 0).unwrap()), ["...", "..."]);
}

#[test]
fn empty_images_compare_equal() {
    let a: Image<Gray> = Image::new(0, 0);
    assert_eq!(compare(&a, &a, 0).unwrap(), Diff { total: 0, differing: 0, max_delta: 0, first: None });
}

#[test]
fn mixed_pixel_types_are_compared_as_rgba() {
    let grey: DynImage = gray(&[&[7, 9]]).into();
    let rgb: DynImage = Image::from_vec(2, 1, vec![Rgb::new(7, 7, 7), Rgb::new(9, 9, 12)]).unwrap().into();
    let diff = compare_dyn(&grey, &rgb, 0).unwrap();
    assert_eq!((diff.differing, diff.max_delta, diff.first), (1, 3, Some((1, 0))));
    // An opaque RGBA image equals its RGB counterpart; a translucent one does not.
    let opaque: DynImage = rgb.to_rgba().into();
    assert!(compare_dyn(&rgb, &opaque, 0).unwrap().is_match());
    let mut translucent = rgb.to_rgba();
    translucent[(0, 0)].a = 200;
    let diff = compare_dyn(&rgb, &translucent.into(), 0).unwrap();
    assert_eq!((diff.differing, diff.max_delta), (1, 55));
    assert!(compare_dyn(&grey, &grey, 0).unwrap().is_match());
}
