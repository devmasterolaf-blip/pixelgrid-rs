//! Image filters: convolution and the neighbourhood and point operations built around it.

pub mod dither;
pub mod kernel;
pub mod spatial;
pub mod tone;

pub use dither::{dither_floyd_steinberg, dither_ordered, quantize, BayerMatrix, Palette};
pub use kernel::{convolve, convolve_2d, convolve_separable, BorderMode, Kernel};
pub use spatial::{box_blur, gaussian_blur, gaussian_kernel, median, sharpen, sobel};
pub use tone::{
    apply_lut, brightness, contrast, equalize, gamma, histogram, invert, luma_histogram,
    otsu_level, otsu_threshold, threshold,
};
