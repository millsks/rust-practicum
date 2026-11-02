import pytest

from practicum.exercises import Exercise, check_exercise

pytestmark = pytest.mark.integration


def make(solution: str, tests: str) -> Exercise:
    return Exercise("x-e1", solution, tests)


def test_passing_solution() -> None:
    exercise = make(
        "pub fn double(n: i32) -> i32 {\n    n * 2\n}\n",
        "#[test]\nfn doubles() {\n    assert_eq!(double(2), 4);\n}\n",
    )
    assert check_exercise(exercise).ok


def test_solution_with_main_is_allowed() -> None:
    exercise = make(
        'fn main() {\n    println!("{}", label());\n}\nfn label() -> &\'static str {\n    "pixi"\n}\n',
        '#[test]\nfn label_is_pixi() {\n    assert_eq!(label(), "pixi");\n}\n',
    )
    assert check_exercise(exercise).ok


def test_failing_assertion_is_reported() -> None:
    result = check_exercise(
        make("fn x() -> i32 {\n    1\n}\n", '#[test]\nfn x_is_2() {\n    assert_eq!(x(), 2, "x should be 2");\n}\n')
    )
    assert not result.ok
    assert result.detail.startswith("x_is_2: ")
    assert "x should be 2" in result.detail


def test_compile_error_is_reported() -> None:
    result = check_exercise(make('fn x() -> i32 {\n    "one"\n}\n', "#[test]\nfn t() {\n    x();\n}\n"))
    assert not result.ok
    assert "does not compile" in result.detail
    assert "E0308" in result.detail


def test_no_tests_ran() -> None:
    result = check_exercise(make("fn x() {}\n", "// no test functions here\n"))
    assert "no tests ran" in result.detail


def test_timeout() -> None:
    result = check_exercise(make("fn spin() {\n    loop {}\n}\n", "#[test]\nfn t() {\n    spin();\n}\n"), timeout=20)
    assert result.detail == "timed out after 20s"
