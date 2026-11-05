mod common;

use common::{gray, gray_rows, rgb_pattern};
use pixelgrid::filter::{convolve, convolve_2d, convolve_separable, BorderMode, Kernel};
use pixelgrid::{Gray, Image, ImageError, Rgba};

fn row_kernel(weights: [f32; 3]) -> Kernel {
    Kernel::from_rows(&[weights]).unwrap()
}

#[test]
fn identity_kernel_copies_the_image() {
    let img = rgb_pattern(4, 3);
    let identity = Kernel::from_rows(&[[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    for border in [BorderMode::Clamp, BorderMode::Mirror, BorderMode::Wrap, BorderMode::Zero] {
        assert_eq!(convolve(&img, &identity, border), img);
        assert_eq!(convolve_2d(&img, &identity, border), img);
    }
}

#[test]
fn kernel_is_applied_without_flipping() {
    // A kernel that reads only the left neighbour shifts the image to the right.
    let img = gray(&[&[1, 2, 3, 4]]);
    let left = row_kernel([1.0, 0.0, 0.0]);
    assert_eq!(gray_rows(&convolve(&img, &left, BorderMode::Zero)), [[0, 1, 2, 3]]);
    let right = row_kernel([0.0, 0.0, 1.0]);
    assert_eq!(gray_rows(&convolve(&img, &right, BorderMode::Zero)), [[2, 3, 4, 0]]);
    let up = Kernel::from_rows(&[[1.0], [0.0], [0.0]]).unwrap();
    let column = gray(&[&[1], &[2], &[3]]);
    assert_eq!(gray_rows(&convolve(&column, &up, BorderMode::Zero)), [[0], [1], [2]]);
}

#[test]
fn border_modes() {
    let img = gray(&[&[1, 2, 3, 4]]);
    let left2 = Kernel::from_rows(&[[1.0, 0.0, 0.0, 0.0, 0.0]]).unwrap();
    let right2 = Kernel::from_rows(&[[0.0, 0.0, 0.0, 0.0, 1.0]]).unwrap();
    let run = |k: &Kernel, border| gray_rows(&convolve(&img, k, border)).remove(0);
    // Reading two pixels to the left: the first two outputs come from beyond the border.
    assert_eq!(run(&left2, BorderMode::Clamp), [1, 1, 1, 2]);
    assert_eq!(run(&left2, BorderMode::Mirror), [2, 1, 1, 2]);
    assert_eq!(run(&left2, BorderMode::Wrap), [3, 4, 1, 2]);
    assert_eq!(run(&left2, BorderMode::Zero), [0, 0, 1, 2]);
    assert_eq!(run(&right2, BorderMode::Clamp), [3, 4, 4, 4]);
    assert_eq!(run(&right2, BorderMode::Mirror), [3, 4, 4, 3]);
    assert_eq!(run(&right2, BorderMode::Wrap), [3, 4, 1, 2]);
    assert_eq!(run(&right2, BorderMode::Zero), [3, 4, 0, 0]);
}

#[test]
fn window_larger_than_the_image() {
    let img = gray(&[&[10, 20]]);
    let far = Kernel::from_rows(&[[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]).unwrap();
    // Five pixels to the left of x = 0 and x = 1.
    let run = |border| gray_rows(&convolve(&img, &far, border)).remove(0);
    assert_eq!(run(BorderMode::Clamp), [10, 10]);
    // Mirrored sequence ... 20 10 | 10 20 | 20 10 10 20 ...: index -5 -> 10, -4 -> 10.
    assert_eq!(run(BorderMode::Mirror), [10, 10]);
    // Index -5 wraps to 1, -4 wraps to 0.
    assert_eq!(run(BorderMode::Wrap), [20, 10]);
    assert_eq!(run(BorderMode::Zero), [0, 0]);
}

#[test]
fn averaging_rounds_to_nearest_and_results_are_clamped() {
    let img = gray(&[&[0, 10, 21, 255]]);
    let mean = row_kernel([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    // (0+0+10)/3 = 3.33, (0+10+21)/3 = 10.33, (10+21+255)/3 = 95.33, (21+255+255)/3 = 177
    assert_eq!(gray_rows(&convolve(&img, &mean, BorderMode::Clamp)), [[3, 10, 95, 177]]);
    let double = row_kernel([0.0, 2.0, 0.0]);
    assert_eq!(gray_rows(&convolve(&img, &double, BorderMode::Clamp)), [[0, 20, 42, 255]]);
    let negate = row_kernel([0.0, -1.0, 0.0]);
    assert_eq!(gray_rows(&convolve(&img, &negate, BorderMode::Clamp)), [[0, 0, 0, 0]]);
}

#[test]
fn two_dimensional_kernel_by_hand() {
    let img = gray(&[&[1, 2, 3], &[4, 5, 6], &[7, 8, 9]]);
    let k = Kernel::from_rows(&[[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]).unwrap();
    assert!(k.separate().is_none());
    // Centre: 1*1 + 2*5 + 3*9 = 38. Top-left with zero border: 2*1 + 3*5 = 17.
    // Bottom-right: 1*5 + 2*9 = 23.
    let out = convolve(&img, &k, BorderMode::Zero);
    assert_eq!(gray_rows(&out), [[17, 22, 6], [32, 38, 14], [14, 20, 23]]);
}

#[test]
fn every_channel_is_filtered_including_alpha() {
    let img = Image::from_vec(
        3,
        1,
        vec![Rgba::new(0, 30, 60, 0), Rgba::new(90, 30, 0, 255), Rgba::new(0, 30, 60, 0)],
    )
    .unwrap();
    let mean = row_kernel([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    let out = convolve(&img, &mean, BorderMode::Clamp);
    assert_eq!(out[(1, 0)], Rgba::new(30, 30, 40, 85));
    assert_eq!(out[(0, 0)], Rgba::new(30, 30, 40, 85));
}

#[test]
fn separable_kernels_are_detected() {
    let box3 = Kernel::new(3, 3, vec![1.0 / 9.0; 9]).unwrap();
    let (h, v) = box3.separate().unwrap();
    assert_eq!((h.len(), v.len()), (3, 3));
    for (y, vy) in v.iter().enumerate() {
        for (x, hx) in h.iter().enumerate() {
            assert!((vy * hx - box3.weights()[y * 3 + x]).abs() < 1e-6);
        }
    }
    let sobel = Kernel::from_rows(&[[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]).unwrap();
    let (h, v) = sobel.separate().unwrap();
    // The pivot is the first entry of largest magnitude, the -2 in the middle row.
    assert_eq!(h, [-2.0, 0.0, 2.0]);
    assert_eq!(v, [0.5, 1.0, 0.5]);
    let wide = Kernel::from_rows(&[[1.0, 2.0, 3.0, 2.0, 1.0]]).unwrap();
    assert_eq!(wide.separate().unwrap(), (vec![1.0, 2.0, 3.0, 2.0, 1.0], vec![1.0]));
    let laplace = Kernel::from_rows(&[[0.0, 1.0, 0.0], [1.0, -4.0, 1.0], [0.0, 1.0, 0.0]]).unwrap();
    assert!(laplace.separate().is_none());
    assert!(Kernel::new(3, 1, vec![0.0; 3]).unwrap().separate().is_none());
}

#[test]
fn separable_route_matches_the_direct_route() {
    let img = Image::from_fn(9, 7, |x, y| Gray(((x * 37 + y * 91 + x * y * 13) % 256) as u8));
    let kernels = [
        Kernel::new(3, 3, vec![1.0 / 9.0; 9]).unwrap(),
        Kernel::from_rows(&[[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]]).unwrap().normalized().unwrap(),
        Kernel::from_rows(&[[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]).unwrap(),
        Kernel::new(5, 3, vec![1.0 / 15.0; 15]).unwrap(),
    ];
    for kernel in &kernels {
        assert!(kernel.separate().is_some());
        for border in [BorderMode::Clamp, BorderMode::Mirror, BorderMode::Wrap, BorderMode::Zero] {
            let fast = convolve(&img, kernel, border);
            let direct = convolve_2d(&img, kernel, border);
            for (a, b) in fast.pixels().zip(direct.pixels()) {
                assert!(a.0.abs_diff(b.0) <= 1, "{a:?} vs {b:?} with {border:?}");
            }
        }
    }
}

#[test]
fn separable_passes_do_not_clamp_in_between() {
    // The horizontal pass alone would give -255 and +255; the vertical kernel then scales by
    // one half. Clamping after the first pass would lose the negative lobe entirely.
    let img = gray(&[&[255, 0, 255]]);
    let out = convolve_separable(&img, &[-1.0, 0.0, 1.0], &[-0.5], BorderMode::Zero).unwrap();
    // Horizontal: [0 - 0, 255 - 255, 0 - 0] = [0, 0, 0] with the centre taps... computed below.
    // x = 0: -1 * 0 (border) + 1 * 0 = 0; x = 1: -255 + 255 = 0; x = 2: -0 + 0 = 0.
    assert_eq!(gray_rows(&out), [[0, 0, 0]]);
    let img = gray(&[&[0, 0, 200]]);
    // Horizontal: [0, 200, 0]... x = 1 reads right neighbour 200; x = 2 reads -0 + border 0.
    // A negative horizontal result times a negative vertical weight must come out positive.
    let out = convolve_separable(&img, &[1.0, 0.0, -1.0], &[-0.5], BorderMode::Zero).unwrap();
    assert_eq!(gray_rows(&out), [[0, 100, 0]]);
}

#[test]
fn separable_kernels_must_have_odd_length() {
    let img = gray(&[&[1, 2]]);
    let err = convolve_separable(&img, &[0.5, 0.5], &[1.0], BorderMode::Clamp).unwrap_err();
    assert_eq!(err.to_string(), "invalid horizontal kernel: length must be odd, got 2");
    let err = convolve_separable(&img, &[1.0], &[], BorderMode::Clamp).unwrap_err();
    assert_eq!(err.to_string(), "invalid vertical kernel: length must be odd, got 0");
}

#[test]
fn kernel_validation() {
    let err = Kernel::new(2, 3, vec![0.0; 6]).unwrap_err();
    assert_eq!(err.to_string(), "invalid kernel: dimensions must be odd, got 2x3");
    assert!(Kernel::new(0, 1, vec![]).is_err());
    let err = Kernel::new(3, 3, vec![0.0; 8]).unwrap_err();
    assert_eq!(err.to_string(), "invalid kernel: a 3x3 kernel needs 9 weights, got 8");
    let err = Kernel::new(1, 1, vec![f32::NAN]).unwrap_err();
    assert!(matches!(err, ImageError::InvalidParameter { name: "kernel", .. }));
    assert!(Kernel::from_rows::<3>(&[]).is_err());
    let k = Kernel::from_rows(&[[1.0, 2.0, 1.0]]).unwrap();
    assert_eq!((k.width(), k.height(), k.sum()), (3, 1, 4.0));
    assert_eq!(k.normalized().unwrap().weights(), [0.25, 0.5, 0.25]);
    let zero_sum = Kernel::from_rows(&[[-1.0, 0.0, 1.0]]).unwrap();
    assert_eq!(
        zero_sum.normalized().unwrap_err().to_string(),
        "invalid kernel: weights sum to zero and cannot be normalized"
    );
}

#[test]
fn empty_images_pass_through() {
    let empty: Image<Gray> = Image::new(0, 0);
    let k = Kernel::new(3, 3, vec![1.0; 9]).unwrap();
    assert_eq!(convolve(&empty, &k, BorderMode::Clamp).dimensions(), (0, 0));
    assert_eq!(convolve_2d(&empty, &k, BorderMode::Wrap).dimensions(), (0, 0));
    let thin: Image<Gray> = Image::new(0, 3);
    assert_eq!(convolve(&thin, &k, BorderMode::Mirror).dimensions(), (0, 3));
}
