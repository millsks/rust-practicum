//! `manifest-reader`: print a summary of a manifest, or the order in which a task and its dependencies run.
//!
//! ```text
//! manifest-reader pixi.toml          # summary
//! manifest-reader pixi.toml test     # task order, one task per line
//! ```
//!
//! Errors go to stderr as `error: <path>: <message>` with exit code 1; wrong usage exits with code 2.

use std::process::ExitCode;

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
    let src = std::fs::read_to_string(path).map_err(|err| format!("cannot read {path}: {err}"))?;
    let manifest = parse_manifest(&src).map_err(|err| format!("{path}: {err}"))?;
    match task {
        None => Ok(manifest.summary()),
        Some(task) => manifest
            .task_order(task)
            .map(|order| order.join("\n"))
            .map_err(|err| format!("{path}: {err}")),
    }
}
