//! `manifest-reader`: print a summary of a manifest, or the order in which a task and its dependencies run.
//!
//! ```text
//! manifest-reader pixi.toml          # summary
//! manifest-reader pixi.toml test     # task order, one task per line
//! ```
//!
//! Errors go to stderr as `error: <path>: <message>` with exit code 1; wrong usage exits with code 2.

use std::process::ExitCode;

#[allow(unused_imports)] // TODO(c1): remove once `run` uses it.
use manifest_reader::parse_manifest;

const USAGE: &str = "usage: manifest-reader <path/to/pixi.toml> [task]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (path, task) = match args.as_slice() {
        [path] => (path.as_str(), None),
        [path, task] => (path.as_str(), Some(task.as_str())),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(path, task) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Do the work and return the text to print, or a readable error message.
fn run(path: &str, task: Option<&str>) -> Result<String, String> {
    // TODO(c1) milestone 7:
    //   1. Read the file with `std::fs::read_to_string(path)`; map an I/O error to "cannot read <path>: <err>".
    //   2. `parse_manifest`; map a ManifestError to "<path>: <err>" (its Display, not its Debug).
    //   3. No task: return `manifest.summary()`. A task: return `task_order` joined with "\n", mapping its error
    //      the same way as in step 2.
    // `main` adds the `error: ` prefix and picks the exit code.
    let _ = (path, task);
    todo!("run")
}
