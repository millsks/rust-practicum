//! Questions you can ask a parsed manifest: which packages an environment gets, and in which order tasks run.

// TODO(c1): delete this line once the functions below are written; it only silences warnings in the starter.
#![allow(dead_code, unused_imports, unused_variables, clippy::ptr_arg)]

use std::collections::BTreeMap;

use crate::error::ManifestError;
use crate::model::{Manifest, Spec};

impl Manifest {
    /// The dependencies an environment installs, by package name.
    ///
    /// Start from the default `[dependencies]`, then apply each of the environment's features in the order listed:
    /// a feature's spec for a package replaces any earlier spec for it. The implicit `default` environment (when the
    /// manifest does not define one) has only the default dependencies. The returned map borrows from `self`, so no
    /// spec is copied.
    ///
    /// # Errors
    ///
    /// [`ManifestError::NoSuchEnvironment`] when `env` is not an environment, and [`ManifestError::UnknownFeature`]
    /// if the environment names a feature that does not exist (only possible for a manifest built by hand, since
    /// [`crate::parse_manifest`] already checks it).
    pub fn environment_dependencies(
        &self,
        env: &str,
    ) -> Result<BTreeMap<&str, &Spec>, ManifestError> {
        // TODO(c1) milestone 5:
        //   1. Find the feature list: the environment's `features` if `self.environments` has `env`; an empty
        //      slice (`&[]`) if `env` is "default" and not defined; otherwise `NoSuchEnvironment`.
        //   2. Start a `BTreeMap<&str, &Spec>` from `self.dependencies` (`name.as_str()` and `spec` are borrows).
        //   3. For each feature name, look the feature up (missing: `UnknownFeature`) and `insert` or `extend` its
        //      dependencies; inserting an existing key replaces the old value, so later features win.
        // Elision ties every `&` in the return type to `&self`, so the map may borrow from `self` but not from
        // `env`.
        todo!("Manifest::environment_dependencies")
    }

    /// The order in which to run `task` and everything it depends on, from the default `[tasks]`.
    ///
    /// Dependencies come first, each task appears once, and the order is deterministic: a depth-first walk that
    /// visits `depends-on` entries in the order they are written, adding a task after all of its dependencies.
    ///
    /// # Errors
    ///
    /// [`ManifestError::NoSuchTask`] when `task` (or a task it depends on) is not defined, and
    /// [`ManifestError::DependencyCycle`] when the dependencies loop back on themselves.
    pub fn task_order(&self, task: &str) -> Result<Vec<&str>, ManifestError> {
        // TODO(c1) milestone 6: create two empty Vecs, `order` (the result) and `in_progress` (the chain of tasks
        // being visited right now), call `self.visit_task(task, &mut in_progress, &mut order)?`, and return
        // `order`.
        todo!("Manifest::task_order")
    }

    /// Visit one task: its dependencies first, then itself.
    ///
    /// `in_progress` is the chain of tasks currently being visited. Meeting one of them again means a cycle.
    /// Names pushed onto either list are borrowed from `self.tasks` (via `get_key_value`), not from `name`, so they
    /// live as long as `self`.
    fn visit_task<'a>(
        &'a self,
        name: &str,
        in_progress: &mut Vec<&'a str>,
        order: &mut Vec<&'a str>,
    ) -> Result<(), ManifestError> {
        // TODO(c1) milestone 6:
        //   1. If `order` already contains `name`, it is scheduled: return Ok(()).
        //   2. If `in_progress` contains `name` at position `start`, the cycle is `in_progress[start..]` plus `name`
        //      again: return `DependencyCycle { cycle }` (owned Strings).
        //   3. Look up `(key, task)` with `self.tasks.get_key_value(name)` (missing: `NoSuchTask`). Push `key`, not
        //      `name`, onto `in_progress`: `key` borrows from `self` and lives long enough; `name` might not.
        //   4. Visit each of `task.depends_on()` in order, then pop `in_progress` and push `key` onto `order`.
        todo!("Manifest::visit_task")
    }
}
