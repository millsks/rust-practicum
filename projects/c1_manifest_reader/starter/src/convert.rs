//! Hand-written conversion from an untyped [`toml::Table`] into the typed [`Manifest`].
//!
//! Every function takes the TOML key path of the value it converts, so an error can say exactly where the problem
//! is. Paths are built with [`join`] (`tasks` + `test` = `tasks.test`) and [`index`] (`workspace.platforms` + 1 =
//! `workspace.platforms[1]`).
//!
//! A `toml::Table` is a map from `String` to [`toml::Value`], and `Value` is an enum:
//! `String`, `Integer`, `Float`, `Boolean`, `Datetime`, `Array(Vec<Value>)`, `Table(Table)`. You can `match` on it,
//! or use the helpers `value.as_str()`, `value.as_array()`, `value.as_table()` (each returns an `Option`), and
//! `value.type_str()` (`"string"`, `"integer"`, `"array"`, `"table"`, ...) for error messages.

// TODO(c1): delete this line once every function below is written; it only silences warnings in the starter.
#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::BTreeMap;

use toml::{Table, Value};

use crate::error::ManifestError;
use crate::model::{DetailedSpec, Environment, Feature, Manifest, Spec, Task, Workspace};
use crate::platform::Platform;

// The keys each kind of table accepts. Anything else is a `ManifestError::UnknownKey`.
const MANIFEST_KEYS: &[&str] = &[
    "workspace",
    "dependencies",
    "tasks",
    "feature",
    "environments",
];
const WORKSPACE_KEYS: &[&str] = &["name", "version", "description", "channels", "platforms"];
const SPEC_KEYS: &[&str] = &["version", "build", "channel"];
const TASK_KEYS: &[&str] = &["cmd", "depends-on"];
const FEATURE_KEYS: &[&str] = &["dependencies", "tasks"];
const ENVIRONMENT_KEYS: &[&str] = &["features", "solve-group"];

/// Append a key to a path: `join("tasks", "test")` is `tasks.test`, and `join("", "tasks")` is `tasks`.
fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

/// Append an array index to a path: `index("workspace.platforms", 1)` is `workspace.platforms[1]`.
fn index(path: &str, i: usize) -> String {
    format!("{path}[{i}]")
}

// ---------------------------------------------------------------------------------------------------------------
// Milestone 3, part 1: small helpers. Each one is a few lines; every converter below is built from them.
// ---------------------------------------------------------------------------------------------------------------

/// Build a [`ManifestError::WrongType`] for `found`, which should have been `expected`.
fn wrong_type(path: &str, expected: &'static str, found: &Value) -> ManifestError {
    // TODO(c1): fill the three fields; `found.type_str()` names the TOML type that was actually there.
    todo!("wrong_type")
}

/// Reject any key of `table` that is not in `allowed`. Keys are checked in sorted order, so the error is stable.
fn check_keys(
    table: &Table,
    path: &str,
    allowed: &'static [&'static str],
) -> Result<(), ManifestError> {
    // TODO(c1): find the first key of `table` that `allowed` does not contain. If there is one, return
    // `UnknownKey { path: join(path, key), allowed }`; otherwise `Ok(())`. (`table.keys()` iterates in sorted
    // order because the toml crate stores tables in a BTreeMap.)
    todo!("check_keys")
}

/// Borrow `value` as a string, or report [`ManifestError::WrongType`] at `path`.
fn as_str<'a>(value: &'a Value, path: &str) -> Result<&'a str, ManifestError> {
    // TODO(c1): `value.as_str()` returns `Option<&str>`; `ok_or_else` turns `None` into the error. The lifetime
    // `'a` says the returned `&str` borrows from `value`, not from `path`.
    todo!("as_str")
}

/// Borrow `value` as a table, or report [`ManifestError::WrongType`] at `path`.
fn as_table<'a>(value: &'a Value, path: &str) -> Result<&'a Table, ManifestError> {
    // TODO(c1): like `as_str`, with `value.as_table()` and the expected type "table".
    todo!("as_table")
}

/// Convert an array whose items must all be strings.
fn string_list(value: &Value, path: &str) -> Result<Vec<String>, ManifestError> {
    // TODO(c1): the value must be an array (expected "array"). Then convert each item with `as_str`, using
    // `index(path, i)` as its path, and collect. Collecting an iterator of `Result<String, _>` into
    // `Result<Vec<String>, _>` stops at the first error, which is exactly what you want.
    todo!("string_list")
}

/// Look up a key that must be present, or report [`ManifestError::MissingKey`] at `path.key`.
fn required<'a>(table: &'a Table, path: &str, key: &str) -> Result<&'a Value, ManifestError> {
    // TODO(c1): `table.get(key)` returns an `Option<&Value>`.
    todo!("required")
}

/// Read an optional string key: absent is `Ok(None)`, present but not a string is an error.
fn optional_string(table: &Table, path: &str, key: &str) -> Result<Option<String>, ManifestError> {
    // TODO(c1): `table.get(key)` gives `Option<&Value>`. Map the inner value through `as_str` to get an
    // `Option<Result<..>>`, then `.transpose()` flips it into the `Result<Option<..>>` this function returns.
    todo!("optional_string")
}

/// Borrow an optional sub-table: absent is `Ok(None)`, present but not a table is an error.
fn optional_table<'a>(
    table: &'a Table,
    path: &str,
    key: &str,
) -> Result<Option<&'a Table>, ManifestError> {
    // TODO(c1): the same shape as `optional_string`, with `as_table`.
    todo!("optional_table")
}

// ---------------------------------------------------------------------------------------------------------------
// Milestone 3, part 2: the converters. Write them for valid input first; milestone 4 tests every error path.
// ---------------------------------------------------------------------------------------------------------------

/// Convert the whole document, then validate the references between its parts.
pub(crate) fn manifest_from_table(table: &Table) -> Result<Manifest, ManifestError> {
    // TODO(c1): step by step:
    //   1. `check_keys(table, "", MANIFEST_KEYS)?`
    //   2. `workspace` is required: `workspace_from_value(required(table, "", "workspace")?, "workspace")?`
    //   3. `dependencies` and `tasks` are optional tables: use `optional_table`, then the converters below, or an
    //      empty `BTreeMap` when the table is absent.
    //   4. `feature` is an optional table of tables: for each `(name, value)`, convert `as_table(value, ..)` with
    //      `feature_from_table` at path `feature.<name>`.
    //   5. `environments` is an optional table: convert each entry with `environment_from_value` at path
    //      `environments.<name>`.
    //   6. Build the `Manifest`, run `validate(&manifest)?`, and return it.
    todo!("manifest_from_table")
}

/// Convert `[workspace]`: `name`, `channels`, and `platforms` are required; `version` and `description` are not.
fn workspace_from_value(value: &Value, path: &str) -> Result<Workspace, ManifestError> {
    // TODO(c1): the value must be a table; check its keys against WORKSPACE_KEYS. `name` is a required string and
    // `channels` a required `string_list`. `platforms` is a required array: for each item, get the string (path
    // `workspace.platforms[i]`) and `parse::<Platform>()` it. Turn a `ParsePlatformError` into
    // `ManifestError::UnknownPlatform { path, value }` with `map_err`.
    todo!("workspace_from_value")
}

/// Convert a dependencies table (default or feature) into specs by package name.
fn dependencies_from_table(
    table: &Table,
    path: &str,
) -> Result<BTreeMap<String, Spec>, ManifestError> {
    // TODO(c1): convert each `(name, value)` with `spec_from_value` at path `<path>.<name>`. Collecting an
    // iterator of `Result<(String, Spec), _>` into `Result<BTreeMap<..>, _>` works like it does for a Vec.
    todo!("dependencies_from_table")
}

/// Convert one dependency: a version string or a table of `version`, `build`, `channel`.
fn spec_from_value(value: &Value, path: &str) -> Result<Spec, ManifestError> {
    // TODO(c1): `match value`:
    //   Value::String(version) => Spec::Version
    //   Value::Table(table)    => check SPEC_KEYS, then Spec::Detailed with three `optional_string`s
    //   anything else          => wrong_type(path, "string or table", value)
    todo!("spec_from_value")
}

/// Convert a tasks table (default or feature) into tasks by name.
fn tasks_from_table(table: &Table, path: &str) -> Result<BTreeMap<String, Task>, ManifestError> {
    // TODO(c1): the same shape as `dependencies_from_table`, with `task_from_value`.
    todo!("tasks_from_table")
}

/// Convert one task: a command string, a table with `cmd` (and maybe `depends-on`), or an alias table.
fn task_from_value(value: &Value, path: &str) -> Result<Task, ManifestError> {
    // TODO(c1):
    //   - a string is `Task::Command`
    //   - anything but a string or a table is `wrong_type(path, "string or table", value)`
    //   - a table: check TASK_KEYS, read `cmd` with `optional_string`, and read `depends-on` (if present) with
    //     `depends_on_from_value` at path `<path>.depends-on`. Then match on the pair `(cmd, depends_on)`:
    //       (Some(cmd), deps)  => Task::Execute { cmd, depends_on: deps or an empty Vec }
    //       (None, Some(deps)) => Task::Alias
    //       (None, None)       => MissingKey at `<path>.cmd`
    todo!("task_from_value")
}

/// `depends-on` accepts one task name or a list of names, as it does in Pixi.
fn depends_on_from_value(value: &Value, path: &str) -> Result<Vec<String>, ManifestError> {
    // TODO(c1): a string becomes a one-element Vec, an array goes through `string_list`, and anything else is
    // `wrong_type(path, "string or array", value)`.
    todo!("depends_on_from_value")
}

/// Convert one `[feature.<name>]` table; both of its sub-tables are optional.
fn feature_from_table(table: &Table, path: &str) -> Result<Feature, ManifestError> {
    // TODO(c1): check FEATURE_KEYS, then convert the optional `dependencies` and `tasks` sub-tables (paths
    // `<path>.dependencies` and `<path>.tasks`), using empty maps when they are absent.
    todo!("feature_from_table")
}

/// Convert one environment: a list of feature names, or a table with `features` and `solve-group`.
fn environment_from_value(value: &Value, path: &str) -> Result<Environment, ManifestError> {
    // TODO(c1): `match value`:
    //   Value::Array(_) => the list shorthand: features from `string_list(value, path)`, no solve group
    //   Value::Table(t) => check ENVIRONMENT_KEYS; `features` is a required `string_list` (path
    //                      `<path>.features`); `solve-group` is an optional string
    //   anything else   => wrong_type(path, "array or table", value)
    todo!("environment_from_value")
}

/// Check the references between parts of the manifest: environments name features, tasks name tasks.
///
/// A default task may depend on default tasks; a feature task may depend on tasks of its own feature or default
/// tasks. Cycles are not checked here; [`Manifest::task_order`] reports them when a task is asked for.
fn validate(manifest: &Manifest) -> Result<(), ManifestError> {
    // TODO(c1) milestone 4:
    //   1. For each environment (in sorted order), the first feature name that `manifest.features` does not
    //      contain is `UnknownFeature { environment, feature }`.
    //   2. `check_task_references(&manifest.tasks, "tasks", |name| ...)` with a closure that says whether a
    //      default task of that name exists.
    //   3. For each feature, `check_task_references(&feature.tasks, "feature.<name>.tasks", |name| ...)` where a
    //      name may be one of the feature's own tasks or a default task.
    todo!("validate")
}

/// Check that every `depends-on` entry of `tasks` names a task for which `exists` returns true.
fn check_task_references(
    tasks: &BTreeMap<String, Task>,
    path: &str,
    exists: impl Fn(&str) -> bool,
) -> Result<(), ManifestError> {
    // TODO(c1): for each `(name, task)`, the first entry of `task.depends_on()` for which `exists` is false is
    // `UnknownTask { path: "<path>.<name>.depends-on", task }`.
    todo!("check_task_references")
}
