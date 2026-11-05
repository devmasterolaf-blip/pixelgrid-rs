//! Image filters: convolution and the neighbourhood and point operations built around it.

pub mod kernel;
pub mod spatial;

pub use kernel::{convolve, convolve_2d, convolve_separable, BorderMode, Kernel};
pub use spatial::{box_blur, gaussian_blur, gaussian_kernel, median, sharpen, sobel};
