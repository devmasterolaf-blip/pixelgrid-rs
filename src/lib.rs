// The README is the crate documentation, which also makes its examples run as doc tests.
#![doc = include_str!("../README.md")]

pub mod cli;
pub mod color;
pub mod compare;
pub mod draw;
pub mod dynimage;
pub mod error;
pub mod filter;
pub mod image;
pub mod io;
pub mod label;
pub mod pixel;
pub mod transform;
pub mod view;

pub use dynimage::DynImage;
pub use error::{DecodeError, EncodeError, Error, ImageError};
pub use image::{Connectivity, Image, Rect};
pub use pixel::{Gray, Pixel, Rgb, Rgba};
pub use view::{SubImage, SubImageMut};
