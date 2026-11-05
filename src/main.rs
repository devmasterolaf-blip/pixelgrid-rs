//! The `pixelgrid` command-line tool. All of the logic lives in [`pixelgrid::cli`].

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = stdout.lock();
    let code = pixelgrid::cli::run(&args, &mut out, &mut stderr.lock());
    // A closed pipe on the way out is not worth reporting.
    let _ = out.flush();
    std::process::exit(code);
}
