//! Helpers shared by the integration tests.
#![allow(dead_code)]

use pixelgrid::{Gray, Image, Rgb};

/// Builds a grey image from rows of values.
pub fn gray(rows: &[&[u8]]) -> Image<Gray> {
    let height = rows.len() as u32;
    let width = rows.first().map_or(0, |r| r.len()) as u32;
    let data = rows.iter().flat_map(|r| r.iter().map(|v| Gray(*v))).collect();
    Image::from_vec(width, height, data).expect("rows must have equal length")
}

/// Returns the grey values of an image as rows.
pub fn gray_rows(img: &Image<Gray>) -> Vec<Vec<u8>> {
    img.rows().map(|r| r.iter().map(|p| p.0).collect()).collect()
}

/// Builds a grey image from an ASCII picture: `#` is 255, anything else is 0.
pub fn mask(rows: &[&str]) -> Image<Gray> {
    let height = rows.len() as u32;
    let width = rows.first().map_or(0, |r| r.len()) as u32;
    let data = rows
        .iter()
        .flat_map(|r| r.bytes().map(|c| Gray(if c == b'#' { 255 } else { 0 })))
        .collect();
    Image::from_vec(width, height, data).expect("rows must have equal length")
}

/// Renders a grey image as an ASCII picture: non-zero pixels are `#`, zero pixels are `.`.
pub fn picture(img: &Image<Gray>) -> Vec<String> {
    img.rows().map(|r| r.iter().map(|p| if p.0 != 0 { '#' } else { '.' }).collect()).collect()
}

/// A small RGB test pattern where every pixel is distinct: `(10 * x, 10 * y, x + y)`.
pub fn rgb_pattern(width: u32, height: u32) -> Image<Rgb> {
    Image::from_fn(width, height, |x, y| Rgb::new((10 * x) as u8, (10 * y) as u8, (x + y) as u8))
}
