//! The Netpbm family: PBM, PGM and PPM, each in its ASCII ("plain") and binary ("raw") form.
//!
//! Samples are scaled to 8 bits on reading, so files with a maxval other than 255 (including
//! 16-bit files) lose their extra precision.

use crate::dynimage::DynImage;
use crate::error::DecodeError;
use crate::image::Image;
use crate::pixel::{Gray, Rgb};

/// The parsed header of a Netpbm file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PnmHeader {
    /// The digit of the magic number: 1 to 6.
    pub magic: u8,
    pub width: u32,
    pub height: u32,
    /// Largest sample value. Always 1 for PBM.
    pub maxval: u16,
    /// Byte offset of the first raster byte.
    pub raster_offset: usize,
}

impl PnmHeader {
    /// True for the ASCII variants P1, P2 and P3.
    pub fn is_ascii(&self) -> bool {
        self.magic <= 3
    }

    /// True for the bitmap variants P1 and P4.
    pub fn is_bitmap(&self) -> bool {
        self.magic == 1 || self.magic == 4
    }

    /// Samples per pixel: 3 for PPM, otherwise 1.
    pub fn channels(&self) -> usize {
        if self.magic == 3 || self.magic == 6 {
            3
        } else {
            1
        }
    }

    /// The conventional name of the format, for example `"PGM"`.
    pub fn format_name(&self) -> &'static str {
        match self.magic {
            1 | 4 => "PBM",
            2 | 5 => "PGM",
            _ => "PPM",
        }
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    /// Skips whitespace and `#` comments, which run to the end of the line.
    fn skip_blank(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'#' {
                while !matches!(self.peek(), None | Some(b'\n') | Some(b'\r')) {
                    self.pos += 1;
                }
            } else if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Reads the next whitespace-delimited unsigned decimal number.
    fn number(&mut self, field: &'static str) -> Result<Option<(usize, u64)>, DecodeError> {
        self.skip_blank();
        let start = self.pos;
        while matches!(self.peek(), Some(b) if !b.is_ascii_whitespace() && b != b'#') {
            self.pos += 1;
        }
        if start == self.pos {
            return Ok(None);
        }
        let token = &self.data[start..self.pos];
        let text = std::str::from_utf8(token).ok().filter(|t| t.bytes().all(|b| b.is_ascii_digit()));
        match text.and_then(|t| t.parse::<u64>().ok()) {
            Some(value) => Ok(Some((start, value))),
            None => Err(DecodeError::InvalidNumber {
                offset: start,
                field,
                token: String::from_utf8_lossy(token).into_owned(),
            }),
        }
    }

    fn header_number(&mut self, field: &'static str) -> Result<u64, DecodeError> {
        match self.number(field)? {
            Some((_, value)) => Ok(value),
            None => Err(DecodeError::UnexpectedEof { offset: self.pos, expected: field }),
        }
    }
}

/// Parses the header of a Netpbm file without touching the raster.
pub fn read_pnm_header(data: &[u8]) -> Result<PnmHeader, DecodeError> {
    let magic = match data {
        [b'P', d @ b'1'..=b'6', ..] => d - b'0',
        _ => return Err(DecodeError::BadMagic { found: data.iter().take(2).copied().collect() }),
    };
    let mut cur = Cursor { data, pos: 2 };
    let width = cur.header_number("width")?;
    let height = cur.header_number("height")?;
    let maxval = if magic == 1 || magic == 4 { 1 } else { cur.header_number("maxval")? };
    if width == 0 || height == 0 || width > u64::from(u32::MAX) || height > u64::from(u32::MAX) {
        return Err(DecodeError::InvalidDimensions { width, height });
    }
    if maxval == 0 || maxval > 65535 {
        return Err(DecodeError::InvalidMaxval { value: maxval });
    }
    if magic >= 4 {
        // Exactly one whitespace byte separates the header from binary data; a comment that
        // directly follows the last number ends with that byte.
        if cur.peek() == Some(b'#') {
            while !matches!(cur.peek(), None | Some(b'\n')) {
                cur.pos += 1;
            }
        }
        if cur.peek().is_none() {
            return Err(DecodeError::UnexpectedEof { offset: cur.pos, expected: "raster" });
        }
        cur.pos += 1;
    }
    Ok(PnmHeader {
        magic,
        width: width as u32,
        height: height as u32,
        maxval: maxval as u16,
        raster_offset: cur.pos,
    })
}

/// Scales a sample in `0..=maxval` to `0..=255`, rounding to nearest.
fn scale(value: u32, maxval: u32) -> u8 {
    ((value * 255 + maxval / 2) / maxval) as u8
}

/// Reads all samples of a PGM or PPM raster, scaled to 8 bits.
fn read_samples(data: &[u8], header: &PnmHeader, count: usize) -> Result<Vec<u8>, DecodeError> {
    let maxval = u32::from(header.maxval);
    let check = |offset: usize, value: u64| {
        if value > u64::from(maxval) {
            let value = value.min(u64::from(u32::MAX)) as u32;
            return Err(DecodeError::SampleOutOfRange { offset, value, maxval });
        }
        Ok(scale(value as u32, maxval))
    };
    let raster = &data[header.raster_offset..];
    if header.is_ascii() {
        let mut cur = Cursor { data, pos: header.raster_offset };
        let mut samples = Vec::with_capacity(count.min(raster.len()));
        while samples.len() < count {
            match cur.number("sample")? {
                Some((offset, value)) => samples.push(check(offset, value)?),
                None => {
                    return Err(DecodeError::TruncatedRaster { expected: count, actual: samples.len() })
                }
            }
        }
        return Ok(samples);
    }
    let width = if maxval > 255 { 2 } else { 1 };
    let needed = count.checked_mul(width).filter(|n| *n <= raster.len());
    let Some(needed) = needed else {
        let expected = count.saturating_mul(width);
        return Err(DecodeError::TruncatedRaster { expected, actual: raster.len() });
    };
    raster[..needed]
        .chunks_exact(width)
        .enumerate()
        .map(|(i, bytes)| {
            let value = bytes.iter().fold(0u64, |acc, b| acc << 8 | u64::from(*b));
            check(header.raster_offset + i * width, value)
        })
        .collect()
}

/// Reads a PBM raster. A set bit is black, so it becomes grey 0; a clear bit becomes 255.
fn read_bits(data: &[u8], header: &PnmHeader, count: usize) -> Result<Vec<u8>, DecodeError> {
    let raster = &data[header.raster_offset..];
    let width = header.width as usize;
    if header.is_ascii() {
        // Plain PBM allows the digits to run together without whitespace.
        let mut cur = Cursor { data, pos: header.raster_offset };
        let mut bits = Vec::with_capacity(count.min(raster.len()));
        while bits.len() < count {
            cur.skip_blank();
            match cur.peek() {
                Some(b'0') => bits.push(255),
                Some(b'1') => bits.push(0),
                Some(other) => {
                    return Err(DecodeError::InvalidNumber {
                        offset: cur.pos,
                        field: "bit",
                        token: String::from_utf8_lossy(&[other]).into_owned(),
                    })
                }
                None => return Err(DecodeError::TruncatedRaster { expected: count, actual: bits.len() }),
            }
            cur.pos += 1;
        }
        return Ok(bits);
    }
    let stride = width.div_ceil(8);
    let needed = stride.checked_mul(header.height as usize).filter(|n| *n <= raster.len());
    let Some(needed) = needed else {
        let expected = stride.saturating_mul(header.height as usize);
        return Err(DecodeError::TruncatedRaster { expected, actual: raster.len() });
    };
    let mut bits = Vec::with_capacity(count);
    for row in raster[..needed].chunks_exact(stride) {
        for x in 0..width {
            let set = row[x / 8] & (0x80 >> (x % 8)) != 0;
            bits.push(if set { 0 } else { 255 });
        }
    }
    Ok(bits)
}

/// Decodes any of the six Netpbm variants. PBM and PGM give a grey image, PPM an RGB image.
///
/// Data after the first image is ignored.
pub fn decode_pnm(data: &[u8]) -> Result<DynImage, DecodeError> {
    let header = read_pnm_header(data)?;
    let (width, height) = (header.width, header.height);
    let too_large = DecodeError::InvalidDimensions { width: width.into(), height: height.into() };
    let pixels = (width as usize).checked_mul(height as usize).ok_or(too_large.clone())?;
    let count = pixels.checked_mul(header.channels()).ok_or(too_large)?;
    let samples = if header.is_bitmap() {
        read_bits(data, &header, count)?
    } else {
        read_samples(data, &header, count)?
    };
    let image = if header.channels() == 3 {
        DynImage::Rgb(Image::<Rgb>::from_raw(width, height, &samples).expect("sample count was checked"))
    } else {
        DynImage::Gray(Image::<Gray>::from_raw(width, height, &samples).expect("sample count was checked"))
    };
    Ok(image)
}
