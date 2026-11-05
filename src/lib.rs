//! `pixelgrid` is a small 2D raster image library with no dependencies outside the standard
//! library.

pub mod color;
pub mod dynimage;
pub mod error;
pub mod image;
pub mod io;
pub mod pixel;
pub mod view;

pub use dynimage::DynImage;
pub use error::{DecodeError, EncodeError, Error, ImageError};
pub use image::{Image, Rect};
pub use pixel::{Gray, Pixel, Rgb, Rgba};
pub use view::{SubImage, SubImageMut};
