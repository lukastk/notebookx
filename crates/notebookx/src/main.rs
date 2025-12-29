//! nbx binary entry point.

use std::process::ExitCode;

fn main() -> ExitCode {
    let code = notebookx::cli::run(std::env::args_os());
    ExitCode::from(code as u8)
}
