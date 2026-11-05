mod common;

use common::{gray, gray_rows};
use pixelgrid::color::BlendMode;
use pixelgrid::draw::blit;
use pixelgrid::{Gray, Image, Rgb, Rgba};

#[test]
fn replace_copies_pixels_at_the_offset() {
    let mut dst = gray(&[&[1, 1, 1, 1], &[1, 1, 1, 1], &[1, 1, 1, 1]]);
    let src = gray(&[&[5, 6], &[7, 8]]);
    blit(&mut dst, &src, 1, 1, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[1, 1, 1, 1], [1, 5, 6, 1], [1, 7, 8, 1]]);
}

#[test]
fn clipped_on_every_side() {
    let src = gray(&[&[1, 2, 3], &[4, 5, 6], &[7, 8, 9]]);
    let blank = || gray(&[&[0, 0], &[0, 0]]);
    let mut dst = blank();
    blit(&mut dst, &src, -1, -1, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[5, 6], [8, 9]]);
    let mut dst = blank();
    blit(&mut dst, &src, 1, 1, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[0, 0], [0, 1]]);
    let mut dst = blank();
    blit(&mut dst, &src, -2, 1, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[0, 0], [3, 0]]);
    let mut dst = blank();
    blit(&mut dst, &src, 0, -2, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[7, 8], [0, 0]]);
}

#[test]
fn completely_outside_changes_nothing() {
    let src = gray(&[&[9, 9], &[9, 9]]);
    for (x, y) in [(2, 0), (0, 2), (-2, 0), (0, -2), (i32::MAX, 0), (i32::MIN, i32::MIN)] {
        let mut dst = gray(&[&[1, 2], &[3, 4]]);
        blit(&mut dst, &src, x, y, BlendMode::Replace);
        assert_eq!(gray_rows(&dst), [[1, 2], [3, 4]], "offset ({x}, {y})");
    }
    let mut dst = gray(&[&[1, 2], &[3, 4]]);
    blit(&mut dst, &Image::<Gray>::new(0, 0), 0, 0, BlendMode::Replace);
    assert_eq!(gray_rows(&dst), [[1, 2], [3, 4]]);
}

#[test]
fn over_composites_rgba_onto_rgb() {
    let mut dst = Image::filled(3, 1, Rgb::new(0, 0, 200));
    let src = Image::from_vec(
        3,
        1,
        vec![Rgba::new(255, 0, 0, 255), Rgba::new(255, 0, 0, 0), Rgba::new(255, 0, 0, 51)],
    )
    .unwrap();
    blit(&mut dst, &src, 0, 0, BlendMode::Over);
    // 51/255 = 0.2: red 0.2 * 255 = 51, blue 0.8 * 200 = 160.
    assert_eq!(dst.as_slice(), [Rgb::new(255, 0, 0), Rgb::new(0, 0, 200), Rgb::new(51, 0, 160)]);
}

#[test]
fn replace_onto_rgb_drops_alpha_without_compositing() {
    let mut dst = Image::filled(1, 1, Rgb::new(0, 0, 200));
    let src = Image::filled(1, 1, Rgba::new(10, 20, 30, 0));
    blit(&mut dst, &src, 0, 0, BlendMode::Replace);
    assert_eq!(dst[(0, 0)], Rgb::new(10, 20, 30));
}

#[test]
fn replace_onto_rgba_keeps_source_alpha() {
    let mut dst = Image::filled(2, 1, Rgba::WHITE);
    let src = Image::filled(1, 1, Rgba::new(10, 20, 30, 40));
    blit(&mut dst, &src, 1, 0, BlendMode::Replace);
    assert_eq!(dst.as_slice(), [Rgba::WHITE, Rgba::new(10, 20, 30, 40)]);
}

#[test]
fn separable_modes_on_grey() {
    let src = gray(&[&[100, 200]]);
    let base = || gray(&[&[50, 150]]);
    let run = |mode| {
        let mut dst = base();
        blit(&mut dst, &src, 0, 0, mode);
        gray_rows(&dst)
    };
    assert_eq!(run(BlendMode::Add), [[150, 255]]);
    assert_eq!(run(BlendMode::Darken), [[50, 150]]);
    assert_eq!(run(BlendMode::Lighten), [[100, 200]]);
    assert_eq!(run(BlendMode::Difference), [[50, 50]]);
    // 100 * 50 / 255 = 19.6 -> 20, 200 * 150 / 255 = 117.6 -> 118
    assert_eq!(run(BlendMode::Multiply), [[20, 118]]);
    // 255 - 155 * 205 / 255 = 255 - 124.6 -> 130, 255 - 55 * 105 / 255 = 255 - 22.6 -> 232
    assert_eq!(run(BlendMode::Screen), [[130, 232]]);
}

#[test]
fn grey_source_onto_rgb_destination() {
    let mut dst = Image::filled(2, 2, Rgb::BLACK);
    blit(&mut dst, &gray(&[&[7]]), 1, 1, BlendMode::Over);
    assert_eq!(dst[(1, 1)], Rgb::new(7, 7, 7));
    assert_eq!(dst[(0, 0)], Rgb::BLACK);
}
