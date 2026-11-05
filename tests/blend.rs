use pixelgrid::color::{
    blend, flatten, mul_div255, over, over_premultiplied, premultiply, unpremultiply, BlendMode,
};
use pixelgrid::{Image, Rgb, Rgba};

const BLUE: Rgba = Rgba::new(0, 0, 255, 255);

#[test]
fn mul_div255_rounds_to_nearest() {
    assert_eq!(mul_div255(255, 255), 255);
    assert_eq!(mul_div255(255, 0), 0);
    assert_eq!(mul_div255(128, 128), 64, "16384 / 255 = 64.25");
    assert_eq!(mul_div255(200, 128), 100, "25600 / 255 = 100.39");
    assert_eq!(mul_div255(3, 128), 2, "384 / 255 = 1.506 rounds up");
}

#[test]
fn opaque_source_wins_and_transparent_source_vanishes() {
    let red = Rgba::opaque(255, 0, 0);
    assert_eq!(over(red, BLUE), red);
    assert_eq!(over(Rgba::new(255, 0, 0, 0), BLUE), BLUE);
    assert_eq!(over(Rgba::TRANSPARENT, Rgba::TRANSPARENT), Rgba::TRANSPARENT);
    assert_eq!(over(Rgba::new(9, 9, 9, 0), Rgba::new(7, 7, 7, 0)), Rgba::TRANSPARENT);
}

#[test]
fn half_transparent_red_over_opaque_blue() {
    // r = 255 * 128 / 255 = 128, b = 255 * 127 / 255 = 127
    assert_eq!(over(Rgba::new(255, 0, 0, 128), BLUE), Rgba::new(128, 0, 127, 255));
}

#[test]
fn semi_over_semi_hand_checked() {
    // alpha = 128 + 128 * 127 / 255 = 191.75; r = 128 / 191.75 * 255 = 170.2; b = 63.75 / 191.75 * 255 = 84.8
    let out = over(Rgba::new(255, 0, 0, 128), Rgba::new(0, 0, 255, 128));
    assert_eq!(out, Rgba::new(170, 0, 85, 192));
}

#[test]
fn source_over_transparent_destination_keeps_the_source() {
    let src = Rgba::new(10, 20, 30, 40);
    assert_eq!(over(src, Rgba::TRANSPARENT), src);
    assert_eq!(over(src, Rgba::new(200, 200, 200, 0)), src);
}

#[test]
fn premultiply_and_back() {
    let c = Rgba::new(255, 128, 0, 128);
    assert_eq!(premultiply(c), Rgba::new(128, 64, 0, 128));
    assert_eq!(unpremultiply(Rgba::new(128, 64, 0, 128)), c);
    assert_eq!(premultiply(Rgba::new(9, 9, 9, 255)), Rgba::new(9, 9, 9, 255));
    assert_eq!(premultiply(Rgba::new(9, 9, 9, 0)), Rgba::TRANSPARENT);
    assert_eq!(unpremultiply(Rgba::new(5, 5, 5, 0)), Rgba::TRANSPARENT);
    assert_eq!(unpremultiply(Rgba::new(200, 0, 0, 100)), Rgba::new(255, 0, 0, 100), "clamped");
}

#[test]
fn premultiplied_over_matches_straight_over() {
    let src = Rgba::new(255, 0, 0, 128);
    let out = over_premultiplied(premultiply(src), premultiply(BLUE));
    assert_eq!(out, Rgba::new(128, 0, 127, 255));
    assert_eq!(unpremultiply(out), over(src, BLUE));
    assert_eq!(over_premultiplied(Rgba::TRANSPARENT, premultiply(BLUE)), BLUE);
}

#[test]
fn separable_modes_on_opaque_pixels() {
    let s = Rgba::opaque(200, 100, 50);
    let d = Rgba::opaque(128, 255, 0);
    assert_eq!(blend(BlendMode::Multiply, s, d), Rgba::opaque(100, 100, 0));
    // screen = 255 - (255 - s) * (255 - d) / 255: 255 - 55 * 127 / 255 = 255 - 27
    assert_eq!(blend(BlendMode::Screen, s, d), Rgba::opaque(228, 255, 50));
    assert_eq!(blend(BlendMode::Add, s, d), Rgba::opaque(255, 255, 50));
    assert_eq!(blend(BlendMode::Darken, s, d), Rgba::opaque(128, 100, 0));
    assert_eq!(blend(BlendMode::Lighten, s, d), Rgba::opaque(200, 255, 50));
    assert_eq!(blend(BlendMode::Difference, s, d), Rgba::opaque(72, 155, 50));
}

#[test]
fn replace_and_over_modes() {
    let s = Rgba::new(255, 0, 0, 128);
    assert_eq!(blend(BlendMode::Replace, s, BLUE), s);
    assert_eq!(blend(BlendMode::Over, s, BLUE), over(s, BLUE));
}

#[test]
fn blend_modes_respect_alpha() {
    let s = Rgba::new(200, 100, 50, 128);
    // No destination coverage: the source passes through unchanged.
    assert_eq!(blend(BlendMode::Multiply, s, Rgba::TRANSPARENT), s);
    // Transparent source leaves the destination alone.
    assert_eq!(blend(BlendMode::Multiply, Rgba::new(200, 100, 50, 0), BLUE), BLUE);
    // Multiply with white is the identity on the colour, so this equals plain "over".
    let white = Rgba::WHITE;
    assert_eq!(blend(BlendMode::Multiply, s, white), over(s, white));
    // Half-covered multiply over opaque: mixed = (100, 100, 0), then over (128, 255, 0).
    // r = (100 * 128 + 128 * 127) / 255 = 113.9, g = (100 * 128 + 255 * 127) / 255 = 177.2
    assert_eq!(blend(BlendMode::Multiply, s, Rgba::opaque(128, 255, 0)), Rgba::opaque(114, 177, 0));
}

#[test]
fn mode_names() {
    assert_eq!(BlendMode::from_name("multiply"), Some(BlendMode::Multiply));
    assert_eq!(BlendMode::from_name("difference"), Some(BlendMode::Difference));
    assert_eq!(BlendMode::from_name("Multiply"), None);
    assert_eq!(BlendMode::from_name(""), None);
}

#[test]
fn flatten_composites_over_the_background() {
    let img = Image::from_vec(
        3,
        1,
        vec![Rgba::new(255, 0, 0, 255), Rgba::new(255, 0, 0, 128), Rgba::new(255, 0, 0, 0)],
    )
    .unwrap();
    let flat = flatten(&img, Rgb::new(0, 0, 255));
    assert_eq!(flat.as_slice(), &[Rgb::new(255, 0, 0), Rgb::new(128, 0, 127), Rgb::new(0, 0, 255)]);
}
