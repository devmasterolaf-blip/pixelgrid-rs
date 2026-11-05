//! Colour handling: colour-space conversion, sRGB gamma, parsing and compositing.

pub mod space;

pub use space::{hsl_to_rgb, hsv_to_rgb, rgb_to_hsl, rgb_to_hsv, Hsl, Hsv};
