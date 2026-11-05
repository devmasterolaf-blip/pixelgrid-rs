mod common;

use common::{gray, rgb_pattern, scratch_dir};
use pixelgrid::io::bmp::RowOrder;
use pixelgrid::io::pnm::PnmEncoding;
use pixelgrid::io::{decode, encode, load, save, save_with, EncodeOptions, Format};
use pixelgrid::{DecodeError, DynImage, EncodeError, Error, Image, Rgba};

#[test]
fn format_from_magic() {
    assert_eq!(Format::from_magic(b"P1 "), Some(Format::Pbm));
    assert_eq!(Format::from_magic(b"P4\n"), Some(Format::Pbm));
    assert_eq!(Format::from_magic(b"P2"), Some(Format::Pgm));
    assert_eq!(Format::from_magic(b"P5"), Some(Format::Pgm));
    assert_eq!(Format::from_magic(b"P3"), Some(Format::Ppm));
    assert_eq!(Format::from_magic(b"P6"), Some(Format::Ppm));
    assert_eq!(Format::from_magic(b"BM\x00"), Some(Format::Bmp));
    assert_eq!(Format::from_magic(b"P7"), None);
    assert_eq!(Format::from_magic(b"\x89PNG"), None);
    assert_eq!(Format::from_magic(b""), None);
}

#[test]
fn format_from_path_ignores_case() {
    assert_eq!(Format::from_path("a.pbm"), Some(Format::Pbm));
    assert_eq!(Format::from_path("dir/a.PGM"), Some(Format::Pgm));
    assert_eq!(Format::from_path("a.b.ppm"), Some(Format::Ppm));
    assert_eq!(Format::from_path("a.pnm"), Some(Format::Pnm));
    assert_eq!(Format::from_path("a.Bmp"), Some(Format::Bmp));
    assert_eq!(Format::from_path("a.png"), None);
    assert_eq!(Format::from_path("ppm"), None);
    assert_eq!(Format::Pnm.name(), "PNM");
}

#[test]
fn decode_dispatches_on_the_signature() {
    let img = DynImage::Rgb(rgb_pattern(3, 2));
    let options = EncodeOptions::default();
    for format in [Format::Ppm, Format::Pnm, Format::Bmp] {
        let bytes = encode(&img, format, &options).unwrap();
        assert_eq!(decode(&bytes).unwrap(), img, "{}", format.name());
    }
    assert_eq!(decode(b"GIF89a"), Err(DecodeError::BadMagic { found: vec![b'G', b'I'] }));
    assert_eq!(decode(b""), Err(DecodeError::BadMagic { found: vec![] }));
}

#[test]
fn encode_refuses_to_convert_pixel_types() {
    let options = EncodeOptions::default();
    let grey = DynImage::Gray(gray(&[&[1, 2]]));
    let rgb = DynImage::Rgb(rgb_pattern(2, 1));
    let rgba = DynImage::Rgba(Image::filled(2, 1, Rgba::new(1, 2, 3, 4)));
    let unsupported = |format, pixel| Err(EncodeError::UnsupportedPixel { format, pixel });
    assert_eq!(encode(&rgb, Format::Pgm, &options), unsupported("PGM", "rgb"));
    assert_eq!(encode(&rgb, Format::Pbm, &options), unsupported("PBM", "rgb"));
    assert_eq!(encode(&grey, Format::Ppm, &options), unsupported("PPM", "gray"));
    assert_eq!(encode(&rgba, Format::Ppm, &options), unsupported("PPM", "rgba"));
    assert_eq!(encode(&rgba, Format::Pnm, &options), unsupported("PNM", "rgba"));
    let message = encode(&rgba, Format::Pnm, &options).unwrap_err().to_string();
    assert_eq!(message, "PNM cannot store rgba pixels");
    assert!(encode(&rgba, Format::Bmp, &options).is_ok());
    assert!(encode(&grey, Format::Pnm, &options).unwrap().starts_with(b"P5"));
    assert!(encode(&rgb, Format::Pnm, &options).unwrap().starts_with(b"P6"));
}

#[test]
fn options_select_ascii_and_top_down() {
    let grey = DynImage::Gray(gray(&[&[1, 2]]));
    let options = EncodeOptions { pnm_encoding: PnmEncoding::Ascii, bmp_row_order: RowOrder::TopDown };
    assert_eq!(encode(&grey, Format::Pgm, &options).unwrap(), b"P2\n2 1\n255\n1 2\n");
    let bmp = encode(&grey, Format::Bmp, &options).unwrap();
    assert_eq!(&bmp[22..26], &(-1i32).to_le_bytes());
}

#[test]
fn save_and_load_round_trip_through_files() {
    let dir = scratch_dir("io-round-trip");
    let rgb = DynImage::Rgb(rgb_pattern(4, 3));
    for name in ["a.ppm", "a.pnm", "a.bmp"] {
        save(dir.join(name), &rgb).unwrap();
        assert_eq!(load(dir.join(name)).unwrap(), rgb, "{name}");
    }
    let grey = DynImage::Gray(gray(&[&[0, 255], &[255, 0]]));
    for name in ["g.pgm", "g.pbm", "g.pnm"] {
        save(dir.join(name), &grey).unwrap();
        assert_eq!(load(dir.join(name)).unwrap(), grey, "{name}");
    }
    let options = EncodeOptions { pnm_encoding: PnmEncoding::Ascii, ..EncodeOptions::default() };
    save_with(dir.join("ascii.pgm"), &grey, &options).unwrap();
    assert_eq!(std::fs::read(dir.join("ascii.pgm")).unwrap(), b"P2\n2 2\n255\n0 255\n255 0\n");
}

#[test]
fn load_detects_the_format_from_content_not_from_the_name() {
    let dir = scratch_dir("io-misnamed");
    let rgb = DynImage::Rgb(rgb_pattern(2, 2));
    let bytes = encode(&rgb, Format::Bmp, &EncodeOptions::default()).unwrap();
    std::fs::write(dir.join("really-a-bmp.ppm"), bytes).unwrap();
    assert_eq!(load(dir.join("really-a-bmp.ppm")).unwrap(), rgb);
}

#[test]
fn file_errors_are_typed() {
    let dir = scratch_dir("io-errors");
    let missing = dir.join("missing.ppm");
    match load(&missing).unwrap_err() {
        Error::Io { path, message } => {
            assert_eq!(path, missing.display().to_string());
            assert!(!message.is_empty());
        }
        other => panic!("expected an I/O error, got {other:?}"),
    }
    let grey = DynImage::Gray(gray(&[&[1]]));
    let err = save(dir.join("out.png"), &grey).unwrap_err();
    assert_eq!(err, Error::UnknownFormat { path: dir.join("out.png").display().to_string() });
    assert!(err.to_string().ends_with("out.png: cannot tell the image format from the file name"));
    assert!(matches!(save(dir.join("no-such-dir").join("x.pgm"), &grey), Err(Error::Io { .. })));
    assert!(matches!(save(dir.join("x.ppm"), &grey), Err(Error::Encode(EncodeError::UnsupportedPixel { .. }))));
    assert!(!dir.join("x.ppm").exists(), "nothing is written when encoding fails");
    std::fs::write(dir.join("junk.pgm"), b"P5 2 2 255\n\x00").unwrap();
    assert_eq!(
        load(dir.join("junk.pgm")),
        Err(Error::Decode(DecodeError::TruncatedRaster { expected: 4, actual: 1 }))
    );
}
