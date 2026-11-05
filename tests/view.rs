mod common;

use common::{gray, gray_rows};
use pixelgrid::{Gray, Image, ImageError, Rect};

fn grid() -> Image<Gray> {
    // 4x3, value = 10 * y + x
    Image::from_fn(4, 3, |x, y| Gray((10 * y + x) as u8))
}

#[test]
fn view_reads_relative_coordinates() {
    let img = grid();
    let v = img.view(1, 1, 2, 2).unwrap();
    assert_eq!((v.width(), v.height()), (2, 2));
    assert_eq!(v.rect(), Rect::new(1, 1, 2, 2));
    assert_eq!(v.get(0, 0), Some(Gray(11)));
    assert_eq!(v.get(1, 1), Some(Gray(22)));
    assert_eq!(v.get(2, 0), None, "inside the image but outside the view");
    assert_eq!(v.get(0, 2), None);
}

#[test]
fn view_rows_pixels_and_enumeration() {
    let img = grid();
    let v = img.view(2, 0, 2, 3).unwrap();
    assert_eq!(v.row(1), &[Gray(12), Gray(13)]);
    assert_eq!(v.rows().count(), 3);
    let values: Vec<u8> = v.pixels().map(|p| p.0).collect();
    assert_eq!(values, vec![2, 3, 12, 13, 22, 23]);
    let first: Vec<(u32, u32, u8)> = v.enumerate_pixels().take(3).map(|(x, y, p)| (x, y, p.0)).collect();
    assert_eq!(first, vec![(0, 0, 2), (1, 0, 3), (0, 1, 12)]);
}

#[test]
fn view_must_fit_inside_the_image() {
    let img = grid();
    assert!(img.view(0, 0, 4, 3).is_ok());
    let err = img.view(3, 0, 2, 1).unwrap_err();
    assert_eq!(
        err,
        ImageError::RegionOutOfBounds { x: 3, y: 0, width: 2, height: 1, image_width: 4, image_height: 3 }
    );
    assert_eq!(err.to_string(), "region 2x1 at (3, 0) does not fit in the 4x3 image");
    assert!(img.view(0, 2, 1, 2).is_err());
    assert!(img.view(u32::MAX, 0, 2, 1).is_err(), "no overflow on huge offsets");
}

#[test]
fn zero_sized_views_are_valid() {
    let img = grid();
    let v = img.view(4, 3, 0, 0).unwrap();
    assert_eq!(v.pixels().count(), 0);
    assert!(v.to_image().is_empty());
}

#[test]
fn nested_views_compose_offsets() {
    let img = grid();
    let outer = img.view(1, 0, 3, 3).unwrap();
    let inner = outer.view(1, 1, 2, 2).unwrap();
    assert_eq!(inner.rect(), Rect::new(2, 1, 2, 2));
    assert_eq!(inner.get(0, 0), Some(Gray(12)));
    assert!(outer.view(2, 0, 2, 1).is_err(), "fits the image but not the outer view");
}

#[test]
fn to_image_copies_the_region() {
    let img = grid();
    let copy = img.view(1, 1, 3, 2).unwrap().to_image();
    assert_eq!(gray_rows(&copy), vec![vec![11, 12, 13], vec![21, 22, 23]]);
}

#[test]
fn mutable_view_writes_through() {
    let mut img = grid();
    {
        let mut v = img.view_mut(1, 1, 2, 2).unwrap();
        v.set(0, 0, Gray(100)).unwrap();
        assert_eq!(v.get(0, 0), Some(Gray(100)));
        let err = v.set(2, 0, Gray(1)).unwrap_err();
        assert_eq!(err, ImageError::OutOfBounds { x: 2, y: 0, width: 2, height: 2 });
        v.row_mut(1)[1] = Gray(200);
    }
    assert_eq!(gray_rows(&img), vec![vec![0, 1, 2, 3], vec![10, 100, 12, 13], vec![20, 21, 200, 23]]);
}

#[test]
fn fill_and_apply_touch_only_the_view() {
    let mut img = grid();
    img.view_mut(0, 0, 2, 2).unwrap().fill(Gray(9));
    img.view_mut(2, 2, 2, 1).unwrap().apply(|p| Gray(p.0 + 100));
    assert_eq!(gray_rows(&img), vec![vec![9, 9, 2, 3], vec![9, 9, 12, 13], vec![20, 21, 122, 123]]);
}

#[test]
fn copy_from_requires_matching_size() {
    let mut img = grid();
    let patch = gray(&[&[7, 8], &[9, 6]]);
    img.view_mut(2, 1, 2, 2).unwrap().copy_from(&patch).unwrap();
    assert_eq!(gray_rows(&img), vec![vec![0, 1, 2, 3], vec![10, 11, 7, 8], vec![20, 21, 9, 6]]);
    let err = img.view_mut(0, 0, 3, 2).unwrap().copy_from(&patch).unwrap_err();
    assert_eq!(err, ImageError::DimensionMismatch { left: (3, 2), right: (2, 2) });
    assert_eq!(err.to_string(), "image sizes differ: 3x2 and 2x2");
}

#[test]
fn as_view_reborrows_read_only() {
    let mut img = grid();
    let v = img.view_mut(0, 1, 4, 1).unwrap();
    let values: Vec<u8> = v.as_view().pixels().map(|p| p.0).collect();
    assert_eq!(values, vec![10, 11, 12, 13]);
    assert_eq!((v.width(), v.height(), v.rect()), (4, 1, Rect::new(0, 1, 4, 1)));
}
