//! Milestone 4: the dependencies each environment installs.

use std::collections::BTreeMap;

use manifest_reader::{Environment, Manifest, ManifestError, Spec, parse_manifest};

fn full() -> Manifest {
    parse_manifest(include_str!("fixtures/full.toml")).unwrap_or_else(|err| {
        panic!("tests/fixtures/full.toml is a valid manifest, but parse_manifest failed: {err}")
    })
}

/// Turn the resolved map into `name -> spec text` so failures print readably.
fn rendered(deps: &BTreeMap<&str, &Spec>) -> Vec<String> {
    deps.iter()
        .map(|(name, spec)| format!("{name} {spec}"))
        .collect()
}

#[test]
fn the_implicit_default_environment_has_only_the_default_dependencies() {
    let m = full();
    let deps = m
        .environment_dependencies("default")
        .expect("`default` exists even when [environments] omits it");
    assert_eq!(
        rendered(&deps),
        [
            "numpy >=2,<3",
            "pandas >=2.2 (conda-forge)",
            "python 3.13.*"
        ],
        "default = the [dependencies] table and nothing else"
    );
}

#[test]
fn a_feature_adds_and_overrides_dependencies() {
    let m = full();
    let deps = m
        .environment_dependencies("test")
        .expect("full.toml defines a test environment");
    assert_eq!(
        rendered(&deps),
        [
            "numpy 2.1.*",
            "pandas >=2.2 (conda-forge)",
            "pytest >=8",
            "python 3.13.*"
        ],
        "the test feature adds pytest and replaces the default numpy spec with its own"
    );
}

#[test]
fn later_features_override_earlier_ones() {
    let m = full();
    let deps = m
        .environment_dependencies("ci")
        .expect("full.toml defines a ci environment");
    assert_eq!(
        rendered(&deps),
        [
            "numpy 2.1.*",
            "pandas >=2.2 (conda-forge)",
            "pytest >=8",
            "python 3.12.* *_cpython",
            "ruff *",
        ],
        "ci = [test, lint, py312]: defaults, then each feature in order; py312 replaces python"
    );
}

#[test]
fn resolved_specs_are_borrowed_from_the_manifest() {
    let m = full();
    let deps = m
        .environment_dependencies("lint")
        .expect("full.toml defines a lint environment");
    assert!(
        std::ptr::eq(deps["pandas"], &m.dependencies["pandas"]),
        "environment_dependencies should return references into the manifest, not clones"
    );
    assert!(
        std::ptr::eq(deps["ruff"], &m.features["lint"].dependencies["ruff"]),
        "a feature's spec should be a reference into that feature"
    );
}

#[test]
fn an_explicit_default_environment_replaces_the_implicit_one() {
    let src = "\
[workspace]
name = \"x\"
channels = []
platforms = []
[dependencies]
python = \"3.13.*\"
[feature.extra.dependencies]
rich = \"*\"
[environments]
default = [\"extra\"]
";
    let m = parse_manifest(src)
        .unwrap_or_else(|err| panic!("a default environment may be defined: {err}"));
    let deps = m
        .environment_dependencies("default")
        .expect("default is defined");
    assert_eq!(
        rendered(&deps),
        ["python 3.13.*", "rich *"],
        "an explicit default environment includes its features"
    );
}

#[test]
fn an_unknown_environment_is_an_error() {
    let m = full();
    let err = m
        .environment_dependencies("gpu")
        .expect_err("full.toml has no gpu environment");
    assert!(
        matches!(&err, ManifestError::NoSuchEnvironment { name } if name == "gpu"),
        "expected NoSuchEnvironment {{ name: \"gpu\" }}, got {err:?}"
    );
}

#[test]
fn a_hand_built_environment_with_a_missing_feature_is_an_error() {
    let mut m = full();
    m.environments.insert(
        "broken".to_string(),
        Environment {
            features: vec!["test".to_string(), "ghost".to_string()],
            solve_group: None,
        },
    );
    let err = m
        .environment_dependencies("broken")
        .expect_err("the ghost feature does not exist");
    assert!(
        matches!(&err, ManifestError::UnknownFeature { environment, feature } if environment == "broken" && feature == "ghost"),
        "fields of Manifest are public, so resolution must check features too; got {err:?}"
    );
}
