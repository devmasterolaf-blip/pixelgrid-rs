//! Scenarios that cross module boundaries: drawing, file formats, filters, labelling and
//! comparison working on each other's output.

mod common;

use common::{gray, gray_rows, rgb_pattern, scratch_dir};
use pixelgrid::color::{flatten, BlendMode};
use pixelgrid::compare::{compare, compare_dyn, diff_mask};
use pixelgrid::draw::{blit, draw_text, fill_circle, fill_rect, text_size};
use pixelgrid::filter::tone::luma_histogram;
use pixelgrid::filter::{
    dither_floyd_steinberg, dither_ordered, equalize, gaussian_blur, BayerMatrix, Palette,
};
use pixelgrid::io::bmp::RowOrder;
use pixelgrid::io::pnm::PnmEncoding;
use pixelgrid::io::{self, EncodeOptions, Format};
use pixelgrid::label::{label_components, label_nonzero};
use pixelgrid::transform::{flip_horizontal, resize, rotate, rotate90, Interpolation};
use pixelgrid::{Connectivity, DynImage, Gray, Image, Rect, Rgb, Rgba};

#[test]
fn drawn_shapes_survive_a_bmp_round_trip_and_are_found_as_components() {
    let mut canvas: Image<Rgb> = Image::new(12, 8);
    fill_rect(&mut canvas, 1, 1, 3, 2, Rgb::new(255, 0, 0));
    fill_circle(&mut canvas, 8, 4, 2, Rgb::WHITE);
    let path = scratch_dir("integration_shapes").join("shapes.bmp");
    io::save(&path, &canvas.clone().into()).unwrap();
    let DynImage::Rgb(loaded) = io::load(&path).unwrap() else { panic!("expected an RGB image") };
    assert_eq!(loaded, canvas);

    let labeling = label_components(&loaded, Connectivity::Eight, |p| p != Rgb::BLACK);
    let found: Vec<_> = labeling.components().iter().map(|c| (c.area, c.bounds, c.centroid)).collect();
    // The radius-2 disc covers rows of 3, 5, 5, 5 and 3 pixels.
    assert_eq!(found, [(6, Rect::new(1, 1, 3, 2), (2.0, 1.5)), (21, Rect::new(6, 2, 5, 5), (8.0, 4.0))]);
    assert_eq!(labeling.label_at(8, 4), Some(2));
}

#[test]
fn rendered_text_splits_into_one_component_per_stroke() {
    let (width, height) = text_size("1:", 2);
    assert_eq!((width, height), (14, 10));
    let mut canvas: Image<Gray> = Image::new(width + 2, height + 2);
    draw_text(&mut canvas, 1, 1, "1:", Gray(255), 2);
    let labeling = label_nonzero(&canvas, Connectivity::Four);
    let found: Vec<_> = labeling.components().iter().map(|c| (c.area, c.bounds)).collect();
    // The digit has 8 font pixels and each dot of the colon one, all doubled in both directions.
    assert_eq!(
        found,
        [(32, Rect::new(1, 1, 6, 10)), (4, Rect::new(11, 3, 2, 2)), (4, Rect::new(11, 7, 2, 2))]
    );
    // Removing the digit by its mask leaves exactly the colon.
    let digit = labeling.mask(1);
    let mut colon: Image<Gray> = Image::new(canvas.width(), canvas.height());
    draw_text(&mut colon, 1, 1, " :", Gray(255), 2);
    assert_eq!(diff_mask(&canvas, &colon, 0).unwrap(), digit);
}

#[test]
fn blitting_with_over_matches_flattening_onto_the_same_background() {
    let sprite = Image::from_vec(
        2,
        2,
        vec![Rgba::new(255, 0, 0, 255), Rgba::new(0, 255, 0, 128), Rgba::new(0, 0, 255, 1), Rgba::TRANSPARENT],
    )
    .unwrap();
    let background = Rgb::new(40, 80, 120);
    let mut canvas = Image::filled(2, 2, background);
    blit(&mut canvas, &sprite, 0, 0, BlendMode::Over);
    assert_eq!(canvas, flatten(&sprite, background));
    assert_eq!(canvas[(0, 0)], Rgb::new(255, 0, 0));
    assert_eq!(canvas[(1, 1)], background);
}

#[test]
fn every_writable_format_reads_back_what_was_written() {
    let colour: DynImage = rgb_pattern(5, 3).into();
    let grey: DynImage = gray(&[&[0, 1, 127], &[128, 254, 255]]).into();
    let all_options = [
        EncodeOptions { pnm_encoding: PnmEncoding::Binary, bmp_row_order: RowOrder::BottomUp },
        EncodeOptions { pnm_encoding: PnmEncoding::Ascii, bmp_row_order: RowOrder::TopDown },
    ];
    for options in &all_options {
        for format in [Format::Ppm, Format::Pnm, Format::Bmp] {
            let bytes = io::encode(&colour, format, options).unwrap();
            assert_eq!(io::decode(&bytes).unwrap(), colour, "{format:?} {options:?}");
        }
        for format in [Format::Pgm, Format::Pnm] {
            let bytes = io::encode(&grey, format, options).unwrap();
            assert_eq!(io::decode(&bytes).unwrap(), grey, "{format:?} {options:?}");
        }
        // BMP has no grey mode: the values come back as RGB.
        let bytes = io::encode(&grey, Format::Bmp, options).unwrap();
        let back = io::decode(&bytes).unwrap();
        assert_eq!(back.pixel_name(), "rgb");
        assert!(compare_dyn(&back, &grey, 0).unwrap().is_match());
    }
}

#[test]
fn transforms_commute_with_a_file_round_trip() {
    let dir = scratch_dir("integration_transforms");
    let original = rgb_pattern(4, 3);
    let turned = rotate90(&flip_horizontal(&original));
    io::save(dir.join("turned.ppm"), &turned.clone().into()).unwrap();
    let DynImage::Rgb(loaded) = io::load(dir.join("turned.ppm")).unwrap() else { panic!("expected RGB") };
    // Undo the two steps on the loaded copy.
    let restored = flip_horizontal(&rotate(&loaded, -90.0, Interpolation::Nearest, true, Rgb::BLACK));
    assert_eq!(restored, original);
}

#[test]
fn nearest_upscaling_by_an_integer_factor_is_undone_by_downscaling() {
    let original = rgb_pattern(5, 4);
    let big = resize(&original, 15, 12, Interpolation::Nearest).unwrap();
    assert_eq!(big[(14, 11)], original[(4, 3)]);
    let small = resize(&big, 5, 4, Interpolation::Nearest).unwrap();
    assert!(compare(&small, &original, 0).unwrap().is_match());
    // Bilinear sampling of the enlarged picture lands on block centres as well.
    let smooth = resize(&big, 5, 4, Interpolation::Bilinear).unwrap();
    assert_eq!(smooth, original);
}

#[test]
fn equalization_spreads_four_equal_groups_evenly() {
    let image = Image::from_fn(4, 4, |x, _| Gray(10 * (x as u8 + 1)));
    let flat = equalize(&image);
    // Cumulative counts 4, 8, 12, 16 map to (c - 4) / 12 of the full range.
    assert_eq!(gray_rows(&flat)[0], [0, 85, 170, 255]);
    let histogram = luma_histogram(&flat);
    assert_eq!([histogram[0], histogram[85], histogram[170], histogram[255]], [4; 4]);
    assert_eq!(histogram.iter().sum::<u32>(), 16);
}

#[test]
fn dithering_keeps_the_average_brightness() {
    let palette = Palette::black_white();
    let white = |img: &Image<Gray>| img.pixels().filter(|p| p.0 == 255).count();
    // Rank 8 and above of the 16 Bayer cells push mid-grey over the threshold: exactly half.
    let ordered = dither_ordered(&Image::filled(16, 16, Gray(128)), &palette, BayerMatrix::B4, 255.0);
    assert_eq!(white(&ordered), 128);
    assert!(ordered.pixels().all(|p| p.0 == 0 || p.0 == 255));
    // Error diffusion loses a little at the borders, so allow a few pixels either way around
    // 256 * 100 / 255 = 100.
    let diffused = dither_floyd_steinberg(&Image::filled(16, 16, Gray(100)), &palette);
    assert!(diffused.pixels().all(|p| p.0 == 0 || p.0 == 255));
    let count = white(&diffused);
    assert!((96..=104).contains(&count), "{count} white pixels");
}

#[test]
fn blurring_a_flat_image_changes_nothing_and_a_step_stays_monotonic() {
    let flat = Image::filled(6, 5, Rgb::new(12, 200, 99));
    assert_eq!(gaussian_blur(&flat, 2.0).unwrap(), flat);
    let step = Image::from_fn(10, 3, |x, _| Gray(if x < 5 { 0 } else { 255 }));
    let blurred = gaussian_blur(&step, 1.0).unwrap();
    let row = &gray_rows(&blurred)[1];
    assert!(row.windows(2).all(|w| w[0] <= w[1]), "{row:?}");
    assert_eq!((row[0], row[9]), (0, 255));
    // The kernel is symmetric, so the two sides of the edge mirror each other.
    assert_eq!(u16::from(row[4]) + u16::from(row[5]), 255);
    assert_eq!(u16::from(row[3]) + u16::from(row[6]), 255);
}
