//! Borrowed rectangular views into an [`Image`].

use crate::error::ImageError;
use crate::image::{Image, Rect};
use crate::pixel::Pixel;

fn check_region<P: Pixel>(image: &Image<P>, rect: Rect) -> Result<(), ImageError> {
    if rect.fits_in(image.width(), image.height()) {
        Ok(())
    } else {
        Err(ImageError::RegionOutOfBounds {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            image_width: image.width(),
            image_height: image.height(),
        })
    }
}

/// A read-only view of a rectangle inside an image. Coordinates are relative to the view.
#[derive(Debug, Clone, Copy)]
pub struct SubImage<'a, P: Pixel> {
    image: &'a Image<P>,
    rect: Rect,
}

/// A mutable view of a rectangle inside an image. Coordinates are relative to the view.
#[derive(Debug)]
pub struct SubImageMut<'a, P: Pixel> {
    image: &'a mut Image<P>,
    rect: Rect,
}

impl<P: Pixel> Image<P> {
    /// Borrows the `width` x `height` rectangle whose top-left corner is `(x, y)`.
    pub fn view(&self, x: u32, y: u32, width: u32, height: u32) -> Result<SubImage<'_, P>, ImageError> {
        let rect = Rect::new(x, y, width, height);
        check_region(self, rect)?;
        Ok(SubImage { image: self, rect })
    }

    /// Mutably borrows the `width` x `height` rectangle whose top-left corner is `(x, y)`.
    pub fn view_mut(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<SubImageMut<'_, P>, ImageError> {
        let rect = Rect::new(x, y, width, height);
        check_region(self, rect)?;
        Ok(SubImageMut { image: self, rect })
    }
}

impl<'a, P: Pixel> SubImage<'a, P> {
    pub fn width(&self) -> u32 {
        self.rect.width
    }

    pub fn height(&self) -> u32 {
        self.rect.height
    }

    /// The viewed rectangle in the coordinates of the underlying image.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Checked read relative to the view's top-left corner.
    pub fn get(&self, x: u32, y: u32) -> Option<P> {
        if x < self.rect.width && y < self.rect.height {
            self.image.get(self.rect.x + x, self.rect.y + y)
        } else {
            None
        }
    }

    /// One row of the view.
    ///
    /// # Panics
    /// Panics when `y >= height`.
    pub fn row(&self, y: u32) -> &'a [P] {
        assert!(y < self.rect.height, "row {y} is outside the view of height {}", self.rect.height);
        let start = self.rect.x as usize;
        &self.image.row(self.rect.y + y)[start..start + self.rect.width as usize]
    }

    /// Iterates over the rows of the view, top to bottom.
    pub fn rows(&self) -> impl Iterator<Item = &'a [P]> + '_ {
        (0..self.rect.height).map(move |y| self.row(y))
    }

    /// Iterates over the pixels of the view in row-major order.
    pub fn pixels(&self) -> impl Iterator<Item = P> + '_ {
        self.rows().flat_map(|row| row.iter().copied())
    }

    /// Iterates over `(x, y, pixel)` with coordinates relative to the view.
    pub fn enumerate_pixels(&self) -> impl Iterator<Item = (u32, u32, P)> + '_ {
        (0..self.rect.height)
            .flat_map(move |y| self.row(y).iter().enumerate().map(move |(x, p)| (x as u32, y, *p)))
    }

    /// Narrows the view. The rectangle is relative to this view and must fit inside it.
    pub fn view(&self, x: u32, y: u32, width: u32, height: u32) -> Result<SubImage<'a, P>, ImageError> {
        if !Rect::new(x, y, width, height).fits_in(self.rect.width, self.rect.height) {
            return Err(ImageError::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                image_width: self.rect.width,
                image_height: self.rect.height,
            });
        }
        Ok(SubImage { image: self.image, rect: Rect::new(self.rect.x + x, self.rect.y + y, width, height) })
    }

    /// Copies the viewed pixels into a new image.
    pub fn to_image(&self) -> Image<P> {
        let mut data = Vec::with_capacity(self.rect.width as usize * self.rect.height as usize);
        for row in self.rows() {
            data.extend_from_slice(row);
        }
        Image::from_vec(self.rect.width, self.rect.height, data).expect("view size matches its pixels")
    }
}

impl<'a, P: Pixel> SubImageMut<'a, P> {
    pub fn width(&self) -> u32 {
        self.rect.width
    }

    pub fn height(&self) -> u32 {
        self.rect.height
    }

    /// The viewed rectangle in the coordinates of the underlying image.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Reborrows as a read-only view.
    pub fn as_view(&self) -> SubImage<'_, P> {
        SubImage { image: self.image, rect: self.rect }
    }

    /// Checked read relative to the view's top-left corner.
    pub fn get(&self, x: u32, y: u32) -> Option<P> {
        self.as_view().get(x, y)
    }

    /// Checked write relative to the view's top-left corner.
    pub fn set(&mut self, x: u32, y: u32, pixel: P) -> Result<(), ImageError> {
        if x < self.rect.width && y < self.rect.height {
            self.image.set(self.rect.x + x, self.rect.y + y, pixel)
        } else {
            Err(ImageError::OutOfBounds { x, y, width: self.rect.width, height: self.rect.height })
        }
    }

    /// Mutable access to one row of the view.
    ///
    /// # Panics
    /// Panics when `y >= height`.
    pub fn row_mut(&mut self, y: u32) -> &mut [P] {
        assert!(y < self.rect.height, "row {y} is outside the view of height {}", self.rect.height);
        let start = self.rect.x as usize;
        &mut self.image.row_mut(self.rect.y + y)[start..start + self.rect.width as usize]
    }

    /// Sets every pixel of the view to `pixel`.
    pub fn fill(&mut self, pixel: P) {
        for y in 0..self.rect.height {
            self.row_mut(y).fill(pixel);
        }
    }

    /// Applies `f` to every pixel of the view in place.
    pub fn apply<F: FnMut(P) -> P>(&mut self, mut f: F) {
        for y in 0..self.rect.height {
            for p in self.row_mut(y) {
                *p = f(*p);
            }
        }
    }

    /// Copies `source` over the view. Both must have the same size.
    pub fn copy_from(&mut self, source: &Image<P>) -> Result<(), ImageError> {
        if source.dimensions() != (self.rect.width, self.rect.height) {
            return Err(ImageError::DimensionMismatch {
                left: (self.rect.width, self.rect.height),
                right: source.dimensions(),
            });
        }
        for y in 0..self.rect.height {
            self.row_mut(y).copy_from_slice(source.row(y));
        }
        Ok(())
    }
}
