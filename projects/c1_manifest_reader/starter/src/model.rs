//! The typed manifest: what a `pixi.toml` means once it has been checked.

#[allow(unused_imports)] // TODO(c1): remove once `environment_names` uses BTreeSet.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::platform::Platform;

/// A parsed and validated manifest.
///
/// Every map is a [`BTreeMap`], so iteration is sorted by name and output is deterministic. Pixi uses `IndexMap` to
/// keep the order the user wrote; sorted order is enough here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The `[workspace]` table.
    pub workspace: Workspace,
    /// The default `[dependencies]`, by package name.
    pub dependencies: BTreeMap<String, Spec>,
    /// The default `[tasks]`, by task name.
    pub tasks: BTreeMap<String, Task>,
    /// The `[feature.<name>]` tables, by feature name.
    pub features: BTreeMap<String, Feature>,
    /// The `[environments]` table, by environment name.
    pub environments: BTreeMap<String, Environment>,
}

/// The `[workspace]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// `name`: required.
    pub name: String,
    /// `version`: optional.
    pub version: Option<String>,
    /// `description`: optional.
    pub description: Option<String>,
    /// `channels`: required, in priority order.
    pub channels: Vec<String>,
    /// `platforms`: required.
    pub platforms: Vec<Platform>,
}

/// What a dependency asks for: `numpy = ">=2"` or `numpy = { version = ">=2", channel = "conda-forge" }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spec {
    /// The short form: a bare version constraint.
    Version(String),
    /// The table form.
    Detailed(DetailedSpec),
}

/// The table form of a [`Spec`]. Every field is optional; `{}` means "any version".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetailedSpec {
    /// `version`, for example `">=2,<3"`.
    pub version: Option<String>,
    /// `build`, for example `"py312*"`.
    pub build: Option<String>,
    /// `channel`, for example `"conda-forge"`.
    pub channel: Option<String>,
}

impl Spec {
    /// The version constraint, if one was given.
    pub fn version(&self) -> Option<&str> {
        // TODO(c1) milestone 2: `Spec::Version(v)` gives `Some(v)`; `Spec::Detailed(d)` gives `d.version` as an
        // `Option<&str>` (look at `Option::as_deref`).
        todo!("Spec::version")
    }

    /// The channel the package must come from, if one was given.
    pub fn channel(&self) -> Option<&str> {
        // TODO(c1) milestone 2: only the table form can name a channel.
        todo!("Spec::channel")
    }
}

impl fmt::Display for Spec {
    /// Formats as the version (or `*` when there is none), then the build string, then the channel in parentheses:
    /// `>=2 py312* (conda-forge)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(c1) milestone 2: the short form writes the version as is. The table form writes the version (or `*`),
        // then ` <build>` if there is a build, then ` (<channel>)` if there is a channel: `>=2 py312* (conda-forge)`.
        // `write!(f, ...)?` lets you write the pieces one at a time.
        let _ = f;
        todo!("Display for Spec")
    }
}

/// A task from a `[tasks]` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Task {
    /// `build = "cargo build"`: a command with no dependencies.
    Command(String),
    /// `test = { cmd = "cargo test", depends-on = ["build"] }`: a command that runs after other tasks.
    Execute {
        /// The command line.
        cmd: String,
        /// Tasks to run first, in order.
        depends_on: Vec<String>,
    },
    /// `all = { depends-on = ["lint", "test"] }`: no command of its own, only dependencies.
    Alias {
        /// Tasks to run, in order.
        depends_on: Vec<String>,
    },
}

impl Task {
    /// The command this task runs, or `None` for an alias.
    pub fn command(&self) -> Option<&str> {
        // TODO(c1) milestone 2: `Command` and `Execute` have a command; an `Alias` does not. One match arm can cover
        // two variants with `|` when they bind the same names: `Task::Command(cmd) | Task::Execute { cmd, .. }`.
        todo!("Task::command")
    }

    /// The tasks that must run before this one (empty for a plain command).
    pub fn depends_on(&self) -> &[String] {
        // TODO(c1) milestone 2: return a slice. A `Command` has no dependencies: `&[]` is an empty slice.
        todo!("Task::depends_on")
    }
}

/// A `[feature.<name>]` table: dependencies and tasks that environments can opt into.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Feature {
    /// `[feature.<name>.dependencies]`.
    pub dependencies: BTreeMap<String, Spec>,
    /// `[feature.<name>.tasks]`.
    pub tasks: BTreeMap<String, Task>,
}

/// An entry of `[environments]`: `test = ["test"]` or `test = { features = ["test"], solve-group = "default" }`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Environment {
    /// The features to include, in order. Later features override earlier ones.
    pub features: Vec<String>,
    /// `solve-group`, only available in the table form.
    pub solve_group: Option<String>,
}

impl Manifest {
    /// Every environment name, sorted, including the implicit `default` environment.
    pub fn environment_names(&self) -> Vec<&str> {
        // TODO(c1) milestone 3: collect the keys of `self.environments` as `&str`, add "default", sort, and remove
        // duplicates (an explicit `default` must not appear twice). A `BTreeSet<&str>` does all three for you.
        todo!("Manifest::environment_names")
    }

    /// A short, human-readable summary, one `label: values` line per part of the manifest.
    ///
    /// Values are comma-separated in sorted order (channels and platforms in manifest order), and an empty list is
    /// written as `(none)`. There is no trailing newline.
    pub fn summary(&self) -> String {
        // TODO(c1) milestone 3: build the seven lines in the README, in this order:
        //   workspace: <name>
        //   channels: <channels in manifest order>
        //   platforms: <platforms in manifest order>
        //   dependencies: <names, sorted>
        //   tasks: <names, sorted>
        //   features: <names, sorted>
        //   environments: <self.environment_names()>
        // Lists are joined with ", " and an empty list is written as "(none)". Join the lines with "\n" (no trailing
        // newline). A small helper that turns an iterator of `&str` into the list text keeps this short.
        todo!("Manifest::summary")
    }
}
