//! Pixel types: 8-bit grey, RGB and RGBA, and the [`Pixel`] trait the rest of the crate is
//! generic over.

use std::fmt::Debug;

/// A pixel made of one to four 8-bit channels.
///
/// Colour channels come first; when [`Pixel::HAS_ALPHA`] is true the last channel is a straight
/// (non-premultiplied) alpha value.
pub trait Pixel: Copy + PartialEq + Default + Debug + 'static {
    /// Number of channels stored per pixel.
    const CHANNELS: usize;
    /// Whether the last channel is alpha.
    const HAS_ALPHA: bool;
    /// Short lower-case name used in messages, for example `"rgb"`.
    const NAME: &'static str;

    /// Returns the channels in storage order. Entries past [`Pixel::CHANNELS`] are zero.
    fn channels(self) -> [u8; 4];

    /// Builds a pixel from the first [`Pixel::CHANNELS`] entries of `c`.
    fn from_channels(c: [u8; 4]) -> Self;

    /// Widens the pixel to RGBA. Pixels without alpha become fully opaque.
    fn to_rgba(self) -> Rgba;

    /// Narrows an RGBA value to this pixel type. Grey uses [`luma`]; types without alpha drop
    /// the alpha channel without compositing.
    fn from_rgba(c: Rgba) -> Self;

    /// Number of colour (non-alpha) channels.
    fn color_channels() -> usize {
        Self::CHANNELS - usize::from(Self::HAS_ALPHA)
    }

    /// Applies `f` to every colour channel and leaves alpha untouched.
    fn map_color<F: FnMut(u8) -> u8>(self, mut f: F) -> Self {
        let mut c = self.channels();
        for v in c.iter_mut().take(Self::color_channels()) {
            *v = f(*v);
        }
        Self::from_channels(c)
    }

    /// Converts to another pixel type by way of RGBA.
    fn convert<Q: Pixel>(self) -> Q {
        Q::from_rgba(self.to_rgba())
    }

    /// Largest absolute channel difference between two pixels.
    fn max_delta(self, other: Self) -> u8 {
        let (a, b) = (self.channels(), other.channels());
        (0..Self::CHANNELS).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
    }
}

/// Integer BT.601 luma of an RGB triple, rounded to nearest.
pub fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((299 * u32::from(r) + 587 * u32::from(g) + 114 * u32::from(b) + 500) / 1000) as u8
}

/// An 8-bit grey value, 0 is black and 255 is white.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Gray(pub u8);

/// An 8-bit RGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// An 8-bit RGB colour with straight alpha. The default value is transparent black.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    /// Adds an alpha channel.
    pub const fn with_alpha(self, a: u8) -> Rgba {
        Rgba::new(self.r, self.g, self.b, a)
    }
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba::new(0, 0, 0, 0);
    pub const BLACK: Rgba = Rgba::new(0, 0, 0, 255);
    pub const WHITE: Rgba = Rgba::new(255, 255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba { r, g, b, a }
    }

    /// A fully opaque colour.
    pub const fn opaque(r: u8, g: u8, b: u8) -> Self {
        Rgba::new(r, g, b, 255)
    }

    /// Drops the alpha channel without compositing.
    pub const fn rgb(self) -> Rgb {
        Rgb::new(self.r, self.g, self.b)
    }
}

impl Pixel for Gray {
    const CHANNELS: usize = 1;
    const HAS_ALPHA: bool = false;
    const NAME: &'static str = "gray";

    fn channels(self) -> [u8; 4] {
        [self.0, 0, 0, 0]
    }

    fn from_channels(c: [u8; 4]) -> Self {
        Gray(c[0])
    }

    fn to_rgba(self) -> Rgba {
        Rgba::opaque(self.0, self.0, self.0)
    }

    fn from_rgba(c: Rgba) -> Self {
        Gray(luma(c.r, c.g, c.b))
    }
}

impl Pixel for Rgb {
    const CHANNELS: usize = 3;
    const HAS_ALPHA: bool = false;
    const NAME: &'static str = "rgb";

    fn channels(self) -> [u8; 4] {
        [self.r, self.g, self.b, 0]
    }

    fn from_channels(c: [u8; 4]) -> Self {
        Rgb::new(c[0], c[1], c[2])
    }

    fn to_rgba(self) -> Rgba {
        self.with_alpha(255)
    }

    fn from_rgba(c: Rgba) -> Self {
        c.rgb()
    }
}

impl Pixel for Rgba {
    const CHANNELS: usize = 4;
    const HAS_ALPHA: bool = true;
    const NAME: &'static str = "rgba";

    fn channels(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    fn from_channels(c: [u8; 4]) -> Self {
        Rgba::new(c[0], c[1], c[2], c[3])
    }

    fn to_rgba(self) -> Rgba {
        self
    }

    fn from_rgba(c: Rgba) -> Self {
        c
    }
}

impl From<Gray> for Rgb {
    fn from(g: Gray) -> Self {
        Rgb::new(g.0, g.0, g.0)
    }
}

impl From<Rgb> for Rgba {
    fn from(c: Rgb) -> Self {
        c.with_alpha(255)
    }
}

impl From<Gray> for Rgba {
    fn from(g: Gray) -> Self {
        g.to_rgba()
    }
}

impl From<[u8; 3]> for Rgb {
    fn from(c: [u8; 3]) -> Self {
        Rgb::new(c[0], c[1], c[2])
    }
}

impl From<[u8; 4]> for Rgba {
    fn from(c: [u8; 4]) -> Self {
        Rgba::new(c[0], c[1], c[2], c[3])
    }
}
