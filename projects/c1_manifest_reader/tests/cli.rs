//! Milestone 6: the `manifest-reader` binary.

use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_manifest-reader"))
        .args(args)
        .output()
        .expect("the manifest-reader binary should start")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn prints_the_summary_and_exits_zero() {
    let out = run(&[&fixture("full.toml")]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a valid manifest should exit 0; stderr was: {}",
        text(&out.stderr)
    );
    let stdout = text(&out.stdout);
    assert!(
        stdout.starts_with("workspace: weather-station\n"),
        "stdout should be manifest.summary(), got:\n{stdout}"
    );
    assert!(
        stdout.contains("environments: ci, default, lint, test"),
        "the summary ends with environments:\n{stdout}"
    );
}

#[test]
fn prints_the_task_order_one_per_line() {
    let out = run(&[&fixture("full.toml"), "report"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a known task should exit 0; stderr was: {}",
        text(&out.stderr)
    );
    assert_eq!(
        text(&out.stdout),
        "fetch\nclean\nplot\nreport\n",
        "with a task argument, print task_order one per line"
    );
}

#[test]
fn a_bad_manifest_prints_a_readable_error_and_exits_one() {
    let path = fixture("unknown-platform.toml");
    let out = run(&[&path]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an invalid manifest should exit with code 1"
    );
    let stderr = text(&out.stderr);
    assert!(
        stderr.starts_with("error: "),
        "errors go to stderr and start with `error: `, got:\n{stderr}"
    );
    assert!(
        stderr.contains(&path),
        "the error should name the file, got:\n{stderr}"
    );
    assert!(
        stderr.contains("unknown platform `linux-arm64`"),
        "the error should use ManifestError's Display:\n{stderr}"
    );
    assert!(
        text(&out.stdout).is_empty(),
        "nothing should be printed to stdout on error"
    );
}

#[test]
fn a_missing_file_and_a_missing_task_exit_one() {
    let out = run(&["does/not/exist.toml"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "an unreadable file should exit with code 1"
    );
    let stderr = text(&out.stderr);
    assert!(
        stderr.starts_with("error: cannot read does/not/exist.toml"),
        "say which file could not be read:\n{stderr}"
    );

    let out = run(&[&fixture("cycle.toml"), "a"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a dependency cycle should exit with code 1"
    );
    assert!(
        text(&out.stderr).contains("dependency cycle: a -> b -> c -> a"),
        "print the cycle:\n{}",
        text(&out.stderr)
    );
}

#[test]
fn wrong_usage_prints_usage_and_exits_two() {
    let out = run(&[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "no arguments is a usage error: exit code 2"
    );
    assert!(
        text(&out.stderr).starts_with("usage: manifest-reader"),
        "print the usage line to stderr"
    );
}
