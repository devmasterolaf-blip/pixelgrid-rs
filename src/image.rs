//! The generic [`Image`] pixel buffer.

use std::ops::{Index, IndexMut};

use crate::error::ImageError;
use crate::pixel::Pixel;

/// An axis-aligned rectangle in pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect { x, y, width, height }
    }

    /// Whether the point lies inside the rectangle.
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && y >= self.y && x - self.x < self.width && y - self.y < self.height
    }

    /// Whether the rectangle lies completely inside a `width` x `height` image.
    pub fn fits_in(&self, width: u32, height: u32) -> bool {
        u64::from(self.x) + u64::from(self.width) <= u64::from(width)
            && u64::from(self.y) + u64::from(self.height) <= u64::from(height)
    }
}

/// Which neighbours of a pixel count as adjacent to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Connectivity {
    /// The pixels above, below, left and right.
    Four,
    /// The four edge neighbours plus the four diagonal ones.
    Eight,
}

impl Connectivity {
    /// The `(dx, dy)` offsets of the neighbours.
    pub fn offsets(self) -> &'static [(i32, i32)] {
        const ALL: [(i32, i32); 8] =
            [(0, -1), (-1, 0), (1, 0), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)];
        match self {
            Connectivity::Four => &ALL[..4],
            Connectivity::Eight => &ALL,
        }
    }

    /// Parses `"4"` or `"8"`.
    pub fn from_name(name: &str) -> Option<Connectivity> {
        match name {
            "4" => Some(Connectivity::Four),
            "8" => Some(Connectivity::Eight),
            _ => None,
        }
    }
}

/// A `width` x `height` grid of pixels stored row by row, top row first.
///
/// Either dimension may be zero, which gives an empty image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image<P: Pixel> {
    width: u32,
    height: u32,
    data: Vec<P>,
}

impl<P: Pixel> Image<P> {
    /// Creates an image filled with the default pixel (black, or transparent black for RGBA).
    pub fn new(width: u32, height: u32) -> Self {
        Self::filled(width, height, P::default())
    }

    /// Creates an image with every pixel set to `pixel`.
    pub fn filled(width: u32, height: u32, pixel: P) -> Self {
        Image { width, height, data: vec![pixel; width as usize * height as usize] }
    }

    /// Wraps an existing row-major pixel vector.
    pub fn from_vec(width: u32, height: u32, data: Vec<P>) -> Result<Self, ImageError> {
        let expected = width as usize * height as usize;
        if data.len() != expected {
            return Err(ImageError::BufferSize { width, height, expected, actual: data.len() });
        }
        Ok(Image { width, height, data })
    }

    /// Builds an image by calling `f(x, y)` for every pixel in row-major order.
    pub fn from_fn<F: FnMut(u32, u32) -> P>(width: u32, height: u32, mut f: F) -> Self {
        let mut data = Vec::with_capacity(width as usize * height as usize);
        for y in 0..height {
            for x in 0..width {
                data.push(f(x, y));
            }
        }
        Image { width, height, data }
    }

    /// Builds an image from interleaved channel bytes (`P::CHANNELS` bytes per pixel).
    pub fn from_raw(width: u32, height: u32, bytes: &[u8]) -> Result<Self, ImageError> {
        let expected = width as usize * height as usize * P::CHANNELS;
        if bytes.len() != expected {
            return Err(ImageError::BufferSize { width, height, expected, actual: bytes.len() });
        }
        let data = bytes
            .chunks_exact(P::CHANNELS)
            .map(|chunk| {
                let mut c = [0u8; 4];
                c[..P::CHANNELS].copy_from_slice(chunk);
                P::from_channels(c)
            })
            .collect();
        Ok(Image { width, height, data })
    }

    /// Returns the pixels as interleaved channel bytes.
    pub fn to_raw(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len() * P::CHANNELS);
        for p in &self.data {
            out.extend_from_slice(&p.channels()[..P::CHANNELS]);
        }
        out
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// `(width, height)`.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The rectangle covering the whole image.
    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// True when the image holds no pixels.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn offset(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y as usize * self.width as usize + x as usize)
        } else {
            None
        }
    }

    /// Checked read: `None` when `(x, y)` is outside the image.
    pub fn get(&self, x: u32, y: u32) -> Option<P> {
        self.offset(x, y).map(|i| self.data[i])
    }

    /// Checked mutable access.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut P> {
        self.offset(x, y).map(move |i| &mut self.data[i])
    }

    /// Checked write.
    pub fn set(&mut self, x: u32, y: u32, pixel: P) -> Result<(), ImageError> {
        match self.offset(x, y) {
            Some(i) => {
                self.data[i] = pixel;
                Ok(())
            }
            None => Err(ImageError::OutOfBounds { x, y, width: self.width, height: self.height }),
        }
    }

    /// Unchecked read.
    ///
    /// # Safety
    /// `x < width` and `y < height` must hold.
    pub unsafe fn get_unchecked(&self, x: u32, y: u32) -> P {
        *self.data.get_unchecked(y as usize * self.width as usize + x as usize)
    }

    /// Unchecked write.
    ///
    /// # Safety
    /// `x < width` and `y < height` must hold.
    pub unsafe fn set_unchecked(&mut self, x: u32, y: u32, pixel: P) {
        *self.data.get_unchecked_mut(y as usize * self.width as usize + x as usize) = pixel;
    }

    /// Writes a pixel given signed coordinates and silently skips anything outside the image.
    /// Returns whether a pixel was written. This is what the drawing code clips with.
    pub fn put_clipped(&mut self, x: i32, y: i32, pixel: P) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        self.set(x as u32, y as u32, pixel).is_ok()
    }

    /// Reads the pixel nearest to `(x, y)` inside the image (clamp-to-edge addressing).
    ///
    /// # Panics
    /// Panics when the image is empty.
    pub fn get_clamped(&self, x: i64, y: i64) -> P {
        assert!(!self.is_empty(), "get_clamped on an empty image");
        let cx = x.clamp(0, i64::from(self.width) - 1) as usize;
        let cy = y.clamp(0, i64::from(self.height) - 1) as usize;
        self.data[cy * self.width as usize + cx]
    }

    /// One row of pixels.
    ///
    /// # Panics
    /// Panics when `y >= height`.
    pub fn row(&self, y: u32) -> &[P] {
        assert!(y < self.height, "row {y} is outside the image of height {}", self.height);
        let w = self.width as usize;
        &self.data[y as usize * w..(y as usize + 1) * w]
    }

    /// Mutable access to one row.
    ///
    /// # Panics
    /// Panics when `y >= height`.
    pub fn row_mut(&mut self, y: u32) -> &mut [P] {
        assert!(y < self.height, "row {y} is outside the image of height {}", self.height);
        let w = self.width as usize;
        &mut self.data[y as usize * w..(y as usize + 1) * w]
    }

    /// Iterates over the rows, top to bottom. An empty image yields no rows.
    pub fn rows(&self) -> std::slice::ChunksExact<'_, P> {
        self.data.chunks_exact((self.width as usize).max(1))
    }

    /// Iterates over the pixels in row-major order.
    pub fn pixels(&self) -> std::slice::Iter<'_, P> {
        self.data.iter()
    }

    /// Iterates mutably over the pixels in row-major order.
    pub fn pixels_mut(&mut self) -> std::slice::IterMut<'_, P> {
        self.data.iter_mut()
    }

    /// Iterates over `(x, y, pixel)` in row-major order.
    pub fn enumerate_pixels(&self) -> impl Iterator<Item = (u32, u32, P)> + '_ {
        let w = self.width as usize;
        self.data.iter().enumerate().map(move |(i, p)| ((i % w) as u32, (i / w) as u32, *p))
    }

    pub fn as_slice(&self) -> &[P] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [P] {
        &mut self.data
    }

    pub fn into_vec(self) -> Vec<P> {
        self.data
    }

    /// Sets every pixel to `pixel`.
    pub fn fill(&mut self, pixel: P) {
        self.data.fill(pixel);
    }

    /// Applies `f` to every pixel, producing an image of the same size.
    pub fn map<Q: Pixel, F: FnMut(P) -> Q>(&self, mut f: F) -> Image<Q> {
        Image { width: self.width, height: self.height, data: self.data.iter().map(|p| f(*p)).collect() }
    }

    /// Converts every pixel to another pixel type (see [`Pixel::convert`]).
    pub fn convert<Q: Pixel>(&self) -> Image<Q> {
        self.map(|p| p.convert())
    }
}

impl<P: Pixel> Index<(u32, u32)> for Image<P> {
    type Output = P;

    /// # Panics
    /// Panics when `(x, y)` is outside the image.
    fn index(&self, (x, y): (u32, u32)) -> &P {
        match self.offset(x, y) {
            Some(i) => &self.data[i],
            None => panic!("pixel ({x}, {y}) is outside the {}x{} image", self.width, self.height),
        }
    }
}

impl<P: Pixel> IndexMut<(u32, u32)> for Image<P> {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut P {
        match self.offset(x, y) {
            Some(i) => &mut self.data[i],
            None => panic!("pixel ({x}, {y}) is outside the {}x{} image", self.width, self.height),
        }
    }
}
