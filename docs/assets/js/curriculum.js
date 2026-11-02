/* Curriculum manifest. The object below must stay valid JSON: practicum check-site parses it. */
window.CURRICULUM = {
  "title": "Rust Practicum",
  "rust": "1.95",
  "pixiSnapshot": "451aa9596402530f1a4d8a50b5fe137570ad42a5",
  "parts": [
    {
      "id": "part0",
      "number": 0,
      "title": "Orientation",
      "level": "Start here",
      "summary": "Why Rust for a Pixi maintainer, how the course runs Rust in your browser and through Pixi, and the frozen Pixi snapshot every lesson reads.",
      "modules": [
        {
          "id": "m00",
          "title": "How This Practicum Works",
          "href": "modules/m00-orientation.html",
          "minutes": 60,
          "summary": "The path from Python developer to Pixi contributor, the Rust Playground, the pinned toolchain, and reading compiler errors."
        }
      ],
      "projects": []
    },
    {
      "id": "part1",
      "number": 1,
      "title": "Stage 1 · The Core Language",
      "level": "Level 1 · Reading Rust",
      "summary": "Cargo and modules, types and enums, ownership and borrowing, Option and Result, and collections: enough to read any function in Pixi.",
      "modules": [
        {
          "id": "m01",
          "title": "Cargo, Crates & Modules",
          "href": "modules/m01-cargo-crates-modules.html",
          "minutes": 120,
          "summary": "cargo new/build/run/test, Cargo.toml, crates and packages, modules and use, and how Pixi's 60-crate workspace fits together."
        },
        {
          "id": "m02",
          "title": "Types, Structs & Enums",
          "href": "modules/m02-types-structs-enums.html",
          "minutes": 150,
          "summary": "Scalars, String vs &str, structs and impl blocks, enums that carry data, and exhaustive match."
        },
        {
          "id": "m03",
          "title": "Ownership & Borrowing",
          "href": "modules/m03-ownership-borrowing.html",
          "minutes": 180,
          "summary": "Moves, Copy and Clone, shared and mutable references, slices, and reading borrow-checker errors without fear."
        },
        {
          "id": "m04",
          "title": "Option, Result & ?",
          "href": "modules/m04-option-result.html",
          "minutes": 150,
          "summary": "Absence and failure as values: match, if let, let-else, combinators, the ? operator, and custom error enums."
        },
        {
          "id": "m05",
          "title": "Collections & Iterators",
          "href": "modules/m05-collections-iterators.html",
          "minutes": 150,
          "summary": "Vec, HashMap, BTreeMap and HashSet, iter vs iter_mut vs into_iter, adapters, and collect."
        }
      ],
      "projects": []
    },
    {
      "id": "part2",
      "number": 2,
      "title": "Stage 2 · Idiomatic Rust",
      "level": "Level 1 · Reading Rust",
      "summary": "Traits and generics, lifetimes, smart pointers, closures, crate design, and tests: the patterns that fill the Pixi codebase.",
      "modules": [
        {
          "id": "m06",
          "title": "Traits & Generics",
          "href": "modules/m06-traits-generics.html",
          "minutes": 180,
          "summary": "Defining and implementing traits, generic functions and bounds, impl Trait, dyn Trait, derive, and the standard conversion traits."
        },
        {
          "id": "m07",
          "title": "Lifetimes in Practice",
          "href": "modules/m07-lifetimes.html",
          "minutes": 120,
          "summary": "What 'a means, elision rules, structs that borrow, 'static, and the lifetime errors you will actually meet."
        },
        {
          "id": "m08",
          "title": "Smart Pointers & Shared State",
          "href": "modules/m08-smart-pointers.html",
          "minutes": 150,
          "summary": "Box, Rc and Arc, RefCell and Mutex, Cow, OnceLock and LazyLock, and why Pixi reaches for Arc everywhere."
        },
        {
          "id": "m09",
          "title": "Closures & Iterator Chains",
          "href": "modules/m09-closures-iterators.html",
          "minutes": 120,
          "summary": "Fn, FnMut and FnOnce, move closures, writing your own iterator, and reading long iterator chains in Pixi."
        },
        {
          "id": "m10",
          "title": "Visibility, Features & Crate Design",
          "href": "modules/m10-crate-design.html",
          "minutes": 90,
          "summary": "pub, pub(crate) and re-exports, Cargo features and cfg, workspace dependencies, and doc comments."
        },
        {
          "id": "m11",
          "title": "Testing Like Pixi Does",
          "href": "modules/m11-testing.html",
          "minutes": 120,
          "summary": "Unit and integration tests, doc tests, test helpers and fixtures, snapshot testing with insta, and cargo nextest."
        }
      ],
      "projects": [
        {
          "id": "c1",
          "title": "Capstone 1: Manifest Reader",
          "href": "projects/c1-manifest-reader.html",
          "summary": "Turn a pixi.toml into typed Rust structs with clear errors, resolve environments, and order tasks, graded by cargo test."
        }
      ]
    }
  ],
  "reference": [
    {
      "id": "ref-roadmap",
      "title": "Roadmap: Levels 2-3",
      "href": "reference/roadmap.html",
      "summary": "What comes after Level 1: the Pixi stack (errors, serde, async) and working in the Pixi codebase itself."
    },
    {
      "id": "ref-python-to-rust",
      "title": "Python to Rust Cheat Sheet",
      "href": "reference/python-to-rust.html",
      "summary": "Side-by-side Python and Rust for the things you do every day."
    },
    {
      "id": "ref-pixi-map",
      "title": "Pixi Codebase Map",
      "href": "reference/pixi-codebase-map.html",
      "summary": "The crates in the frozen Pixi snapshot, what each one does, and where to start reading."
    },
    {
      "id": "ref-glossary",
      "title": "Glossary",
      "href": "reference/glossary.html",
      "summary": "Terms used in the course, with links to where each is taught."
    }
  ]
};
