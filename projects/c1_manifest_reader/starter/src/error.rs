//! The error type for everything that can go wrong while reading a manifest.

use std::fmt;

use crate::platform::Platform;

/// Everything that can go wrong while reading or querying a manifest.
///
/// Variants that come from the manifest text carry the TOML key path of the problem, written the way you would
/// look it up: `tasks.test.depends-on`, `workspace.platforms[1]`. Pixi's `TomlError` plays this role; it is built
/// with `thiserror` and `miette` (Level 2), which generate the same `Display` and `Error` impls written here by hand.
#[derive(Debug)]
pub enum ManifestError {
    /// The text is not valid TOML.
    Toml(toml::de::Error),
    /// A required key is absent, for example `workspace.name`.
    MissingKey {
        /// Path of the missing key.
        path: String,
    },
    /// A key holds the wrong kind of value, for example a number where a string belongs.
    WrongType {
        /// Path of the value.
        path: String,
        /// What the key should hold, for example `"string"` or `"string or table"`.
        expected: &'static str,
        /// The TOML type that was found, as returned by [`toml::Value::type_str`].
        found: &'static str,
    },
    /// A table has a key this format does not define (often a typo such as `depends_on`).
    UnknownKey {
        /// Path of the unexpected key.
        path: String,
        /// The keys that table accepts.
        allowed: &'static [&'static str],
    },
    /// A platform name that is not one of [`Platform::ALL`].
    UnknownPlatform {
        /// Path of the value, for example `workspace.platforms[2]`.
        path: String,
        /// The text that was not recognized.
        value: String,
    },
    /// An environment includes a feature that no `[feature.<name>]` table defines.
    UnknownFeature {
        /// Name of the environment.
        environment: String,
        /// Name of the missing feature.
        feature: String,
    },
    /// A `depends-on` entry names a task that does not exist.
    UnknownTask {
        /// Path of the `depends-on` key, for example `tasks.test.depends-on`.
        path: String,
        /// The task that was named.
        task: String,
    },
    /// A query asked for an environment the manifest does not define.
    NoSuchEnvironment {
        /// The requested name.
        name: String,
    },
    /// A query asked for a task the manifest does not define.
    NoSuchTask {
        /// The requested name.
        name: String,
    },
    /// Tasks depend on each other in a loop. The cycle starts and ends with the same task: `[a, b, a]`.
    DependencyCycle {
        /// The tasks in the cycle, in dependency order.
        cycle: Vec<String>,
    },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(c1) milestone 2: `match self` with one arm per variant and `write!` the message from the README:
        //   Toml(err)          invalid TOML: <err>
        //   MissingKey         missing key `<path>`
        //   WrongType          wrong type for `<path>`: expected <expected>, found <found>
        //   UnknownKey         unknown key `<path>` (expected one of: <allowed joined with ", ">)
        //   UnknownPlatform    unknown platform `<value>` at `<path>` (known platforms: <Platform::ALL joined with ", ">)
        //   UnknownFeature     environment `<environment>` includes unknown feature `<feature>`
        //   UnknownTask        `<path>` refers to unknown task `<task>`
        //   NoSuchEnvironment  no environment named `<name>`
        //   NoSuchTask         no task named `<name>`
        //   DependencyCycle    dependency cycle: <cycle joined with " -> ">
        let _ = (f, Platform::ALL);
        todo!("Display for ManifestError")
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // TODO(c1) milestone 2: the `Toml` variant wraps another error; return `Some(err)` for it and `None` for every
        // other variant, so callers can walk the chain of causes.
        todo!("Error::source for ManifestError")
    }
}

impl From<toml::de::Error> for ManifestError {
    fn from(err: toml::de::Error) -> Self {
        // TODO(c1) milestone 2: wrap the error in the `Toml` variant. This impl is what lets `?` convert a
        // `toml::de::Error` into a `ManifestError` inside `parse_manifest`.
        let _ = err;
        todo!("From<toml::de::Error> for ManifestError")
    }
}
