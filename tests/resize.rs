mod common;

use common::{gray, gray_rows, rgb_pattern};
use pixelgrid::transform::{resize, sample, Interpolation};
use pixelgrid::{Gray, Image, ImageError, Rgba};

const ALL: [Interpolation; 3] =
    [Interpolation::Nearest, Interpolation::Bilinear, Interpolation::Bicubic];

#[test]
fn same_size_is_an_exact_copy() {
    let img = rgb_pattern(5, 4);
    for interpolation in ALL {
        assert_eq!(resize(&img, 5, 4, interpolation).unwrap(), img, "{interpolation:?}");
    }
}

#[test]
fn nearest_upscale_repeats_pixels() {
    let img = gray(&[&[1, 2], &[3, 4]]);
    let big = resize(&img, 4, 4, Interpolation::Nearest).unwrap();
    assert_eq!(gray_rows(&big), [[1, 1, 2, 2], [1, 1, 2, 2], [3, 3, 4, 4], [3, 3, 4, 4]]);
    let wide = resize(&img, 6, 2, Interpolation::Nearest).unwrap();
    assert_eq!(gray_rows(&wide), [[1, 1, 1, 2, 2, 2], [3, 3, 3, 4, 4, 4]]);
}

#[test]
fn nearest_downscale_picks_the_pixel_under_each_centre() {
    let img = gray(&[&[0, 1, 2, 3, 4, 5]]);
    // Centres of the three output pixels fall at 1.0, 3.0 and 5.0 source pixels from the left
    // edge, which are the left borders of pixels 1, 3 and 5.
    assert_eq!(gray_rows(&resize(&img, 3, 1, Interpolation::Nearest).unwrap()), [[1, 3, 5]]);
    // 6 -> 4: centres at 0.75, 2.25, 3.75 and 5.25.
    assert_eq!(gray_rows(&resize(&img, 4, 1, Interpolation::Nearest).unwrap()), [[0, 2, 3, 5]]);
    // 6 -> 1: the centre at 3.0.
    assert_eq!(gray_rows(&resize(&img, 1, 1, Interpolation::Nearest).unwrap()), [[3]]);
}

#[test]
fn bilinear_upscale_of_two_pixels() {
    let img = gray(&[&[0, 100]]);
    // Sample positions -0.25, 0.25, 0.75, 1.25; the outer two clamp to the edge pixels.
    assert_eq!(gray_rows(&resize(&img, 4, 1, Interpolation::Bilinear).unwrap()), [[0, 25, 75, 100]]);
    let tall = resize(&gray(&[&[0], &[100]]), 1, 4, Interpolation::Bilinear).unwrap();
    assert_eq!(gray_rows(&tall), [[0], [25], [75], [100]]);
}

#[test]
fn bilinear_halving_averages_pairs() {
    let img = gray(&[&[10, 20, 30, 50], &[30, 40, 70, 90]]);
    // Each output pixel sits exactly between four source pixels.
    assert_eq!(gray_rows(&resize(&img, 2, 1, Interpolation::Bilinear).unwrap()), [[25, 60]]);
    // Halving only the width averages horizontal pairs.
    assert_eq!(gray_rows(&resize(&img, 2, 2, Interpolation::Bilinear).unwrap()), [[15, 40], [35, 80]]);
}

#[test]
fn bicubic_upscale_of_two_pixels() {
    let img = gray(&[&[0, 100]]);
    // Catmull-Rom weights at t = 0.25 are (-0.0703125, 0.8671875, 0.2265625, -0.0234375).
    // At 0.25 the taps read 0, 0, 100, 100: 100 * (0.2265625 - 0.0234375) = 20.3.
    // At 0.75 by symmetry 79.7. The outer samples ring past the step: at -0.25 the taps read
    // 0, 0, 0, 100 and give -7.03, which is clamped to 0; at 1.25 they read 0, 100, 100, 100
    // and give 100 * (1 + 0.0703125) = 107.03, a legal value that is kept.
    assert_eq!(gray_rows(&resize(&img, 4, 1, Interpolation::Bicubic).unwrap()), [[0, 20, 80, 107]]);
}

#[test]
fn bicubic_overshoot_is_clamped_not_wrapped() {
    let img = gray(&[&[0, 0, 255, 255]]);
    let out = resize(&img, 8, 1, Interpolation::Bicubic).unwrap();
    let values = &gray_rows(&out)[0];
    assert_eq!(values[..3], [0, 0, 0]);
    assert_eq!(values[5..], [255, 255, 255]);
    // 255 * (0.2265625 - 0.0234375) = 51.8 and its mirror image 203.2.
    assert_eq!(values[3..5], [52, 203]);
}

#[test]
fn constant_images_stay_constant() {
    let img: Image<Gray> = Image::filled(3, 5, Gray(77));
    for interpolation in ALL {
        for (w, h) in [(1, 1), (7, 2), (2, 9), (6, 10)] {
            let out = resize(&img, w, h, interpolation).unwrap();
            assert_eq!(out.dimensions(), (w, h));
            assert!(out.pixels().all(|p| p.0 == 77), "{interpolation:?} {w}x{h}");
        }
    }
}

#[test]
fn alpha_is_interpolated_like_any_channel() {
    let img = Image::from_vec(2, 1, vec![Rgba::new(0, 0, 0, 0), Rgba::new(200, 100, 40, 255)]).unwrap();
    let out = resize(&img, 4, 1, Interpolation::Bilinear).unwrap();
    // 25 % of each channel: 50, 25, 10 and 63.75 -> 64.
    assert_eq!(out[(1, 0)], Rgba::new(50, 25, 10, 64));
}

#[test]
fn sample_reads_between_pixel_centres() {
    let img = gray(&[&[0, 100], &[200, 100]]);
    for interpolation in ALL {
        assert_eq!(sample(&img, 1.0, 0.0, interpolation), Gray(100));
        assert_eq!(sample(&img, 0.0, 1.0, interpolation), Gray(200));
        // Far outside: clamp to the nearest edge pixel.
        assert_eq!(sample(&img, -50.0, -50.0, interpolation), Gray(0));
        assert_eq!(sample(&img, 50.0, 0.0, interpolation), Gray(100));
    }
    assert_eq!(sample(&img, 0.5, 0.5, Interpolation::Bilinear), Gray(100));
    assert_eq!(sample(&img, 0.5, 0.0, Interpolation::Bilinear), Gray(50));
    assert_eq!(sample(&img, 0.4, 0.0, Interpolation::Nearest), Gray(0));
    assert_eq!(sample(&img, 0.5, 0.0, Interpolation::Nearest), Gray(100));
}

#[test]
fn rejects_empty_sources_and_empty_targets() {
    let empty: Image<Gray> = Image::new(0, 4);
    let err = resize(&empty, 2, 2, Interpolation::Nearest).unwrap_err();
    assert_eq!(err, ImageError::EmptyImage { width: 0, height: 4 });
    assert_eq!(err.to_string(), "the operation needs a non-empty image, got 0x4");
    let img = gray(&[&[1]]);
    for (w, h) in [(0, 1), (1, 0), (0, 0)] {
        let err = resize(&img, w, h, Interpolation::Bilinear).unwrap_err();
        assert_eq!(err.to_string(), format!("invalid size: cannot resize to {w}x{h}"));
    }
}

#[test]
fn interpolation_names() {
    assert_eq!(Interpolation::from_name("nearest"), Some(Interpolation::Nearest));
    assert_eq!(Interpolation::from_name("bilinear"), Some(Interpolation::Bilinear));
    assert_eq!(Interpolation::from_name("bicubic"), Some(Interpolation::Bicubic));
    assert_eq!(Interpolation::from_name("lanczos"), None);
}
