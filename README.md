# pixelgrid

A small 2D raster image library and command-line tool written in plain Rust. It depends on
nothing but the standard library.

pixelgrid is a young project (version 0.1.0, not published on crates.io). It aims to be easy to
read and to have exactly defined results, so its tests compare against hand-checked pixel
values. It does not aim to be fast or to cover many file formats; see
[Limitations](#limitations) before you rely on it.

## What is in it

- **Pixels and buffers**: 8-bit `Gray`, `Rgb` and `Rgba` pixels behind one `Pixel` trait, a
  generic `Image<P>` with checked and unchecked access, row and pixel iterators, borrowed
  sub-image views, and `DynImage` for images whose pixel type is only known at run time.
- **Colour**: RGB to HSV and HSL and back, the sRGB transfer curve, straight and premultiplied
  alpha compositing, eight blend modes, and parsing of colour names and `#rgb`, `#rgba`,
  `#rrggbb`, `#rrggbbaa` strings with typed errors.
- **File formats**: Netpbm (PBM, PGM and PPM, plain and raw, comments, maxval up to 65535) and
  uncompressed 24-bit and 32-bit BMP (bottom-up and top-down). Malformed input gives a typed
  `DecodeError` that says what was wrong.
- **Drawing**, clipped to the image: lines (Bresenham), thick lines, rectangles, circles and
  ellipses (midpoint algorithms), polygon outlines and scanline fills with the even-odd and
  non-zero rules, flood fill with a tolerance, blits with blend modes, and text in a built-in
  3x5 font.
- **Transforms**: crop, flips, quarter turns, rotation by any angle, and resizing with
  nearest, bilinear or bicubic sampling.
- **Filters**: convolution with a fast path for separable kernels, box and Gaussian blur,
  sharpen, Sobel, median, threshold and Otsu's method, histograms and equalization,
  brightness, contrast, gamma, and dithering to a palette (Floyd-Steinberg and Bayer).
- **Analysis**: connected-component labelling with area, bounding box and centroid, and
  pixel-wise comparison of two images with a tolerance.
- **`pixelgrid` binary**: chains the operations above between an input and an output file,
  plus `info`, `diff` and `components` commands.

## Build

You need a Rust toolchain (the code is developed with Rust 1.93, edition 2021).

```sh
git clone <this repository>
cd pixelgrid-rs
cargo build --release        # the binary is target/release/pixelgrid
cargo test                   # see "Tests" below
```

To use the library from another crate, point a path or git dependency at this repository:

```toml
[dependencies]
pixelgrid = { path = "../pixelgrid-rs" }
```

## Using the library

The example below is compiled and run as a documentation test, so it stays correct.

```rust
use pixelgrid::draw::{fill_rect, line};
use pixelgrid::filter::threshold;
use pixelgrid::label::label_nonzero;
use pixelgrid::transform::{resize, rotate90, Interpolation};
use pixelgrid::{io, Connectivity, DynImage, Image, Rect, Rgb};

// A 6x4 RGB canvas with a red square and a white vertical line.
let mut canvas: Image<Rgb> = Image::new(6, 4);
fill_rect(&mut canvas, 1, 1, 2, 2, "red".parse().unwrap());
line(&mut canvas, 5, 0, 5, 3, Rgb::WHITE);
assert_eq!(canvas[(1, 1)], Rgb::new(255, 0, 0));
assert_eq!(canvas.get(6, 0), None); // checked access outside the image

// Geometry: a quarter turn swaps the sides, nearest-neighbour scaling repeats pixels.
assert_eq!(rotate90(&canvas).dimensions(), (4, 6));
let big = resize(&canvas, 12, 8, Interpolation::Nearest).unwrap();
assert_eq!(big[(3, 3)], Rgb::new(255, 0, 0));

// Analysis: binarize on luma, then find the connected regions in reading order.
let mask = threshold(&canvas, 50);
let regions = label_nonzero(&mask, Connectivity::Four);
assert_eq!(regions.count(), 2);
assert_eq!(regions.components()[0].bounds, Rect::new(5, 0, 1, 4)); // the line
assert_eq!(regions.components()[1].area, 4); // the square

// File formats work on byte buffers; `io::load` and `io::save` wrap them for files.
let image = DynImage::from(canvas);
let bytes = io::encode(&image, io::Format::Ppm, &io::EncodeOptions::default()).unwrap();
assert!(bytes.starts_with(b"P6\n6 4\n255\n"));
assert_eq!(io::decode(&bytes).unwrap(), image);
```

Decoders return a `DynImage`. The `map_dyn!` macro runs the same generic code on whichever
pixel type it holds:

```rust,no_run
use pixelgrid::filter::gaussian_blur;
use pixelgrid::{io, map_dyn};

let image = io::load("photo.ppm").unwrap(); // the format is detected from the content
let blurred = map_dyn!(image, |img| gaussian_blur(&img, 1.5).unwrap());
io::save("photo.bmp", &blurred).unwrap(); // the format follows the extension
```

Conventions worth knowing:

- `(0, 0)` is the top-left pixel, `x` grows to the right and `y` downwards.
- Positive rotation angles turn clockwise as seen on screen.
- Alpha is straight (not premultiplied) unless a function says otherwise.
- Drawing functions take signed coordinates and clip silently. They write the colour as it is
  and do not blend; use `draw::blit` with a `BlendMode` to composite.
- Functions that can fail return `Result` with one of the error types in `pixelgrid::error`.
  A few documented preconditions panic instead (for example indexing outside the image, or a
  rotation angle that is not finite).

## Using the command-line tool

```text
pixelgrid [options] <input> [operation...] <output>
pixelgrid info <file>
pixelgrid diff <a> <b> [--tolerance N]
pixelgrid components <file> [--threshold N] [--connectivity 4|8]
pixelgrid help | --help | --version
```

The input format is detected from the file content, the output format follows the extension
(`.pbm`, `.pgm`, `.ppm`, `.pnm`, `.bmp`). Operations run from left to right. The whole command
line is checked before the input is read, so a typo never costs you a long computation.

The session below starts from a 4x4 image typed by hand. Run it after
`cargo build --release`; `pixelgrid` stands for `target/release/pixelgrid`. The lines that
start with `#` show what the command before them prints.

```sh
cat > demo.ppm <<'EOF'
P3
# a 4x4 test card
4 4
255
255 0 0   255 0 0   0 0 255   0 0 255
255 0 0   255 0 0   0 0 255   0 0 255
0 255 0   0 255 0   255 255 255   255 255 255
0 255 0   0 255 0   255 255 255   255 255 255
EOF

pixelgrid info demo.ppm
# demo.ppm: PPM 4x4 rgb

pixelgrid demo.ppm resize 64x64 blur 1.5 rotate 90 out.bmp
pixelgrid info out.bmp
# out.bmp: BMP 64x64 rgb

pixelgrid --ascii demo.ppm gray small.pgm     # small.pgm is a plain-text P2 file

pixelgrid demo.ppm gray threshold otsu mask.pgm
pixelgrid components mask.pgm
# 1 component
# 1: area 8, bounds 0,2 4x2, centroid 1.50,2.50

pixelgrid demo.ppm brightness 10 brighter.ppm
pixelgrid diff demo.ppm brighter.ppm          # exit code 1
# 12 of 16 pixels differ (max delta 10, first at 0,0)
pixelgrid diff demo.ppm brighter.ppm --tolerance 10
# images match (16 pixels, max delta 10)

pixelgrid demo.ppm resize 64x64 nearest rgba \
    fill-circle 32 32 12 '#00000080' text 4 4 2 white HELLO \
    flatten white card.ppm
```

### Operations

| Operation | Effect |
| --- | --- |
| `resize WxH [nearest\|bilinear\|bicubic]` | scale to `W` by `H` pixels (bilinear by default) |
| `crop X Y W H` | keep the `W` by `H` rectangle whose top-left corner is `X,Y` |
| `flip h\|v` | mirror left-right or top-bottom |
| `rotate DEGREES` | rotate clockwise; the canvas grows to fit and the new corners get the default pixel (black, or transparent for RGBA) |
| `blur SIGMA` | Gaussian blur |
| `box RADIUS` | box blur |
| `median RADIUS` | median filter |
| `sharpen` | 3x3 sharpening kernel |
| `sobel` | edge strength as a grey image |
| `gray`, `rgb`, `rgba` | convert the pixel type (`rgb` drops alpha without compositing) |
| `flatten COLOUR` | composite an RGBA image over `COLOUR`, giving RGB; other images pass through |
| `threshold N\|otsu` | white where luma is above `N`, black elsewhere; `otsu` picks `N` |
| `equalize` | histogram equalization |
| `brightness N` | add `N` (may be negative) to every colour channel |
| `contrast FACTOR` | scale the distance from mid-grey |
| `gamma G` | gamma correction; values above 1 brighten the mid-tones |
| `invert` | negative |
| `dither fs\|bayer2\|bayer4\|bayer8 LEVELS` | reduce to `LEVELS` values per channel (2 to 256 for grey, 2 to 6 for colour) |
| `line X0 Y0 X1 Y1 COLOUR` | one-pixel line |
| `rect X Y W H COLOUR`, `fill-rect ...` | rectangle outline or filled rectangle |
| `circle CX CY RADIUS COLOUR`, `fill-circle ...` | circle outline or disc |
| `text X Y SCALE COLOUR STRING` | text in the built-in font, `SCALE` pixels per font pixel |
| `flood X Y COLOUR TOLERANCE` | flood fill from the seed `X,Y` (4-connected) |

Options, given before the input file: `--ascii` writes plain-text Netpbm files, `--top-down`
writes BMP files with the top row first.

Colours are one of about twenty names (`black`, `white`, `red`, `orange`, ...,
`transparent`) or a hex value such as `#f80`, `#ff8800` or `#ff880080`. On a grey image a
colour is reduced to its luma; on an image without alpha its alpha is ignored.

Pixel types are only changed when you ask for it. A grey result is widened to RGB when the
output is a `.ppm` file because that loses nothing. Anything else that would drop colour or
alpha is an error that names the step to add:

```text
$ pixelgrid demo.ppm out.pgm
pixelgrid: out.pgm: PGM cannot store rgb pixels; add 'gray' before the output
```

### Exit codes

| Code | Meaning |
| --- | --- |
| 0 | success; for `diff`, the images match |
| 1 | `diff` found differences (including different sizes) |
| 2 | the command line could not be understood |
| 3 | a file could not be read or written |
| 4 | an input file is not a valid image |
| 5 | an operation or the encoder rejected the image |

## Project layout

```text
src/
  lib.rs            crate root and re-exports
  pixel.rs          Pixel trait, Gray, Rgb, Rgba
  image.rs          Image<P>, Rect, Connectivity
  view.rs           SubImage and SubImageMut
  dynimage.rs       DynImage and the map_dyn! macro
  error.rs          ImageError, DecodeError, EncodeError, Error
  color/            space.rs (HSV, HSL), gamma.rs (sRGB), parse.rs, blend.rs
  io/               mod.rs (format detection, load, save), pnm.rs, bmp.rs
  draw/             line.rs, shapes.rs, polygon.rs, flood.rs, blit.rs, text.rs
  transform/        geometry.rs, resample.rs, rotate.rs
  filter/           kernel.rs, spatial.rs, tone.rs, dither.rs
  label.rs          connected-component labelling
  compare.rs        image comparison
  cli.rs, cli/ops.rs   the command-line front end, as a testable library function
  main.rs           the pixelgrid binary
tests/              one integration test file per area, plus common/ helpers
```

## Tests

```sh
cargo test                      # everything: the files in tests/ and the README examples
cargo test --test resize        # one file
cargo test --test cli diff      # tests in tests/cli.rs whose name contains "diff"
```

The tests build their tiny input images in code; no image files are stored in the
repository. Tests that need files write them under `target/test-scratch/`.

## Limitations

- Channels are 8 bits. Netpbm files with a maxval above 255 are read but scaled down to 8
  bits, and are never written.
- Only the first image of a multi-image Netpbm file is read. PAM (`P7`) is not supported.
- BMP support is limited to uncompressed 24-bit and 32-bit files. Palettes, RLE, bit fields
  and 16-bit files are rejected with an `Unsupported` error. The fourth byte of a 32-bit
  pixel is always treated as alpha.
- There are no PNG, JPEG or GIF codecs.
- `resize` does not low-pass filter when shrinking, so strong downscaling aliases.
- Resampling and filters work on straight alpha and on gamma-encoded values.
- Drawing is not anti-aliased, and the font only has digits, upper-case Latin letters and a
  little punctuation.
- Everything is single-threaded and written for clarity. Blur and median filters cost time
  proportional to their radius per pixel.
- Image sizes are limited by memory only; the tool does not guard against requests such as
  `resize 100000x100000`.

## License

MIT, see [LICENSE](LICENSE).
