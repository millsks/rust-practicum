import subprocess
from pathlib import Path
from typing import Any

import pytest

from practicum import projects
from practicum.projects import (
    ProjectError,
    Source,
    discover_projects,
    get_project,
    grade_project,
    lint_project,
    load_project,
    resolve_source,
    stage_project,
    start_project,
)


def test_load_project_reads_fields(fake_repo: Path) -> None:
    project = load_project(fake_repo / "projects" / "p01_demo")
    assert (project.id, project.title, project.part, project.package) == ("p01", "Demo", 1, "demo")
    assert project.starter_dir.name == "starter"
    assert project.solution_dir.name == "solution"
    assert project.tests_dir.name == "tests"


def test_load_project_missing_file(tmp_path: Path) -> None:
    with pytest.raises(ProjectError, match="missing"):
        load_project(tmp_path)


def test_load_project_invalid_toml(tmp_path: Path) -> None:
    (tmp_path / "project.toml").write_text("id = ", encoding="utf-8")
    with pytest.raises(ProjectError, match="invalid TOML"):
        load_project(tmp_path)


def test_load_project_missing_key(tmp_path: Path) -> None:
    (tmp_path / "project.toml").write_text('id = "p9"\n', encoding="utf-8")
    with pytest.raises(ProjectError, match="missing key 'title'"):
        load_project(tmp_path)


def test_discover_sorts_by_id(fake_repo: Path) -> None:
    other = fake_repo / "projects" / "a_first"
    other.mkdir()
    (other / "project.toml").write_text('id = "p00"\ntitle = "Zero"\npart = 0\npackage = "z"\n', encoding="utf-8")
    assert [p.id for p in discover_projects(fake_repo)] == ["p00", "p01"]


def test_get_project_by_id_and_directory(fake_repo: Path) -> None:
    assert get_project(fake_repo, "p01").title == "Demo"
    assert get_project(fake_repo, "p01_demo").id == "p01"


def test_get_project_unknown(fake_repo: Path) -> None:
    with pytest.raises(ProjectError, match="unknown project 'p99'"):
        get_project(fake_repo, "p99")


def test_start_project_copies_starter(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    destination = start_project(fake_repo, project)
    assert destination == fake_repo / "work" / "p01_demo"
    assert "todo!()" in (destination / "src" / "lib.rs").read_text(encoding="utf-8")


def test_start_project_refuses_to_overwrite(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    start_project(fake_repo, project)
    with pytest.raises(ProjectError, match="already exists"):
        start_project(fake_repo, project)


def test_start_project_force_replaces(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    destination = start_project(fake_repo, project)
    (destination / "scratch.txt").write_text("old", encoding="utf-8")
    start_project(fake_repo, project, force=True)
    assert not (destination / "scratch.txt").exists()


def test_resolve_source_prefers_work_dir(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    assert resolve_source(fake_repo, project, None) == (Source.STARTER, project.starter_dir)
    start_project(fake_repo, project)
    assert resolve_source(fake_repo, project, None) == (Source.WORK, project.work_dir(fake_repo))
    assert resolve_source(fake_repo, project, Source.SOLUTION) == (Source.SOLUTION, project.solution_dir)


@pytest.fixture
def fake_cargo(monkeypatch: pytest.MonkeyPatch) -> list[dict[str, Any]]:
    calls: list[dict[str, Any]] = []
    codes = iter([0, 3])

    def fake_run(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
        calls.append({"command": command, **kwargs})
        return subprocess.CompletedProcess(command, next(codes))

    monkeypatch.setattr(projects.shutil, "which", lambda name: f"/bin/{name}")
    monkeypatch.setattr(projects.subprocess, "run", fake_run)
    return calls


def test_stage_project_adds_tests_without_touching_source(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    staged = stage_project(fake_repo, project, Source.SOLUTION)
    assert staged == fake_repo / "target" / "practicum" / "p01_demo-solution"
    assert (staged / "tests" / "grade.rs").is_file()
    assert (staged / "src" / "lib.rs").read_text(encoding="utf-8").count("42") == 1
    assert not (project.solution_dir / "tests").exists()
    (staged / "stale.txt").write_text("x", encoding="utf-8")
    assert not (stage_project(fake_repo, project, Source.SOLUTION) / "stale.txt").exists()


def test_stage_project_requires_cargo_toml(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    (project.starter_dir / "Cargo.toml").unlink()
    with pytest.raises(ProjectError, match=r"no Cargo\.toml"):
        stage_project(fake_repo, project, Source.STARTER)


def test_grade_project_runs_cargo_test(fake_repo: Path, fake_cargo: list[dict[str, Any]]) -> None:
    project = get_project(fake_repo, "p01")
    assert grade_project(fake_repo, project, source=Source.SOLUTION, extra_args=["--quiet"]) == 0
    call = fake_cargo[0]
    staged = fake_repo / "target" / "practicum" / "p01_demo-solution"
    assert call["command"] == ["/bin/cargo", "test", "--manifest-path", str(staged / "Cargo.toml"), "--quiet"]
    assert call["cwd"] == fake_repo
    assert call["env"]["CARGO_TARGET_DIR"] == str(fake_repo / "target" / "practicum" / "build" / "p01_demo-solution")


def test_grade_project_defaults_to_starter(fake_repo: Path, fake_cargo: list[dict[str, Any]]) -> None:
    grade_project(fake_repo, get_project(fake_repo, "p01"))
    assert "p01_demo-starter" in fake_cargo[0]["command"][3]


def test_lint_project_runs_fmt_then_clippy(fake_repo: Path, fake_cargo: list[dict[str, Any]]) -> None:
    assert lint_project(fake_repo, get_project(fake_repo, "p01")) == 3
    assert [c["command"][1] for c in fake_cargo] == ["fmt", "clippy"]
    assert fake_cargo[1]["command"][-3:] == ["--", "-D", "warnings"]


def test_lint_project_stops_at_first_failure(fake_repo: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr(projects.shutil, "which", lambda name: name)

    def fail(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
        calls.append(command)
        return subprocess.CompletedProcess(command, 1)

    monkeypatch.setattr(projects.subprocess, "run", fail)
    assert lint_project(fake_repo, get_project(fake_repo, "p01")) == 1
    assert len(calls) == 1


def test_missing_cargo(fake_repo: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(projects.shutil, "which", lambda name: None)
    with pytest.raises(ProjectError, match="cargo not found"):
        grade_project(fake_repo, get_project(fake_repo, "p01"))
