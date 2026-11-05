use pixelgrid::color::{hsl_to_rgb, hsv_to_rgb, rgb_to_hsl, rgb_to_hsv, Hsl, Hsv};
use pixelgrid::Rgb;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

fn assert_hsv(c: Rgb, h: f32, s: f32, v: f32) {
    let got = rgb_to_hsv(c);
    assert!(close(got.h, h) && close(got.s, s) && close(got.v, v), "{c:?} -> {got:?}");
}

fn assert_hsl(c: Rgb, h: f32, s: f32, l: f32) {
    let got = rgb_to_hsl(c);
    assert!(close(got.h, h) && close(got.s, s) && close(got.l, l), "{c:?} -> {got:?}");
}

#[test]
fn primary_and_secondary_hues() {
    assert_hsv(Rgb::new(255, 0, 0), 0.0, 1.0, 1.0);
    assert_hsv(Rgb::new(255, 255, 0), 60.0, 1.0, 1.0);
    assert_hsv(Rgb::new(0, 255, 0), 120.0, 1.0, 1.0);
    assert_hsv(Rgb::new(0, 255, 255), 180.0, 1.0, 1.0);
    assert_hsv(Rgb::new(0, 0, 255), 240.0, 1.0, 1.0);
    assert_hsv(Rgb::new(255, 0, 255), 300.0, 1.0, 1.0);
}

#[test]
fn greys_have_zero_hue_and_saturation() {
    assert_hsv(Rgb::new(0, 0, 0), 0.0, 0.0, 0.0);
    assert_hsv(Rgb::new(255, 255, 255), 0.0, 0.0, 1.0);
    assert_hsv(Rgb::new(128, 128, 128), 0.0, 0.0, 128.0 / 255.0);
    assert_hsl(Rgb::new(255, 255, 255), 0.0, 0.0, 1.0);
    assert_hsl(Rgb::new(51, 51, 51), 0.0, 0.0, 0.2);
}

#[test]
fn rgb_to_hsv_hand_checked() {
    // max = 0.8 (blue), min = 0.4, delta = 0.4: h = 60 * ((0.4 - 0.6) / 0.4 + 4) = 210
    assert_hsv(Rgb::new(102, 153, 204), 210.0, 0.5, 0.8);
    // a hue just below 360: h = 60 * ((0 - 0.2) / 1 mod 6) = 348
    assert_hsv(Rgb::new(255, 0, 51), 348.0, 1.0, 1.0);
}

#[test]
fn hsv_to_rgb_hand_checked() {
    // chroma 0.4, x 0.2, m 0.4 in sector 3 gives (0.4, 0.6, 0.8)
    assert_eq!(hsv_to_rgb(Hsv { h: 210.0, s: 0.5, v: 0.8 }), Rgb::new(102, 153, 204));
    // x = 0.5 gives 127.5, which rounds up
    assert_eq!(hsv_to_rgb(Hsv { h: 30.0, s: 1.0, v: 1.0 }), Rgb::new(255, 128, 0));
    assert_eq!(hsv_to_rgb(Hsv { h: 0.0, s: 0.0, v: 0.2 }), Rgb::new(51, 51, 51));
}

#[test]
fn rgb_to_hsl_hand_checked() {
    assert_hsl(Rgb::new(255, 0, 0), 0.0, 1.0, 0.5);
    // l = (0.8 + 0.4) / 2 = 0.6, s = 0.4 / (1 - |1.2 - 1|) = 0.5
    assert_hsl(Rgb::new(102, 153, 204), 210.0, 0.5, 0.6);
    assert_hsl(Rgb::new(0, 0, 128), 240.0, 1.0, 64.0 / 255.0);
}

#[test]
fn hsl_to_rgb_hand_checked() {
    assert_eq!(hsl_to_rgb(Hsl { h: 210.0, s: 0.5, l: 0.6 }), Rgb::new(102, 153, 204));
    assert_eq!(hsl_to_rgb(Hsl { h: 120.0, s: 1.0, l: 0.25 }), Rgb::new(0, 128, 0));
    assert_eq!(hsl_to_rgb(Hsl { h: 77.0, s: 0.3, l: 1.0 }), Rgb::new(255, 255, 255));
    assert_eq!(hsl_to_rgb(Hsl { h: 77.0, s: 0.3, l: 0.0 }), Rgb::new(0, 0, 0));
}

#[test]
fn hue_wraps_and_other_components_clamp() {
    let blue = Rgb::new(0, 0, 255);
    assert_eq!(hsv_to_rgb(Hsv { h: -120.0, s: 1.0, v: 1.0 }), blue);
    assert_eq!(hsv_to_rgb(Hsv { h: 600.0, s: 1.0, v: 1.0 }), blue);
    assert_eq!(hsv_to_rgb(Hsv { h: 360.0, s: 1.0, v: 1.0 }), Rgb::new(255, 0, 0));
    assert_eq!(hsv_to_rgb(Hsv { h: 240.0, s: 7.0, v: 3.0 }), blue);
    assert_eq!(hsl_to_rgb(Hsl { h: 240.0, s: 2.0, l: 0.5 }), blue);
    assert_eq!(hsl_to_rgb(Hsl { h: 240.0, s: -1.0, l: 0.5 }), Rgb::new(128, 128, 128));
}

#[test]
fn round_trips_are_exact_on_a_colour_grid() {
    for r in (0..=255u32).step_by(17) {
        for g in (0..=255u32).step_by(17) {
            for b in (0..=255u32).step_by(17) {
                let c = Rgb::new(r as u8, g as u8, b as u8);
                assert_eq!(hsv_to_rgb(rgb_to_hsv(c)), c, "hsv round trip");
                assert_eq!(hsl_to_rgb(rgb_to_hsl(c)), c, "hsl round trip");
            }
        }
    }
}

#[test]
fn from_impls_match_the_functions() {
    let c = Rgb::new(12, 200, 99);
    assert_eq!(Hsv::from(c), rgb_to_hsv(c));
    assert_eq!(Hsl::from(c), rgb_to_hsl(c));
    assert_eq!(Rgb::from(Hsv::from(c)), c);
    assert_eq!(Rgb::from(Hsl::from(c)), c);
}
