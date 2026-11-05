mod common;

use common::{gray, gray_rows};
use pixelgrid::{Gray, Image, ImageError, Rect, Rgb, Rgba};

#[test]
fn new_is_filled_with_the_default_pixel() {
    let img: Image<Rgba> = Image::new(3, 2);
    assert_eq!(img.dimensions(), (3, 2));
    assert!(img.pixels().all(|p| *p == Rgba::TRANSPARENT));
    assert_eq!(img.as_slice().len(), 6);
    assert_eq!(img.bounds(), Rect::new(0, 0, 3, 2));
}

#[test]
fn from_fn_visits_pixels_in_row_major_order() {
    let img = Image::from_fn(3, 2, |x, y| Gray((10 * y + x) as u8));
    assert_eq!(gray_rows(&img), vec![vec![0, 1, 2], vec![10, 11, 12]]);
}

#[test]
fn from_vec_checks_the_length() {
    let err = Image::from_vec(2, 2, vec![Gray(0); 3]).unwrap_err();
    assert_eq!(err, ImageError::BufferSize { width: 2, height: 2, expected: 4, actual: 3 });
    assert_eq!(err.to_string(), "a 2x2 image needs 4 buffer elements, got 3");
    assert!(Image::from_vec(2, 2, vec![Gray(0); 4]).is_ok());
}

#[test]
fn raw_bytes_round_trip() {
    let img: Image<Rgb> = Image::from_raw(2, 1, &[1, 2, 3, 4, 5, 6]).unwrap();
    assert_eq!(img[(0, 0)], Rgb::new(1, 2, 3));
    assert_eq!(img[(1, 0)], Rgb::new(4, 5, 6));
    assert_eq!(img.to_raw(), vec![1, 2, 3, 4, 5, 6]);
    let err = Image::<Rgba>::from_raw(2, 1, &[0; 6]).unwrap_err();
    assert_eq!(err, ImageError::BufferSize { width: 2, height: 1, expected: 8, actual: 6 });
}

#[test]
fn checked_access_reports_out_of_bounds() {
    let mut img = gray(&[&[1, 2], &[3, 4]]);
    assert_eq!(img.get(1, 1), Some(Gray(4)));
    assert_eq!(img.get(2, 0), None);
    assert_eq!(img.get(0, 2), None);
    assert!(img.set(1, 0, Gray(9)).is_ok());
    let err = img.set(2, 1, Gray(9)).unwrap_err();
    assert_eq!(err, ImageError::OutOfBounds { x: 2, y: 1, width: 2, height: 2 });
    assert_eq!(err.to_string(), "pixel (2, 1) is outside the 2x2 image");
    *img.get_mut(0, 1).unwrap() = Gray(7);
    assert!(img.get_mut(5, 5).is_none());
    assert_eq!(gray_rows(&img), vec![vec![1, 9], vec![7, 4]]);
}

#[test]
fn index_reads_and_writes() {
    let mut img = gray(&[&[1, 2], &[3, 4]]);
    img[(0, 0)] = Gray(50);
    assert_eq!(img[(0, 0)], Gray(50));
    assert_eq!(img[(1, 1)], Gray(4));
}

#[test]
#[should_panic(expected = "pixel (2, 0) is outside the 2x2 image")]
fn index_panics_outside_the_image() {
    let img = gray(&[&[1, 2], &[3, 4]]);
    let _ = img[(2, 0)];
}

#[test]
fn unchecked_access_matches_checked_access() {
    let mut img = gray(&[&[1, 2, 3], &[4, 5, 6]]);
    // SAFETY: both coordinates are inside the 3x2 image.
    unsafe {
        assert_eq!(img.get_unchecked(2, 1), Gray(6));
        img.set_unchecked(0, 1, Gray(40));
    }
    assert_eq!(img.get(0, 1), Some(Gray(40)));
}

#[test]
fn put_clipped_ignores_pixels_outside() {
    let mut img = gray(&[&[0, 0], &[0, 0]]);
    assert!(img.put_clipped(1, 1, Gray(5)));
    assert!(!img.put_clipped(-1, 0, Gray(5)));
    assert!(!img.put_clipped(0, -1, Gray(5)));
    assert!(!img.put_clipped(2, 0, Gray(5)));
    assert!(!img.put_clipped(0, 2, Gray(5)));
    assert_eq!(gray_rows(&img), vec![vec![0, 0], vec![0, 5]]);
}

#[test]
fn get_clamped_replicates_the_edges() {
    let img = gray(&[&[1, 2], &[3, 4]]);
    assert_eq!(img.get_clamped(-5, -5), Gray(1));
    assert_eq!(img.get_clamped(7, 0), Gray(2));
    assert_eq!(img.get_clamped(0, 9), Gray(3));
    assert_eq!(img.get_clamped(9, 9), Gray(4));
    assert_eq!(img.get_clamped(1, 0), Gray(2));
}

#[test]
fn rows_and_row_access() {
    let mut img = gray(&[&[1, 2, 3], &[4, 5, 6]]);
    assert_eq!(img.row(1), &[Gray(4), Gray(5), Gray(6)]);
    img.row_mut(0)[2] = Gray(30);
    let rows: Vec<&[Gray]> = img.rows().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], &[Gray(1), Gray(2), Gray(30)]);
}

#[test]
fn enumerate_pixels_yields_coordinates() {
    let img = gray(&[&[1, 2], &[3, 4], &[5, 6]]);
    let all: Vec<(u32, u32, u8)> = img.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)).collect();
    assert_eq!(all, vec![(0, 0, 1), (1, 0, 2), (0, 1, 3), (1, 1, 4), (0, 2, 5), (1, 2, 6)]);
}

#[test]
fn pixels_mut_and_fill() {
    let mut img = gray(&[&[1, 2], &[3, 4]]);
    for p in img.pixels_mut() {
        p.0 *= 2;
    }
    assert_eq!(gray_rows(&img), vec![vec![2, 4], vec![6, 8]]);
    img.fill(Gray(9));
    assert_eq!(img.into_vec(), vec![Gray(9); 4]);
}

#[test]
fn map_and_convert_change_the_pixel_type() {
    let img = gray(&[&[10, 20]]);
    let rgb: Image<Rgb> = img.convert();
    assert_eq!(rgb.as_slice(), &[Rgb::new(10, 10, 10), Rgb::new(20, 20, 20)]);
    let inverted = img.map(|p| Gray(255 - p.0));
    assert_eq!(gray_rows(&inverted), vec![vec![245, 235]]);
}

#[test]
fn empty_images_are_allowed() {
    let img: Image<Gray> = Image::new(0, 5);
    assert!(img.is_empty());
    assert_eq!(img.dimensions(), (0, 5));
    assert_eq!(img.get(0, 0), None);
    assert_eq!(img.rows().count(), 0);
    assert_eq!(img.enumerate_pixels().count(), 0);
    assert!(Image::<Rgb>::from_raw(0, 0, &[]).unwrap().is_empty());
}

#[test]
fn rect_contains_and_fits() {
    let r = Rect::new(2, 3, 4, 5);
    assert!(r.contains(2, 3) && r.contains(5, 7));
    assert!(!r.contains(6, 3) && !r.contains(2, 8) && !r.contains(1, 3));
    assert!(r.fits_in(6, 8));
    assert!(!r.fits_in(5, 8));
    assert!(!Rect::new(u32::MAX, 0, 2, 1).fits_in(u32::MAX, 1));
}
