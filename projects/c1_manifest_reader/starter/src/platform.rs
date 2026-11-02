//! The platforms a workspace can support.

use std::fmt;
use std::str::FromStr;

/// A platform name from `workspace.platforms`, such as `linux-64`.
///
/// Pixi uses `rattler_conda_types::Platform`, which knows many more platforms; these six cover what most manifests
/// list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Platform {
    /// `linux-64`: Linux on x86-64.
    Linux64,
    /// `linux-aarch64`: Linux on 64-bit ARM.
    LinuxAarch64,
    /// `osx-64`: macOS on Intel.
    Osx64,
    /// `osx-arm64`: macOS on Apple silicon.
    OsxArm64,
    /// `win-64`: Windows on x86-64.
    Win64,
    /// `noarch`: packages that run anywhere.
    NoArch,
}

impl Platform {
    /// Every platform, in the order used by error messages.
    pub const ALL: [Platform; 6] = [
        Platform::Linux64,
        Platform::LinuxAarch64,
        Platform::Osx64,
        Platform::OsxArm64,
        Platform::Win64,
        Platform::NoArch,
    ];

    /// The name of the platform as it is written in a manifest.
    pub fn as_str(self) -> &'static str {
        // TODO(c1) milestone 1: `match self` and return the name for each variant: "linux-64", "linux-aarch64",
        // "osx-64", "osx-arm64", "win-64", "noarch". The compiler tells you if you forget one.
        todo!("Platform::as_str")
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(c1) milestone 1: write `self.as_str()` to the formatter (`f.write_str(...)`).
        let _ = f;
        todo!("Display for Platform")
    }
}

/// The error returned when a string is not one of the known platform names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePlatformError {
    /// The text that failed to parse.
    pub value: String,
}

impl fmt::Display for ParsePlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(c1) milestone 1: write "unknown platform `<value>`" (with the backticks) using `write!`.
        let _ = f;
        todo!("Display for ParsePlatformError")
    }
}

impl std::error::Error for ParsePlatformError {}

impl FromStr for Platform {
    type Err = ParsePlatformError;

    /// Parse a platform name. Names are case-sensitive, as they are in Pixi.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO(c1) milestone 1: find the platform in `Platform::ALL` whose `as_str()` equals `s` (an iterator and
        // `find` work well), or return `ParsePlatformError { value: s.to_string() }`. `Option::ok_or_else` turns the
        // `Option` from `find` into a `Result`.
        let _ = s;
        todo!("FromStr for Platform")
    }
}
