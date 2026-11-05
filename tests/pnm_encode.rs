mod common;

use common::{gray, rgb_pattern};
use pixelgrid::io::pnm::{decode_pnm, encode_pbm, encode_pgm, encode_ppm, PnmEncoding};
use pixelgrid::{DynImage, EncodeError, Gray, Image, Rgb};

#[test]
fn binary_pgm_bytes() {
    let img = gray(&[&[0, 127], &[128, 255]]);
    assert_eq!(encode_pgm(&img, PnmEncoding::Binary).unwrap(), b"P5\n2 2\n255\n\x00\x7f\x80\xff");
}

#[test]
fn ascii_pgm_has_one_line_per_row() {
    let img = gray(&[&[0, 10, 200], &[30, 40, 255]]);
    assert_eq!(encode_pgm(&img, PnmEncoding::Ascii).unwrap(), b"P2\n3 2\n255\n0 10 200\n30 40 255\n");
}

#[test]
fn binary_and_ascii_ppm_bytes() {
    let img = Image::from_vec(2, 1, vec![Rgb::new(255, 0, 0), Rgb::new(1, 2, 3)]).unwrap();
    assert_eq!(encode_ppm(&img, PnmEncoding::Binary).unwrap(), b"P6\n2 1\n255\n\xff\x00\x00\x01\x02\x03");
    assert_eq!(encode_ppm(&img, PnmEncoding::Ascii).unwrap(), b"P3\n2 1\n255\n255 0 0 1 2 3\n");
}

#[test]
fn pbm_thresholds_at_128_and_packs_bits() {
    let img = gray(&[&[0, 127, 128, 255, 0, 0, 0, 0, 0, 255]]);
    assert_eq!(encode_pbm(&img, PnmEncoding::Ascii).unwrap(), b"P1\n10 1\n1 1 0 0 1 1 1 1 1 0\n");
    // 1100_1111 1000_0000: the second byte holds two pixels and six padding bits
    assert_eq!(encode_pbm(&img, PnmEncoding::Binary).unwrap(), b"P4\n10 1\n\xcf\x80");
}

#[test]
fn long_ascii_rows_wrap_before_70_characters() {
    let img = Image::from_fn(40, 2, |x, _| Gray(200 + x as u8));
    let text = String::from_utf8(encode_pgm(&img, PnmEncoding::Ascii).unwrap()).unwrap();
    assert!(text.lines().all(|l| l.len() <= 70), "a line is too long");
    // 40 values of "2xx" need 159 characters, so each row spans three lines.
    assert_eq!(text.lines().count(), 3 + 2 * 3);
    assert_eq!(decode_pnm(text.as_bytes()).unwrap(), DynImage::Gray(img));
}

#[test]
fn every_variant_round_trips() {
    let grey = Image::from_fn(5, 4, |x, y| Gray((x * 50 + y * 3) as u8));
    let rgb = rgb_pattern(5, 4);
    for encoding in [PnmEncoding::Ascii, PnmEncoding::Binary] {
        let pgm = encode_pgm(&grey, encoding).unwrap();
        assert_eq!(decode_pnm(&pgm).unwrap(), DynImage::Gray(grey.clone()));
        let ppm = encode_ppm(&rgb, encoding).unwrap();
        assert_eq!(decode_pnm(&ppm).unwrap(), DynImage::Rgb(rgb.clone()));
    }
}

#[test]
fn pbm_round_trips_bilevel_images() {
    let img = Image::from_fn(11, 3, |x, y| Gray(if (x + y) % 3 == 0 { 0 } else { 255 }));
    for encoding in [PnmEncoding::Ascii, PnmEncoding::Binary] {
        let pbm = encode_pbm(&img, encoding).unwrap();
        assert_eq!(decode_pnm(&pbm).unwrap(), DynImage::Gray(img.clone()));
    }
}

#[test]
fn default_encoding_is_binary() {
    assert_eq!(PnmEncoding::default(), PnmEncoding::Binary);
}

#[test]
fn empty_images_cannot_be_encoded() {
    let empty: Image<Gray> = Image::new(0, 3);
    let err = encode_pgm(&empty, PnmEncoding::Binary).unwrap_err();
    assert_eq!(err, EncodeError::EmptyImage { width: 0, height: 3 });
    assert_eq!(err.to_string(), "cannot encode an empty 0x3 image");
    assert!(encode_pbm(&empty, PnmEncoding::Ascii).is_err());
    assert!(encode_ppm(&Image::<Rgb>::new(2, 0), PnmEncoding::Ascii).is_err());
}
