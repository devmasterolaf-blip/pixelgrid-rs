# Contributing to pixelgrid

Thanks for taking a look. pixelgrid is a small project, so the process is light: open an issue
or a pull request and describe what you saw and what you expected.

## Getting started

```sh
cargo build
cargo test
```

Both must finish without warnings. There is no CI yet, so please run the full test suite
before you send a change. The crate has no dependencies and should stay that way unless there
is a very good reason.

## Ground rules

- **Results are defined exactly.** Public functions document their rounding, edge behaviour
  and coordinate conventions, and the tests pin them with literal pixel values.
  If a change alters output pixels, say so in the pull request and update the affected
  expectations by hand; do not regenerate them from the new code without checking them.
- **Small, hand-checked test images.** Tests build their inputs in code (see
  `tests/common/mod.rs`). Do not add binary image files. A 3x3 image whose expected result
  you can work out on paper is better than a large one compared against a checksum.
- **Typed errors, no panics on bad data.** Anything that depends on file content or
  user-supplied numbers returns one of the error types in `src/error.rs`. Panics are reserved
  for programmer errors that are documented under a `# Panics` heading.
- **Decoders never trust the header.** Check sizes before allocating and report truncated
  input with the number of bytes or samples that were expected and found.
- **Drawing clips, it never fails.** Drawing functions take signed coordinates and silently
  skip what falls outside the image.
- **The command-line tool validates first.** All operations and the output file name are
  parsed before the input is opened. New operations go into `src/cli/ops.rs` (parser, `Op`
  variant and `apply`), the usage text in `src/cli.rs`, the table in the README and a test in
  `tests/cli.rs`.

## Style

`rustfmt` and `clippy` are welcome but not required; the code is formatted by hand in the
default rustfmt style with lines of about 100 columns. Please match the surrounding code:

- four spaces, no tabs, trailing commas in multi-line lists;
- `use` lines sorted, standard library first, then `crate::`;
- doc comments on new public items, with `# Errors` and `# Panics` sections where they apply;
- British spelling in prose ("colour", "grey"), American spelling in identifiers (`color`,
  `Gray`), because that is what the wider Rust image ecosystem uses.

## Tests

Each area has its own file under `tests/`, named after the module it covers. Add new cases
next to the related ones and name them after the behaviour they check
(`bilinear_halving_averages_pairs`, not `test_resize_2`). Tests that need real files use
`common::scratch_dir`, which creates a directory below `target/`.

The Rust code blocks in the README are compiled and run by `cargo test`, so examples there
must stay complete and correct.

## Commits

Write messages in the form `area: what changed` or `type(area): what changed`, for example
`fix(io): reject BMP files whose row size overflows`. Keep unrelated changes in separate
commits and add a line to `CHANGELOG.md` under "Unreleased" for anything a user would notice.

## Licence

By contributing you agree that your work is released under the MIT licence of the project.
