"""Discover, start, grade, and lint the curriculum's Cargo capstone projects.

A project directory holds ``project.toml``, a ``starter/`` crate, a ``solution/`` crate, and ``tests/`` (Cargo
integration tests). Grading copies the chosen crate into ``target/practicum/`` and adds the tests, so the learner's
copy in ``work/`` never contains the grading tests and cannot accidentally edit them.
"""

import os
import shutil
import subprocess
import tomllib
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path

_IGNORED = shutil.ignore_patterns("target", ".git", "*.orig", "*.rej")


class ProjectError(RuntimeError):
    """Raised for an unknown project, an invalid project definition, or a missing Cargo toolchain."""


class Source(StrEnum):
    """Which copy of a project's crate a grading run builds."""

    WORK = "work"
    STARTER = "starter"
    SOLUTION = "solution"


@dataclass(frozen=True, slots=True)
class Project:
    """A Cargo capstone defined by a ``project.toml`` file.

    Attributes:
        id: Short identifier such as ``c1``.
        title: Human-readable title.
        part: Curriculum part the project closes.
        package: Cargo package name of the crate the learner implements.
        path: Directory holding the project definition.
    """

    id: str
    title: str
    part: int
    package: str
    path: Path

    @property
    def starter_dir(self) -> Path:
        """Directory containing the starter crate."""
        return self.path / "starter"

    @property
    def solution_dir(self) -> Path:
        """Directory containing the reference solution crate."""
        return self.path / "solution"

    @property
    def tests_dir(self) -> Path:
        """Directory containing the grading integration tests."""
        return self.path / "tests"

    def work_dir(self, root: Path) -> Path:
        """Return the learner's working copy location.

        Args:
            root: Repository root.

        Returns:
            The path ``work/<project directory name>`` under ``root``.
        """
        return root / "work" / self.path.name


def load_project(project_dir: Path) -> Project:
    """Load a project definition from ``project_dir/project.toml``.

    Args:
        project_dir: Directory containing ``project.toml``.

    Returns:
        The parsed project.

    Raises:
        ProjectError: If the file is missing or lacks required keys.
    """
    config_path = project_dir / "project.toml"
    try:
        data = tomllib.loads(config_path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ProjectError(f"missing {config_path}") from exc
    except tomllib.TOMLDecodeError as exc:
        raise ProjectError(f"invalid TOML in {config_path}: {exc}") from exc
    try:
        return Project(
            id=str(data["id"]),
            title=str(data["title"]),
            part=int(data["part"]),
            package=str(data["package"]),
            path=project_dir,
        )
    except KeyError as exc:
        raise ProjectError(f"{config_path} is missing key {exc.args[0]!r}") from exc


def discover_projects(root: Path) -> list[Project]:
    """Return every project under ``root/projects`` sorted by id.

    Args:
        root: Repository root.

    Returns:
        Projects ordered by identifier.
    """
    found = [load_project(p.parent) for p in (root / "projects").glob("*/project.toml")]
    return sorted(found, key=lambda p: p.id)


def get_project(root: Path, project_id: str) -> Project:
    """Look up a project by id or directory name.

    Args:
        root: Repository root.
        project_id: Identifier such as ``c1`` or a directory name such as ``c1_manifest_reader``.

    Returns:
        The matching project.

    Raises:
        ProjectError: If no project matches.
    """
    for project in discover_projects(root):
        if project_id in (project.id, project.path.name):
            return project
    raise ProjectError(f"unknown project {project_id!r}; run `pixi run projects` to list them")


def start_project(root: Path, project: Project, *, force: bool = False) -> Path:
    """Copy a project's starter crate into the learner's ``work/`` directory.

    Args:
        root: Repository root.
        project: Project to start.
        force: Replace an existing working copy.

    Returns:
        The working copy directory.

    Raises:
        ProjectError: If a working copy exists and ``force`` is false.
    """
    destination = project.work_dir(root)
    if destination.exists():
        if not force:
            raise ProjectError(f"{destination} already exists; pass --force to overwrite your work")
        shutil.rmtree(destination)
    shutil.copytree(project.starter_dir, destination, ignore=_IGNORED)
    return destination


def resolve_source(root: Path, project: Project, source: Source | None) -> tuple[Source, Path]:
    """Pick which crate a grading run builds.

    With no explicit choice the learner's working copy is preferred, falling back to the starter.

    Args:
        root: Repository root.
        project: Project being graded.
        source: Explicit source, or ``None`` to choose automatically.

    Returns:
        The chosen source and its directory.
    """
    if source is None:
        source = Source.WORK if project.work_dir(root).is_dir() else Source.STARTER
    directories = {
        Source.WORK: project.work_dir(root),
        Source.STARTER: project.starter_dir,
        Source.SOLUTION: project.solution_dir,
    }
    return source, directories[source]


def find_cargo() -> str:
    """Locate ``cargo``.

    Returns:
        The path to ``cargo``.

    Raises:
        ProjectError: If it is not on ``PATH``; run through ``pixi run`` so the pinned toolchain is active.
    """
    cargo = shutil.which("cargo")
    if cargo is None:
        raise ProjectError("cargo not found; run this through `pixi run` so the course toolchain is on PATH")
    return cargo


def stage_project(root: Path, project: Project, source: Source) -> Path:
    """Copy a crate plus the grading tests into ``target/practicum/<project>-<source>/``.

    Args:
        root: Repository root.
        project: Project to stage.
        source: Which crate to copy.

    Returns:
        The staged crate directory, ready for ``cargo test``.

    Raises:
        ProjectError: If the chosen crate has no ``Cargo.toml``.
    """
    _, code_dir = resolve_source(root, project, source)
    if not (code_dir / "Cargo.toml").is_file():
        raise ProjectError(f"{code_dir} has no Cargo.toml")
    staged = root / "target" / "practicum" / f"{project.path.name}-{source.value}"
    if staged.exists():
        shutil.rmtree(staged)
    # Fresh mtimes (shutil.copy, not copy2) so Cargo never mistakes a new copy for an up-to-date build.
    shutil.copytree(code_dir, staged, ignore=_IGNORED, copy_function=shutil.copy)
    if project.tests_dir.is_dir():
        shutil.copytree(project.tests_dir, staged / "tests", dirs_exist_ok=True, copy_function=shutil.copy)
    return staged


def _cargo_env(staged: Path) -> dict[str, str]:
    # One target directory per staged copy: starter, solution, and work share a package name, and Cargo would
    # otherwise reuse one copy's build artifacts for another.
    return {**os.environ, "CARGO_TARGET_DIR": str(staged.parent / "build" / staged.name)}


def grade_project(
    root: Path, project: Project, *, source: Source | None = None, extra_args: list[str] | None = None
) -> int:
    """Run a project's grading tests against the chosen crate with ``cargo test``.

    Args:
        root: Repository root.
        project: Project to grade.
        source: Which crate to build; see :func:`resolve_source`.
        extra_args: Additional arguments forwarded to ``cargo test``.

    Returns:
        The cargo exit code (0 means every test passed).
    """
    chosen, _ = resolve_source(root, project, source)
    staged = stage_project(root, project, chosen)
    command = [find_cargo(), "test", "--manifest-path", str(staged / "Cargo.toml"), *(extra_args or [])]
    return subprocess.run(command, cwd=root, env=_cargo_env(staged), check=False).returncode


def lint_project(root: Path, project: Project) -> int:
    """Check that a project's solution is formatted and clippy-clean, tests included.

    Args:
        root: Repository root.
        project: Project to lint.

    Returns:
        0 when ``cargo fmt --check`` and ``cargo clippy -D warnings`` both pass, otherwise the first failing code.
    """
    staged = stage_project(root, project, Source.SOLUTION)
    manifest = str(staged / "Cargo.toml")
    cargo = find_cargo()
    env = _cargo_env(staged)
    for command in (
        [cargo, "fmt", "--manifest-path", manifest, "--check"],
        [cargo, "clippy", "--manifest-path", manifest, "--all-targets", "--quiet", "--", "-D", "warnings"],
    ):
        code = subprocess.run(command, cwd=root, env=env, check=False).returncode
        if code != 0:
            return code
    return 0
