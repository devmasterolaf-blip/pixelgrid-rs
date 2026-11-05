//! Geometric transforms. Every function returns a new image and leaves its input untouched.

pub mod geometry;

pub use geometry::{crop, flip_horizontal, flip_vertical, rotate180, rotate270, rotate90};
