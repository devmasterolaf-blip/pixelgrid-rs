use pixelgrid::color::{linear_to_srgb, mix_linear, relative_luminance, srgb_to_linear};
use pixelgrid::Rgb;

#[test]
fn end_points_are_exact() {
    assert_eq!(srgb_to_linear(0), 0.0);
    assert!((srgb_to_linear(255) - 1.0).abs() < 1e-6);
    assert_eq!(linear_to_srgb(0.0), 0);
    assert_eq!(linear_to_srgb(1.0), 255);
}

#[test]
fn known_values_of_the_transfer_curve() {
    // The linear segment: 10 / 255 / 12.92
    assert!((srgb_to_linear(10) - 0.003_035_27).abs() < 1e-6);
    // Mid grey 128 decodes to about 21.6 % linear light.
    assert!((srgb_to_linear(128) - 0.215_86).abs() < 1e-4);
    // Half linear light encodes to 188 (1.055 * 0.5^(1/2.4) - 0.055 = 0.7354).
    assert_eq!(linear_to_srgb(0.5), 188);
    assert_eq!(linear_to_srgb(0.215_86), 128);
    assert_eq!(linear_to_srgb(0.001), 3, "12.92 * 0.001 * 255 = 3.29");
}

#[test]
fn every_byte_round_trips() {
    for v in 0..=255u8 {
        assert_eq!(linear_to_srgb(srgb_to_linear(v)), v);
    }
}

#[test]
fn decoding_is_monotonic() {
    for v in 1..=255u8 {
        assert!(srgb_to_linear(v) > srgb_to_linear(v - 1));
    }
}

#[test]
fn encoding_clamps_out_of_range_input() {
    assert_eq!(linear_to_srgb(-0.5), 0);
    assert_eq!(linear_to_srgb(1.5), 255);
}

#[test]
fn luminance_weights_sum_to_one() {
    assert_eq!(relative_luminance(Rgb::BLACK), 0.0);
    assert!((relative_luminance(Rgb::WHITE) - 1.0).abs() < 1e-5);
    assert!((relative_luminance(Rgb::new(255, 0, 0)) - 0.2126).abs() < 1e-5);
    assert!((relative_luminance(Rgb::new(0, 255, 0)) - 0.7152).abs() < 1e-5);
    assert!((relative_luminance(Rgb::new(0, 0, 255)) - 0.0722).abs() < 1e-5);
}

#[test]
fn mixing_happens_in_linear_light() {
    let (black, white) = (Rgb::BLACK, Rgb::WHITE);
    assert_eq!(mix_linear(black, white, 0.0), black);
    assert_eq!(mix_linear(black, white, 1.0), white);
    // A naive mix would give 128; the linear-light midpoint is brighter.
    assert_eq!(mix_linear(black, white, 0.5), Rgb::new(188, 188, 188));
    assert_eq!(mix_linear(Rgb::new(255, 0, 0), Rgb::new(0, 0, 255), 0.5), Rgb::new(188, 0, 188));
    assert_eq!(mix_linear(black, white, 7.0), white, "t is clamped");
}
