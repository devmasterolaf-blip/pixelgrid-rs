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
