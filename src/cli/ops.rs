//! The operations a pipeline can chain: parsing them from the command line and applying them.

use std::str::FromStr;

use super::CliError;
use crate::color::flatten;
use crate::dynimage::DynImage;
use crate::error::ImageError;
use crate::filter::{self, BayerMatrix, Palette};
use crate::image::Image;
use crate::map_dyn;
use crate::pixel::{Pixel, Rgb};
use crate::transform::{self, Interpolation};

/// How `dither` spreads the quantization error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Dither {
    FloydSteinberg,
    Ordered(BayerMatrix),
}

/// One parsed pipeline step.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Op {
    Resize(u32, u32, Interpolation),
    Crop { x: u32, y: u32, width: u32, height: u32 },
    FlipHorizontal,
    FlipVertical,
    Rotate(f64),
    Blur(f32),
    BoxBlur(u32),
    Sharpen,
    Median(u32),
    Sobel,
    ToGray,
    ToRgb,
    ToRgba,
    Flatten(Rgb),
    /// `None` picks the level with Otsu's method.
    Threshold(Option<u8>),
    Equalize,
    Brightness(i32),
    Contrast(f32),
    Gamma(f32),
    Invert,
    Dither(Dither, u32),
}

/// A parsed step together with the name it was given under, for error messages.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Step<'a> {
    pub name: &'a str,
    pub op: Op,
}

/// Walks over the arguments of the operations.
struct Cursor<'a> {
    tokens: &'a [String],
    position: usize,
}

impl<'a> Cursor<'a> {
    /// The next argument of operation `op`; `what` names it in the error when it is missing.
    fn next(&mut self, op: &str, what: &str) -> Result<&'a str, CliError> {
        let token = self
            .tokens
            .get(self.position)
            .ok_or_else(|| CliError::usage(format!("{op}: missing {what}")))?;
        self.position += 1;
        Ok(token)
    }

    /// The next argument parsed as a value of type `T`.
    fn value<T: FromStr>(&mut self, op: &str, what: &str) -> Result<T, CliError> {
        let token = self.next(op, what)?;
        token.parse().map_err(|_| invalid(op, what, token))
    }

    /// The next argument parsed as a finite floating point number.
    fn float(&mut self, op: &str, what: &str) -> Result<f64, CliError> {
        let token = self.next(op, what)?;
        match token.parse::<f64>() {
            Ok(v) if v.is_finite() => Ok(v),
            _ => Err(invalid(op, what, token)),
        }
    }

    /// Consumes the next argument only when `parse` accepts it.
    fn optional<T>(&mut self, parse: impl Fn(&str) -> Option<T>) -> Option<T> {
        let parsed = parse(self.tokens.get(self.position)?)?;
        self.position += 1;
        Some(parsed)
    }
}

fn invalid(op: &str, what: &str, token: &str) -> CliError {
    CliError::usage(format!("{op}: invalid {what} '{token}'"))
}

/// Parses every operation up front, so that a typo is reported before any file is touched.
pub(super) fn parse_ops(tokens: &[String]) -> Result<Vec<Step<'_>>, CliError> {
    let mut cursor = Cursor { tokens, position: 0 };
    let mut steps = Vec::new();
    while let Some(name) = tokens.get(cursor.position) {
        cursor.position += 1;
        steps.push(Step { name, op: parse_op(name, &mut cursor)? });
    }
    Ok(steps)
}

fn parse_op(name: &str, args: &mut Cursor) -> Result<Op, CliError> {
    Ok(match name {
        "resize" => {
            let size = args.next(name, "size")?;
            let (width, height) = size
                .split_once('x')
                .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                .ok_or_else(|| invalid(name, "size", size))?;
            let interpolation = args.optional(Interpolation::from_name).unwrap_or(Interpolation::Bilinear);
            Op::Resize(width, height, interpolation)
        }
        "crop" => Op::Crop {
            x: args.value(name, "x")?,
            y: args.value(name, "y")?,
            width: args.value(name, "width")?,
            height: args.value(name, "height")?,
        },
        "flip" => match args.next(name, "direction")? {
            "h" | "horizontal" => Op::FlipHorizontal,
            "v" | "vertical" => Op::FlipVertical,
            other => return Err(invalid(name, "direction", other)),
        },
        "rotate" => Op::Rotate(args.float(name, "angle")?),
        "blur" => Op::Blur(args.float(name, "sigma")? as f32),
        "box" => Op::BoxBlur(args.value(name, "radius")?),
        "sharpen" => Op::Sharpen,
        "median" => Op::Median(args.value(name, "radius")?),
        "sobel" => Op::Sobel,
        "gray" | "grey" => Op::ToGray,
        "rgb" => Op::ToRgb,
        "rgba" => Op::ToRgba,
        "flatten" => {
            let token = args.next(name, "colour")?;
            let color = token
                .parse()
                .map_err(|e| CliError::usage(format!("{name}: invalid colour '{token}': {e}")))?;
            Op::Flatten(color)
        }
        "threshold" => match args.next(name, "level")? {
            "otsu" => Op::Threshold(None),
            token => Op::Threshold(Some(token.parse().map_err(|_| invalid(name, "level", token))?)),
        },
        "equalize" => Op::Equalize,
        "brightness" => Op::Brightness(args.value(name, "amount")?),
        "contrast" => Op::Contrast(args.float(name, "factor")? as f32),
        "gamma" => Op::Gamma(args.float(name, "gamma")? as f32),
        "invert" => Op::Invert,
        "dither" => {
            let method = match args.next(name, "method")? {
                "fs" => Dither::FloydSteinberg,
                "bayer2" => Dither::Ordered(BayerMatrix::B2),
                "bayer4" => Dither::Ordered(BayerMatrix::B4),
                "bayer8" => Dither::Ordered(BayerMatrix::B8),
                other => return Err(invalid(name, "method", other)),
            };
            Op::Dither(method, args.value(name, "levels")?)
        }
        other => return Err(CliError::usage(format!("unknown operation '{other}'"))),
    })
}

/// Applies one step. Most operations keep the pixel type; `gray`, `rgb`, `rgba`, `flatten`,
/// `threshold` and `sobel` may change it.
pub(super) fn apply(image: DynImage, op: &Op) -> Result<DynImage, ImageError> {
    Ok(match op {
        Op::ToGray => image.to_gray().into(),
        Op::ToRgb => image.to_rgb().into(),
        Op::ToRgba => image.to_rgba().into(),
        Op::Flatten(background) => match image {
            DynImage::Rgba(img) => flatten(&img, *background).into(),
            opaque => opaque,
        },
        Op::Threshold(level) => match image {
            DynImage::Gray(img) => threshold(&img, *level),
            DynImage::Rgb(img) => threshold(&img, *level),
            DynImage::Rgba(img) => threshold(&img, *level),
        },
        Op::Sobel => match image {
            DynImage::Gray(img) => filter::sobel(&img).into(),
            DynImage::Rgb(img) => filter::sobel(&img).into(),
            DynImage::Rgba(img) => filter::sobel(&img).into(),
        },
        _ => map_dyn!(image, |img| apply_typed(img, op)?),
    })
}

fn threshold<P: Pixel>(image: &Image<P>, level: Option<u8>) -> DynImage {
    match level {
        Some(level) => filter::threshold(image, level).into(),
        None => filter::otsu_threshold(image).0.into(),
    }
}

/// The operations that return an image of the pixel type they were given.
fn apply_typed<P: Pixel>(image: Image<P>, op: &Op) -> Result<Image<P>, ImageError> {
    Ok(match *op {
        Op::Resize(width, height, interpolation) => transform::resize(&image, width, height, interpolation)?,
        Op::Crop { x, y, width, height } => transform::crop(&image, x, y, width, height)?,
        Op::FlipHorizontal => transform::flip_horizontal(&image),
        Op::FlipVertical => transform::flip_vertical(&image),
        Op::Rotate(degrees) => {
            transform::rotate(&image, degrees, Interpolation::Bilinear, true, P::default())
        }
        Op::Blur(sigma) => filter::gaussian_blur(&image, sigma)?,
        Op::BoxBlur(radius) => filter::box_blur(&image, radius),
        Op::Sharpen => filter::sharpen(&image),
        Op::Median(radius) => filter::median(&image, radius),
        Op::Equalize => filter::equalize(&image),
        Op::Brightness(amount) => filter::brightness(&image, amount),
        Op::Contrast(factor) => filter::contrast(&image, factor)?,
        Op::Gamma(gamma) => filter::gamma(&image, gamma)?,
        Op::Invert => filter::invert(&image),
        Op::Dither(method, levels) => {
            // Grey images get a grey ramp, colour images a levels^3 colour cube.
            let palette =
                if P::CHANNELS == 1 { Palette::grays(levels)? } else { Palette::rgb_cube(levels)? };
            match method {
                Dither::FloydSteinberg => filter::dither_floyd_steinberg(&image, &palette),
                Dither::Ordered(matrix) => {
                    filter::dither_ordered(&image, &palette, matrix, 255.0 / (levels - 1) as f32)
                }
            }
        }
        Op::ToGray | Op::ToRgb | Op::ToRgba | Op::Flatten(_) | Op::Threshold(_) | Op::Sobel => {
            unreachable!("handled by apply")
        }
    })
}
