//! Readers and writers for image file formats, and helpers that pick the format for you.

pub mod bmp;
pub mod pnm;

use std::path::Path;

use crate::dynimage::DynImage;
use crate::error::{DecodeError, EncodeError, Error};

use bmp::RowOrder;
use pnm::PnmEncoding;

/// A file format this crate can write. All but [`Format::Pnm`] are also detected on reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Netpbm bitmap, one bit per pixel.
    Pbm,
    /// Netpbm greymap.
    Pgm,
    /// Netpbm pixmap (RGB).
    Ppm,
    /// "Any Netpbm": written as PGM for grey images and as PPM for RGB images.
    Pnm,
    /// Windows bitmap.
    Bmp,
}

impl Format {
    /// Recognises a format from the first bytes of a file.
    pub fn from_magic(data: &[u8]) -> Option<Format> {
        match data {
            [b'P', b'1' | b'4', ..] => Some(Format::Pbm),
            [b'P', b'2' | b'5', ..] => Some(Format::Pgm),
            [b'P', b'3' | b'6', ..] => Some(Format::Ppm),
            [b'B', b'M', ..] => Some(Format::Bmp),
            _ => None,
        }
    }

    /// Picks a format from a file extension, ignoring case.
    pub fn from_path(path: impl AsRef<Path>) -> Option<Format> {
        let ext = path.as_ref().extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "pbm" => Some(Format::Pbm),
            "pgm" => Some(Format::Pgm),
            "ppm" => Some(Format::Ppm),
            "pnm" => Some(Format::Pnm),
            "bmp" => Some(Format::Bmp),
            _ => None,
        }
    }

    /// The upper-case name of the format.
    pub fn name(self) -> &'static str {
        match self {
            Format::Pbm => "PBM",
            Format::Pgm => "PGM",
            Format::Ppm => "PPM",
            Format::Pnm => "PNM",
            Format::Bmp => "BMP",
        }
    }
}

/// Knobs for [`encode`]. The default writes binary Netpbm and bottom-up BMP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EncodeOptions {
    pub pnm_encoding: PnmEncoding,
    pub bmp_row_order: RowOrder,
}

/// Decodes an image, detecting the format from its signature.
pub fn decode(data: &[u8]) -> Result<DynImage, DecodeError> {
    match Format::from_magic(data) {
        Some(Format::Bmp) => bmp::decode_bmp(data),
        Some(_) => pnm::decode_pnm(data),
        None => Err(DecodeError::BadMagic { found: data.iter().take(2).copied().collect() }),
    }
}

/// Encodes an image in the given format.
///
/// The pixel type must match what the format stores: grey for PBM and PGM, RGB for PPM, grey or
/// RGB for PNM. BMP takes all three. Nothing is converted silently; a mismatch is an
/// [`EncodeError::UnsupportedPixel`].
pub fn encode(image: &DynImage, format: Format, options: &EncodeOptions) -> Result<Vec<u8>, EncodeError> {
    let encoding = options.pnm_encoding;
    match (format, image) {
        (Format::Bmp, _) => bmp::encode_bmp(image, options.bmp_row_order),
        (Format::Pbm, DynImage::Gray(img)) => pnm::encode_pbm(img, encoding),
        (Format::Pgm | Format::Pnm, DynImage::Gray(img)) => pnm::encode_pgm(img, encoding),
        (Format::Ppm | Format::Pnm, DynImage::Rgb(img)) => pnm::encode_ppm(img, encoding),
        _ => Err(EncodeError::UnsupportedPixel { format: format.name(), pixel: image.pixel_name() }),
    }
}

fn io_error(path: &Path, e: std::io::Error) -> Error {
    Error::Io { path: path.display().to_string(), message: e.to_string() }
}

/// Reads and decodes an image file. The format is detected from the content, not the name.
pub fn load(path: impl AsRef<Path>) -> Result<DynImage, Error> {
    let path = path.as_ref();
    let data = std::fs::read(path).map_err(|e| io_error(path, e))?;
    Ok(decode(&data)?)
}

/// Encodes and writes an image file with default options. The format follows the extension.
pub fn save(path: impl AsRef<Path>, image: &DynImage) -> Result<(), Error> {
    save_with(path, image, &EncodeOptions::default())
}

/// Encodes and writes an image file. The format follows the extension.
pub fn save_with(path: impl AsRef<Path>, image: &DynImage, options: &EncodeOptions) -> Result<(), Error> {
    let path = path.as_ref();
    let format = Format::from_path(path)
        .ok_or_else(|| Error::UnknownFormat { path: path.display().to_string() })?;
    let bytes = encode(image, format, options)?;
    std::fs::write(path, bytes).map_err(|e| io_error(path, e))
}
