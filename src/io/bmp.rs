//! Uncompressed Windows BMP files with 24 or 32 bits per pixel.
//!
//! Both row orders are read and written: the usual bottom-up layout (positive height) and the
//! top-down layout (negative height). Rows of 24-bit files are padded to a multiple of four
//! bytes. In 32-bit files the fourth byte is treated as straight alpha.

use crate::dynimage::DynImage;
use crate::error::{DecodeError, EncodeError};
use crate::image::Image;
use crate::pixel::{Pixel, Rgb, Rgba};

const FILE_HEADER_LEN: usize = 14;
const INFO_HEADER_LEN: usize = 40;
/// 72 dots per inch, in pixels per metre.
const PIXELS_PER_METRE: u32 = 2835;

/// The order in which a BMP file stores its rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RowOrder {
    /// The first stored row is the bottom row of the image (the BMP default).
    #[default]
    BottomUp,
    /// The first stored row is the top row; the header carries a negative height.
    TopDown,
}

/// The fields of a BMP header this crate uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BmpHeader {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u16,
    pub row_order: RowOrder,
    /// Byte offset of the pixel data.
    pub data_offset: usize,
}

impl BmpHeader {
    /// Bytes per stored row, including padding.
    pub fn stride(&self) -> usize {
        row_stride(self.width, self.bits_per_pixel)
    }
}

fn row_stride(width: u32, bits_per_pixel: u16) -> usize {
    (width as usize * usize::from(bits_per_pixel)).div_ceil(32) * 4
}

fn field<const N: usize>(
    data: &[u8],
    offset: usize,
    expected: &'static str,
) -> Result<[u8; N], DecodeError> {
    match data.get(offset..offset + N) {
        Some(bytes) => Ok(bytes.try_into().expect("slice has length N")),
        None => Err(DecodeError::UnexpectedEof { offset: data.len(), expected }),
    }
}

fn u16_at(data: &[u8], offset: usize, expected: &'static str) -> Result<u16, DecodeError> {
    field::<2>(data, offset, expected).map(u16::from_le_bytes)
}

fn u32_at(data: &[u8], offset: usize, expected: &'static str) -> Result<u32, DecodeError> {
    field::<4>(data, offset, expected).map(u32::from_le_bytes)
}

fn i32_at(data: &[u8], offset: usize, expected: &'static str) -> Result<i32, DecodeError> {
    field::<4>(data, offset, expected).map(i32::from_le_bytes)
}

/// Parses and validates the file and info headers of a BMP file.
pub fn read_bmp_header(data: &[u8]) -> Result<BmpHeader, DecodeError> {
    if data.len() < 2 || &data[..2] != b"BM" {
        return Err(DecodeError::BadMagic { found: data.iter().take(2).copied().collect() });
    }
    let data_offset = u32_at(data, 10, "file header")? as usize;
    let header_len = u32_at(data, 14, "info header size")?;
    if header_len < INFO_HEADER_LEN as u32 {
        return Err(DecodeError::Unsupported {
            feature: format!("BMP info header of {header_len} bytes"),
        });
    }
    let width = i32_at(data, 18, "width")?;
    let height = i32_at(data, 22, "height")?;
    let planes = u16_at(data, 26, "planes")?;
    let bits_per_pixel = u16_at(data, 28, "bits per pixel")?;
    let compression = u32_at(data, 30, "compression")?;
    if width <= 0 {
        return Err(DecodeError::InvalidHeader { field: "width", value: width.into() });
    }
    if height == 0 || height == i32::MIN {
        return Err(DecodeError::InvalidHeader { field: "height", value: height.into() });
    }
    if planes != 1 {
        return Err(DecodeError::InvalidHeader { field: "planes", value: planes.into() });
    }
    if bits_per_pixel != 24 && bits_per_pixel != 32 {
        return Err(DecodeError::Unsupported { feature: format!("{bits_per_pixel}-bit BMP") });
    }
    if compression != 0 {
        return Err(DecodeError::Unsupported { feature: format!("BMP compression {compression}") });
    }
    if data_offset < FILE_HEADER_LEN + header_len as usize {
        return Err(DecodeError::InvalidHeader { field: "data offset", value: data_offset as i64 });
    }
    Ok(BmpHeader {
        width: width as u32,
        height: height.unsigned_abs(),
        bits_per_pixel,
        row_order: if height < 0 { RowOrder::TopDown } else { RowOrder::BottomUp },
        data_offset,
    })
}

fn read_rows<P: Pixel>(data: &[u8], header: &BmpHeader, pixel: fn(&[u8]) -> P) -> Image<P> {
    let bytes = usize::from(header.bits_per_pixel / 8);
    let stride = header.stride();
    Image::from_fn(header.width, header.height, |x, y| {
        let row = match header.row_order {
            RowOrder::TopDown => y,
            RowOrder::BottomUp => header.height - 1 - y,
        };
        let start = header.data_offset + row as usize * stride + x as usize * bytes;
        pixel(&data[start..start + bytes])
    })
}

/// Decodes a BMP file. 24-bit files give an RGB image, 32-bit files an RGBA image.
pub fn decode_bmp(data: &[u8]) -> Result<DynImage, DecodeError> {
    let header = read_bmp_header(data)?;
    let stride = header.stride();
    // The padding of the last stored row may be missing.
    let last_row = header.width as usize * usize::from(header.bits_per_pixel / 8);
    let needed = stride
        .checked_mul(header.height as usize - 1)
        .and_then(|n| n.checked_add(last_row))
        .ok_or(DecodeError::InvalidDimensions {
            width: header.width.into(),
            height: header.height.into(),
        })?;
    let available = data.len().saturating_sub(header.data_offset);
    if available < needed {
        return Err(DecodeError::TruncatedRaster { expected: needed, actual: available });
    }
    Ok(if header.bits_per_pixel == 24 {
        DynImage::Rgb(read_rows(data, &header, |b| Rgb::new(b[2], b[1], b[0])))
    } else {
        DynImage::Rgba(read_rows(data, &header, |b| Rgba::new(b[2], b[1], b[0], b[3])))
    })
}

fn write_bmp<P: Pixel>(
    image: &Image<P>,
    bits_per_pixel: u16,
    order: RowOrder,
    put: fn(P, &mut Vec<u8>),
) -> Result<Vec<u8>, EncodeError> {
    let (width, height) = image.dimensions();
    if image.is_empty() {
        return Err(EncodeError::EmptyImage { width, height });
    }
    let stride = row_stride(width, bits_per_pixel);
    let data_len = stride.checked_mul(height as usize);
    let file_len = data_len.and_then(|n| n.checked_add(FILE_HEADER_LEN + INFO_HEADER_LEN));
    let (Some(data_len), Some(file_len)) = (data_len, file_len) else {
        return Err(EncodeError::TooLarge { width, height });
    };
    if width > i32::MAX as u32 || height > i32::MAX as u32 || file_len > u32::MAX as usize {
        return Err(EncodeError::TooLarge { width, height });
    }
    let stored_height = match order {
        RowOrder::BottomUp => height as i32,
        RowOrder::TopDown => -(height as i32),
    };
    let mut out = Vec::with_capacity(file_len);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_len as u32).to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&((FILE_HEADER_LEN + INFO_HEADER_LEN) as u32).to_le_bytes());
    out.extend_from_slice(&(INFO_HEADER_LEN as u32).to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&stored_height.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&bits_per_pixel.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    out.extend_from_slice(&PIXELS_PER_METRE.to_le_bytes());
    out.extend_from_slice(&PIXELS_PER_METRE.to_le_bytes());
    out.extend_from_slice(&[0; 8]);
    for i in 0..height {
        let y = match order {
            RowOrder::TopDown => i,
            RowOrder::BottomUp => height - 1 - i,
        };
        let row_start = out.len();
        for p in image.row(y) {
            put(*p, &mut out);
        }
        out.resize(row_start + stride, 0);
    }
    Ok(out)
}

/// Encodes an image as BMP: 32 bits per pixel for RGBA, 24 bits per pixel for RGB and grey
/// (grey values are written as equal blue, green and red bytes).
pub fn encode_bmp(image: &DynImage, order: RowOrder) -> Result<Vec<u8>, EncodeError> {
    let bgr = |p: Rgb, out: &mut Vec<u8>| out.extend_from_slice(&[p.b, p.g, p.r]);
    match image {
        DynImage::Gray(img) => write_bmp(img, 24, order, |p, out| out.extend_from_slice(&[p.0; 3])),
        DynImage::Rgb(img) => write_bmp(img, 24, order, bgr),
        DynImage::Rgba(img) => {
            write_bmp(img, 32, order, |p, out| out.extend_from_slice(&[p.b, p.g, p.r, p.a]))
        }
    }
}
