mod common;

use common::{gray, rgb_pattern};
use pixelgrid::io::bmp::{decode_bmp, encode_bmp, read_bmp_header, RowOrder};
use pixelgrid::{DecodeError, DynImage, EncodeError, Gray, Image, Rgb, Rgba};

/// Builds a BMP by hand so the decoder is not only tested against our own encoder.
fn raw_bmp(width: i32, height: i32, bits: u16, pixels: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54 + pixels.len() as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(&[0; 24]);
    out.extend_from_slice(pixels);
    out
}

fn two_by_two() -> Image<Rgb> {
    Image::from_vec(2, 2, vec![Rgb::new(255, 0, 0), Rgb::new(0, 255, 0), Rgb::new(0, 0, 255), Rgb::new(1, 2, 3)])
        .unwrap()
}

#[test]
fn encodes_a_bottom_up_24_bit_file_byte_for_byte() {
    let bytes = encode_bmp(&DynImage::Rgb(two_by_two()), RowOrder::BottomUp).unwrap();
    // Stored rows are bottom first, pixels are BGR, each 6-byte row is padded to 8 bytes.
    let pixels = [255, 0, 0, 3, 2, 1, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0];
    assert_eq!(bytes.len(), 54 + 16);
    assert_eq!(&bytes[..2], b"BM");
    assert_eq!(&bytes[2..6], &70u32.to_le_bytes(), "file size");
    assert_eq!(&bytes[10..14], &54u32.to_le_bytes(), "data offset");
    assert_eq!(&bytes[14..18], &40u32.to_le_bytes(), "info header size");
    assert_eq!(&bytes[18..22], &2i32.to_le_bytes(), "width");
    assert_eq!(&bytes[22..26], &2i32.to_le_bytes(), "height");
    assert_eq!(&bytes[26..30], &[1, 0, 24, 0], "planes and bit count");
    assert_eq!(&bytes[30..34], &[0; 4], "no compression");
    assert_eq!(&bytes[34..38], &16u32.to_le_bytes(), "image size");
    assert_eq!(&bytes[54..], &pixels);
}

#[test]
fn top_down_files_store_a_negative_height() {
    let bytes = encode_bmp(&DynImage::Rgb(two_by_two()), RowOrder::TopDown).unwrap();
    assert_eq!(&bytes[22..26], &(-2i32).to_le_bytes());
    assert_eq!(&bytes[54..], &[0, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0, 3, 2, 1, 0, 0]);
    let header = read_bmp_header(&bytes).unwrap();
    assert_eq!((header.width, header.height, header.row_order), (2, 2, RowOrder::TopDown));
    assert_eq!(decode_bmp(&bytes).unwrap(), DynImage::Rgb(two_by_two()));
}

#[test]
fn decodes_hand_built_files_in_both_row_orders() {
    let pixels = [255, 0, 0, 3, 2, 1, 0, 0, 0, 0, 255, 0, 255, 0, 0, 0];
    assert_eq!(decode_bmp(&raw_bmp(2, 2, 24, &pixels)).unwrap(), DynImage::Rgb(two_by_two()));
    let flipped = Image::from_vec(
        2,
        2,
        vec![Rgb::new(0, 0, 255), Rgb::new(1, 2, 3), Rgb::new(255, 0, 0), Rgb::new(0, 255, 0)],
    )
    .unwrap();
    assert_eq!(decode_bmp(&raw_bmp(2, -2, 24, &pixels)).unwrap(), DynImage::Rgb(flipped));
}

#[test]
fn row_padding_for_every_width_modulo_four() {
    for width in 1..=5u32 {
        let img = rgb_pattern(width, 3);
        let bytes = encode_bmp(&DynImage::Rgb(img.clone()), RowOrder::BottomUp).unwrap();
        let stride = (width as usize * 3).div_ceil(4) * 4;
        assert_eq!(read_bmp_header(&bytes).unwrap().stride(), stride);
        assert_eq!(bytes.len(), 54 + stride * 3, "width {width}");
        assert_eq!(decode_bmp(&bytes).unwrap(), DynImage::Rgb(img), "width {width}");
    }
}

#[test]
fn thirty_two_bit_files_keep_alpha() {
    let img = Image::from_vec(1, 2, vec![Rgba::new(10, 20, 30, 40), Rgba::new(50, 60, 70, 255)]).unwrap();
    let bytes = encode_bmp(&DynImage::Rgba(img.clone()), RowOrder::BottomUp).unwrap();
    assert_eq!(&bytes[28..30], &[32, 0]);
    assert_eq!(&bytes[54..], &[70, 60, 50, 255, 30, 20, 10, 40], "BGRA, bottom row first");
    assert_eq!(decode_bmp(&bytes).unwrap(), DynImage::Rgba(img));
}

#[test]
fn grey_images_are_written_as_24_bit() {
    let bytes = encode_bmp(&DynImage::Gray(gray(&[&[7, 200]])), RowOrder::BottomUp).unwrap();
    assert_eq!(&bytes[54..], &[7, 7, 7, 200, 200, 200, 0, 0]);
    let back = decode_bmp(&bytes).unwrap();
    assert_eq!(back.pixel_name(), "rgb");
    assert_eq!(back.to_gray(), gray(&[&[7, 200]]));
}

#[test]
fn larger_info_headers_are_accepted() {
    // A BITMAPV4HEADER is 108 bytes; the extra fields sit between the header and the pixels.
    let mut bytes = raw_bmp(1, 1, 24, &[]);
    bytes[14..18].copy_from_slice(&108u32.to_le_bytes());
    bytes[10..14].copy_from_slice(&122u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 68]);
    bytes.extend_from_slice(&[3, 2, 1, 0]);
    assert_eq!(decode_bmp(&bytes).unwrap().to_rgb().as_slice(), &[Rgb::new(1, 2, 3)]);
}

#[test]
fn missing_padding_on_the_last_row_is_tolerated() {
    let pixels = [9, 8, 7, 0, 3, 2, 1];
    let img = decode_bmp(&raw_bmp(1, 2, 24, &pixels)).unwrap().to_rgb();
    assert_eq!(img.as_slice(), &[Rgb::new(1, 2, 3), Rgb::new(7, 8, 9)]);
}

#[test]
fn bad_magic_and_truncated_headers() {
    assert_eq!(decode_bmp(b""), Err(DecodeError::BadMagic { found: vec![] }));
    assert_eq!(decode_bmp(b"P6\n1 1\n"), Err(DecodeError::BadMagic { found: vec![b'P', b'6'] }));
    assert_eq!(decode_bmp(b"BM"), Err(DecodeError::UnexpectedEof { offset: 2, expected: "file header" }));
    let full = raw_bmp(1, 1, 24, &[1, 2, 3, 0]);
    assert_eq!(decode_bmp(&full[..16]), Err(DecodeError::UnexpectedEof { offset: 16, expected: "info header size" }));
    assert_eq!(decode_bmp(&full[..24]), Err(DecodeError::UnexpectedEof { offset: 24, expected: "height" }));
    assert_eq!(decode_bmp(&full[..29]), Err(DecodeError::UnexpectedEof { offset: 29, expected: "bits per pixel" }));
}

#[test]
fn truncated_pixel_data() {
    let full = raw_bmp(2, 2, 24, &[0; 16]);
    let err = decode_bmp(&full[..full.len() - 3]).unwrap_err();
    // 8 bytes for the first stored row plus 6 for the last one.
    assert_eq!(err, DecodeError::TruncatedRaster { expected: 14, actual: 13 });
    assert_eq!(decode_bmp(&full[..54]), Err(DecodeError::TruncatedRaster { expected: 14, actual: 0 }));
}

#[test]
fn invalid_header_values() {
    let px = [0u8; 4];
    assert_eq!(decode_bmp(&raw_bmp(0, 1, 24, &px)), Err(DecodeError::InvalidHeader { field: "width", value: 0 }));
    assert_eq!(decode_bmp(&raw_bmp(-4, 1, 24, &px)), Err(DecodeError::InvalidHeader { field: "width", value: -4 }));
    assert_eq!(decode_bmp(&raw_bmp(1, 0, 24, &px)), Err(DecodeError::InvalidHeader { field: "height", value: 0 }));
    assert_eq!(
        decode_bmp(&raw_bmp(1, i32::MIN, 24, &px)),
        Err(DecodeError::InvalidHeader { field: "height", value: i64::from(i32::MIN) })
    );
    let mut planes = raw_bmp(1, 1, 24, &px);
    planes[26] = 3;
    assert_eq!(decode_bmp(&planes), Err(DecodeError::InvalidHeader { field: "planes", value: 3 }));
    let mut offset = raw_bmp(1, 1, 24, &px);
    offset[10] = 20;
    let err = decode_bmp(&offset).unwrap_err();
    assert_eq!(err, DecodeError::InvalidHeader { field: "data offset", value: 20 });
    assert_eq!(err.to_string(), "invalid header field data offset: 20");
}

#[test]
fn unsupported_variants_are_named() {
    let px = [0u8; 4];
    let err = decode_bmp(&raw_bmp(1, 1, 8, &px)).unwrap_err();
    assert_eq!(err, DecodeError::Unsupported { feature: "8-bit BMP".into() });
    assert_eq!(err.to_string(), "unsupported: 8-bit BMP");
    let mut rle = raw_bmp(1, 1, 24, &px);
    rle[30] = 1;
    assert_eq!(decode_bmp(&rle), Err(DecodeError::Unsupported { feature: "BMP compression 1".into() }));
    let mut core = raw_bmp(1, 1, 24, &px);
    core[14] = 12;
    assert_eq!(decode_bmp(&core), Err(DecodeError::Unsupported { feature: "BMP info header of 12 bytes".into() }));
}

#[test]
fn huge_declared_sizes_fail_without_allocating() {
    let err = decode_bmp(&raw_bmp(i32::MAX, i32::MAX, 32, &[0; 4])).unwrap_err();
    assert!(matches!(err, DecodeError::TruncatedRaster { actual: 4, .. }), "{err:?}");
}

#[test]
fn empty_images_cannot_be_encoded() {
    let empty = DynImage::Gray(Image::<Gray>::new(0, 0));
    assert_eq!(encode_bmp(&empty, RowOrder::BottomUp), Err(EncodeError::EmptyImage { width: 0, height: 0 }));
}
