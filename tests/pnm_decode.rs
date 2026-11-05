mod common;

use common::gray_rows;
use pixelgrid::io::pnm::{decode_pnm, read_pnm_header};
use pixelgrid::{DecodeError, DynImage, Gray, Image, Rgb};

fn decode_gray(data: &[u8]) -> Image<Gray> {
    match decode_pnm(data).expect("decode") {
        DynImage::Gray(img) => img,
        other => panic!("expected a grey image, got {}", other.pixel_name()),
    }
}

fn decode_rgb(data: &[u8]) -> Image<Rgb> {
    match decode_pnm(data).expect("decode") {
        DynImage::Rgb(img) => img,
        other => panic!("expected an RGB image, got {}", other.pixel_name()),
    }
}

#[test]
fn ascii_pgm() {
    let img = decode_gray(b"P2\n3 2\n255\n0 10 20\n30 40 255\n");
    assert_eq!(gray_rows(&img), vec![vec![0, 10, 20], vec![30, 40, 255]]);
}

#[test]
fn binary_pgm() {
    let img = decode_gray(b"P5\n2 2\n255\n\x00\x7f\x80\xff");
    assert_eq!(gray_rows(&img), vec![vec![0, 127], vec![128, 255]]);
}

#[test]
fn ascii_and_binary_ppm() {
    let ascii = decode_rgb(b"P3 2 1 255 255 0 0 1 2 3");
    assert_eq!(ascii.as_slice(), &[Rgb::new(255, 0, 0), Rgb::new(1, 2, 3)]);
    let binary = decode_rgb(b"P6\n2 1\n255\n\xff\x00\x00\x01\x02\x03");
    assert_eq!(binary, ascii);
}

#[test]
fn ascii_pbm_maps_one_to_black() {
    let img = decode_gray(b"P1\n3 2\n1 0 1\n0 1 0\n");
    assert_eq!(gray_rows(&img), vec![vec![0, 255, 0], vec![255, 0, 255]]);
    // The digits may run together.
    assert_eq!(decode_gray(b"P1 3 2 101010"), img);
}

#[test]
fn binary_pbm_rows_are_padded_to_whole_bytes() {
    // 10 pixels per row need two bytes: 0b1010_0000 0b0100_0000 -> bits 1,0,1,0,0,0,0,0,0,1
    let img = decode_gray(b"P4\n10 2\n\xa0\x40\x00\xff");
    assert_eq!(
        gray_rows(&img),
        vec![vec![0, 255, 0, 255, 255, 255, 255, 255, 255, 0], vec![255, 255, 255, 255, 255, 255, 255, 255, 0, 0]]
    );
}

#[test]
fn comments_are_allowed_anywhere_in_the_header() {
    let data = b"P2 # magic\n# a full-line comment\n2 # width\n# before height\n1\n# before maxval\n255\n7 9\n";
    assert_eq!(gray_rows(&decode_gray(data)), vec![vec![7, 9]]);
    let header = read_pnm_header(b"P5\n1 1\n255# glued comment\n\x2a").unwrap();
    assert_eq!(header.raster_offset, 26);
    assert_eq!(gray_rows(&decode_gray(b"P5\n1 1\n255# glued comment\n\x2a")), vec![vec![42]]);
}

#[test]
fn comments_in_an_ascii_raster_are_skipped() {
    let img = decode_gray(b"P2\n2 1\n255\n1 # first\n2\n");
    assert_eq!(gray_rows(&img), vec![vec![1, 2]]);
}

#[test]
fn small_maxval_is_scaled_to_eight_bits() {
    // maxval 3: 0, 1, 2, 3 -> 0, 85, 170, 255
    assert_eq!(gray_rows(&decode_gray(b"P2 4 1 3 0 1 2 3")), vec![vec![0, 85, 170, 255]]);
    // maxval 100: 50 -> 127.5 rounds to 128
    assert_eq!(gray_rows(&decode_gray(b"P2 1 1 100 50")), vec![vec![128]]);
    assert_eq!(gray_rows(&decode_gray(b"P5 2 1 15\n\x00\x0f")), vec![vec![0, 255]]);
}

#[test]
fn sixteen_bit_binary_samples_are_big_endian() {
    // 0x0000, 0x8000 (32768 -> 127.5 -> 128), 0xffff
    let img = decode_gray(b"P5\n3 1\n65535\n\x00\x00\x80\x00\xff\xff");
    assert_eq!(gray_rows(&img), vec![vec![0, 128, 255]]);
    // maxval 256 already needs two bytes per sample: 0x0100 = 256 -> 255
    assert_eq!(gray_rows(&decode_gray(b"P5\n1 1\n256\n\x01\x00")), vec![vec![255]]);
    let rgb = decode_rgb(b"P6 1 1 65535\n\xff\xff\x00\x00\x80\x00");
    assert_eq!(rgb.as_slice(), &[Rgb::new(255, 0, 128)]);
}

#[test]
fn sixteen_bit_ascii_samples() {
    assert_eq!(gray_rows(&decode_gray(b"P2 2 1 65535 65535 257")), vec![vec![255, 1]]);
}

#[test]
fn header_fields_are_reported() {
    let h = read_pnm_header(b"P6\n4 3\n255\n").unwrap();
    assert_eq!((h.magic, h.width, h.height, h.maxval, h.raster_offset), (6, 4, 3, 255, 11));
    assert!(!h.is_ascii() && !h.is_bitmap());
    assert_eq!((h.channels(), h.format_name()), (3, "PPM"));
    let h = read_pnm_header(b"P1 5 6 ").unwrap();
    assert!(h.is_ascii() && h.is_bitmap());
    assert_eq!((h.maxval, h.channels(), h.format_name()), (1, 1, "PBM"));
    assert_eq!(read_pnm_header(b"P2 1 1 9").unwrap().format_name(), "PGM");
}

#[test]
fn data_after_the_raster_is_ignored() {
    assert_eq!(gray_rows(&decode_gray(b"P5 1 1 255\n\x07trailing bytes")), vec![vec![7]]);
    assert_eq!(gray_rows(&decode_gray(b"P2 1 1 255 7 8 9")), vec![vec![7]]);
}

#[test]
fn bad_magic() {
    assert_eq!(decode_pnm(b""), Err(DecodeError::BadMagic { found: vec![] }));
    assert_eq!(decode_pnm(b"P"), Err(DecodeError::BadMagic { found: vec![b'P'] }));
    assert_eq!(decode_pnm(b"P7\n1 1\n"), Err(DecodeError::BadMagic { found: vec![b'P', b'7'] }));
    assert_eq!(decode_pnm(b"BM...."), Err(DecodeError::BadMagic { found: vec![b'B', b'M'] }));
}

#[test]
fn truncated_headers_name_the_missing_field() {
    assert_eq!(decode_pnm(b"P5"), Err(DecodeError::UnexpectedEof { offset: 2, expected: "width" }));
    assert_eq!(decode_pnm(b"P5 3"), Err(DecodeError::UnexpectedEof { offset: 4, expected: "height" }));
    assert_eq!(decode_pnm(b"P5 3 2\n"), Err(DecodeError::UnexpectedEof { offset: 7, expected: "maxval" }));
    assert_eq!(decode_pnm(b"P5 3 2 255"), Err(DecodeError::UnexpectedEof { offset: 10, expected: "raster" }));
    assert_eq!(decode_pnm(b"P4 3 2"), Err(DecodeError::UnexpectedEof { offset: 6, expected: "raster" }));
    let err = decode_pnm(b"P5 3 # only a comment").unwrap_err();
    assert_eq!(err, DecodeError::UnexpectedEof { offset: 21, expected: "height" });
    assert_eq!(err.to_string(), "unexpected end of data at byte 21 while reading height");
}

#[test]
fn malformed_header_numbers() {
    let err = decode_pnm(b"P2\n3 x2\n255\n").unwrap_err();
    assert_eq!(err, DecodeError::InvalidNumber { offset: 5, field: "height", token: "x2".into() });
    assert_eq!(err.to_string(), "invalid height 'x2' at byte 5");
    assert_eq!(
        decode_pnm(b"P2 -3 2 255"),
        Err(DecodeError::InvalidNumber { offset: 3, field: "width", token: "-3".into() })
    );
    assert_eq!(
        decode_pnm(b"P2 3 2 2.5"),
        Err(DecodeError::InvalidNumber { offset: 7, field: "maxval", token: "2.5".into() })
    );
    assert!(matches!(
        decode_pnm(b"P2 99999999999999999999999 2 255"),
        Err(DecodeError::InvalidNumber { field: "width", .. })
    ));
}

#[test]
fn invalid_dimensions_and_maxval() {
    assert_eq!(decode_pnm(b"P2 0 2 255"), Err(DecodeError::InvalidDimensions { width: 0, height: 2 }));
    assert_eq!(decode_pnm(b"P1 2 0 "), Err(DecodeError::InvalidDimensions { width: 2, height: 0 }));
    assert_eq!(
        decode_pnm(b"P2 4294967296 1 255"),
        Err(DecodeError::InvalidDimensions { width: 4294967296, height: 1 })
    );
    assert_eq!(decode_pnm(b"P2 1 1 0 0"), Err(DecodeError::InvalidMaxval { value: 0 }));
    let err = decode_pnm(b"P5 1 1 65536\n\x00\x00").unwrap_err();
    assert_eq!(err, DecodeError::InvalidMaxval { value: 65536 });
    assert_eq!(err.to_string(), "maxval 65536 is outside 1..=65535");
}

#[test]
fn truncated_binary_raster_counts_bytes() {
    let err = decode_pnm(b"P5\n3 2\n255\n\x01\x02\x03\x04").unwrap_err();
    assert_eq!(err, DecodeError::TruncatedRaster { expected: 6, actual: 4 });
    assert_eq!(err.to_string(), "pixel data is truncated: expected 6, found 4");
    assert_eq!(
        decode_pnm(b"P6\n2 1\n65535\n\x00\x00\x00"),
        Err(DecodeError::TruncatedRaster { expected: 12, actual: 3 })
    );
    assert_eq!(decode_pnm(b"P4\n9 2\n\xff\xff\xff"), Err(DecodeError::TruncatedRaster { expected: 4, actual: 3 }));
}

#[test]
fn truncated_ascii_raster_counts_samples() {
    assert_eq!(decode_pnm(b"P2 3 2 255 1 2 3 4"), Err(DecodeError::TruncatedRaster { expected: 6, actual: 4 }));
    assert_eq!(decode_pnm(b"P3 1 1 255 1 2"), Err(DecodeError::TruncatedRaster { expected: 3, actual: 2 }));
    assert_eq!(decode_pnm(b"P1 2 2 1 0 1"), Err(DecodeError::TruncatedRaster { expected: 4, actual: 3 }));
}

#[test]
fn huge_declared_sizes_fail_without_allocating() {
    assert_eq!(
        decode_pnm(b"P5 4000000000 4000000000 255\n\x00"),
        Err(DecodeError::TruncatedRaster { expected: 16_000_000_000_000_000_000, actual: 1 })
    );
    assert!(matches!(decode_pnm(b"P2 4000000000 4000000000 255 1 2"), Err(DecodeError::TruncatedRaster { actual: 2, .. })));
    assert!(matches!(decode_pnm(b"P6 4294967295 4294967295 255\n"), Err(DecodeError::InvalidDimensions { .. })));
}

#[test]
fn samples_above_maxval_are_rejected() {
    let err = decode_pnm(b"P2 2 1 100 100 101").unwrap_err();
    assert_eq!(err, DecodeError::SampleOutOfRange { offset: 15, value: 101, maxval: 100 });
    assert_eq!(err.to_string(), "sample 101 at byte 15 exceeds the maxval 100");
    assert_eq!(
        decode_pnm(b"P5 2 1 100\n\x64\x65"),
        Err(DecodeError::SampleOutOfRange { offset: 12, value: 101, maxval: 100 })
    );
    assert_eq!(
        decode_pnm(b"P5 1 1 300\n\x01\x2d"),
        Err(DecodeError::SampleOutOfRange { offset: 11, value: 301, maxval: 300 })
    );
}

#[test]
fn malformed_ascii_samples() {
    assert_eq!(
        decode_pnm(b"P2 2 1 255 12 ab"),
        Err(DecodeError::InvalidNumber { offset: 14, field: "sample", token: "ab".into() })
    );
    assert_eq!(
        decode_pnm(b"P1 2 1 1 2"),
        Err(DecodeError::InvalidNumber { offset: 9, field: "bit", token: "2".into() })
    );
}
