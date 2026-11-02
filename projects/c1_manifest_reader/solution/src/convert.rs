//! Hand-written conversion from an untyped [`toml::Table`] into the typed [`Manifest`].
//!
//! Every function takes the TOML key path of the value it converts, so an error can say exactly where the problem
//! is. Paths are built with [`join`] (`tasks` + `test` = `tasks.test`) and [`index`] (`workspace.platforms` + 1 =
//! `workspace.platforms[1]`).

use std::collections::BTreeMap;

use toml::{Table, Value};

use crate::error::ManifestError;
use crate::model::{DetailedSpec, Environment, Feature, Manifest, Spec, Task, Workspace};
use crate::platform::Platform;

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

/// Build a [`ManifestError::WrongType`] for `found`, which should have been `expected`.
fn wrong_type(path: &str, expected: &'static str, found: &Value) -> ManifestError {
    ManifestError::WrongType {
        path: path.to_string(),
        expected,
        found: found.type_str(),
    }
}

/// Reject any key of `table` that is not in `allowed`. Keys are checked in sorted order, so the error is stable.
fn check_keys(
    table: &Table,
    path: &str,
    allowed: &'static [&'static str],
) -> Result<(), ManifestError> {
    match table.keys().find(|key| !allowed.contains(&key.as_str())) {
        Some(key) => Err(ManifestError::UnknownKey {
            path: join(path, key),
            allowed,
        }),
        None => Ok(()),
    }
}

/// Borrow `value` as a string, or report [`ManifestError::WrongType`] at `path`.
fn as_str<'a>(value: &'a Value, path: &str) -> Result<&'a str, ManifestError> {
    value
        .as_str()
        .ok_or_else(|| wrong_type(path, "string", value))
}

/// Borrow `value` as a table, or report [`ManifestError::WrongType`] at `path`.
fn as_table<'a>(value: &'a Value, path: &str) -> Result<&'a Table, ManifestError> {
    value
        .as_table()
        .ok_or_else(|| wrong_type(path, "table", value))
}

/// Convert an array whose items must all be strings.
fn string_list(value: &Value, path: &str) -> Result<Vec<String>, ManifestError> {
    let items = value
        .as_array()
        .ok_or_else(|| wrong_type(path, "array", value))?;
    items
        .iter()
        .enumerate()
        .map(|(i, item)| as_str(item, &index(path, i)).map(str::to_string))
        .collect()
}

/// Look up a key that must be present, or report [`ManifestError::MissingKey`] at `path.key`.
fn required<'a>(table: &'a Table, path: &str, key: &str) -> Result<&'a Value, ManifestError> {
    table.get(key).ok_or_else(|| ManifestError::MissingKey {
        path: join(path, key),
    })
}

/// Read an optional string key: absent is `Ok(None)`, present but not a string is an error.
fn optional_string(table: &Table, path: &str, key: &str) -> Result<Option<String>, ManifestError> {
    table
        .get(key)
        .map(|value| as_str(value, &join(path, key)).map(str::to_string))
        .transpose()
}

/// Borrow an optional sub-table: absent is `Ok(None)`, present but not a table is an error.
fn optional_table<'a>(
    table: &'a Table,
    path: &str,
    key: &str,
) -> Result<Option<&'a Table>, ManifestError> {
    table
        .get(key)
        .map(|value| as_table(value, &join(path, key)))
        .transpose()
}

/// Convert the whole document, then validate the references between its parts.
pub(crate) fn manifest_from_table(table: &Table) -> Result<Manifest, ManifestError> {
    check_keys(table, "", MANIFEST_KEYS)?;
    let workspace = workspace_from_value(required(table, "", "workspace")?, "workspace")?;
    let dependencies = match optional_table(table, "", "dependencies")? {
        Some(deps) => dependencies_from_table(deps, "dependencies")?,
        None => BTreeMap::new(),
    };
    let tasks = match optional_table(table, "", "tasks")? {
        Some(tasks) => tasks_from_table(tasks, "tasks")?,
        None => BTreeMap::new(),
    };
    let mut features = BTreeMap::new();
    if let Some(feature_table) = optional_table(table, "", "feature")? {
        for (name, value) in feature_table {
            let path = join("feature", name);
            features.insert(
                name.clone(),
                feature_from_table(as_table(value, &path)?, &path)?,
            );
        }
    }
    let mut environments = BTreeMap::new();
    if let Some(env_table) = optional_table(table, "", "environments")? {
        for (name, value) in env_table {
            environments.insert(
                name.clone(),
                environment_from_value(value, &join("environments", name))?,
            );
        }
    }
    let manifest = Manifest {
        workspace,
        dependencies,
        tasks,
        features,
        environments,
    };
    validate(&manifest)?;
    Ok(manifest)
}

/// Convert `[workspace]`: `name`, `channels`, and `platforms` are required; `version` and `description` are not.
fn workspace_from_value(value: &Value, path: &str) -> Result<Workspace, ManifestError> {
    let table = as_table(value, path)?;
    check_keys(table, path, WORKSPACE_KEYS)?;
    let name = as_str(required(table, path, "name")?, &join(path, "name"))?.to_string();
    let channels = string_list(required(table, path, "channels")?, &join(path, "channels"))?;
    let platforms_path = join(path, "platforms");
    let platforms_value = required(table, path, "platforms")?;
    let items = platforms_value
        .as_array()
        .ok_or_else(|| wrong_type(&platforms_path, "array", platforms_value))?;
    let mut platforms = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let item_path = index(&platforms_path, i);
        let text = as_str(item, &item_path)?;
        let platform = text
            .parse::<Platform>()
            .map_err(|err| ManifestError::UnknownPlatform {
                path: item_path,
                value: err.value,
            })?;
        platforms.push(platform);
    }
    Ok(Workspace {
        name,
        version: optional_string(table, path, "version")?,
        description: optional_string(table, path, "description")?,
        channels,
        platforms,
    })
}

/// Convert a dependencies table (default or feature) into specs by package name.
fn dependencies_from_table(
    table: &Table,
    path: &str,
) -> Result<BTreeMap<String, Spec>, ManifestError> {
    table
        .iter()
        .map(|(name, value)| Ok((name.clone(), spec_from_value(value, &join(path, name))?)))
        .collect()
}

/// Convert one dependency: a version string or a table of `version`, `build`, `channel`.
fn spec_from_value(value: &Value, path: &str) -> Result<Spec, ManifestError> {
    match value {
        Value::String(version) => Ok(Spec::Version(version.clone())),
        Value::Table(table) => {
            check_keys(table, path, SPEC_KEYS)?;
            Ok(Spec::Detailed(DetailedSpec {
                version: optional_string(table, path, "version")?,
                build: optional_string(table, path, "build")?,
                channel: optional_string(table, path, "channel")?,
            }))
        }
        other => Err(wrong_type(path, "string or table", other)),
    }
}

/// Convert a tasks table (default or feature) into tasks by name.
fn tasks_from_table(table: &Table, path: &str) -> Result<BTreeMap<String, Task>, ManifestError> {
    table
        .iter()
        .map(|(name, value)| Ok((name.clone(), task_from_value(value, &join(path, name))?)))
        .collect()
}

/// Convert one task: a command string, a table with `cmd` (and maybe `depends-on`), or an alias table.
fn task_from_value(value: &Value, path: &str) -> Result<Task, ManifestError> {
    let table = match value {
        Value::String(cmd) => return Ok(Task::Command(cmd.clone())),
        Value::Table(table) => table,
        other => return Err(wrong_type(path, "string or table", other)),
    };
    check_keys(table, path, TASK_KEYS)?;
    let cmd = optional_string(table, path, "cmd")?;
    let depends_on = match table.get("depends-on") {
        Some(value) => Some(depends_on_from_value(value, &join(path, "depends-on"))?),
        None => None,
    };
    match (cmd, depends_on) {
        (Some(cmd), depends_on) => Ok(Task::Execute {
            cmd,
            depends_on: depends_on.unwrap_or_default(),
        }),
        (None, Some(depends_on)) => Ok(Task::Alias { depends_on }),
        (None, None) => Err(ManifestError::MissingKey {
            path: join(path, "cmd"),
        }),
    }
}

/// `depends-on` accepts one task name or a list of names, as it does in Pixi.
fn depends_on_from_value(value: &Value, path: &str) -> Result<Vec<String>, ManifestError> {
    match value {
        Value::String(name) => Ok(vec![name.clone()]),
        Value::Array(_) => string_list(value, path),
        other => Err(wrong_type(path, "string or array", other)),
    }
}

/// Convert one `[feature.<name>]` table; both of its sub-tables are optional.
fn feature_from_table(table: &Table, path: &str) -> Result<Feature, ManifestError> {
    check_keys(table, path, FEATURE_KEYS)?;
    let dependencies = match optional_table(table, path, "dependencies")? {
        Some(deps) => dependencies_from_table(deps, &join(path, "dependencies"))?,
        None => BTreeMap::new(),
    };
    let tasks = match optional_table(table, path, "tasks")? {
        Some(tasks) => tasks_from_table(tasks, &join(path, "tasks"))?,
        None => BTreeMap::new(),
    };
    Ok(Feature {
        dependencies,
        tasks,
    })
}

/// Convert one environment: a list of feature names, or a table with `features` and `solve-group`.
fn environment_from_value(value: &Value, path: &str) -> Result<Environment, ManifestError> {
    match value {
        Value::Array(_) => Ok(Environment {
            features: string_list(value, path)?,
            solve_group: None,
        }),
        Value::Table(table) => {
            check_keys(table, path, ENVIRONMENT_KEYS)?;
            let features =
                string_list(required(table, path, "features")?, &join(path, "features"))?;
            Ok(Environment {
                features,
                solve_group: optional_string(table, path, "solve-group")?,
            })
        }
        other => Err(wrong_type(path, "array or table", other)),
    }
}

/// Check the references between parts of the manifest: environments name features, tasks name tasks.
///
/// A default task may depend on default tasks; a feature task may depend on tasks of its own feature or default
/// tasks. Cycles are not checked here; [`Manifest::task_order`] reports them when a task is asked for.
fn validate(manifest: &Manifest) -> Result<(), ManifestError> {
    for (env_name, environment) in &manifest.environments {
        if let Some(missing) = environment
            .features
            .iter()
            .find(|f| !manifest.features.contains_key(*f))
        {
            return Err(ManifestError::UnknownFeature {
                environment: env_name.clone(),
                feature: missing.clone(),
            });
        }
    }
    check_task_references(&manifest.tasks, "tasks", |name| {
        manifest.tasks.contains_key(name)
    })?;
    for (feature_name, feature) in &manifest.features {
        let path = format!("feature.{feature_name}.tasks");
        check_task_references(&feature.tasks, &path, |name| {
            feature.tasks.contains_key(name) || manifest.tasks.contains_key(name)
        })?;
    }
    Ok(())
}

/// Check that every `depends-on` entry of `tasks` names a task for which `exists` returns true.
fn check_task_references(
    tasks: &BTreeMap<String, Task>,
    path: &str,
    exists: impl Fn(&str) -> bool,
) -> Result<(), ManifestError> {
    for (name, task) in tasks {
        if let Some(missing) = task.depends_on().iter().find(|dep| !exists(dep)) {
            return Err(ManifestError::UnknownTask {
                path: format!("{}.depends-on", join(path, name)),
                task: missing.clone(),
            });
        }
    }
    Ok(())
}
