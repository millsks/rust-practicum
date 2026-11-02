# Capstone 1: Manifest Reader

Turn a simplified `pixi.toml` into typed Rust structs, report every mistake with the TOML key path where it happened,
work out which packages each environment installs, and order tasks by `depends-on`. This is the job Pixi's
`pixi_manifest` crate does, at a size you can finish in a weekend with only what Level 1 teaches.

The full page, with background, hints, and a self-review checklist, is
`docs/projects/c1-manifest-reader.html` (`pixi run serve`, then open Capstone 1).

## Commands

Run these from the repository root.

```sh
pixi run projects                              # list the capstones and which you have started
pixi run start c1                              # copy starter/ to work/c1_manifest_reader/
pixi run grade c1                              # run every grading test against work/ (cargo test)
pixi run grade c1 --no-fail-fast            # keep going after the first failing test file
pixi run grade c1 --test tasks              # one test file: parse, errors, environments, tasks, or cli
pixi run grade c1 --test parse platform     # one test file, only tests whose names contain "platform"
```

Anything after `--` that `practicum grade` does not recognize is passed to `cargo test`. Inside your working copy the
usual Cargo commands work too:

```sh
cd work/c1_manifest_reader
pixi run cargo build
pixi run cargo run -- ../../projects/c1_manifest_reader/tests/fixtures/full.toml
pixi run cargo clippy --all-targets -- -D warnings
pixi run cargo fmt
```

The grading tests live in `projects/c1_manifest_reader/tests/` and are copied next to your code only when grading, so
you cannot edit them by accident. Read them: every assertion message says what was expected and why.

## What is in the crate

| File | Status | What it holds |
|---|---|---|
| `Cargo.toml` | given | Package `manifest_reader`, edition 2024, one dependency (`toml`), a `manifest-reader` binary |
| `src/lib.rs` | yours (one function) | Module tree, re-exports, `parse_manifest` |
| `src/platform.rs` | yours | `Platform` with `as_str`, `Display`, `FromStr`; `ParsePlatformError` |
| `src/model.rs` | types given, methods yours | `Manifest`, `Workspace`, `Spec`, `DetailedSpec`, `Task`, `Feature`, `Environment` |
| `src/error.rs` | enum given, impls yours | `ManifestError` with `Display`, `Error::source`, `From<toml::de::Error>` |
| `src/convert.rs` | yours | `toml::Table` to `Manifest`, plus validation |
| `src/resolve.rs` | yours | `environment_dependencies`, `task_order` |
| `src/main.rs` | `main` given, `run` yours | The command line |

The `toml` crate is used only to parse text into an untyped `toml::Table`. You write every conversion by hand with
`match`, `Option`, and `Result`; no serde derives (those are Level 2).

## The manifest format

A manifest has five top-level keys. Only `workspace` is required; any other top-level key is an error.

```toml
[workspace]
name = "weather-station"                   # required string
version = "0.3.0"                          # optional string
description = "Readings from the garden"   # optional string
channels = ["conda-forge", "bioconda"]     # required array of strings, order kept
platforms = ["linux-64", "osx-arm64"]      # required array of platform names, order kept

[dependencies]
python = "3.13.*"                                          # Spec::Version
pandas = { version = ">=2.2", channel = "conda-forge" }    # Spec::Detailed (keys: version, build, channel; all optional)

[tasks]
fetch = "python -m station.fetch"                                   # Task::Command
clean = { cmd = "python -m station.clean", depends-on = "fetch" }   # Task::Execute; one name is shorthand for a list
plot = { cmd = "python -m station.plot", depends-on = ["clean"] }   # Task::Execute
all = { depends-on = ["plot", "fetch"] }                            # Task::Alias (no cmd)

[feature.test.dependencies]                # a feature has optional `dependencies` and `tasks` tables
pytest = ">=8"

[feature.test.tasks]
test = { cmd = "pytest", depends-on = ["fetch"] }

[environments]
test = ["test"]                                            # list shorthand
lint = { features = ["test"], solve-group = "default" }    # table form (keys: features required, solve-group optional)
```

Platforms are `linux-64`, `linux-aarch64`, `osx-64`, `osx-arm64`, `win-64`, and `noarch`, spelled exactly.

Validation after conversion:

- every feature an environment lists must be defined under `[feature.<name>]`;
- every `depends-on` entry of a default task must name a default task;
- every `depends-on` entry of a feature task must name a task of the same feature or a default task.

Cycles are not a parse error; `task_order` reports them.

## The API the tests call

```rust
pub fn parse_manifest(src: &str) -> Result<Manifest, ManifestError>;

impl Platform {
    pub const ALL: [Platform; 6];
    pub fn as_str(self) -> &'static str;
}
impl FromStr for Platform { type Err = ParsePlatformError; }
impl Display for Platform {}

impl Spec {
    pub fn version(&self) -> Option<&str>;
    pub fn channel(&self) -> Option<&str>;
}
impl Display for Spec {}   // ">=2", or for the table form: version-or-*, then " <build>", then " (<channel>)"

impl Task {
    pub fn command(&self) -> Option<&str>;   // None for an alias
    pub fn depends_on(&self) -> &[String];   // empty for a plain command
}

impl Manifest {
    pub fn environment_names(&self) -> Vec<&str>;
    pub fn summary(&self) -> String;
    pub fn environment_dependencies(&self, env: &str) -> Result<BTreeMap<&str, &Spec>, ManifestError>;
    pub fn task_order(&self, task: &str) -> Result<Vec<&str>, ManifestError>;
}
```

- `environment_names`: the keys of `[environments]` plus `default`, sorted, without duplicates.
- `environment_dependencies`: the default `[dependencies]`, then each feature of the environment in order, a later
  spec for a package replacing an earlier one. `default` (when not defined) has only the default dependencies.
- `task_order`: a depth-first walk of the default `[tasks]` that visits `depends-on` in the order written and adds a
  task after all its dependencies. Each task appears once. Feature tasks are not included.

### `summary()`

Seven lines, joined with `\n`, no trailing newline. Channels and platforms keep manifest order; the other lists are
sorted. An empty list is `(none)`.

```text
workspace: weather-station
channels: conda-forge, bioconda
platforms: linux-64, osx-arm64, win-64
dependencies: numpy, pandas, python
tasks: all, clean, fetch, plot, report
features: lint, py312, test
environments: ci, default, lint, test
```

### Errors

Key paths join table keys with `.` and array positions with `[i]`: `tasks.test.depends-on`, `workspace.platforms[2]`.
`found` is `toml::Value::type_str()` (`string`, `integer`, `float`, `boolean`, `datetime`, `array`, `table`).

| Variant | When | `Display` |
|---|---|---|
| `Toml(toml::de::Error)` | Invalid TOML | ``invalid TOML: <err>`` |
| `MissingKey { path }` | A required key is absent; a task table with neither `cmd` nor `depends-on` reports `<task>.cmd` | ``missing key `<path>` `` |
| `WrongType { path, expected, found }` | A value of the wrong type | ``wrong type for `<path>`: expected <expected>, found <found>`` |
| `UnknownKey { path, allowed }` | A key the table does not accept (first one in sorted order) | ``unknown key `<path>` (expected one of: <allowed, comma-separated>)`` |
| `UnknownPlatform { path, value }` | A platform name that is not one of the six | ``unknown platform `<value>` at `<path>` (known platforms: linux-64, linux-aarch64, osx-64, osx-arm64, win-64, noarch)`` |
| `UnknownFeature { environment, feature }` | An environment lists an undefined feature | ``environment `<environment>` includes unknown feature `<feature>` `` |
| `UnknownTask { path, task }` | A `depends-on` entry names no task; `path` ends in `.depends-on` | ``` `<path>` refers to unknown task `<task>` ``` |
| `NoSuchEnvironment { name }` | `environment_dependencies` was asked for an undefined environment | ``no environment named `<name>` `` |
| `NoSuchTask { name }` | `task_order` was asked for an undefined task | ``no task named `<name>` `` |
| `DependencyCycle { cycle }` | `task_order` found a loop; `cycle` starts and ends with the repeated task | ``dependency cycle: a -> b -> a`` |

`expected` is one of `"string"`, `"table"`, `"array"`, `"string or table"` (dependencies and tasks),
`"string or array"` (`depends-on`), and `"array or table"` (environments). `source()` returns the wrapped error for
`Toml` and `None` otherwise.

## The command line

```text
manifest-reader <path/to/pixi.toml>          print summary(), exit 0
manifest-reader <path/to/pixi.toml> <task>   print task_order(task), one name per line, exit 0
```

On failure print one line to stderr and exit with code 1: `error: cannot read <path>: <io error>` or
`error: <path>: <ManifestError>`. Any other number of arguments prints the usage line and exits with code 2.

## Milestones

| # | Goal | Check with |
|---|---|---|
| 1 | `Platform`: `as_str`, `Display`, `FromStr`, and `ParsePlatformError`'s `Display` | `pixi run grade c1 --test parse platform` |
| 2 | Model methods (`Spec`, `Task`) and the error type (`Display`, `source`, `From`) | `pixi run grade c1 --test parse spec_display` and `pixi run grade c1 --test errors query_errors` |
| 3 | Conversion helpers, the converters, `parse_manifest`, `environment_names`, `summary` | `pixi run grade c1 --test parse` |
| 4 | Every error path, and `validate` | `pixi run grade c1 --test errors` |
| 5 | `environment_dependencies` | `pixi run grade c1 --test environments` |
| 6 | `task_order` with cycle detection | `pixi run grade c1 --test tasks` |
| 7 | The `run` function of the CLI | `pixi run grade c1 --test cli`, then `pixi run grade c1` |

Done means `pixi run grade c1` passes all 46 tests and `pixi run cargo clippy --all-targets -- -D warnings` and
`pixi run cargo fmt --check` are clean in `work/c1_manifest_reader/`.
