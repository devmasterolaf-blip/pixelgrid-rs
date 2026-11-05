//! Colour handling: colour-space conversion, sRGB gamma, parsing and compositing.

pub mod space;

pub use space::{hsl_to_rgb, hsv_to_rgb, rgb_to_hsl, rgb_to_hsv, Hsl, Hsv};

pub mod gamma;

pub use gamma::{linear_to_srgb, mix_linear, relative_luminance, srgb_to_linear};

pub mod parse;

pub use parse::{named_color, parse_color, ColorParseError};
