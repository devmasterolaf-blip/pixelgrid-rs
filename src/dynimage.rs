//! [`DynImage`]: an image whose pixel type is only known at run time, as produced by decoders.

use crate::image::Image;
use crate::pixel::{Gray, Pixel, Rgb, Rgba};

/// An image of one of the three supported pixel types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynImage {
    Gray(Image<Gray>),
    Rgb(Image<Rgb>),
    Rgba(Image<Rgba>),
}

/// Runs the same expression on whichever image a [`DynImage`] holds and wraps the result back
/// into a `DynImage` of the same pixel type.
#[macro_export]
macro_rules! map_dyn {
    ($image:expr, |$img:ident| $body:expr) => {
        match $image {
            $crate::DynImage::Gray($img) => $crate::DynImage::Gray($body),
            $crate::DynImage::Rgb($img) => $crate::DynImage::Rgb($body),
            $crate::DynImage::Rgba($img) => $crate::DynImage::Rgba($body),
        }
    };
}

impl DynImage {
    pub fn width(&self) -> u32 {
        self.dimensions().0
    }

    pub fn height(&self) -> u32 {
        self.dimensions().1
    }

    /// `(width, height)`.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            DynImage::Gray(img) => img.dimensions(),
            DynImage::Rgb(img) => img.dimensions(),
            DynImage::Rgba(img) => img.dimensions(),
        }
    }

    /// The pixel type name: `"gray"`, `"rgb"` or `"rgba"`.
    pub fn pixel_name(&self) -> &'static str {
        match self {
            DynImage::Gray(_) => Gray::NAME,
            DynImage::Rgb(_) => Rgb::NAME,
            DynImage::Rgba(_) => Rgba::NAME,
        }
    }

    /// Converts to grey using BT.601 luma. Alpha is dropped without compositing.
    pub fn to_gray(&self) -> Image<Gray> {
        match self {
            DynImage::Gray(img) => img.clone(),
            DynImage::Rgb(img) => img.convert(),
            DynImage::Rgba(img) => img.convert(),
        }
    }

    /// Converts to RGB. Alpha is dropped without compositing.
    pub fn to_rgb(&self) -> Image<Rgb> {
        match self {
            DynImage::Gray(img) => img.convert(),
            DynImage::Rgb(img) => img.clone(),
            DynImage::Rgba(img) => img.convert(),
        }
    }

    /// Converts to RGBA. Images without alpha become fully opaque.
    pub fn to_rgba(&self) -> Image<Rgba> {
        match self {
            DynImage::Gray(img) => img.convert(),
            DynImage::Rgb(img) => img.convert(),
            DynImage::Rgba(img) => img.clone(),
        }
    }
}

impl From<Image<Gray>> for DynImage {
    fn from(img: Image<Gray>) -> Self {
        DynImage::Gray(img)
    }
}

impl From<Image<Rgb>> for DynImage {
    fn from(img: Image<Rgb>) -> Self {
        DynImage::Rgb(img)
    }
}

impl From<Image<Rgba>> for DynImage {
    fn from(img: Image<Rgba>) -> Self {
        DynImage::Rgba(img)
    }
}
