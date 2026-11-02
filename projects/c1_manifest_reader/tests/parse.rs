//! Milestones 1 and 2: platforms, then turning a valid manifest into typed structs.

use manifest_reader::{DetailedSpec, Environment, Manifest, Platform, Spec, Task, parse_manifest};

fn full() -> Manifest {
    parse_manifest(include_str!("fixtures/full.toml")).unwrap_or_else(|err| {
        panic!("tests/fixtures/full.toml is a valid manifest, but parse_manifest failed: {err}")
    })
}

fn minimal() -> Manifest {
    parse_manifest(include_str!("fixtures/minimal.toml")).unwrap_or_else(|err| {
        panic!("tests/fixtures/minimal.toml is a valid manifest, but parse_manifest failed: {err}")
    })
}

#[test]
fn platform_names_round_trip_through_from_str_and_display() {
    let names = [
        "linux-64",
        "linux-aarch64",
        "osx-64",
        "osx-arm64",
        "win-64",
        "noarch",
    ];
    for (platform, name) in Platform::ALL.into_iter().zip(names) {
        assert_eq!(
            platform.as_str(),
            name,
            "Platform::{platform:?}.as_str() should be the manifest name"
        );
        assert_eq!(
            platform.to_string(),
            name,
            "Display for Platform::{platform:?} should write the manifest name"
        );
        assert_eq!(
            name.parse::<Platform>(),
            Ok(platform),
            "\"{name}\".parse::<Platform>() should succeed"
        );
    }
}

#[test]
fn platform_from_str_rejects_unknown_names() {
    for bad in ["linux-arm64", "Linux-64", "osx", ""] {
        let err = bad
            .parse::<Platform>()
            .expect_err("only the six exact, lowercase names are platforms");
        assert_eq!(
            err.value, bad,
            "ParsePlatformError.value should hold the text that failed to parse"
        );
        assert_eq!(
            err.to_string(),
            format!("unknown platform `{bad}`"),
            "Display for ParsePlatformError"
        );
    }
}

#[test]
fn reads_the_workspace_table() {
    let ws = full().workspace;
    assert_eq!(ws.name, "weather-station", "workspace.name");
    assert_eq!(
        ws.version.as_deref(),
        Some("0.3.0"),
        "workspace.version is optional, but this fixture sets it"
    );
    assert_eq!(
        ws.description.as_deref(),
        Some("Collect and plot readings from a backyard weather station"),
        "workspace.description"
    );
    assert_eq!(
        ws.channels,
        ["conda-forge", "bioconda"],
        "channels keep the order they are written in"
    );
    assert_eq!(
        ws.platforms,
        [Platform::Linux64, Platform::OsxArm64, Platform::Win64],
        "platforms are parsed into Platform values, in manifest order"
    );
}

#[test]
fn a_minimal_manifest_has_empty_optional_parts() {
    let m = minimal();
    assert_eq!(m.workspace.name, "empty", "workspace.name");
    assert_eq!(
        m.workspace.version, None,
        "a missing workspace.version is None, not an error"
    );
    assert!(
        m.workspace.channels.is_empty(),
        "channels = [] is a valid, empty list"
    );
    assert!(
        m.dependencies.is_empty(),
        "no [dependencies] table means no dependencies"
    );
    assert!(m.tasks.is_empty(), "no [tasks] table means no tasks");
    assert!(
        m.features.is_empty(),
        "no [feature] tables means no features"
    );
    assert!(
        m.environments.is_empty(),
        "no [environments] table means no explicit environments"
    );
}

#[test]
fn dependencies_accept_the_string_and_table_forms() {
    let m = full();
    assert_eq!(
        m.dependencies.len(),
        3,
        "full.toml has three default dependencies: python, numpy, pandas"
    );
    assert_eq!(
        m.dependencies.get("numpy"),
        Some(&Spec::Version(">=2,<3".to_string())),
        "numpy = \">=2,<3\" should become Spec::Version"
    );
    let pandas = m
        .dependencies
        .get("pandas")
        .expect("pandas is a default dependency in full.toml");
    let expected = DetailedSpec {
        version: Some(">=2.2".to_string()),
        build: None,
        channel: Some("conda-forge".to_string()),
    };
    assert_eq!(
        pandas,
        &Spec::Detailed(expected),
        "pandas = {{ version, channel }} should become Spec::Detailed"
    );
    assert_eq!(
        pandas.version(),
        Some(">=2.2"),
        "Spec::version() reads the version from either form"
    );
    assert_eq!(
        pandas.channel(),
        Some("conda-forge"),
        "Spec::channel() reads the channel of the table form"
    );
    assert_eq!(
        m.dependencies["numpy"].channel(),
        None,
        "a bare version string has no channel"
    );
}

#[test]
fn spec_display_shows_version_build_and_channel() {
    let cases = [
        (Spec::Version(">=2".to_string()), ">=2"),
        (Spec::Detailed(DetailedSpec::default()), "*"),
        (
            Spec::Detailed(DetailedSpec {
                version: Some("3.12.*".to_string()),
                build: Some("*_cpython".to_string()),
                channel: None,
            }),
            "3.12.* *_cpython",
        ),
        (
            Spec::Detailed(DetailedSpec {
                version: None,
                build: Some("py312*".to_string()),
                channel: Some("conda-forge".to_string()),
            }),
            "* py312* (conda-forge)",
        ),
    ];
    for (spec, expected) in cases {
        assert_eq!(spec.to_string(), expected, "Display for {spec:?}");
    }
}

#[test]
fn tasks_come_in_three_forms() {
    let m = full();
    assert_eq!(
        m.tasks["fetch"],
        Task::Command("python -m station.fetch".to_string()),
        "a string task is Task::Command"
    );
    assert_eq!(
        m.tasks["plot"],
        Task::Execute {
            cmd: "python -m station.plot".to_string(),
            depends_on: vec!["clean".to_string()]
        },
        "a table with cmd is Task::Execute"
    );
    assert_eq!(
        m.tasks["clean"],
        Task::Execute {
            cmd: "python -m station.clean".to_string(),
            depends_on: vec!["fetch".to_string()]
        },
        "depends-on = \"fetch\" (one string) is shorthand for depends-on = [\"fetch\"]"
    );
    assert_eq!(
        m.tasks["all"],
        Task::Alias {
            depends_on: vec!["report".to_string(), "fetch".to_string()]
        },
        "a table with depends-on but no cmd is Task::Alias, and depends-on keeps its order"
    );
}

#[test]
fn task_accessors_work_for_every_form() {
    let m = full();
    assert_eq!(
        m.tasks["fetch"].command(),
        Some("python -m station.fetch"),
        "command() of a Command"
    );
    assert_eq!(
        m.tasks["report"].command(),
        Some("python -m station.report"),
        "command() of an Execute"
    );
    assert_eq!(m.tasks["all"].command(), None, "an alias has no command");
    assert!(
        m.tasks["fetch"].depends_on().is_empty(),
        "a Command has no dependencies"
    );
    assert_eq!(
        m.tasks["report"].depends_on(),
        ["plot", "clean"],
        "depends_on() of an Execute"
    );
    assert_eq!(
        m.tasks["all"].depends_on(),
        ["report", "fetch"],
        "depends_on() of an Alias"
    );
}

#[test]
fn features_hold_dependencies_and_tasks() {
    let m = full();
    let names: Vec<&str> = m.features.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        ["lint", "py312", "test"],
        "full.toml defines three features (sorted by name)"
    );
    let test = &m.features["test"];
    assert_eq!(
        test.dependencies.get("pytest"),
        Some(&Spec::Version(">=8".to_string())),
        "feature.test.dependencies"
    );
    assert_eq!(
        test.tasks["test"].depends_on(),
        ["fetch"],
        "feature.test.tasks.test depends on the default fetch task"
    );
    assert!(
        m.features["lint"].tasks.is_empty(),
        "a feature without a tasks table has no tasks"
    );
}

#[test]
fn environments_accept_the_list_and_table_forms() {
    let m = full();
    assert_eq!(
        m.environments["test"],
        Environment {
            features: vec!["test".to_string()],
            solve_group: None
        },
        "test = [\"test\"] is the list shorthand"
    );
    assert_eq!(
        m.environments["lint"],
        Environment {
            features: vec!["lint".to_string()],
            solve_group: Some("default".to_string())
        },
        "lint = {{ features = [...], solve-group = \"default\" }} is the table form"
    );
    assert_eq!(
        m.environments["ci"].features,
        ["test", "lint", "py312"],
        "features keep the order they are written in"
    );
}

#[test]
fn environment_names_include_the_implicit_default() {
    assert_eq!(
        full().environment_names(),
        ["ci", "default", "lint", "test"],
        "sorted, with `default` added"
    );
    assert_eq!(
        minimal().environment_names(),
        ["default"],
        "every manifest has the default environment"
    );
    let explicit = parse_manifest(
        "[workspace]\nname = \"x\"\nchannels = []\nplatforms = []\n[environments]\ndefault = []\n",
    )
    .expect("an explicit default environment is allowed");
    assert_eq!(
        explicit.environment_names(),
        ["default"],
        "an explicit `default` must not appear twice"
    );
}

#[test]
fn summary_lists_each_part_of_the_manifest() {
    let expected = "\
workspace: weather-station
channels: conda-forge, bioconda
platforms: linux-64, osx-arm64, win-64
dependencies: numpy, pandas, python
tasks: all, clean, fetch, plot, report
features: lint, py312, test
environments: ci, default, lint, test";
    assert_eq!(
        full().summary(),
        expected,
        "summary() of full.toml (see the README for the format)"
    );

    let expected = "\
workspace: empty
channels: (none)
platforms: noarch
dependencies: (none)
tasks: (none)
features: (none)
environments: default";
    assert_eq!(
        minimal().summary(),
        expected,
        "empty lists are written as (none)"
    );
}
