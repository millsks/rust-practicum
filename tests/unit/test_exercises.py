import pytest

from practicum import exercises
from practicum.exercises import (
    CheckOutcome,
    Exercise,
    RustToolchainError,
    check_exercise,
    extract_exercises,
    parse_test_output,
    wrap_tests,
)

HTML = """
<div class="exercise" data-id="m01-e1">
  <div class="exercise-prompt"><p>Do it &amp; win</p></div>
  <textarea class="starter">fn f() -> bool {\n    todo!()\n}</textarea>
  <script type="text/x-rust" class="tests">
    #[test]
    fn f_is_true() {
        assert!(f() == (1 &lt; 2));
    }
  </script>
  <details class="hint"><summary>Hint</summary><p><code>not this</code></p></details>
  <details class="solution"><summary>Show</summary>
<pre><code class="language-rust">fn f() -> bool {
    1 &lt; 2
}</code></pre>
  </details>
</div>
<div class="exercise" data-id="m01-e2"><script class="tests">#[test] fn x() {}</script></div>
"""

LIBTEST_OUTPUT = """
running 3 tests
test pp_checks::adds ... ok
test pp_checks::skipped ... ignored
test pp_checks::subtracts ... FAILED

failures:

---- pp_checks::subtracts stdout ----

thread 'pp_checks::subtracts' (50) panicked at exercise.rs:9:9:
assertion `left == right` failed: 3 - 1 should be 2
  left: 4
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    pp_checks::subtracts

test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out
"""


def test_extract_exercises_reads_solution_and_tests() -> None:
    first, second = extract_exercises(HTML)
    assert first.id == "m01-e1"
    assert first.solution == "fn f() -> bool {\n    1 < 2\n}\n"
    assert first.tests == "#[test]\nfn f_is_true() {\n    assert!(f() == (1 &lt; 2));\n}\n"
    assert second.id == "m01-e2"
    assert second.solution == "\n"


def test_wrap_tests_puts_tests_in_a_cfg_test_module() -> None:
    wrapped = wrap_tests("fn f() {}\n\n", "#[test]\nfn t() {\n\n    f();\n}\n")
    assert wrapped.startswith(
        "fn f() {}\n\n#[cfg(test)]\n#[allow(unused_imports)]\nmod pp_checks {\n    use super::*;\n"
    )
    assert "    #[test]\n    fn t() {\n\n        f();\n    }\n" in wrapped
    assert wrapped.endswith("}\n")


def test_parse_test_output_reads_passes_failures_and_messages() -> None:
    assert parse_test_output(LIBTEST_OUTPUT) == [
        CheckOutcome("adds", True),
        CheckOutcome("subtracts", False, "assertion `left == right` failed: 3 - 1 should be 2 left: 4 right: 2"),
    ]


def test_parse_test_output_with_nothing() -> None:
    assert parse_test_output("error: could not compile") == []


def test_check_exercise_needs_solution_and_tests() -> None:
    assert "no reference solution" in check_exercise(Exercise("x", "  \n", "#[test] fn t() {}")).detail
    assert "no tests" in check_exercise(Exercise("x", "fn f() {}", "\n")).detail


def test_missing_rustc_raises(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(exercises.shutil, "which", lambda name: None)
    with pytest.raises(RustToolchainError, match="pixi run"):
        check_exercise(Exercise("x", "fn f() {}", "#[test] fn t() {}"))
