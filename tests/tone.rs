mod common;

use common::{gray, gray_rows};
use pixelgrid::filter::{
    apply_lut, brightness, contrast, equalize, gamma, histogram, invert, luma_histogram,
    otsu_level, otsu_threshold, threshold,
};
use pixelgrid::{Gray, Image, Rgb, Rgba};

fn counts(pairs: &[(usize, u32)]) -> [u32; 256] {
    let mut h = [0u32; 256];
    for (value, count) in pairs {
        h[*value] = *count;
    }
    h
}

#[test]
fn histogram_counts_one_channel() {
    let img = gray(&[&[0, 0, 7], &[255, 7, 0]]);
    assert_eq!(histogram(&img, 0).unwrap(), counts(&[(0, 3), (7, 2), (255, 1)]));
    let rgba = Image::from_vec(2, 1, vec![Rgba::new(1, 2, 3, 4), Rgba::new(1, 5, 3, 0)]).unwrap();
    assert_eq!(histogram(&rgba, 0).unwrap(), counts(&[(1, 2)]));
    assert_eq!(histogram(&rgba, 1).unwrap(), counts(&[(2, 1), (5, 1)]));
    assert_eq!(histogram(&rgba, 3).unwrap(), counts(&[(4, 1), (0, 1)]));
    let err = histogram(&img, 1).unwrap_err();
    assert_eq!(err.to_string(), "invalid channel: gray pixels have 1 channels, got index 1");
    let empty: Image<Gray> = Image::new(0, 0);
    assert_eq!(histogram(&empty, 0).unwrap(), [0u32; 256]);
}

#[test]
fn luma_histogram_uses_the_grey_conversion() {
    let img = Image::from_vec(3, 1, vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::WHITE]).unwrap();
    // 0.299 * 255 = 76.2, 0.587 * 255 = 149.7
    assert_eq!(luma_histogram(&img), counts(&[(76, 1), (150, 1), (255, 1)]));
}

#[test]
fn equalize_spreads_four_levels_over_the_full_range() {
    let img = gray(&[&[50, 51, 52, 53]]);
    // cdf = 1, 2, 3, 4; (cdf - 1) * 255 / 3 = 0, 85, 170, 255.
    assert_eq!(gray_rows(&equalize(&img)), [[0, 85, 170, 255]]);
}

#[test]
fn equalize_weights_levels_by_frequency() {
    let img = gray(&[&[10, 10, 10, 20], &[20, 30, 30, 30]]);
    // cdf(10) = 3, cdf(20) = 5, cdf(30) = 8, cdf_min = 3: 0, 2 * 255 / 5 = 102, 255.
    assert_eq!(gray_rows(&equalize(&img)), [[0, 0, 0, 102], [102, 255, 255, 255]]);
}

#[test]
fn equalize_leaves_constant_and_empty_images_alone() {
    let flat: Image<Gray> = Image::filled(3, 2, Gray(99));
    assert_eq!(equalize(&flat), flat);
    let empty: Image<Gray> = Image::new(0, 0);
    assert_eq!(equalize(&empty).dimensions(), (0, 0));
}

#[test]
fn equalize_colour_uses_one_table_and_keeps_alpha() {
    let img = Image::from_vec(2, 1, vec![Rgba::new(100, 100, 100, 7), Rgba::new(200, 200, 200, 9)]).unwrap();
    let out = equalize(&img);
    assert_eq!(out.as_slice(), [Rgba::new(0, 0, 0, 7), Rgba::new(255, 255, 255, 9)]);
}

#[test]
fn threshold_is_strictly_greater_than() {
    let img = gray(&[&[0, 99, 100, 101, 255]]);
    assert_eq!(gray_rows(&threshold(&img, 100)), [[0, 0, 0, 255, 255]]);
    assert_eq!(gray_rows(&threshold(&img, 0)), [[0, 255, 255, 255, 255]]);
    assert_eq!(gray_rows(&threshold(&img, 255)), [[0, 0, 0, 0, 0]]);
    let colour = Image::from_vec(2, 1, vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0)]).unwrap();
    assert_eq!(gray_rows(&threshold(&colour, 100)), [[0, 255]]);
}

#[test]
fn otsu_separates_two_clusters_at_the_lowest_tied_level() {
    // Every level from 10 to 199 splits the clusters equally well; the smallest wins.
    assert_eq!(otsu_level(&counts(&[(10, 5), (200, 5)])), 10);
    assert_eq!(otsu_level(&counts(&[(10, 1), (200, 99)])), 10);
    assert_eq!(otsu_level(&counts(&[(0, 4), (255, 4)])), 0);
}

#[test]
fn otsu_by_hand_on_three_levels() {
    // Levels 0, 100, 250 with counts 2, 1, 1 (total 4, mean 87.5).
    // t = 0:   classes {0,0} | {100,250}: w = 2 * 2, means 0 and 175   -> 4 * 175^2 = 122500
    // t = 100: classes {0,0,100} | {250}: w = 3 * 1, means 33.3 and 250 -> 3 * 216.7^2 = 140833
    assert_eq!(otsu_level(&counts(&[(0, 2), (100, 1), (250, 1)])), 100);
    // Moving the middle level up to 200 makes the first split better:
    // t = 0: 4 * 225^2 = 202500; t = 200: 3 * (250 - 66.7)^2 = 100833.
    assert_eq!(otsu_level(&counts(&[(0, 2), (200, 1), (250, 1)])), 0);
}

#[test]
fn otsu_degenerate_histograms() {
    assert_eq!(otsu_level(&[0u32; 256]), 0);
    assert_eq!(otsu_level(&counts(&[(77, 10)])), 0);
}

#[test]
fn otsu_threshold_binarizes() {
    let img = gray(&[&[12, 10, 200], &[11, 201, 199]]);
    let (out, level) = otsu_threshold(&img);
    assert_eq!(level, 12);
    assert_eq!(gray_rows(&out), [[0, 0, 255], [0, 255, 255]]);
}

#[test]
fn brightness_saturates() {
    let img = gray(&[&[0, 100, 250]]);
    assert_eq!(gray_rows(&brightness(&img, 10)), [[10, 110, 255]]);
    assert_eq!(gray_rows(&brightness(&img, -101)), [[0, 0, 149]]);
    assert_eq!(gray_rows(&brightness(&img, 0)), [[0, 100, 250]]);
    assert_eq!(gray_rows(&brightness(&img, i32::MAX)), [[255, 255, 255]]);
    assert_eq!(gray_rows(&brightness(&img, i32::MIN)), [[0, 0, 0]]);
}

#[test]
fn contrast_pivots_on_mid_grey() {
    let img = gray(&[&[0, 64, 128, 192, 255]]);
    assert_eq!(gray_rows(&contrast(&img, 1.0).unwrap()), [[0, 64, 128, 192, 255]]);
    assert_eq!(gray_rows(&contrast(&img, 0.0).unwrap()), [[128, 128, 128, 128, 128]]);
    // (v - 128) * 2 + 128 = -128, 0, 128, 256, 382, clamped.
    assert_eq!(gray_rows(&contrast(&img, 2.0).unwrap()), [[0, 0, 128, 255, 255]]);
    // (v - 128) * 0.5 + 128 = 64, 96, 128, 160, 191.5 -> 192.
    assert_eq!(gray_rows(&contrast(&img, 0.5).unwrap()), [[64, 96, 128, 160, 192]]);
    for bad in [-0.5, f32::NAN, f32::INFINITY] {
        let err = contrast(&img, bad).unwrap_err();
        assert_eq!(
            err.to_string(),
            format!("invalid contrast factor: must be a non-negative number, got {bad}")
        );
    }
}

#[test]
fn gamma_keeps_end_points_and_moves_mid_tones() {
    let img = gray(&[&[0, 64, 255]]);
    assert_eq!(gray_rows(&gamma(&img, 1.0).unwrap()), [[0, 64, 255]]);
    // 255 * (64 / 255) ^ 0.5 = 255 * 0.50098 = 127.75
    assert_eq!(gray_rows(&gamma(&img, 2.0).unwrap()), [[0, 128, 255]]);
    // 255 * (64 / 255) ^ 2 = 16.06
    assert_eq!(gray_rows(&gamma(&img, 0.5).unwrap()), [[0, 16, 255]]);
    for bad in [0.0, -2.0, f32::NAN] {
        assert_eq!(
            gamma(&img, bad).unwrap_err().to_string(),
            format!("invalid gamma: must be a positive number, got {bad}")
        );
    }
}

#[test]
fn invert_and_lut_leave_alpha_alone() {
    let img = Image::from_vec(1, 1, vec![Rgba::new(0, 100, 255, 30)]).unwrap();
    assert_eq!(invert(&img)[(0, 0)], Rgba::new(255, 155, 0, 30));
    assert_eq!(invert(&invert(&img)), img);
    assert_eq!(brightness(&img, 5)[(0, 0)], Rgba::new(5, 105, 255, 30));
    let mut lut = [0u8; 256];
    lut[100] = 1;
    lut[255] = 2;
    lut[30] = 99;
    assert_eq!(apply_lut(&img, &lut)[(0, 0)], Rgba::new(0, 1, 2, 30));
}
