mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{gray, gray_rows, mask, picture, rgb_pattern, scratch_dir};
use pixelgrid::{cli, io, DynImage, Gray, Image, Rgb, Rgba};

/// Runs the CLI in-process and returns `(exit code, stdout, stderr)`.
fn run(args: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(&args, &mut out, &mut err);
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("scratch paths are valid UTF-8")
}

/// Runs a pipeline that is expected to succeed silently and returns the image it wrote.
fn pipe(input: &Path, ops: &[&str], output: &Path) -> DynImage {
    let mut args = vec![path_str(input)];
    args.extend_from_slice(ops);
    args.push(path_str(output));
    assert_eq!(run(&args), (0, String::new(), String::new()), "pipeline {ops:?}");
    io::load(output).unwrap()
}

/// Saves an image into `dir` and returns its path.
fn write(dir: &Path, name: &str, image: impl Into<DynImage>) -> PathBuf {
    let path = dir.join(name);
    io::save(&path, &image.into()).unwrap();
    path
}

#[test]
fn help_and_version() {
    let (code, out, err) = run(&["help"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("pixelgrid - a small raster image tool\n"));
    assert!(out.contains("Usage:") && out.contains("Exit codes:"));
    assert_eq!(err, "");
    assert_eq!(run(&["--help"]), (0, out, String::new()));
    assert_eq!(run(&["--version"]), (0, format!("pixelgrid {}\n", env!("CARGO_PKG_VERSION")), String::new()));
}

#[test]
fn missing_or_unknown_command_is_a_usage_error() {
    let hint = "Try 'pixelgrid help'.\n";
    assert_eq!(run(&[]), (2, String::new(), format!("pixelgrid: no command given\n{hint}")));
    assert_eq!(run(&["frobnicate"]), (2, String::new(), format!("pixelgrid: unknown command 'frobnicate'\n{hint}")));
    assert_eq!(run(&["--ascii"]), (2, String::new(), format!("pixelgrid: no input file given\n{hint}")));
    assert_eq!(run(&["--fast", "a.pgm", "b.pgm"]), (2, String::new(), format!("pixelgrid: unknown option '--fast'\n{hint}")));
    assert_eq!(run(&["info"]).0, 2);
    assert_eq!(run(&["info", "a.pgm", "b.pgm"]).0, 2);
}

#[test]
fn info_reports_format_size_and_pixel_type() {
    let dir = scratch_dir("cli_info");
    let ppm = write(&dir, "a.ppm", rgb_pattern(4, 3));
    let pgm = write(&dir, "b.pgm", gray(&[&[1, 2], &[3, 4], &[5, 6]]));
    let bmp = write(&dir, "c.bmp", rgb_pattern(2, 5).convert::<pixelgrid::Rgba>());
    let (ppm, pgm, bmp) = (path_str(&ppm), path_str(&pgm), path_str(&bmp));
    assert_eq!(run(&["info", ppm]), (0, format!("{ppm}: PPM 4x3 rgb\n"), String::new()));
    assert_eq!(run(&["info", pgm]), (0, format!("{pgm}: PGM 2x3 gray\n"), String::new()));
    assert_eq!(run(&["info", bmp]), (0, format!("{bmp}: BMP 2x5 rgba\n"), String::new()));
}

#[test]
fn unreadable_and_invalid_files_have_their_own_exit_codes() {
    let dir = scratch_dir("cli_bad_files");
    let missing = dir.join("missing.ppm");
    let (code, out, err) = run(&["info", path_str(&missing)]);
    assert_eq!((code, out.as_str()), (3, ""));
    assert!(err.starts_with(&format!("pixelgrid: {}: ", path_str(&missing))), "{err}");

    let garbage = dir.join("garbage.ppm");
    std::fs::write(&garbage, b"P6\n2 2\n255\n\x01\x02\x03").unwrap();
    let garbage = path_str(&garbage);
    let expected = format!("pixelgrid: {garbage}: pixel data is truncated: expected 12, found 3\n");
    assert_eq!(run(&["info", garbage]), (4, String::new(), expected));

    let not_an_image = dir.join("notes.txt");
    std::fs::write(&not_an_image, b"hello").unwrap();
    assert_eq!(run(&["info", path_str(&not_an_image)]).0, 4);
}

#[test]
fn diff_reports_matches_and_differences() {
    let dir = scratch_dir("cli_diff");
    let a = write(&dir, "a.pgm", gray(&[&[10, 20, 30], &[40, 50, 60]]));
    let b = write(&dir, "b.pgm", gray(&[&[10, 22, 30], &[45, 50, 50]]));
    let (a, b) = (path_str(&a), path_str(&b));
    assert_eq!(run(&["diff", a, a]), (0, "images match (6 pixels, max delta 0)\n".to_string(), String::new()));
    assert_eq!(run(&["diff", a, b]), (1, "3 of 6 pixels differ (max delta 10, first at 1,0)\n".to_string(), String::new()));
    assert_eq!(
        run(&["diff", a, b, "--tolerance", "5"]),
        (1, "1 of 6 pixels differ (max delta 10, first at 2,1)\n".to_string(), String::new())
    );
    // The option may come first and has a short form.
    assert_eq!(run(&["diff", "-t", "10", a, b]), (0, "images match (6 pixels, max delta 10)\n".to_string(), String::new()));
}

#[test]
fn diff_compares_across_formats_and_pixel_types() {
    let dir = scratch_dir("cli_diff_formats");
    let grey = gray(&[&[0, 128], &[255, 7]]);
    let pgm = write(&dir, "g.pgm", grey.clone());
    let bmp = write(&dir, "g.bmp", grey.convert::<pixelgrid::Rgb>());
    assert_eq!(run(&["diff", path_str(&pgm), path_str(&bmp)]).0, 0);
}

#[test]
fn diff_treats_a_size_mismatch_as_a_difference() {
    let dir = scratch_dir("cli_diff_sizes");
    let a = write(&dir, "a.pgm", gray(&[&[1, 2, 3]]));
    let b = write(&dir, "b.pgm", gray(&[&[1, 2], &[3, 4]]));
    assert_eq!(
        run(&["diff", path_str(&a), path_str(&b)]),
        (1, "image sizes differ: 3x1 and 2x2\n".to_string(), String::new())
    );
}

#[test]
fn diff_validates_its_arguments() {
    let dir = scratch_dir("cli_diff_args");
    let a = write(&dir, "a.pgm", gray(&[&[1]]));
    let a = path_str(&a);
    let hint = "Try 'pixelgrid help'.\n";
    assert_eq!(run(&["diff", a]), (2, String::new(), format!("pixelgrid: diff: expected exactly two files\n{hint}")));
    assert_eq!(
        run(&["diff", a, a, "--tolerance", "300"]),
        (2, String::new(), format!("pixelgrid: diff: invalid tolerance '300'\n{hint}"))
    );
    assert_eq!(
        run(&["diff", a, a, "--tolerance"]),
        (2, String::new(), format!("pixelgrid: diff: --tolerance needs a value\n{hint}"))
    );
    assert_eq!(run(&["diff", a, "/nonexistent/b.pgm"]).0, 3);
}

#[test]
fn the_binary_sets_the_process_exit_code() {
    let dir = scratch_dir("cli_binary");
    let a = write(&dir, "a.pgm", gray(&[&[1, 2]]));
    let b = write(&dir, "b.pgm", gray(&[&[1, 3]]));
    let exe = env!("CARGO_BIN_EXE_pixelgrid");
    let same = Command::new(exe).args(["diff", path_str(&a), path_str(&a)]).output().unwrap();
    assert_eq!(same.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&same.stdout), "images match (2 pixels, max delta 0)\n");
    let different = Command::new(exe).args(["diff", path_str(&a), path_str(&b)]).output().unwrap();
    assert_eq!(different.status.code(), Some(1));
    let usage = Command::new(exe).output().unwrap();
    assert_eq!(usage.status.code(), Some(2));
    assert!(usage.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&usage.stderr), "pixelgrid: no command given\nTry 'pixelgrid help'.\n");
}

#[test]
fn pipeline_without_operations_converts_between_formats() {
    let dir = scratch_dir("cli_convert");
    let input = write(&dir, "in.ppm", rgb_pattern(3, 2));
    let out = dir.join("out.bmp");
    assert_eq!(pipe(&input, &[], &out), DynImage::Rgb(rgb_pattern(3, 2)));
    let out = path_str(&out);
    assert_eq!(run(&["info", out]), (0, format!("{out}: BMP 3x2 rgb\n"), String::new()));
}

#[test]
fn pipeline_applies_operations_from_left_to_right() {
    let dir = scratch_dir("cli_chain");
    let input = write(&dir, "in.pgm", gray(&[&[1, 2, 3], &[4, 5, 6]]));
    let out = dir.join("out.pgm");
    assert_eq!(gray_rows(&pipe(&input, &["flip", "h"], &out).to_gray()), [[3, 2, 1], [6, 5, 4]]);
    assert_eq!(gray_rows(&pipe(&input, &["flip", "v"], &out).to_gray()), [[4, 5, 6], [1, 2, 3]]);
    // crop -> [[2, 3], [5, 6]], quarter turn clockwise -> [[5, 2], [6, 3]], then the negative.
    let chained = pipe(&input, &["crop", "1", "0", "2", "2", "rotate", "90", "invert"], &out);
    assert_eq!(gray_rows(&chained.to_gray()), [[250, 253], [249, 252]]);
    // The same steps in another order give another picture.
    let reordered = pipe(&input, &["rotate", "90", "crop", "1", "0", "1", "3"], &out);
    assert_eq!(gray_rows(&reordered.to_gray()), [[1], [2], [3]]);
}

#[test]
fn pipeline_resize_and_tone_operations() {
    let dir = scratch_dir("cli_resize_tone");
    let input = write(&dir, "in.pgm", gray(&[&[10, 20, 200, 210]]));
    let out = dir.join("out.pgm");
    let doubled = pipe(&input, &["resize", "8x2", "nearest"], &out);
    assert_eq!(gray_rows(&doubled.to_gray()), [[10, 10, 20, 20, 200, 200, 210, 210]; 2]);
    // Bilinear is the default: the two middle samples sit half way between their neighbours.
    let halved = pipe(&input, &["resize", "2x1"], &out);
    assert_eq!(gray_rows(&halved.to_gray()), [[15, 205]]);
    assert_eq!(gray_rows(&pipe(&input, &["brightness", "-15"], &out).to_gray()), [[0, 5, 185, 195]]);
    assert_eq!(gray_rows(&pipe(&input, &["brightness", "50"], &out).to_gray()), [[60, 70, 250, 255]]);
    assert_eq!(gray_rows(&pipe(&input, &["threshold", "20"], &out).to_gray()), [[0, 0, 255, 255]]);
    assert_eq!(gray_rows(&pipe(&input, &["threshold", "10"], &out).to_gray()), [[0, 255, 255, 255]]);
    assert_eq!(gray_rows(&pipe(&input, &["threshold", "otsu"], &out).to_gray()), [[0, 0, 255, 255]]);
    assert_eq!(gray_rows(&pipe(&input, &["contrast", "0"], &out).to_gray()), [[128, 128, 128, 128]]);
}

#[test]
fn pipeline_dithers_to_the_requested_number_of_levels() {
    let dir = scratch_dir("cli_dither");
    let input = write(&dir, "in.pgm", gray(&[&[128, 128]]));
    let out = dir.join("out.pgm");
    // 128 rounds up to white and pushes 7/16 of the error (-127) onto its right neighbour,
    // which lands at 72 and rounds down to black.
    assert_eq!(gray_rows(&pipe(&input, &["dither", "fs", "2"], &out).to_gray()), [[255, 0]]);
    // Bayer 2x2 offsets the first row by -3/8 and +1/8 of the level distance (255).
    assert_eq!(gray_rows(&pipe(&input, &["dither", "bayer2", "2"], &out).to_gray()), [[0, 255]]);
    let (code, _, err) = run(&[path_str(&input), "dither", "fs", "1", path_str(&out)]);
    assert_eq!((code, err.as_str()), (5, "pixelgrid: dither: invalid palette: grey levels must be between 2 and 256, got 1\n"));
}

#[test]
fn pipeline_converts_pixel_types_only_when_asked() {
    let dir = scratch_dir("cli_pixel_types");
    let grey = write(&dir, "grey.pgm", gray(&[&[7, 9]]));
    let colour = write(&dir, "colour.ppm", rgb_pattern(2, 2));
    let translucent: Image<Rgba> =
        Image::from_vec(2, 1, vec![Rgba::new(255, 0, 0, 128), Rgba::new(0, 0, 255, 255)]).unwrap();
    let alpha = write(&dir, "alpha.bmp", translucent);

    // Grey widens to RGB without loss, so a PPM target needs no extra step.
    let ppm = dir.join("out.ppm");
    let widened = pipe(&grey, &[], &ppm);
    assert_eq!(widened, DynImage::Rgb(Image::from_vec(2, 1, vec![Rgb::new(7, 7, 7), Rgb::new(9, 9, 9)]).unwrap()));

    // Dropping colour or alpha is never done silently.
    let pgm = dir.join("out.pgm");
    let (colour_s, alpha_s, pgm_s, ppm_s) = (path_str(&colour), path_str(&alpha), path_str(&pgm), path_str(&ppm));
    assert_eq!(
        run(&[colour_s, pgm_s]),
        (5, String::new(), format!("pixelgrid: {pgm_s}: PGM cannot store rgb pixels; add 'gray' before the output\n"))
    );
    assert!(!pgm.exists());
    assert_eq!(
        run(&[alpha_s, ppm_s]),
        (
            5,
            String::new(),
            format!("pixelgrid: {ppm_s}: PPM cannot store rgba pixels; add 'flatten COLOUR' or 'rgb' before the output\n")
        )
    );
    // Luma of (0, 0, 0) and (10, 0, 1) in the top row, (0, 10, 1) and (10, 10, 2) below.
    assert_eq!(gray_rows(&pipe(&colour, &["gray"], &pgm).to_gray()), [[0, 3], [6, 9]]);
    // Half-transparent red over white: the green and blue channels end at 255 * 127 / 255.
    let flat = pipe(&alpha, &["flatten", "white"], &ppm);
    assert_eq!(flat, DynImage::Rgb(Image::from_vec(2, 1, vec![Rgb::new(255, 127, 127), Rgb::new(0, 0, 255)]).unwrap()));
    // 'rgb' just drops the alpha channel.
    let dropped = pipe(&alpha, &["rgb"], &ppm);
    assert_eq!(dropped, DynImage::Rgb(Image::from_vec(2, 1, vec![Rgb::new(255, 0, 0), Rgb::new(0, 0, 255)]).unwrap()));
    assert_eq!(pipe(&grey, &["rgba"], &dir.join("out.bmp")).pixel_name(), "rgba");
}

#[test]
fn pipeline_output_options() {
    let dir = scratch_dir("cli_options");
    let input = write(&dir, "in.pgm", gray(&[&[10, 20], &[30, 40]]));
    let (input, pgm, bmp) = (path_str(&input), dir.join("out.pgm"), dir.join("out.bmp"));
    assert_eq!(run(&["--ascii", input, path_str(&pgm)]).0, 0);
    assert_eq!(std::fs::read_to_string(&pgm).unwrap(), "P2\n2 2\n255\n10 20\n30 40\n");
    assert_eq!(run(&[input, path_str(&pgm)]).0, 0);
    assert_eq!(std::fs::read(&pgm).unwrap(), b"P5\n2 2\n255\n\x0a\x14\x1e\x28");

    assert_eq!(run(&["--top-down", input, "rgb", path_str(&bmp)]).0, 0);
    let top_down = std::fs::read(&bmp).unwrap();
    // A negative height in the header marks a top-down bitmap; the first stored row is row 0.
    assert_eq!(i32::from_le_bytes(top_down[22..26].try_into().unwrap()), -2);
    assert_eq!(&top_down[54..60], [10, 10, 10, 20, 20, 20]);
    assert_eq!(run(&[input, "rgb", path_str(&bmp)]).0, 0);
    let bottom_up = std::fs::read(&bmp).unwrap();
    assert_eq!(i32::from_le_bytes(bottom_up[22..26].try_into().unwrap()), 2);
    assert_eq!(&bottom_up[54..60], [30, 30, 30, 40, 40, 40]);
    assert_eq!(io::decode(&top_down).unwrap(), io::decode(&bottom_up).unwrap());
}

#[test]
fn pipeline_rejects_bad_command_lines_before_touching_any_file() {
    let dir = scratch_dir("cli_usage");
    let out = dir.join("out.pgm");
    let out_s = path_str(&out);
    // The input does not exist, yet the usage problem is what gets reported.
    let cases: &[(&[&str], &str)] = &[
        (&["sparkle"], "unknown operation 'sparkle'"),
        (&["blur"], "blur: missing sigma"),
        (&["blur", "soft"], "blur: invalid sigma 'soft'"),
        (&["blur", "nan"], "blur: invalid sigma 'nan'"),
        (&["resize", "4by4"], "resize: invalid size '4by4'"),
        (&["resize", "4x"], "resize: invalid size '4x'"),
        (&["crop", "1", "2", "3"], "crop: missing height"),
        (&["crop", "1", "-2", "3", "4"], "crop: invalid y '-2'"),
        (&["flip", "sideways"], "flip: invalid direction 'sideways'"),
        (&["rotate", "inf"], "rotate: invalid angle 'inf'"),
        (&["threshold", "256"], "threshold: invalid level '256'"),
        (&["dither", "noise", "2"], "dither: invalid method 'noise'"),
        (&["flatten", "#12345"], "flatten: invalid colour '#12345': a hex colour needs 3, 4, 6 or 8 digits, got 5"),
        (&["flatten", "#ff000080"], "flatten: invalid colour '#ff000080': expected an opaque colour, got alpha 128"),
        (&["invert", "extra"], "unknown operation 'extra'"),
    ];
    for (ops, message) in cases {
        let mut args = vec!["/nonexistent/in.pgm"];
        args.extend_from_slice(ops);
        args.push(out_s);
        let expected = format!("pixelgrid: {message}\nTry 'pixelgrid help'.\n");
        assert_eq!(run(&args), (2, String::new(), expected), "{ops:?}");
    }
    let expected = "pixelgrid: out.jpeg: cannot tell the image format from the file name\nTry 'pixelgrid help'.\n";
    assert_eq!(run(&["/nonexistent/in.pgm", "out.jpeg"]), (2, String::new(), expected.to_string()));
    assert_eq!(run(&["/nonexistent/in.pgm", out_s]).0, 3);
    assert!(!out.exists());
}

#[test]
fn pipeline_reports_operations_that_fail_on_the_image() {
    let dir = scratch_dir("cli_failures");
    let input = write(&dir, "in.pgm", gray(&[&[1, 2], &[3, 4]]));
    let out = dir.join("out.pgm");
    let (input, out_s) = (path_str(&input), path_str(&out));
    let cases: &[(&[&str], &str)] = &[
        (&["crop", "1", "1", "2", "2"], "crop: region 2x2 at (1, 1) does not fit in the 2x2 image"),
        (&["invert", "resize", "0x5"], "resize: invalid size: cannot resize to 0x5"),
        (&["blur", "0"], "blur: invalid sigma: must be a positive number, got 0"),
        (&["gamma", "-1"], "gamma: invalid gamma: must be a positive number, got -1"),
    ];
    for (ops, message) in cases {
        let mut args = vec![input];
        args.extend_from_slice(ops);
        args.push(out_s);
        assert_eq!(run(&args), (5, String::new(), format!("pixelgrid: {message}\n")), "{ops:?}");
    }
    assert!(!out.exists(), "a failed pipeline must not leave an output file behind");
    let unwritable = dir.join("no-such-dir").join("out.pgm");
    assert_eq!(run(&[input, path_str(&unwritable)]).0, 3);
}

#[test]
fn the_example_from_the_readme_runs() {
    let dir = scratch_dir("cli_example");
    let input = write(&dir, "in.ppm", rgb_pattern(16, 12));
    let out = dir.join("out.bmp");
    let image = pipe(&input, &["resize", "64x64", "blur", "1.5", "rotate", "90"], &out);
    assert_eq!((image.dimensions(), image.pixel_name()), ((64, 64), "rgb"));
    // A quarter turn after a symmetric blur equals the blur after the quarter turn.
    let other = dir.join("other.bmp");
    assert_eq!(pipe(&input, &["resize", "64x64", "rotate", "90", "blur", "1.5"], &other), image);
}

#[test]
fn pipeline_draws_shapes_and_text() {
    let dir = scratch_dir("cli_draw");
    let blank = write(&dir, "blank.pgm", Image::<Gray>::new(7, 5));
    let out = dir.join("out.pgm");
    let draw = |ops: &[&str]| picture(&pipe(&blank, ops, &out).to_gray());
    assert_eq!(
        draw(&["line", "0", "0", "4", "4", "white"]),
        ["#......", ".#.....", "..#....", "...#...", "....#.."]
    );
    assert_eq!(
        draw(&["rect", "1", "0", "5", "4", "white"]),
        [".#####.", ".#...#.", ".#...#.", ".#####.", "......."]
    );
    assert_eq!(
        draw(&["fill-rect", "5", "3", "9", "9", "white", "fill-rect", "-2", "-2", "3", "4", "white"]),
        ["#......", "#......", ".......", ".....##", ".....##"]
    );
    assert_eq!(
        draw(&["circle", "3", "2", "2", "white"]),
        ["..###..", ".#...#.", ".#...#.", ".#...#.", "..###.."]
    );
    assert_eq!(
        draw(&["fill-circle", "0", "0", "1", "white"]),
        ["##.....", "#......", ".......", ".......", "......."]
    );
    assert_eq!(
        draw(&["text", "0", "0", "1", "white", "Hi"]),
        ["#.#.###", "#.#..#.", "###..#.", "#.#..#.", "#.#.###"]
    );
    // A scale far beyond the image is fine: one font pixel of the T covers the whole canvas.
    assert_eq!(draw(&["text", "0", "0", "4000000000", "white", "T"]), ["#######"; 5]);
    // Later steps draw over earlier ones, and black erases.
    assert_eq!(
        draw(&["fill-rect", "0", "0", "7", "5", "white", "line", "0", "2", "6", "2", "black"]),
        ["#######", "#######", ".......", "#######", "#######"]
    );
}

#[test]
fn pipeline_narrows_drawing_colours_to_the_pixel_type() {
    let dir = scratch_dir("cli_draw_colours");
    let blank = write(&dir, "blank.pgm", Image::<Gray>::new(3, 1));
    let out = dir.join("out.pgm");
    // Grey images receive the luma of the colour: red is 76, #808080 stays 128.
    let ops = ["fill-rect", "0", "0", "1", "1", "red", "fill-rect", "1", "0", "1", "1", "#808080"];
    assert_eq!(gray_rows(&pipe(&blank, &ops, &out).to_gray()), [[76, 128, 0]]);
    // After 'rgba' the alpha of the colour is stored as given.
    let bmp = dir.join("out.bmp");
    let image = pipe(&blank, &["rgba", "fill-rect", "2", "0", "1", "1", "#0a141e28"], &bmp).to_rgba();
    assert_eq!(image.as_slice(), [Rgba::opaque(0, 0, 0), Rgba::opaque(0, 0, 0), Rgba::new(10, 20, 30, 40)]);
}

#[test]
fn pipeline_flood_fill() {
    let dir = scratch_dir("cli_flood");
    let walls = write(&dir, "walls.pgm", mask(&["..#..", "..#..", "###.."]));
    let out = dir.join("out.pgm");
    let filled = pipe(&walls, &["flood", "0", "0", "#646464", "0"], &out);
    assert_eq!(gray_rows(&filled.to_gray()), [[100, 100, 255, 0, 0], [100, 100, 255, 0, 0], [255, 255, 255, 0, 0]]);
    // With the full tolerance the wall no longer stops the fill.
    let flooded = pipe(&walls, &["flood", "0", "0", "#646464", "255"], &out);
    assert_eq!(gray_rows(&flooded.to_gray()), [[100; 5]; 3]);
    let (walls, out) = (path_str(&walls), path_str(&out));
    assert_eq!(
        run(&[walls, "flood", "5", "0", "white", "0", out]),
        (5, String::new(), "pixelgrid: flood: pixel (5, 0) is outside the 5x3 image\n".to_string())
    );
}

#[test]
fn drawing_operations_validate_their_arguments() {
    let cases: &[(&[&str], &str)] = &[
        (&["line", "0", "0", "4", "4"], "line: missing colour"),
        (&["line", "0", "0", "4", "x", "red"], "line: invalid y1 'x'"),
        (&["rect", "0", "0", "-4", "4", "red"], "rect: invalid width '-4'"),
        (&["fill-circle", "0", "0", "2", "reddish"], "fill-circle: invalid colour 'reddish': unknown colour name 'reddish'"),
        (&["text", "0", "0", "0", "red", "hi"], "text: invalid scale '0'"),
        (&["text", "0", "0", "2", "red"], "text: missing text"),
        (&["flood", "0", "0", "red", "300"], "flood: invalid tolerance '300'"),
        (&["flood", "-1", "0", "red", "0"], "flood: invalid x '-1'"),
    ];
    for (ops, message) in cases {
        let mut args = vec!["/nonexistent/in.pgm"];
        args.extend_from_slice(ops);
        args.push("out.pgm");
        let expected = format!("pixelgrid: {message}\nTry 'pixelgrid help'.\n");
        assert_eq!(run(&args), (2, String::new(), expected), "{ops:?}");
    }
}

#[test]
fn components_lists_connected_regions() {
    let dir = scratch_dir("cli_components");
    let blobs = write(&dir, "blobs.pgm", mask(&["##..#", "#...#", "....#"]));
    let blobs = path_str(&blobs);
    let expected = "2 components\n\
                    1: area 3, bounds 0,0 2x2, centroid 0.33,0.33\n\
                    2: area 3, bounds 4,0 1x3, centroid 4.00,1.00\n";
    assert_eq!(run(&["components", blobs]), (0, expected.to_string(), String::new()));

    // Diagonal neighbours join only with 8-connectivity, which is the default.
    let diagonal = write(&dir, "diagonal.pgm", mask(&["#.", ".#"]));
    let diagonal = path_str(&diagonal);
    let joined = "1 component\n1: area 2, bounds 0,0 2x2, centroid 0.50,0.50\n";
    assert_eq!(run(&["components", diagonal]), (0, joined.to_string(), String::new()));
    let (code, out, _) = run(&["components", "--connectivity", "4", diagonal]);
    assert_eq!((code, out.lines().next()), (0, Some("2 components")));

    // Foreground is luma above the threshold.
    let levels = write(&dir, "levels.pgm", gray(&[&[100, 0, 200]]));
    let levels = path_str(&levels);
    assert_eq!(run(&["components", levels]).1, "1 component\n1: area 1, bounds 2,0 1x1, centroid 2.00,0.00\n");
    assert_eq!(run(&["components", levels, "--threshold", "50"]).1.lines().next(), Some("2 components"));
    assert_eq!(run(&["components", levels, "--threshold", "200"]).1, "0 components\n");
}

#[test]
fn components_validates_its_arguments() {
    let hint = "Try 'pixelgrid help'.\n";
    let usage = |args: &[&str], message: &str| {
        assert_eq!(run(args), (2, String::new(), format!("pixelgrid: components: {message}\n{hint}")), "{args:?}");
    };
    usage(&["components"], "expected exactly one file");
    usage(&["components", "a.pgm", "b.pgm"], "expected exactly one file");
    usage(&["components", "a.pgm", "--threshold"], "--threshold needs a value");
    usage(&["components", "a.pgm", "--threshold", "high"], "invalid threshold 'high'");
    usage(&["components", "a.pgm", "--connectivity", "6"], "invalid connectivity '6'");
    assert_eq!(run(&["components", "/nonexistent/a.pgm"]).0, 3);
}
