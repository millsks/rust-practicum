"""Grade and lint the fake repository's dependency-free crate with the real Cargo toolchain."""

from pathlib import Path

import pytest

from practicum.projects import Source, get_project, grade_project, lint_project, start_project

pytestmark = pytest.mark.integration


def test_solution_passes_and_starter_fails(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    assert grade_project(fake_repo, project, source=Source.SOLUTION, extra_args=["--quiet"]) == 0
    assert grade_project(fake_repo, project, source=Source.STARTER, extra_args=["--quiet"]) != 0


def test_work_copy_is_graded_after_start(fake_repo: Path) -> None:
    project = get_project(fake_repo, "p01")
    work = start_project(fake_repo, project)
    (work / "src" / "lib.rs").write_text("/// The answer.\npub fn answer() -> u32 {\n    42\n}\n", encoding="utf-8")
    assert grade_project(fake_repo, project, extra_args=["--quiet"]) == 0


def test_lint_project_accepts_clean_solution(fake_repo: Path) -> None:
    assert lint_project(fake_repo, get_project(fake_repo, "p01")) == 0
