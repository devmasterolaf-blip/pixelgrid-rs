use pixelgrid::color::{named_color, parse_color, ColorParseError};
use pixelgrid::{Rgb, Rgba};

#[test]
fn six_and_eight_digit_hex() {
    assert_eq!(parse_color("#ff8000"), Ok(Rgba::new(255, 128, 0, 255)));
    assert_eq!(parse_color("#FF800040"), Ok(Rgba::new(255, 128, 0, 64)));
    assert_eq!(parse_color("#000000"), Ok(Rgba::BLACK));
    assert_eq!(parse_color("#00000000"), Ok(Rgba::TRANSPARENT));
}

#[test]
fn short_hex_repeats_each_digit() {
    assert_eq!(parse_color("#f80"), Ok(Rgba::new(255, 136, 0, 255)));
    assert_eq!(parse_color("#f80c"), Ok(Rgba::new(255, 136, 0, 204)));
    assert_eq!(parse_color("#abc"), parse_color("#aabbcc"));
}

#[test]
fn names_are_case_insensitive_and_trimmed() {
    assert_eq!(parse_color("red"), Ok(Rgba::opaque(255, 0, 0)));
    assert_eq!(parse_color("  Navy "), Ok(Rgba::opaque(0, 0, 128)));
    assert_eq!(parse_color("ORANGE"), Ok(Rgba::opaque(255, 165, 0)));
    assert_eq!(parse_color("grey"), parse_color("gray"));
    assert_eq!(parse_color("cyan"), parse_color("aqua"));
    assert_eq!(parse_color("transparent"), Ok(Rgba::TRANSPARENT));
    assert_eq!(named_color("lime"), Some(Rgb::new(0, 255, 0)));
    assert_eq!(named_color("Lime"), None, "the lookup itself is exact");
}

#[test]
fn empty_input_is_its_own_error() {
    assert_eq!(parse_color(""), Err(ColorParseError::Empty));
    assert_eq!(parse_color("   "), Err(ColorParseError::Empty));
    assert_eq!(ColorParseError::Empty.to_string(), "empty colour");
}

#[test]
fn unknown_names_keep_the_original_spelling() {
    let err = parse_color("Blurple").unwrap_err();
    assert_eq!(err, ColorParseError::UnknownName("Blurple".to_string()));
    assert_eq!(err.to_string(), "unknown colour name 'Blurple'");
    assert!(matches!(parse_color("ff0000"), Err(ColorParseError::UnknownName(_))), "missing '#'");
}

#[test]
fn hex_length_is_validated() {
    for (text, len) in [("#", 0), ("#f", 1), ("#ff", 2), ("#fffff", 5), ("#fffffff", 7), ("#fffffffff", 9)] {
        assert_eq!(parse_color(text), Err(ColorParseError::InvalidHexLength { len }), "{text}");
    }
    let message = ColorParseError::InvalidHexLength { len: 5 }.to_string();
    assert_eq!(message, "a hex colour needs 3, 4, 6 or 8 digits, got 5");
}

#[test]
fn bad_hex_digits_report_their_position() {
    let err = parse_color("#12g456").unwrap_err();
    assert_eq!(err, ColorParseError::InvalidHexDigit { index: 2, found: 'g' });
    assert_eq!(err.to_string(), "invalid hex digit 'g' at position 2");
    assert_eq!(parse_color("#-12"), Err(ColorParseError::InvalidHexDigit { index: 0, found: '-' }));
    assert_eq!(parse_color("#ff é0"), Err(ColorParseError::InvalidHexDigit { index: 2, found: ' ' }));
}

#[test]
fn from_str_for_rgba_and_rgb() {
    assert_eq!("#01020304".parse::<Rgba>(), Ok(Rgba::new(1, 2, 3, 4)));
    assert_eq!("teal".parse::<Rgb>(), Ok(Rgb::new(0, 128, 128)));
    assert_eq!("#010203ff".parse::<Rgb>(), Ok(Rgb::new(1, 2, 3)));
    assert_eq!("#01020380".parse::<Rgb>(), Err(ColorParseError::UnexpectedAlpha { alpha: 128 }));
    assert_eq!("transparent".parse::<Rgb>(), Err(ColorParseError::UnexpectedAlpha { alpha: 0 }));
}

#[test]
fn to_hex_round_trips() {
    assert_eq!(Rgb::new(255, 128, 0).to_hex(), "#ff8000");
    assert_eq!(Rgba::new(255, 128, 0, 255).to_hex(), "#ff8000");
    assert_eq!(Rgba::new(1, 2, 3, 4).to_hex(), "#01020304");
    for text in ["#000000", "#12ab9f", "#12ab9f07"] {
        assert_eq!(parse_color(text).unwrap().to_hex(), text);
    }
}
