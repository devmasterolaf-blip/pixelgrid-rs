# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the version numbers follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html). While the version is below 1.0
the API may change between minor versions.

## [Unreleased]

Nothing yet.

## [0.1.0] - 2026-11-05

First version.

### Added

- `Gray`, `Rgb` and `Rgba` 8-bit pixel types behind the `Pixel` trait, with conversions
  between them.
- `Image<P>`: a generic pixel buffer with checked, unchecked and clipped access, row and
  pixel iterators, `map`, and the `SubImage` / `SubImageMut` rectangular views.
- `DynImage` and the `map_dyn!` macro for images whose pixel type is known only at run time.
- Colour handling: RGB/HSV/HSL conversion, the sRGB transfer function and linear-light
  mixing, straight and premultiplied alpha compositing, eight blend modes, and parsing of
  named and hexadecimal colours with `ColorParseError`.
- Netpbm decoding (P1 to P6, comments, maxval up to 65535) and encoding (PBM, PGM, PPM in
  plain and raw form).
- BMP decoding and encoding for uncompressed 24-bit and 32-bit images, bottom-up and
  top-down.
- Format detection from file content and from file extensions, `io::load`, `io::save` and
  `io::save_with`.
- Typed errors: `ImageError`, `DecodeError`, `EncodeError` and the umbrella `Error`.
- Drawing with clipping: lines, thick lines, rectangles, circles, ellipses, polygon
  outlines, polygon fill with the even-odd and non-zero rules, flood fill with tolerance and
  4- or 8-connectivity, blitting with blend modes, and text in a built-in 3x5 font.
- Transforms: crop, horizontal and vertical flips, quarter-turn rotations, rotation by any
  angle with optional canvas expansion, and resizing with nearest, bilinear and bicubic
  sampling.
- Filters: 2D and separable convolution with four border modes, box blur, Gaussian blur,
  sharpen, Sobel edge strength, median, histograms, equalization, fixed and Otsu thresholds,
  brightness, contrast, gamma, invert, palettes, nearest-colour quantization, and
  Floyd-Steinberg and ordered (Bayer 2x2, 4x4, 8x8) dithering.
- Connected-component labelling with area, bounding box and centroid per component.
- Image comparison with a per-channel tolerance and a difference mask.
- The `pixelgrid` command-line tool: an operation pipeline between an input and an output
  file, plus the `info`, `diff` and `components` commands, with distinct exit codes for
  usage, I/O, decode and operation errors.

### Fixed

- `text_size` and `draw_text` no longer overflow for very large scale factors.
- `gaussian_kernel` no longer produces NaN weights for extremely small sigma values.
