//! Questions you can ask a parsed manifest: which packages an environment gets, and in which order tasks run.

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
        let features: &[String] = match self.environments.get(env) {
            Some(environment) => &environment.features,
            None if env == "default" => &[],
            None => {
                return Err(ManifestError::NoSuchEnvironment {
                    name: env.to_string(),
                });
            }
        };
        let mut deps: BTreeMap<&str, &Spec> = self
            .dependencies
            .iter()
            .map(|(name, spec)| (name.as_str(), spec))
            .collect();
        for feature_name in features {
            let feature =
                self.features
                    .get(feature_name)
                    .ok_or_else(|| ManifestError::UnknownFeature {
                        environment: env.to_string(),
                        feature: feature_name.clone(),
                    })?;
            deps.extend(
                feature
                    .dependencies
                    .iter()
                    .map(|(name, spec)| (name.as_str(), spec)),
            );
        }
        Ok(deps)
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
        let mut order = Vec::new();
        let mut in_progress = Vec::new();
        self.visit_task(task, &mut in_progress, &mut order)?;
        Ok(order)
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
        if order.contains(&name) {
            return Ok(());
        }
        if let Some(start) = in_progress.iter().position(|n| *n == name) {
            let mut cycle: Vec<String> =
                in_progress[start..].iter().map(|n| n.to_string()).collect();
            cycle.push(name.to_string());
            return Err(ManifestError::DependencyCycle { cycle });
        }
        let (key, task) =
            self.tasks
                .get_key_value(name)
                .ok_or_else(|| ManifestError::NoSuchTask {
                    name: name.to_string(),
                })?;
        in_progress.push(key);
        for dependency in task.depends_on() {
            self.visit_task(dependency, in_progress, order)?;
        }
        in_progress.pop();
        order.push(key);
        Ok(())
    }
}
