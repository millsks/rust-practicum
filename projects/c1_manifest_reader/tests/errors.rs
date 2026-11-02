//! Milestone 3: invalid manifests produce a ManifestError that names the TOML key path of the problem.

use manifest_reader::{ManifestError, parse_manifest};

const WORKSPACE: &str =
    "[workspace]\nname = \"x\"\nchannels = [\"conda-forge\"]\nplatforms = [\"linux-64\"]\n";

/// Parse text that must fail, and return the error.
fn error_for(src: &str) -> ManifestError {
    match parse_manifest(src) {
        Ok(manifest) => {
            panic!("parse_manifest should have failed for:\n{src}\nbut it returned {manifest:?}")
        }
        Err(err) => err,
    }
}

/// Parse a valid workspace followed by `rest`, which must make it fail.
fn error_after_workspace(rest: &str) -> ManifestError {
    error_for(&format!("{WORKSPACE}{rest}"))
}

#[test]
fn invalid_toml_is_a_toml_error_with_a_source() {
    let err = error_for("[workspace\nname = ");
    assert!(
        matches!(err, ManifestError::Toml(_)),
        "a syntax error should be ManifestError::Toml, got {err:?}"
    );
    assert!(
        err.to_string().starts_with("invalid TOML: "),
        "Display should start with `invalid TOML: `, got: {err}"
    );
    assert!(
        std::error::Error::source(&err).is_some(),
        "source() should return the wrapped toml::de::Error for the Toml variant"
    );

    fn takes_std_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}
    fn parse_with_question_mark(src: &str) -> Result<toml::Table, ManifestError> {
        Ok(src.parse::<toml::Table>()?)
    }
    let err = parse_with_question_mark("= 1").expect_err("`= 1` is not valid TOML");
    takes_std_error(&err);
    assert!(
        matches!(err, ManifestError::Toml(_)),
        "`?` should turn toml::de::Error into ManifestError::Toml"
    );
}

#[test]
fn missing_keys_name_the_full_path() {
    let err = error_for("[dependencies]\nnumpy = \"*\"\n");
    assert!(
        matches!(&err, ManifestError::MissingKey { path } if path == "workspace"),
        "expected MissingKey {{ path: \"workspace\" }}, got {err:?}"
    );
    assert_eq!(
        err.to_string(),
        "missing key `workspace`",
        "Display for MissingKey"
    );

    for (key, src) in [
        ("name", "[workspace]\nchannels = []\nplatforms = []\n"),
        ("channels", "[workspace]\nname = \"x\"\nplatforms = []\n"),
        ("platforms", "[workspace]\nname = \"x\"\nchannels = []\n"),
    ] {
        let err = error_for(src);
        let expected = format!("workspace.{key}");
        assert!(
            matches!(&err, ManifestError::MissingKey { path } if *path == expected),
            "a workspace without `{key}` should give MissingKey {{ path: \"{expected}\" }}, got {err:?}"
        );
    }
}

#[test]
fn a_wrong_type_reports_expected_and_found() {
    let err = error_for("[workspace]\nname = 42\nchannels = []\nplatforms = []\n");
    match &err {
        ManifestError::WrongType {
            path,
            expected,
            found,
        } => {
            assert_eq!(path, "workspace.name", "path of the bad value");
            assert_eq!(*expected, "string", "workspace.name must be a string");
            assert_eq!(*found, "integer", "use toml::Value::type_str() for `found`");
        }
        other => panic!("name = 42 should be WrongType, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "wrong type for `workspace.name`: expected string, found integer",
        "Display"
    );
}

#[test]
fn array_items_are_addressed_by_index() {
    let err =
        error_for("[workspace]\nname = \"x\"\nchannels = [\"conda-forge\", 7]\nplatforms = []\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "string", found: "integer" } if path == "workspace.channels[1]"),
        "the second channel is not a string: expected WrongType at workspace.channels[1], got {err:?}"
    );
    let err = error_for("[workspace]\nname = \"x\"\nchannels = \"conda-forge\"\nplatforms = []\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "array", found: "string" } if path == "workspace.channels"),
        "channels must be an array: expected WrongType at workspace.channels, got {err:?}"
    );
}

#[test]
fn an_unknown_platform_names_its_position() {
    let err = error_for(include_str!("fixtures/unknown-platform.toml"));
    assert!(
        matches!(&err, ManifestError::UnknownPlatform { path, value } if path == "workspace.platforms[2]" && value == "linux-arm64"),
        "expected UnknownPlatform at workspace.platforms[2] with value linux-arm64, got {err:?}"
    );
    assert_eq!(
        err.to_string(),
        "unknown platform `linux-arm64` at `workspace.platforms[2]` \
         (known platforms: linux-64, linux-aarch64, osx-64, osx-arm64, win-64, noarch)",
        "Display lists the known platforms so the user can fix the typo"
    );
}

#[test]
fn a_dependency_must_be_a_string_or_a_table() {
    let err = error_after_workspace("[dependencies]\nnumpy = 2\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "string or table", found: "integer" } if path == "dependencies.numpy"),
        "numpy = 2 should be WrongType at dependencies.numpy expecting \"string or table\", got {err:?}"
    );
    let err = error_after_workspace("[dependencies]\nnumpy = { version = 2 }\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "string", .. } if path == "dependencies.numpy.version"),
        "a non-string version inside a spec table should be WrongType at dependencies.numpy.version, got {err:?}"
    );
}

#[test]
fn unknown_keys_are_rejected_with_the_allowed_list() {
    let err =
        error_after_workspace("[tasks]\ntest = { cmd = \"pytest\", depends_on = [\"build\"] }\n");
    match &err {
        ManifestError::UnknownKey { path, allowed } => {
            assert_eq!(
                path, "tasks.test.depends_on",
                "the typo `depends_on` should be named by its full path"
            );
            assert_eq!(
                *allowed,
                ["cmd", "depends-on"],
                "a task table allows exactly cmd and depends-on"
            );
        }
        other => panic!("depends_on (underscore) should be UnknownKey, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "unknown key `tasks.test.depends_on` (expected one of: cmd, depends-on)",
        "Display for UnknownKey"
    );
}

#[test]
fn unknown_keys_are_rejected_at_every_level() {
    for (rest, expected) in [
        ("[project]\nname = \"old\"\n", "project"),
        (
            "[dependencies]\nnumpy = { version = \"*\", chanel = \"conda-forge\" }\n",
            "dependencies.numpy.chanel",
        ),
        (
            "[feature.test.dependecies]\npytest = \"*\"\n",
            "feature.test.dependecies",
        ),
        (
            "[environments]\ntest = { features = [], solve_group = \"a\" }\n",
            "environments.test.solve_group",
        ),
    ] {
        let err = error_after_workspace(rest);
        assert!(
            matches!(&err, ManifestError::UnknownKey { path, .. } if path == expected),
            "expected UnknownKey at `{expected}` for:\n{rest}\ngot {err:?}"
        );
    }
}

#[test]
fn a_task_table_needs_cmd_or_depends_on() {
    let err = error_after_workspace("[tasks]\nempty = {}\n");
    assert!(
        matches!(&err, ManifestError::MissingKey { path } if path == "tasks.empty.cmd"),
        "a task table with neither cmd nor depends-on should be MissingKey at tasks.empty.cmd, got {err:?}"
    );
    let err = error_after_workspace(
        "[tasks]\nbuild = \"make\"\ntest = { cmd = \"make test\", depends-on = 3 }\n",
    );
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "string or array", found: "integer" } if path == "tasks.test.depends-on"),
        "depends-on = 3 should be WrongType at tasks.test.depends-on expecting \"string or array\", got {err:?}"
    );
    let err = error_after_workspace("[tasks]\nbuild = [\"make\"]\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "string or table", found: "array" } if path == "tasks.build"),
        "a task must be a string or a table, got {err:?}"
    );
}

#[test]
fn environments_must_be_a_list_or_a_table_with_features() {
    let err = error_after_workspace("[environments]\ntest = \"test\"\n");
    assert!(
        matches!(&err, ManifestError::WrongType { path, expected: "array or table", found: "string" } if path == "environments.test"),
        "test = \"test\" should be WrongType at environments.test expecting \"array or table\", got {err:?}"
    );
    let err = error_after_workspace("[environments]\ntest = { solve-group = \"default\" }\n");
    assert!(
        matches!(&err, ManifestError::MissingKey { path } if path == "environments.test.features"),
        "the table form requires features: expected MissingKey at environments.test.features, got {err:?}"
    );
}

#[test]
fn an_environment_cannot_include_an_undefined_feature() {
    let err = error_for(include_str!("fixtures/unknown-feature.toml"));
    assert!(
        matches!(&err, ManifestError::UnknownFeature { environment, feature } if environment == "docs" && feature == "theme"),
        "expected UnknownFeature {{ environment: \"docs\", feature: \"theme\" }}, got {err:?}"
    );
    assert_eq!(
        err.to_string(),
        "environment `docs` includes unknown feature `theme`",
        "Display for UnknownFeature"
    );
}

#[test]
fn depends_on_must_name_an_existing_task() {
    let err = error_for(include_str!("fixtures/unknown-task.toml"));
    assert!(
        matches!(&err, ManifestError::UnknownTask { path, task } if path == "tasks.test.depends-on" && task == "biuld"),
        "expected UnknownTask {{ path: \"tasks.test.depends-on\", task: \"biuld\" }}, got {err:?}"
    );
    assert_eq!(
        err.to_string(),
        "`tasks.test.depends-on` refers to unknown task `biuld`",
        "Display for UnknownTask"
    );
}

#[test]
fn feature_tasks_may_depend_on_their_own_and_default_tasks_only() {
    let ok = format!(
        "{WORKSPACE}[tasks]\nbuild = \"make\"\n[feature.test.tasks]\nunit = \"pytest\"\nall = {{ depends-on = [\"build\", \"unit\"] }}\n"
    );
    parse_manifest(&ok).unwrap_or_else(|err| {
        panic!("a feature task may depend on default tasks and tasks of its own feature, but got: {err}")
    });
    let err = error_after_workspace(
        "[feature.test.tasks]\nunit = \"pytest\"\n[feature.lint.tasks]\ncheck = { cmd = \"ruff\", depends-on = [\"unit\"] }\n",
    );
    assert!(
        matches!(&err, ManifestError::UnknownTask { path, task } if path == "feature.lint.tasks.check.depends-on" && task == "unit"),
        "a lint task cannot see the test feature's tasks: expected UnknownTask at feature.lint.tasks.check.depends-on, got {err:?}"
    );
}

#[test]
fn query_errors_display_clearly() {
    let cases = [
        (
            ManifestError::NoSuchEnvironment {
                name: "gpu".to_string(),
            },
            "no environment named `gpu`",
        ),
        (
            ManifestError::NoSuchTask {
                name: "deploy".to_string(),
            },
            "no task named `deploy`",
        ),
        (
            ManifestError::DependencyCycle {
                cycle: vec!["a".to_string(), "b".to_string(), "a".to_string()],
            },
            "dependency cycle: a -> b -> a",
        ),
    ];
    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected, "Display for {err:?}");
        assert!(
            std::error::Error::source(&err).is_none(),
            "only the Toml variant has a source, not {err:?}"
        );
    }
}
