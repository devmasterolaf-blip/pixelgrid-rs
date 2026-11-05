//! Error types shared across the crate.

use std::fmt;

/// Errors raised by pixel-buffer operations: bad coordinates, sizes and parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    /// A pixel coordinate lies outside the image.
    OutOfBounds { x: u32, y: u32, width: u32, height: u32 },
    /// A rectangle does not fit inside the image it refers to.
    RegionOutOfBounds { x: u32, y: u32, width: u32, height: u32, image_width: u32, image_height: u32 },
    /// A buffer does not hold exactly `width * height` pixels (or that many times the channels).
    BufferSize { width: u32, height: u32, expected: usize, actual: usize },
    /// Two images that must have the same size do not.
    DimensionMismatch { left: (u32, u32), right: (u32, u32) },
    /// The operation needs at least one pixel.
    EmptyImage { width: u32, height: u32 },
    /// A parameter is outside the range the operation accepts.
    InvalidParameter { name: &'static str, reason: String },
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::OutOfBounds { x, y, width, height } => {
                write!(f, "pixel ({x}, {y}) is outside the {width}x{height} image")
            }
            ImageError::RegionOutOfBounds { x, y, width, height, image_width, image_height } => write!(
                f,
                "region {width}x{height} at ({x}, {y}) does not fit in the \
                 {image_width}x{image_height} image"
            ),
            ImageError::BufferSize { width, height, expected, actual } => write!(
                f,
                "a {width}x{height} image needs {expected} buffer elements, got {actual}"
            ),
            ImageError::DimensionMismatch { left, right } => write!(
                f,
                "image sizes differ: {}x{} and {}x{}",
                left.0, left.1, right.0, right.1
            ),
            ImageError::EmptyImage { width, height } => {
                write!(f, "the operation needs a non-empty image, got {width}x{height}")
            }
            ImageError::InvalidParameter { name, reason } => {
                write!(f, "invalid {name}: {reason}")
            }
        }
    }
}

impl std::error::Error for ImageError {}

/// Why image data could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The data does not start with a signature this crate knows. `found` holds the first bytes
    /// (at most two).
    BadMagic { found: Vec<u8> },
    /// The data ends inside the header, while `expected` was being read.
    UnexpectedEof { offset: usize, expected: &'static str },
    /// A header field or ASCII sample is not an unsigned decimal number.
    InvalidNumber { offset: usize, field: &'static str, token: String },
    /// Width or height is zero, or the image is too large to address.
    InvalidDimensions { width: u64, height: u64 },
    /// A Netpbm maxval outside `1..=65535`.
    InvalidMaxval { value: u64 },
    /// A sample is larger than the maxval declared in the header.
    SampleOutOfRange { offset: usize, value: u32, maxval: u32 },
    /// The pixel data is shorter than the header promises. Counts are bytes for binary data and
    /// samples for ASCII data.
    TruncatedRaster { expected: usize, actual: usize },
    /// A header field holds a value the format does not allow.
    InvalidHeader { field: &'static str, value: i64 },
    /// The file is valid but uses a feature this crate does not implement.
    Unsupported { feature: String },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::BadMagic { found } => write!(f, "unrecognised file signature {found:?}"),
            DecodeError::UnexpectedEof { offset, expected } => {
                write!(f, "unexpected end of data at byte {offset} while reading {expected}")
            }
            DecodeError::InvalidNumber { offset, field, token } => {
                write!(f, "invalid {field} '{token}' at byte {offset}")
            }
            DecodeError::InvalidDimensions { width, height } => {
                write!(f, "invalid image size {width}x{height}")
            }
            DecodeError::InvalidMaxval { value } => {
                write!(f, "maxval {value} is outside 1..=65535")
            }
            DecodeError::SampleOutOfRange { offset, value, maxval } => {
                write!(f, "sample {value} at byte {offset} exceeds the maxval {maxval}")
            }
            DecodeError::TruncatedRaster { expected, actual } => {
                write!(f, "pixel data is truncated: expected {expected}, found {actual}")
            }
            DecodeError::InvalidHeader { field, value } => {
                write!(f, "invalid header field {field}: {value}")
            }
            DecodeError::Unsupported { feature } => write!(f, "unsupported: {feature}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Why an image could not be encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// None of the supported formats can store an image without pixels.
    EmptyImage { width: u32, height: u32 },
    /// The format cannot store this pixel type, for example RGBA in a PPM file.
    UnsupportedPixel { format: &'static str, pixel: &'static str },
    /// The image is larger than the format's header fields can describe.
    TooLarge { width: u32, height: u32 },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::EmptyImage { width, height } => {
                write!(f, "cannot encode an empty {width}x{height} image")
            }
            EncodeError::UnsupportedPixel { format, pixel } => {
                write!(f, "{format} cannot store {pixel} pixels")
            }
            EncodeError::TooLarge { width, height } => {
                write!(f, "a {width}x{height} image is too large for the format")
            }
        }
    }
}

impl std::error::Error for EncodeError {}
