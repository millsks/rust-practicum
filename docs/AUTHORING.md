# Authoring Guide

How lesson, project, and reference pages in `docs/` are built. The live gallery of every component is
`docs/authoring/components.html` (`pixi run serve`, then open `/authoring/components.html`).

Two commands must pass before a page is done:

- `pixi run check-site` validates links, anchors, quizzes, exercise test blocks, and visualizer names.
- `pixi run check-exercises docs/modules/<page>.html` compiles every exercise's reference solution with its tests
  using the course's pinned `rustc` (1.95, edition 2024) and runs them. Every exercise must pass.

## Who the course is for

A working Python developer who uses Pixi daily and wants to help maintain the Pixi codebase (prefix-dev/pixi, written
in Rust). They know functions, classes, exceptions, dicts, type hints, and tests in Python. They do not know Rust,
C, or systems programming. Level 1 ("Reading Rust") ends when they can read any function in Pixi and explain who
owns what, and can write a small, well-tested crate from a blank file.

Teach every idea in three steps:

1. **From Python.** Start from what the learner already knows ("In Python, `b = a` makes two names for one list...").
2. **The Rust idea.** Explain what Rust does, and *why* (the guarantee it buys), with a runnable example.
3. **In Pixi.** Show where the idea appears in the frozen Pixi snapshot, with a permalink, and explain the lines.

## The frozen Pixi snapshot

Every reference to Pixi source code points at one commit, so links and quoted code never drift:

- Commit: `451aa9596402530f1a4d8a50b5fe137570ad42a5` (pixi 0.81.0 development, 2026-09-18)
- Permalink form: `https://github.com/prefix-dev/pixi/blob/451aa9596402530f1a4d8a50b5fe137570ad42a5/<path>#L<start>-L<end>`
- A local checkout at that commit lives at `../pixi` (relative to this repository) for authors. Quote only code that
  exists there, and check line numbers in permalinks against it.
- Toolchain at the snapshot: `rust-toolchain` is `1.95.0`, workspace `edition = "2024"`, `rust-version = "1.90"`.

Quote short excerpts (usually under 25 lines), trimmed with `// ...` where you cut. Pixi is BSD-3-Clause licensed;
attribute each excerpt with its permalink. Never invent Pixi APIs; if you simplify Pixi code for teaching, say
"simplified from" and link the original.

## Page skeleton

Module pages live at `docs/modules/<id>-<slug>.html` with the exact `href` from `docs/assets/js/curriculum.js`.
Project pages live at `docs/projects/`, reference pages at `docs/reference/`. All use `data-root="../"`.

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>M03 · Ownership &amp; Borrowing · Rust Practicum</title>
<link rel="stylesheet" href="../assets/css/course.css">
<script src="../assets/js/curriculum.js"></script>
<script src="../assets/js/viz.js" defer></script>
<script src="../assets/js/viz-rust.js" defer></script>
<script src="../assets/js/course.js" defer></script>
</head>
<body data-root="../" data-module="m03">
<main class="lesson">
  <header class="lesson-hero">
    <p class="eyebrow">Stage 1 · The Core Language · Module 03</p>
    <h1>Ownership &amp; Borrowing</h1>
    <p class="lede">One or two sentences on why this matters.</p>
    <div class="lesson-meta"><span>⏱ ~3 hours</span><span>Prerequisites: <a href="m02-types-structs-enums.html">M02</a></span></div>
    <ul class="objectives"><li>…</li></ul>
  </header>
  <section class="lesson-section" id="some-topic"><h2>Some topic</h2> … </section>
  …
</main>
</body>
</html>
```

`course.js` wraps `<main>` with the header, sidebar, table of contents (built from each
`section.lesson-section[id] > h2`), the "Mark complete" box, notes, and previous/next links. Do not write those yourself.
Capstone pages use `data-module="c1"` (the manifest id) so completion tracking works; reference pages use
`data-page="ref-glossary"` etc.

## Required module structure

1. Hero: eyebrow, `h1`, lede, meta (time and prerequisites), objectives (4–7 items).
2. Six to ten teaching sections. Each one explains the idea in plain language, shows a worked example, and makes the
   learner do something: a runnable cell, a `data-predict` cell, a `data-compile-fail` cell to read, the ownership
   visualizer, or a "think" callout.
3. `id="pixi-reading"`: **Reading Pixi.** One or two real excerpts from the snapshot that use the module's ideas,
   with a permalink, a line-by-line explanation, and two or three questions in `<details class="reveal">` blocks.
4. `id="practice"`: three to six graded exercises, from easy to hard.
5. `id="mini-project"`: a `.project-box` with a small build the learner does locally in a Cargo project created with
   Pixi (spec, acceptance criteria as a `.checklist`, stretch goals).
6. `id="quiz"`: an end-of-module quiz of 10–15 questions mixing `single`, `multi`, `output`, and `text`.
7. `id="recap"`: a summary list, flashcards (8–12 cards), and "Going further" links to the Rust Book
   (doc.rust-lang.org/book), Rust by Example, the std docs, the Rust Reference, and the relevant Pixi source.

Write for a motivated developer who becomes a maintainer: define every term on first use, explain *why* as well as
*how*, show the mistakes Python developers actually make, and finish each section with what to remember. Aim for the
depth of a good textbook chapter (a module page is typically 900–1400 lines of HTML).

## Components

### Static code

```html
<pre><code class="language-rust">fn add(a: i32, b: i32) -&gt; i32 { a + b }</code></pre>
<pre data-label="Terminal"><code class="language-bash">pixi run cargo test</code></pre>
<pre data-label="Python"><code class="language-python">def add(a: int, b: int) -&gt; int: ...</code></pre>
```

Escape `<`, `>`, and `&` inside `<pre>` (`&lt;`, `&gt;`, `&amp;`). This matters constantly in Rust: `Vec&lt;String&gt;`,
`&amp;str`, `-&gt;`. Languages: rust, python, bash, toml, yaml, json, diff.

### Runnable cells (the official Rust Playground)

```html
<div class="runnable" data-title="Try it">
<textarea>
fn main() {
    let v: Vec<i32> = vec![1, 2, 3];
    println!("{v:?}");
}
</textarea>
</div>
```

- Code inside `<textarea>` is written literally (no escaping: `Vec<i32>` and `&str` as-is) and dedented
  automatically. Never write `</textarea>` inside it.
- Run compiles with Rust stable, edition 2024, debug mode, on play.rust-lang.org. Code with `fn main` runs; code
  without it is compiled as a library and reports "Compiled with no errors".
- `data-predict`: the learner types a prediction of stdout, then Run compares it.
- `data-compile-fail`: the cell is **meant** not to compile. The button reads "Compile" and the label "Compiler says
  no". Use these generously: show the real error, then explain how to read it (error code, the primary span, the
  `help:` lines) and how to fix it. Explain errors with their codes (`E0382`, `E0499`, `E0502`, `E0505`, `E0106`,
  `E0308`, `E0277`, ...).
- `data-local="reason"`: code that cannot run on the Playground (it uses Pixi or rattler crates, the filesystem, the
  network, environment variables, or a multi-file crate). The reason is shown with instructions to run it locally.
- The Playground has the top ~100 crates.io crates (serde, serde_json, itertools, thiserror, tokio, regex, ...).
  Runnable cells may use them when a lesson is about them, but Level 1 cells should prefer `std`.
- Keep cells short (under ~40 lines) and deterministic: no `HashMap` iteration order in printed output (use
  `BTreeMap` or sort first), no timing, no randomness.

### Exercises (graded by `cargo test` on the Playground, and by `rustc --test` locally)

```html
<div class="exercise" data-id="m03-e1" data-title="Borrow instead of move">
  <div class="exercise-prompt"><p>Task description…</p></div>
<textarea class="starter">
fn total_len(names: Vec<String>) -> usize {
    todo!()
}
</textarea>
  <script type="text/x-rust" class="tests">
#[test]
fn counts_bytes() {
    let names = vec![String::from("pixi"), String::from("rattler")];
    assert_eq!(total_len(&names), 11, "pixi (4) + rattler (7) = 11");
}
  </script>
  <details class="hint"><summary>Hint</summary><p>…</p></details>
  <details class="solution"><summary>Show solution</summary><pre><code class="language-rust">fn total_len(names: &amp;[String]) -&gt; usize {
    names.iter().map(|n| n.len()).sum()
}</code></pre></details>
</div>
```

How grading works: the learner's code is followed by

```rust
#[cfg(test)]
#[allow(unused_imports)]
mod pp_checks {
    use super::*;
    // ...the test functions from script.tests, indented...
}
```

and run with `cargo test`. Each `#[test]` function is one check; its panic message is shown to the learner. So:

- `data-id` must be unique across the site: `<module id>-e<n>` (for example `m03-e2`).
- Tests are ordinary `#[test]` functions. Use `assert_eq!(actual, expected, "helpful message")` and
  `assert!(cond, "helpful message")` with a message on **every** assert, written for the learner.
- Tests see everything in the learner's code through `use super::*;` (private items too). Do not write `mod` or
  `use super::*` yourself. Tests may define helper functions; they must not contain `</script>`.
- **Exercises are `std`-only.** The local checker runs plain `rustc` with no crates.
- Starters must compile (use `todo!()` for bodies), so the first Check shows failing tests rather than a compile
  error, unless the exercise is "fix this compile error", in which case say so in the prompt.
- An exercise may require a signature change (borrow instead of move, add a lifetime, add a trait bound). Tests then
  call the *expected* signature, and the starter will not compile against them; say this in the prompt so the
  learner knows the compiler error is the first clue.
- `println!` output is not graded. Test return values and state.
- The first `<code>` inside `details.solution` is the reference solution. It must pass its tests:
  `pixi run check-exercises docs/modules/<page>.html`.

### Quizzes

```html
<div class="quiz" data-id="m03-quiz">
<script type="application/json">
{"title": "Ownership quiz", "passScore": 0.7, "questions": [
  {"type": "single", "prompt": "HTML allowed", "code": "optional Rust shown above options", "options": ["a", "b"], "answer": 1, "explanation": "<p>Why.</p>"},
  {"type": "multi", "prompt": "…", "options": ["…", "…", "…"], "answer": [0, 2], "explanation": "…"},
  {"type": "output", "prompt": "What does this print?", "code": "fn main() {\n    println!(\"{}\", 1 + 1);\n}", "answer": "2", "explanation": "…"},
  {"type": "text", "prompt": "Which error code means 'use of moved value'?", "answer": ["E0382"], "explanation": "…"}
]}
</script>
</div>
```

The payload must be strict JSON (double quotes, no trailing commas, `\n` for newlines inside strings, `\"` for quotes).
`answer` is a 0-based index (`single`), a list of indexes (`multi`), or the exact text/accepted list
(`output`/`text`; `text` is case-insensitive). Every question needs an `explanation` that teaches. Inside JSON
strings use HTML entities for markup in prose (`&lt;T&gt;`), but `code` fields are shown as text, so write `Vec<T>` there.
Include "does this compile?" questions; they are the best check of ownership and lifetime understanding.

### Callouts, tags, and layout helpers

```html
<aside class="callout note|tip|warn|danger|pixi|pro|think"><p>…</p></aside>
<aside class="callout note" data-title="Custom heading"><p>…</p></aside>
<span class="tag pixi">pixi</span>
<div class="two-col">…</div>   <div class="card-grid"><div class="card"><h4>…</h4>…</div></div>
<div class="tabs"><div class="tab" data-label="Python">…</div><div class="tab" data-label="Rust">…</div></div>
<details class="reveal"><summary>Reveal</summary>…</details>
<ul class="checklist" data-id="m03-mini">…</ul>          (persisted checkboxes; unique data-id)
<pre class="mermaid">flowchart LR; A-->B</pre>          (Mermaid diagrams)
<figure class="figure"><svg class="diagram" viewBox="…">…</svg><figcaption>…</figcaption></figure>
```

Use `tabs` with `Python` and `Rust` labels for side-by-side comparisons. Use `callout pixi` for "how Pixi does it"
notes and `callout pro` for maintainer habits (clippy, rustfmt, review conventions).

Inline SVG diagrams should use CSS variables for color (`fill="var(--viz-1)"`, `stroke="var(--fg-muted)"`) so they work
in light and dark themes.

### Project box

```html
<div class="project-box">
  <div class="widget-head"><span class="w-kind">Mini-project</span><span class="w-title">Title</span></div>
  <div class="widget-body"> spec, steps, <ul class="checklist" data-id="m03-mini">…</ul> </div>
</div>
```

### Visualizers

```html
<div class="viz" data-viz="ownership" data-config='{"scenario": "borrow"}'></div>
```

| Name | Purpose | Config (all optional) |
|---|---|---|
| `ownership` | Steps through code showing stack slots, heap buffers, owner arrows, borrow arrows, moves and drops | `scenario`: `move` \| `clone` \| `borrow` \| `mut-borrow` \| `fn-move` (no scenario: a picker of all five). Or a custom scenario: `title`, `code` (array of lines), `steps` (see `viz-rust.js` for the step format) |

## Rust facts to teach (and never contradict)

- Toolchain: Rust 1.95 stable, **edition 2024**. Pixi pins `1.95.0` in `rust-toolchain`.
- Edition 2024 specifics worth knowing: `gen` is a reserved keyword; `unsafe extern` blocks; `impl Trait` in return
  position captures all in-scope lifetimes by default; `if let` temporaries drop before `else`; `let` chains
  (`if let Some(x) = a && x > 3 { ... }`) are stable in edition 2024 since Rust 1.88.
- `std::sync::LazyLock` and `OnceLock` are stable (prefer them to `once_cell`/`lazy_static` in new code; Pixi still
  uses `once_cell` in places). `async fn` in traits is stable (Pixi still uses `async-trait` in places).
- Format strings capture identifiers: `println!("{name} {count:>4} {items:?}")`.
- Integer overflow panics in debug builds and wraps in release builds.
- `HashMap` iteration order is unspecified and varies between runs.

## Pixi conventions taught by the course

Pixi installs and runs everything, including Rust. Never tell the learner to install rustup, use `curl | sh`, `pip`,
`uv`, or bare `python`. For local Rust work:

```sh
pixi init hello-rust && cd hello-rust
pixi add rust                   # conda-forge's Rust toolchain: rustc, cargo, clippy, rustfmt
pixi run cargo init --name hello
pixi run cargo run
pixi run cargo test
pixi run cargo clippy --all-targets -- -D warnings
pixi run cargo fmt
```

Inside this repository the toolchain is already in `pixi.toml`; capstones use `pixi run projects`,
`pixi run start c1`, and `pixi run grade c1`. Mini-projects say to create their own Pixi workspace as above.

Pixi itself is built the same way: its `pixi.toml` has a `rust` feature (pinning `rust-src` 1.95) and, in the
`rust-test` environment, tasks such as `pixi run build-debug` (`cargo build --workspace --bin pixi`), `pixi run
test-fast` (`cargo nextest run --workspace --all-targets`), and `pixi run insta`; `pixi run lint` runs its
lefthook checks (cargo-fmt, cargo-clippy, typos, and more). Level 3 covers building Pixi; Level 1 only reads its code.
