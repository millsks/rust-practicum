//! Milestone 5: run order for tasks and their depends-on, with cycle detection.

use manifest_reader::{Manifest, ManifestError, parse_manifest};

fn load(src: &str, name: &str) -> Manifest {
    parse_manifest(src).unwrap_or_else(|err| {
        panic!("tests/fixtures/{name} is a valid manifest, but parsing failed: {err}")
    })
}

fn order(m: &Manifest, task: &str) -> Vec<String> {
    match m.task_order(task) {
        Ok(order) => order.into_iter().map(str::to_string).collect(),
        Err(err) => panic!("task_order({task:?}) should succeed, got {err}"),
    }
}

#[test]
fn dependencies_run_first() {
    let m = load(include_str!("fixtures/full.toml"), "full.toml");
    assert_eq!(
        order(&m, "fetch"),
        ["fetch"],
        "fetch depends on nothing, so it runs alone"
    );
    assert_eq!(
        order(&m, "plot"),
        ["fetch", "clean", "plot"],
        "plot -> clean -> fetch, so fetch runs first"
    );
    assert_eq!(
        order(&m, "report"),
        ["fetch", "clean", "plot", "report"],
        "report depends on plot and clean; clean is already done when report's second dependency is visited"
    );
}

#[test]
fn an_alias_runs_its_dependencies_and_itself_once_each() {
    let m = load(include_str!("fixtures/full.toml"), "full.toml");
    assert_eq!(
        order(&m, "all"),
        ["fetch", "clean", "plot", "report", "all"],
        "every task appears once, even though fetch is reachable three ways; the alias itself comes last"
    );
}

#[test]
fn a_diamond_runs_the_shared_task_once() {
    let m = load(include_str!("fixtures/tasks.toml"), "tasks.toml");
    assert_eq!(
        order(&m, "top"),
        ["base", "left", "right", "top"],
        "base is shared by left and right"
    );
}

#[test]
fn depends_on_order_decides_ties() {
    let m = load(include_str!("fixtures/tasks.toml"), "tasks.toml");
    assert_eq!(
        order(&m, "top-reversed"),
        ["base", "right", "left", "top-reversed"],
        "visit depends-on in the order written (right before left), not alphabetically"
    );
    assert_eq!(
        order(&m, "everything"),
        ["base", "left", "right", "top", "everything"],
        "tasks already scheduled are skipped when they come up again"
    );
}

#[test]
fn the_order_borrows_from_the_manifest_not_the_argument() {
    let m = load(include_str!("fixtures/full.toml"), "full.toml");
    let result = {
        let name = String::from("clean");
        m.task_order(&name).expect("clean is a task")
    }; // `name` is dropped here; the &str values in `result` must point into `m`.
    assert_eq!(
        result,
        ["fetch", "clean"],
        "task_order returns names borrowed from the manifest"
    );
}

#[test]
fn an_unknown_task_is_an_error() {
    let m = load(include_str!("fixtures/full.toml"), "full.toml");
    let err = m
        .task_order("deploy")
        .expect_err("full.toml has no deploy task");
    assert!(
        matches!(&err, ManifestError::NoSuchTask { name } if name == "deploy"),
        "expected NoSuchTask {{ name: \"deploy\" }}, got {err:?}"
    );
    let err = m
        .task_order("test")
        .expect_err("`test` is a feature task, and task_order only reads [tasks]");
    assert!(
        matches!(&err, ManifestError::NoSuchTask { .. }),
        "feature tasks are not default tasks; got {err:?}"
    );
}

#[test]
fn a_cycle_is_reported_with_the_tasks_in_it() {
    let m = load(include_str!("fixtures/cycle.toml"), "cycle.toml");
    let err = m.task_order("a").expect_err("a -> b -> c -> a is a cycle");
    match &err {
        ManifestError::DependencyCycle { cycle } => {
            assert_eq!(
                cycle,
                &["a", "b", "c", "a"],
                "the cycle starts and ends with the repeated task"
            );
        }
        other => panic!("expected DependencyCycle, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "dependency cycle: a -> b -> c -> a",
        "Display for DependencyCycle"
    );
}

#[test]
fn a_cycle_reached_from_outside_starts_where_the_loop_starts() {
    let m = load(include_str!("fixtures/cycle.toml"), "cycle.toml");
    let err = m
        .task_order("start")
        .expect_err("start -> b -> c -> a -> b loops");
    assert!(
        matches!(&err, ManifestError::DependencyCycle { cycle } if cycle == &["b", "c", "a", "b"]),
        "start is not part of the loop, so the cycle is [b, c, a, b]; got {err:?}"
    );
    let err = m
        .task_order("selfish")
        .expect_err("selfish depends on itself");
    assert!(
        matches!(&err, ManifestError::DependencyCycle { cycle } if cycle == &["selfish", "selfish"]),
        "a task that depends on itself is a cycle of one: [selfish, selfish]; got {err:?}"
    );
    assert_eq!(
        order(&m, "fine"),
        ["fine"],
        "a cycle elsewhere in the manifest does not affect other tasks"
    );
}
