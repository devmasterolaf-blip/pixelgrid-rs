//! Image filters: convolution and the neighbourhood and point operations built around it.

pub mod kernel;

pub use kernel::{convolve, convolve_2d, convolve_separable, BorderMode, Kernel};
