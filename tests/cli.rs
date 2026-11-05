mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{gray, rgb_pattern, scratch_dir};
use pixelgrid::{cli, io, DynImage};

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
