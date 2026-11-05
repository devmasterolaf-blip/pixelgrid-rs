use pixelgrid::pixel::luma;
use pixelgrid::{Gray, Pixel, Rgb, Rgba};

#[test]
fn channel_counts_and_names() {
    assert_eq!((Gray::CHANNELS, Rgb::CHANNELS, Rgba::CHANNELS), (1, 3, 4));
    assert_eq!((Gray::color_channels(), Rgb::color_channels(), Rgba::color_channels()), (1, 3, 3));
    assert!(Rgba::HAS_ALPHA && !Rgb::HAS_ALPHA && !Gray::HAS_ALPHA);
    assert_eq!((Gray::NAME, Rgb::NAME, Rgba::NAME), ("gray", "rgb", "rgba"));
}

#[test]
fn channels_round_trip() {
    assert_eq!(Gray(7).channels(), [7, 0, 0, 0]);
    assert_eq!(Rgb::new(1, 2, 3).channels(), [1, 2, 3, 0]);
    assert_eq!(Rgba::new(1, 2, 3, 4).channels(), [1, 2, 3, 4]);
    assert_eq!(Gray::from_channels([9, 8, 7, 6]), Gray(9));
    assert_eq!(Rgb::from_channels([9, 8, 7, 6]), Rgb::new(9, 8, 7));
    assert_eq!(Rgba::from_channels([9, 8, 7, 6]), Rgba::new(9, 8, 7, 6));
}

#[test]
fn luma_uses_bt601_weights_rounded() {
    assert_eq!(luma(0, 0, 0), 0);
    assert_eq!(luma(255, 255, 255), 255);
    // 0.299 * 255 = 76.245, 0.587 * 255 = 149.685, 0.114 * 255 = 29.07
    assert_eq!(luma(255, 0, 0), 76);
    assert_eq!(luma(0, 255, 0), 150);
    assert_eq!(luma(0, 0, 255), 29);
    // (299 * 10 + 587 * 20 + 114 * 30 + 500) / 1000 = 18650 / 1000
    assert_eq!(luma(10, 20, 30), 18);
}

#[test]
fn widening_conversions_are_lossless() {
    assert_eq!(Gray(40).to_rgba(), Rgba::new(40, 40, 40, 255));
    assert_eq!(Rgb::from(Gray(40)), Rgb::new(40, 40, 40));
    assert_eq!(Rgba::from(Rgb::new(1, 2, 3)), Rgba::new(1, 2, 3, 255));
    assert_eq!(Gray(40).convert::<Rgb>().convert::<Gray>(), Gray(40));
}

#[test]
fn narrowing_drops_alpha_and_uses_luma() {
    let c = Rgba::new(255, 0, 0, 10);
    assert_eq!(Rgb::from_rgba(c), Rgb::new(255, 0, 0));
    assert_eq!(Gray::from_rgba(c), Gray(76));
    assert_eq!(c.rgb().with_alpha(10), c);
}

#[test]
fn map_color_leaves_alpha_alone() {
    let c = Rgba::new(10, 20, 30, 40).map_color(|v| v * 2);
    assert_eq!(c, Rgba::new(20, 40, 60, 40));
    assert_eq!(Gray(5).map_color(|v| v + 1), Gray(6));
    assert_eq!(Rgb::new(1, 2, 3).map_color(|v| 255 - v), Rgb::new(254, 253, 252));
}

#[test]
fn max_delta_is_the_largest_channel_difference() {
    assert_eq!(Rgb::new(10, 20, 30).max_delta(Rgb::new(12, 15, 30)), 5);
    assert_eq!(Rgba::new(0, 0, 0, 0).max_delta(Rgba::new(0, 0, 0, 255)), 255);
    assert_eq!(Gray(3).max_delta(Gray(3)), 0);
}

#[test]
fn defaults_and_constants() {
    assert_eq!(Rgba::default(), Rgba::TRANSPARENT);
    assert_eq!(Rgb::default(), Rgb::BLACK);
    assert_eq!(Rgba::opaque(1, 2, 3), Rgba::new(1, 2, 3, 255));
    assert_eq!(Rgb::from([1, 2, 3]), Rgb::new(1, 2, 3));
    assert_eq!(Rgba::from([1, 2, 3, 4]), Rgba::new(1, 2, 3, 4));
}
