mod common;

use common::{gray, gray_rows};
use pixelgrid::filter::{dither_floyd_steinberg, dither_ordered, quantize, BayerMatrix, Palette};
use pixelgrid::{Gray, Image, Rgb, Rgba};

#[test]
fn palette_constructors() {
    assert_eq!(Palette::black_white().colors(), [Rgb::BLACK, Rgb::WHITE]);
    let greys = Palette::grays(4).unwrap();
    let values: Vec<u8> = greys.colors().iter().map(|c| c.r).collect();
    assert_eq!(values, [0, 85, 170, 255]);
    assert!(greys.colors().iter().all(|c| c.r == c.g && c.g == c.b));
    assert_eq!(Palette::grays(256).unwrap().len(), 256);
    assert_eq!(Palette::grays(256).unwrap().colors()[200], Rgb::new(200, 200, 200));
    let cube = Palette::rgb_cube(2).unwrap();
    assert_eq!(cube.len(), 8);
    assert_eq!(cube.colors()[0], Rgb::BLACK);
    assert_eq!(cube.colors()[1], Rgb::new(0, 0, 255));
    assert_eq!(cube.colors()[4], Rgb::new(255, 0, 0));
    assert_eq!(cube.colors()[7], Rgb::WHITE);
    assert_eq!(Palette::rgb_cube(6).unwrap().len(), 216);
    assert_eq!(Palette::rgb_cube(3).unwrap().colors()[13], Rgb::new(128, 128, 128));
    assert!(!cube.is_empty());
}

#[test]
fn palette_validation() {
    assert_eq!(
        Palette::new(vec![]).unwrap_err().to_string(),
        "invalid palette: needs 1 to 256 colours, got 0"
    );
    assert!(Palette::new(vec![Rgb::BLACK; 257]).is_err());
    assert!(Palette::new(vec![Rgb::BLACK; 256]).is_ok());
    assert_eq!(
        Palette::grays(1).unwrap_err().to_string(),
        "invalid palette: grey levels must be between 2 and 256, got 1"
    );
    assert!(Palette::grays(257).is_err());
    assert_eq!(
        Palette::rgb_cube(7).unwrap_err().to_string(),
        "invalid palette: cube levels must be between 2 and 6, got 7"
    );
    assert!(Palette::rgb_cube(1).is_err());
}

#[test]
fn nearest_colour_and_ties() {
    let palette = Palette::new(vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255)]).unwrap();
    assert_eq!(palette.nearest_index(Rgb::new(200, 10, 30)), 0);
    assert_eq!(palette.nearest_index(Rgb::new(10, 100, 90)), 1);
    assert_eq!(palette.nearest(Rgb::new(10, 90, 100)), Rgb::new(0, 0, 255));
    // Grey is equally far from all three: the first entry wins.
    assert_eq!(palette.nearest_index(Rgb::new(128, 128, 128)), 0);
    let bw = Palette::black_white();
    assert_eq!(bw.nearest(Rgb::new(127, 127, 127)), Rgb::BLACK);
    assert_eq!(bw.nearest(Rgb::new(128, 128, 128)), Rgb::WHITE);
}

#[test]
fn quantize_snaps_to_levels_and_keeps_alpha() {
    let img = gray(&[&[0, 42, 43, 127, 128, 255]]);
    assert_eq!(gray_rows(&quantize(&img, &Palette::grays(4).unwrap())), [[0, 0, 85, 85, 170, 255]]);
    let rgba = Image::from_vec(1, 1, vec![Rgba::new(250, 10, 130, 77)]).unwrap();
    assert_eq!(quantize(&rgba, &Palette::rgb_cube(2).unwrap())[(0, 0)], Rgba::new(255, 0, 255, 77));
}

#[test]
fn floyd_steinberg_single_row_by_hand() {
    let img = gray(&[&[100, 100, 100, 100]]);
    // 100 -> 0, error 100; next 100 + 43.75 = 143.75 -> 255, error -111.25;
    // next 100 - 48.67 = 51.33 -> 0, error 51.33; last 100 + 22.46 = 122.46 -> 0.
    assert_eq!(gray_rows(&dither_floyd_steinberg(&img, &Palette::black_white())), [[0, 255, 0, 0]]);
}

#[test]
fn floyd_steinberg_two_rows_by_hand() {
    let img = gray(&[&[128, 128], &[128, 128]]);
    // (0,0): 128 -> 255, error -127: right 128 - 55.56 = 72.44, below 88.31, below-right 120.06.
    // (1,0): 72.44 -> 0, error 72.44: below-left 88.31 + 13.58 = 101.89, below 120.06 + 22.64.
    // (0,1): 101.89 -> 0, error 101.89: right 142.70 + 44.58 = 187.28 -> 255.
    assert_eq!(gray_rows(&dither_floyd_steinberg(&img, &Palette::black_white())), [[255, 0], [0, 255]]);
}

#[test]
fn floyd_steinberg_preserves_average_brightness() {
    for level in [30u8, 64, 128, 200] {
        let img: Image<Gray> = Image::filled(32, 32, Gray(level));
        let out = dither_floyd_steinberg(&img, &Palette::black_white());
        assert!(out.pixels().all(|p| p.0 == 0 || p.0 == 255));
        let mean = out.pixels().map(|p| f64::from(p.0)).sum::<f64>() / 1024.0;
        assert!((mean - f64::from(level)).abs() < 4.0, "level {level}: mean {mean}");
    }
}

#[test]
fn dithering_leaves_palette_colours_untouched() {
    let img = gray(&[&[0, 255, 255], &[255, 0, 0]]);
    let bw = Palette::black_white();
    assert_eq!(dither_floyd_steinberg(&img, &bw), img);
    assert_eq!(dither_ordered(&img, &bw, BayerMatrix::B4, 255.0), img);
    let empty: Image<Gray> = Image::new(0, 0);
    assert_eq!(dither_floyd_steinberg(&empty, &bw).dimensions(), (0, 0));
    assert_eq!(dither_ordered(&empty, &bw, BayerMatrix::B2, 255.0).dimensions(), (0, 0));
}

#[test]
fn floyd_steinberg_colour_keeps_alpha() {
    let img = Image::filled(4, 4, Rgba::new(200, 60, 128, 99));
    let out = dither_floyd_steinberg(&img, &Palette::rgb_cube(2).unwrap());
    assert!(out.pixels().all(|p| p.a == 99));
    assert!(out.pixels().all(|p| [p.r, p.g, p.b].iter().all(|v| *v == 0 || *v == 255)));
    // The first pixel carries no error yet: 200 -> 255, 60 -> 0 and 128 -> 255.
    assert_eq!(out[(0, 0)], Rgba::new(255, 0, 255, 99));
}

#[test]
fn bayer_matrices() {
    let b2: Vec<u32> = (0..4).map(|i| BayerMatrix::B2.rank(i % 2, i / 2)).collect();
    assert_eq!(b2, [0, 2, 3, 1]);
    let b4: Vec<u32> = (0..16).map(|i| BayerMatrix::B4.rank(i % 4, i / 4)).collect();
    assert_eq!(b4, [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5]);
    let mut b8: Vec<u32> = (0..64).map(|i| BayerMatrix::B8.rank(i % 8, i / 8)).collect();
    assert_eq!(b8[..8], [0, 32, 8, 40, 2, 34, 10, 42]);
    b8.sort_unstable();
    assert_eq!(b8, (0..64).collect::<Vec<u32>>());
    // The matrix tiles the plane.
    assert_eq!(BayerMatrix::B4.rank(5, 6), BayerMatrix::B4.rank(1, 2));
    assert_eq!(BayerMatrix::from_name("4"), Some(BayerMatrix::B4));
    assert_eq!(BayerMatrix::from_name("3"), None);
    assert_eq!((BayerMatrix::B2.size(), BayerMatrix::B8.size()), (2, 8));
}

#[test]
fn ordered_dither_by_hand() {
    let bw = Palette::black_white();
    // Offsets of the 2x2 matrix times 255: -95.6, 31.9 / 63.75, -31.9.
    let mid = gray(&[&[128, 128], &[128, 128]]);
    assert_eq!(gray_rows(&dither_ordered(&mid, &bw, BayerMatrix::B2, 255.0)), [[0, 255], [255, 0]]);
    // 64 + 63.75 = 127.75 rounds to 128, the only cell that reaches white.
    let dark = gray(&[&[64, 64], &[64, 64]]);
    assert_eq!(gray_rows(&dither_ordered(&dark, &bw, BayerMatrix::B2, 255.0)), [[0, 0], [255, 0]]);
    // With no spread the result is plain quantization.
    assert_eq!(dither_ordered(&mid, &bw, BayerMatrix::B2, 0.0), quantize(&mid, &bw));
}

#[test]
fn ordered_dither_density_tracks_brightness() {
    let bw = Palette::black_white();
    for (level, lit) in [(0u8, 0), (32, 2), (128, 8), (224, 14), (255, 16)] {
        let img: Image<Gray> = Image::filled(4, 4, Gray(level));
        let out = dither_ordered(&img, &bw, BayerMatrix::B4, 255.0);
        let count = out.pixels().filter(|p| p.0 == 255).count();
        assert_eq!(count, lit, "level {level}");
    }
}
