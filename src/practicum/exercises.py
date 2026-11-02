"""Verify that each in-page exercise's reference solution passes its own tests with a local ``rustc``."""

import re
import shutil
import subprocess
import tempfile
import textwrap
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path

TEST_MODULE = "pp_checks"
EDITION = "2024"
_TEST_LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored)", re.M)
_FAILURE_BLOCK = re.compile(r"^---- (\S+) stdout ----\n(.*?)(?=^---- |^failures:|^test result:)", re.M | re.S)
_PANIC_LINE = re.compile(r"^thread '.*'( \(\d+\))? panicked at ")


@dataclass(frozen=True, slots=True)
class Exercise:
    """An exercise extracted from a page.

    Attributes:
        id: The exercise ``data-id``.
        solution: Reference solution source, or an empty string when the page has none.
        tests: Test functions (``#[test] fn ...``) that run against the solution.
    """

    id: str
    solution: str
    tests: str


@dataclass(frozen=True, slots=True)
class CheckOutcome:
    """One libtest result.

    Attributes:
        name: Test function name without the wrapper module path.
        ok: Whether it passed.
        message: The panic message for a failure; empty otherwise.
    """

    name: str
    ok: bool
    message: str = ""


@dataclass(frozen=True, slots=True)
class ExerciseResult:
    """Outcome of checking one exercise.

    Attributes:
        id: The exercise ``data-id``.
        ok: Whether the solution passed every test.
        detail: Failure details; empty when ``ok``.
    """

    id: str
    ok: bool
    detail: str = ""


class RustToolchainError(RuntimeError):
    """Raised when ``rustc`` is not available on ``PATH``."""


def wrap_tests(code: str, tests: str) -> str:
    """Append the tests to the code inside a ``#[cfg(test)]`` module, exactly as the browser does.

    Args:
        code: The learner's (or reference) code.
        tests: Test functions to place in the module.

    Returns:
        A single Rust source file.
    """
    body = "\n".join("    " + line if line.strip() else line for line in tests.split("\n"))
    return f"{code.rstrip()}\n\n#[cfg(test)]\n#[allow(unused_imports)]\nmod {TEST_MODULE} {{\n    use super::*;\n\n{body}\n}}\n"


def parse_test_output(stdout: str) -> list[CheckOutcome]:
    """Parse libtest's human-readable output.

    Args:
        stdout: What the test binary printed.

    Returns:
        One outcome per test that ran, in output order; ignored tests are skipped.
    """
    outcomes: dict[str, CheckOutcome] = {}
    for full_name, status in _TEST_LINE.findall(stdout):
        if status != "ignored":
            outcomes[full_name] = CheckOutcome(full_name.removeprefix(f"{TEST_MODULE}::"), status == "ok")
    for full_name, block in _FAILURE_BLOCK.findall(stdout):
        if full_name in outcomes:
            lines = [
                line
                for line in block.splitlines()
                if line.strip() and not line.startswith("note: run with") and not _PANIC_LINE.match(line)
            ]
            outcomes[full_name] = CheckOutcome(outcomes[full_name].name, False, " ".join(" ".join(lines).split()))
    return list(outcomes.values())


class _ExerciseParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.exercises: list[Exercise] = []
        self._depth = 0
        self._current: dict[str, str] | None = None
        self._exercise_depth = 0
        self._solution_depth: int | None = None
        self._capture: str | None = None
        self._buffer: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = {k: v or "" for k, v in attrs}
        classes = set(attributes.get("class", "").split())
        if tag not in {"br", "hr", "img", "input", "meta", "link", "wbr", "source", "col", "area", "base", "embed"}:
            self._depth += 1
        if "exercise" in classes and self._current is None:
            self._current = {"id": attributes.get("data-id", ""), "solution": "", "tests": ""}
            self._exercise_depth = self._depth
        elif self._current is not None:
            if tag == "details" and "solution" in classes:
                self._solution_depth = self._depth
            elif tag == "script" and "tests" in classes:
                self._capture, self._buffer = "tests", []
            elif tag == "code" and self._solution_depth is not None and not self._current["solution"]:
                self._capture, self._buffer = "solution", []

    def handle_data(self, data: str) -> None:
        if self._capture:
            self._buffer.append(data)

    def handle_endtag(self, tag: str) -> None:
        if self._current is not None:
            if (self._capture == "tests" and tag == "script") or (self._capture == "solution" and tag == "code"):
                self._current[self._capture] = "".join(self._buffer)
                self._capture = None
            if self._solution_depth == self._depth and tag == "details":
                self._solution_depth = None
            if self._depth == self._exercise_depth:
                current = self._current
                self.exercises.append(
                    Exercise(id=current["id"], solution=_clean(current["solution"]), tests=_clean(current["tests"]))
                )
                self._current = None
        self._depth -= 1


def _clean(source: str) -> str:
    return textwrap.dedent(source.replace("\t", "    ")).strip("\n") + "\n"


def extract_exercises(html: str) -> list[Exercise]:
    """Extract every exercise from a page.

    Args:
        html: Page source.

    Returns:
        Exercises in page order.
    """
    parser = _ExerciseParser()
    parser.feed(html)
    parser.close()
    return parser.exercises


def find_rustc() -> str:
    """Locate ``rustc``.

    Returns:
        The path to ``rustc``.

    Raises:
        RustToolchainError: If it is not on ``PATH``; run through ``pixi run`` so the pinned toolchain is active.
    """
    rustc = shutil.which("rustc")
    if rustc is None:
        raise RustToolchainError("rustc not found; run this through `pixi run` so the course toolchain is on PATH")
    return rustc


def check_exercise(exercise: Exercise, *, timeout: float = 120.0) -> ExerciseResult:
    """Compile an exercise's reference solution with its tests and run them.

    Args:
        exercise: The exercise to check.
        timeout: Seconds allowed for compiling and for running, each.

    Returns:
        The outcome.
    """
    if not exercise.solution.strip():
        return ExerciseResult(exercise.id, False, "no reference solution in a details.solution block")
    if not exercise.tests.strip():
        return ExerciseResult(exercise.id, False, "no tests in a script.tests block")
    rustc = find_rustc()
    with tempfile.TemporaryDirectory(prefix="pp-ex-") as tmp:
        source = Path(tmp) / "exercise.rs"
        binary = Path(tmp) / "exercise-tests"
        source.write_text(wrap_tests(exercise.solution, exercise.tests), encoding="utf-8")
        command = [rustc, "--edition", EDITION, "--test", "--crate-name", "exercise", "-o", str(binary), str(source)]
        try:
            compiled = subprocess.run(command, capture_output=True, text=True, timeout=timeout, check=False)
            if compiled.returncode != 0:
                return ExerciseResult(exercise.id, False, f"does not compile:\n{compiled.stderr.strip()[-2000:]}")
            ran = subprocess.run(
                [str(binary), "--test-threads=1"], capture_output=True, text=True, timeout=timeout, check=False
            )
        except subprocess.TimeoutExpired:
            return ExerciseResult(exercise.id, False, f"timed out after {timeout:.0f}s")
    outcomes = parse_test_output(ran.stdout)
    if not outcomes:
        return ExerciseResult(exercise.id, False, f"no tests ran:\n{(ran.stdout + ran.stderr).strip()[-1000:]}")
    failures = [f"{o.name}: {o.message}" for o in outcomes if not o.ok]
    return ExerciseResult(exercise.id, not failures, "; ".join(failures))


def check_page_exercises(page: Path) -> list[ExerciseResult]:
    """Check every exercise on one page.

    Args:
        page: HTML file to check.

    Returns:
        One result per exercise.
    """
    return [check_exercise(e) for e in extract_exercises(page.read_text(encoding="utf-8"))]
