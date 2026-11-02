# rust-practicum

An interactive curriculum that takes a Python developer from no Rust to helping maintain the
[Pixi](https://github.com/prefix-dev/pixi) codebase. Lessons tie every idea to Python and to real code from a frozen
Pixi snapshot. Everything, including the Rust toolchain, is managed with Pixi.

[![CI](https://github.com/millsks/rust-practicum/actions/workflows/ci.yml/badge.svg)](https://github.com/millsks/rust-practicum/actions/workflows/ci.yml)
![coverage](https://img.shields.io/badge/coverage-%E2%89%A590%25-brightgreen)
![rust](https://img.shields.io/badge/rust-1.95%20%C2%B7%20edition%202024-b7410e)
![pixi](https://img.shields.io/badge/managed%20by-pixi-orange)

## What's inside

- **Level 1 (Reading Rust), available now:** an orientation module, Stage 1 *The core language* (5 modules) and
  Stage 2 *Idiomatic Rust* (6 modules), published as static HTML in [`docs/`](docs/).
- **Levels 2-3** (the Pixi stack: errors, serde and TOML, async; then rattler, a tour of Pixi's architecture, and a
  change on a frozen Pixi snapshot) are on the [roadmap](docs/reference/roadmap.html).
- **Rust in the browser.** Runnable cells and graded exercises compile on the official
  [Rust Playground](https://play.rust-lang.org) (stable, edition 2024). Exercises are `#[test]` functions, graded from
  real `cargo test` output.
- **A frozen Pixi snapshot.** Every "Reading Pixi" section quotes
  [prefix-dev/pixi@451aa959](https://github.com/prefix-dev/pixi/tree/451aa9596402530f1a4d8a50b5fe137570ad42a5), so
  permalinks and excerpts never drift.
- **Capstones** in [`projects/`](projects/): a starter crate, a reference solution, and Cargo integration tests.

## Installation

Install [Pixi](https://pixi.sh), then:

```sh
git clone https://github.com/millsks/rust-practicum.git
cd rust-practicum
pixi install          # Python tooling plus Rust 1.95 (rustc, cargo, clippy, rustfmt) from conda-forge
pixi run bootstrap    # pre-commit hooks (only needed if you contribute)
```

## Quick start

```sh
pixi run serve        # opens the course at http://127.0.0.1:8000/
```

Start with **Module 00: How this practicum works**. When a level ends, build its capstone:

```sh
pixi run projects     # list capstones and whether you've started them
pixi run start c1     # copy the starter crate into work/c1_manifest_reader/
pixi run grade c1     # run the capstone's cargo tests against your crate in work/
```

## Development

```sh
pixi run test               # unit tests (fast)
pixi run test-integration   # every exercise solution and capstone solution is compiled and tested
pixi run check-site         # validate links, anchors, quizzes, and exercises in docs/
pixi run check-exercises    # compile and test every exercise's reference solution with rustc
pixi run lint-rust          # cargo fmt --check and cargo clippy -D warnings on capstone solutions
pixi run bundle             # build the single-file edition: dist/rust-practicum.html
pixi run ci                 # pre-commit, build, mypy, ruff, Rust lint, and tests with a coverage gate (>= 90%)
```

Lesson authors should read [`docs/AUTHORING.md`](docs/AUTHORING.md).

## License

[MIT](LICENSE)
