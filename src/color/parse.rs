//! Parsing colours from text: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` and a few names.

use std::fmt;
use std::str::FromStr;

use crate::pixel::{Rgb, Rgba};

/// Why a colour string could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    /// The string is empty or only whitespace.
    Empty,
    /// The string is not a hex colour and not a known colour name.
    UnknownName(String),
    /// A hex colour must have 3, 4, 6 or 8 digits after the `#`.
    InvalidHexLength { len: usize },
    /// A character after the `#` is not a hexadecimal digit. `index` counts from the first digit.
    InvalidHexDigit { index: usize, found: char },
    /// An opaque colour was expected but the text carries a non-opaque alpha value.
    UnexpectedAlpha { alpha: u8 },
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorParseError::Empty => write!(f, "empty colour"),
            ColorParseError::UnknownName(name) => write!(f, "unknown colour name '{name}'"),
            ColorParseError::InvalidHexLength { len } => {
                write!(f, "a hex colour needs 3, 4, 6 or 8 digits, got {len}")
            }
            ColorParseError::InvalidHexDigit { index, found } => {
                write!(f, "invalid hex digit '{found}' at position {index}")
            }
            ColorParseError::UnexpectedAlpha { alpha } => {
                write!(f, "expected an opaque colour, got alpha {alpha}")
            }
        }
    }
}

impl std::error::Error for ColorParseError {}

const NAMED: &[(&str, Rgb)] = &[
    ("black", Rgb::new(0, 0, 0)),
    ("silver", Rgb::new(192, 192, 192)),
    ("gray", Rgb::new(128, 128, 128)),
    ("grey", Rgb::new(128, 128, 128)),
    ("white", Rgb::new(255, 255, 255)),
    ("maroon", Rgb::new(128, 0, 0)),
    ("red", Rgb::new(255, 0, 0)),
    ("purple", Rgb::new(128, 0, 128)),
    ("fuchsia", Rgb::new(255, 0, 255)),
    ("magenta", Rgb::new(255, 0, 255)),
    ("green", Rgb::new(0, 128, 0)),
    ("lime", Rgb::new(0, 255, 0)),
    ("olive", Rgb::new(128, 128, 0)),
    ("yellow", Rgb::new(255, 255, 0)),
    ("navy", Rgb::new(0, 0, 128)),
    ("blue", Rgb::new(0, 0, 255)),
    ("teal", Rgb::new(0, 128, 128)),
    ("aqua", Rgb::new(0, 255, 255)),
    ("cyan", Rgb::new(0, 255, 255)),
    ("orange", Rgb::new(255, 165, 0)),
];

/// Looks up a colour by its lower-case name.
pub fn named_color(name: &str) -> Option<Rgb> {
    NAMED.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

fn parse_hex(digits: &str) -> Result<Rgba, ColorParseError> {
    let mut values = Vec::with_capacity(8);
    for (index, found) in digits.chars().enumerate() {
        match found.to_digit(16) {
            Some(d) => values.push(d as u8),
            None => return Err(ColorParseError::InvalidHexDigit { index, found }),
        }
    }
    let channels: Vec<u8> = match values.len() {
        3 | 4 => values.iter().map(|d| d * 17).collect(),
        6 | 8 => values.chunks_exact(2).map(|p| p[0] * 16 + p[1]).collect(),
        len => return Err(ColorParseError::InvalidHexLength { len }),
    };
    let alpha = channels.get(3).copied().unwrap_or(255);
    Ok(Rgba::new(channels[0], channels[1], channels[2], alpha))
}

/// Parses a colour. Leading and trailing whitespace is ignored and names are case-insensitive.
/// `transparent` is transparent black.
pub fn parse_color(text: &str) -> Result<Rgba, ColorParseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ColorParseError::Empty);
    }
    if let Some(digits) = text.strip_prefix('#') {
        return parse_hex(digits);
    }
    let lower = text.to_ascii_lowercase();
    if lower == "transparent" {
        return Ok(Rgba::TRANSPARENT);
    }
    named_color(&lower).map(Rgba::from).ok_or_else(|| ColorParseError::UnknownName(text.to_string()))
}

impl FromStr for Rgba {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_color(s)
    }
}

impl FromStr for Rgb {
    type Err = ColorParseError;

    /// Like [`parse_color`], but rejects colours that are not fully opaque.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = parse_color(s)?;
        if c.a != 255 {
            return Err(ColorParseError::UnexpectedAlpha { alpha: c.a });
        }
        Ok(c.rgb())
    }
}

impl Rgba {
    /// Formats the colour as `#rrggbb`, or `#rrggbbaa` when it is not fully opaque.
    pub fn to_hex(self) -> String {
        if self.a == 255 {
            self.rgb().to_hex()
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl Rgb {
    /// Formats the colour as `#rrggbb`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
