//! `pixelgrid` is a small 2D raster image library with no dependencies outside the standard
//! library.

pub mod error;
pub mod image;
pub mod pixel;
pub mod view;

pub use error::ImageError;
pub use image::{Image, Rect};
pub use pixel::{Gray, Pixel, Rgb, Rgba};
pub use view::{SubImage, SubImageMut};
