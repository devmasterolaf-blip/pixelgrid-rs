//! The command-line front end, kept in the library so that it can be tested without spawning a
//! process: [`run`] takes the arguments and two writers and returns the exit code.

use std::io::Write;

use crate::compare::compare_dyn;
use crate::dynimage::DynImage;
use crate::error::{Error, ImageError};
use crate::io::{self, Format};

/// Everything went well; for `diff`, the images match.
pub const EXIT_OK: i32 = 0;
/// `diff` found differences.
pub const EXIT_DIFFERENT: i32 = 1;
/// The command line could not be understood.
pub const EXIT_USAGE: i32 = 2;
/// A file could not be read or written.
pub const EXIT_IO: i32 = 3;
/// An input file is not a valid image.
pub const EXIT_DECODE: i32 = 4;
/// An operation or the encoder rejected the image it was given.
pub const EXIT_FAILED: i32 = 5;

const USAGE: &str = "\
pixelgrid - a small raster image tool

Usage:
  pixelgrid info <file>
  pixelgrid diff <a> <b> [--tolerance N]
  pixelgrid help | --help | --version

Exit codes:
  0  success (diff: the images match)
  1  diff found differences
  2  usage error
  3  a file could not be read or written
  4  an input file is not a valid image
  5  an operation or the encoder rejected the image
";

/// A failure with the exit code it maps to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CliError {
    code: i32,
    message: String,
}

impl CliError {
    fn usage(message: impl Into<String>) -> CliError {
        CliError { code: EXIT_USAGE, message: message.into() }
    }
}

impl From<Error> for CliError {
    fn from(e: Error) -> Self {
        let code = match e {
            Error::Io { .. } => EXIT_IO,
            Error::Decode(_) => EXIT_DECODE,
            Error::UnknownFormat { .. } => EXIT_USAGE,
            Error::Encode(_) | Error::Image(_) => EXIT_FAILED,
        };
        CliError { code, message: e.to_string() }
    }
}

impl From<ImageError> for CliError {
    fn from(e: ImageError) -> Self {
        CliError { code: EXIT_FAILED, message: e.to_string() }
    }
}

impl From<std::io::Error> for CliError {
    /// Only used for failures to write to standard output.
    fn from(e: std::io::Error) -> Self {
        CliError { code: EXIT_IO, message: format!("cannot write output: {e}") }
    }
}

/// Runs the tool with `args` (without the program name) and returns the process exit code.
///
/// Normal output goes to `out`; error messages go to `err`, prefixed with `pixelgrid: `.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(e) => {
            let _ = writeln!(err, "pixelgrid: {}", e.message);
            if e.code == EXIT_USAGE {
                let _ = writeln!(err, "Try 'pixelgrid help'.");
            }
            e.code
        }
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, CliError> {
    let Some(command) = args.first() else {
        return Err(CliError::usage("no command given"));
    };
    match command.as_str() {
        "help" | "--help" | "-h" => {
            out.write_all(USAGE.as_bytes())?;
            Ok(EXIT_OK)
        }
        "--version" | "-V" => {
            writeln!(out, "pixelgrid {}", env!("CARGO_PKG_VERSION"))?;
            Ok(EXIT_OK)
        }
        "info" => info(&args[1..], out),
        "diff" => diff(&args[1..], out),
        other => Err(CliError::usage(format!("unknown command '{other}'"))),
    }
}

/// Parses a number, naming the command and the argument in the error.
fn number<T: std::str::FromStr>(command: &str, what: &str, token: &str) -> Result<T, CliError> {
    token
        .parse()
        .map_err(|_| CliError::usage(format!("{command}: invalid {what} '{token}'")))
}

/// Reads an image file and also reports the format it was stored in.
fn load(path: &str) -> Result<(DynImage, Format), CliError> {
    let data = std::fs::read(path)
        .map_err(|e| Error::Io { path: path.to_string(), message: e.to_string() })?;
    let image = io::decode(&data).map_err(|e| CliError {
        code: EXIT_DECODE,
        message: format!("{path}: {e}"),
    })?;
    let format = Format::from_magic(&data).expect("decode accepted the signature");
    Ok((image, format))
}

fn info(args: &[String], out: &mut dyn Write) -> Result<i32, CliError> {
    let [path] = args else {
        return Err(CliError::usage("info: expected exactly one file"));
    };
    let (image, format) = load(path)?;
    let (width, height) = image.dimensions();
    writeln!(out, "{path}: {} {width}x{height} {}", format.name(), image.pixel_name())?;
    Ok(EXIT_OK)
}

fn diff(args: &[String], out: &mut dyn Write) -> Result<i32, CliError> {
    let mut paths = Vec::new();
    let mut tolerance = 0u8;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--tolerance" || arg == "-t" {
            let value = rest.next().ok_or_else(|| CliError::usage("diff: --tolerance needs a value"))?;
            tolerance = number("diff", "tolerance", value)?;
        } else {
            paths.push(arg.as_str());
        }
    }
    let [left, right] = paths[..] else {
        return Err(CliError::usage("diff: expected exactly two files"));
    };
    let (a, _) = load(left)?;
    let (b, _) = load(right)?;
    let report = match compare_dyn(&a, &b, tolerance) {
        Ok(report) => report,
        Err(e @ ImageError::DimensionMismatch { .. }) => {
            writeln!(out, "{e}")?;
            return Ok(EXIT_DIFFERENT);
        }
        Err(e) => return Err(e.into()),
    };
    match report.first {
        None => {
            writeln!(out, "images match ({} pixels, max delta {})", report.total, report.max_delta)?;
            Ok(EXIT_OK)
        }
        Some((x, y)) => {
            writeln!(
                out,
                "{} of {} pixels differ (max delta {}, first at {x},{y})",
                report.differing, report.total, report.max_delta
            )?;
            Ok(EXIT_DIFFERENT)
        }
    }
}
