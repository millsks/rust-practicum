//! Read a simplified `pixi.toml` into typed Rust structs.
//!
//! The `toml` crate turns the text into an untyped [`toml::Table`]: a map from keys to [`toml::Value`]s that could be
//! anything. This crate converts that table by hand into [`Manifest`], checking every key and type on the way and
//! reporting problems as a [`ManifestError`] that names the exact TOML key path. Pixi does the same job in its
//! `pixi_manifest` crate (with `toml_span` instead of `toml`, so its errors can also point at line and column).
//!
//! Work through the `TODO(c1)` comments in the order of the milestones in README.md:
//! `platform.rs`, `model.rs`, `error.rs`, `convert.rs`, `resolve.rs`, then `main.rs`.
//!
//! ```
//! let manifest = manifest_reader::parse_manifest(
//!     r#"
//!     [workspace]
//!     name = "demo"
//!     channels = ["conda-forge"]
//!     platforms = ["linux-64"]
//!
//!     [tasks]
//!     build = "cargo build"
//!     test = { cmd = "cargo test", depends-on = ["build"] }
//!     "#,
//! )
//! .unwrap();
//! assert_eq!(manifest.task_order("test").unwrap(), ["build", "test"]);
//! ```

mod convert;
mod error;
mod model;
mod platform;
mod resolve;

pub use error::ManifestError;
pub use model::{DetailedSpec, Environment, Feature, Manifest, Spec, Task, Workspace};
pub use platform::{ParsePlatformError, Platform};

/// Parse the text of a manifest into a validated [`Manifest`].
///
/// Parsing happens in three steps: the `toml` crate checks the syntax and builds a [`toml::Table`], the table is
/// converted into typed structs (every key and type is checked), and finally the cross references are validated:
/// every feature an environment includes must exist, and every `depends-on` entry must name a task.
///
/// # Errors
///
/// Returns [`ManifestError::Toml`] for a syntax error, and one of the other variants for a manifest that is valid
/// TOML but not a valid manifest.
pub fn parse_manifest(src: &str) -> Result<Manifest, ManifestError> {
    // TODO(c1) milestone 3: parse `src` into a `toml::Table` with `src.parse()`. Its error type is
    // `toml::de::Error`; once `From<toml::de::Error> for ManifestError` exists (milestone 2), `?` converts it.
    // Then hand the table to `convert::manifest_from_table`.
    let _ = src;
    todo!("parse_manifest")
}
