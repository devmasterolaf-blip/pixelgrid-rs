mod common;

use common::{gray, gray_rows, mask};
use pixelgrid::filter::{box_blur, gaussian_blur, gaussian_kernel, median, sharpen, sobel};
use pixelgrid::{Gray, Image, Rgb, Rgba};

#[test]
fn box_blur_spreads_an_impulse_evenly() {
    let mut img: Image<Gray> = Image::new(5, 5);
    img[(2, 2)] = Gray(90);
    let out = box_blur(&img, 1);
    let expected = [[0, 0, 0, 0, 0], [0, 10, 10, 10, 0], [0, 10, 10, 10, 0], [0, 10, 10, 10, 0], [0, 0, 0, 0, 0]];
    assert_eq!(gray_rows(&out), expected);
    assert_eq!(box_blur(&img, 0), img);
}

#[test]
fn box_blur_clamps_at_the_border() {
    let img = gray(&[&[90, 0, 0]]);
    // x = 0 sees 90, 90, 0 horizontally (and three identical rows vertically).
    assert_eq!(gray_rows(&box_blur(&img, 1)), [[60, 30, 0]]);
    // Radius 2 on three pixels: windows 90,90,90,0,0 / 90,90,0,0,0 / 90,0,0,0,0.
    assert_eq!(gray_rows(&box_blur(&img, 2)), [[54, 36, 18]]);
}

#[test]
fn gaussian_kernel_shape() {
    let k = gaussian_kernel(1.0).unwrap();
    assert_eq!(k.len(), 7);
    assert!((k.iter().sum::<f32>() - 1.0).abs() < 1e-6);
    for i in 0..3 {
        assert_eq!(k[i], k[6 - i]);
        assert!(k[i] < k[i + 1]);
    }
    // exp(-1/2) = 0.60653: the ratio of a neighbour to the centre.
    assert!((k[2] / k[3] - 0.60653).abs() < 1e-4);
    assert_eq!(gaussian_kernel(0.5).unwrap().len(), 5);
    assert_eq!(gaussian_kernel(0.2).unwrap().len(), 3);
    assert_eq!(gaussian_kernel(2.5).unwrap().len(), 17);
}

#[test]
fn gaussian_blur_keeps_flat_areas_and_softens_edges() {
    let flat: Image<Rgb> = Image::filled(6, 4, Rgb::new(10, 200, 77));
    assert_eq!(gaussian_blur(&flat, 1.3).unwrap(), flat);
    let step = gray(&[&[0, 0, 0, 0, 200, 200, 200, 200]]);
    let out = gray_rows(&gaussian_blur(&step, 1.0).unwrap()).remove(0);
    assert!(out.windows(2).all(|w| w[0] <= w[1]), "{out:?}");
    assert_eq!((out[0], out[7]), (0, 200));
    // The profile is symmetric around the edge.
    for i in 0..4 {
        assert_eq!(out[i] + out[7 - i], 200, "{out:?}");
    }
    assert!(out[3] > 0 && out[3] < 100);
}

#[test]
fn gaussian_blur_impulse_by_hand() {
    // sigma = 0.5: weights exp(-2 i^2) for i = -2..=2 are 0.000335, 0.135335, 1, 0.135335,
    // 0.000335, sum 1.271341; normalized centre 0.786571, neighbour 0.106451, outer 0.000264.
    let mut img: Image<Gray> = Image::new(5, 1);
    img[(2, 0)] = Gray(200);
    // One row: the vertical pass sees the same row everywhere and changes nothing.
    assert_eq!(gray_rows(&gaussian_blur(&img, 0.5).unwrap()), [[0, 21, 157, 21, 0]]);
}

#[test]
fn a_vanishing_sigma_gives_the_identity_kernel() {
    // 2 * sigma^2 underflows to zero for these, which used to turn the centre weight into NaN.
    for sigma in [1e-30, f32::MIN_POSITIVE, 1e-3] {
        assert_eq!(gaussian_kernel(sigma).unwrap(), [0.0, 1.0, 0.0], "sigma {sigma}");
    }
    let img = gray(&[&[0, 60, 255], &[7, 8, 9]]);
    assert_eq!(gaussian_blur(&img, 1e-30).unwrap(), img);
}

#[test]
fn gaussian_blur_rejects_bad_sigma() {
    let img = gray(&[&[1]]);
    for sigma in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let err = gaussian_blur(&img, sigma).unwrap_err();
        assert_eq!(err.to_string(), format!("invalid sigma: must be a positive number, got {sigma}"));
    }
}

#[test]
fn sharpen_exaggerates_a_step_and_keeps_flat_areas() {
    let flat: Image<Gray> = Image::filled(3, 3, Gray(100));
    assert_eq!(sharpen(&flat), flat);
    let step = gray(&[&[100, 100, 120, 120]]);
    // 5c - left - right - up - down with up = down = c: 3c - left - right.
    assert_eq!(gray_rows(&sharpen(&step)), [[100, 80, 140, 120]]);
    let hard = gray(&[&[0, 255]]);
    assert_eq!(gray_rows(&sharpen(&hard)), [[0, 255]]);
}

#[test]
fn sobel_on_a_vertical_edge() {
    let img = gray(&[&[0, 0, 10, 10], &[0, 0, 10, 10], &[0, 0, 10, 10]]);
    // gx = 4 * 10 on the two columns next to the edge, zero elsewhere; gy = 0.
    assert_eq!(gray_rows(&sobel(&img)), [[0, 40, 40, 0], [0, 40, 40, 0], [0, 40, 40, 0]]);
}

#[test]
fn sobel_on_a_horizontal_edge_and_a_corner() {
    let img = gray(&[&[0, 0, 0], &[0, 0, 0], &[10, 10, 10]]);
    assert_eq!(gray_rows(&sobel(&img)), [[0, 0, 0], [40, 40, 40], [40, 40, 40]]);
    // A single bright pixel in the corner of a 2x2 image.
    let img = gray(&[&[0, 0], &[0, 10]]);
    // At (0, 0): gx = (0 + 2*0 + 10) - 0 = 10, gy = (0 + 2*0 + 10) - 0 = 10 -> 14.14.
    assert_eq!(sobel(&img)[(0, 0)], Gray(14));
}

#[test]
fn sobel_saturates_and_uses_luma_for_colour() {
    let img = gray(&[&[0, 255], &[0, 255]]);
    assert!(sobel(&img).pixels().all(|p| p.0 == 255));
    let colour = Image::from_vec(2, 1, vec![Rgb::new(0, 0, 0), Rgb::new(0, 0, 100)]).unwrap();
    // Luma of (0, 0, 100) is 11; gx = 4 * 11.
    assert_eq!(gray_rows(&sobel(&colour)), [[44, 44]]);
}

#[test]
fn median_removes_isolated_pixels() {
    let img = mask(&[".....", "..#..", ".....", "#####", "#####"]);
    let out = median(&img, 1);
    assert_eq!(common::picture(&out), [".....", ".....", ".....", "#####", "#####"]);
    assert_eq!(median(&img, 0), img);
}

#[test]
fn median_by_hand() {
    let img = gray(&[&[1, 9, 3], &[7, 5, 2], &[8, 4, 6]]);
    let out = median(&img, 1);
    // Centre: median of 1..=9 is 5. Top-left window (clamped): 1,1,9,1,1,9,7,7,5 -> sorted
    // 1,1,1,1,5,7,7,9,9 -> 5.
    assert_eq!(out[(1, 1)], Gray(5));
    assert_eq!(out[(0, 0)], Gray(5));
    // Top-right window: 9,3,3,9,3,3,5,2,2 -> sorted 2,2,3,3,3,3,5,9,9 -> 3.
    assert_eq!(out[(2, 0)], Gray(3));
    // Clamped windows repeat the edge pixel, so the two ends keep their own values.
    assert_eq!(median(&gray(&[&[1, 2, 100]]), 1).as_slice(), [Gray(1), Gray(2), Gray(100)]);
}

#[test]
fn median_handles_channels_independently() {
    let a = Rgba::new(1, 200, 9, 255);
    let b = Rgba::new(2, 100, 8, 0);
    let c = Rgba::new(3, 150, 7, 255);
    let img = Image::from_vec(3, 1, vec![a, b, c]).unwrap();
    // The window at x = 1 holds every pixel three times; per-channel medians mix the pixels.
    assert_eq!(median(&img, 1)[(1, 0)], Rgba::new(2, 150, 8, 255));
    let empty: Image<Gray> = Image::new(0, 0);
    assert_eq!(median(&empty, 2).dimensions(), (0, 0));
}
