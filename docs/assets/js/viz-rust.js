/* Rust visualizers: ownership (stack, heap, moves, borrows, drops). */
(() => {
  "use strict";
  const { registerViz } = window.PPViz;

  /*
   * A scenario is { title, code: [lines], steps: [step] }.
   * step = { line (1-based), note (HTML), stack: [slot], heap: [{ id, value }] }
   * slot = { name, type, value?, owns?: heapId, ref?: slotName, mut?: bool, state?: "moved" | "dropped", frame?: "main" | fn name }
   */
  const S = (name, type, extra = {}) => ({ name, type, ...extra });
  const SCENARIOS = {
    move: {
      title: "A move: ownership of the heap buffer changes hands",
      code: [
        "fn main() {",
        "    let a = String::from(\"conda\");",
        "    let b = a;",
        "    // println!(\"{a}\"); // error[E0382]: borrow of moved value",
        "    println!(\"{b}\");",
        "}",
      ],
      steps: [
        { line: 2, note: "<code>a</code> is a <code>String</code>: pointer, length, capacity on the stack; the bytes live on the heap. <code>a</code> owns them.", stack: [S("a", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"conda\"" }] },
        { line: 3, note: "<code>let b = a;</code> copies the three stack words into <code>b</code> and <strong>moves</strong> ownership. The heap bytes are not copied. <code>a</code> is now unusable.", stack: [S("a", "String", { state: "moved" }), S("b", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"conda\"" }] },
        { line: 4, note: "Using <code>a</code> here would not compile: the compiler tracks that it was moved out of. There is exactly one owner, so there is exactly one free.", stack: [S("a", "String", { state: "moved" }), S("b", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"conda\"" }] },
        { line: 5, note: "<code>b</code> is the owner, so <code>b</code> is the name you use.", stack: [S("a", "String", { state: "moved" }), S("b", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"conda\"" }] },
        { line: 6, note: "End of scope: <code>b</code> is dropped and frees the heap buffer. <code>a</code> was moved, so nothing happens for it. No double free, no garbage collector.", stack: [S("a", "String", { state: "dropped" }), S("b", "String", { state: "dropped" })], heap: [] },
      ],
    },
    clone: {
      title: "A clone: a second, independent heap buffer",
      code: [
        "fn main() {",
        "    let a = String::from(\"pixi\");",
        "    let mut b = a.clone();",
        "    b.push_str(\".toml\");",
        "    println!(\"{a} {b}\");",
        "}",
      ],
      steps: [
        { line: 2, note: "<code>a</code> owns a heap buffer holding <code>\"pixi\"</code>.", stack: [S("a", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"pixi\"" }] },
        { line: 3, note: "<code>.clone()</code> allocates a <strong>new</strong> buffer and copies the bytes. <code>b</code> owns the copy; <code>a</code> still owns the original. Cloning costs an allocation, which is why Rust makes you ask for it.", stack: [S("a", "String", { owns: "h1" }), S("b", "String", { owns: "h2", mut: true })], heap: [{ id: "h1", value: "\"pixi\"" }, { id: "h2", value: "\"pixi\"" }] },
        { line: 4, note: "Changing <code>b</code> touches only <code>b</code>'s buffer.", stack: [S("a", "String", { owns: "h1" }), S("b", "String", { owns: "h2", mut: true })], heap: [{ id: "h1", value: "\"pixi\"" }, { id: "h2", value: "\"pixi.toml\"" }] },
        { line: 5, note: "Both names are valid owners of different data.", stack: [S("a", "String", { owns: "h1" }), S("b", "String", { owns: "h2", mut: true })], heap: [{ id: "h1", value: "\"pixi\"" }, { id: "h2", value: "\"pixi.toml\"" }] },
        { line: 6, note: "Both are dropped, in reverse order of declaration: <code>b</code> first, then <code>a</code>. Two buffers, two frees.", stack: [S("a", "String", { state: "dropped" }), S("b", "String", { state: "dropped" })], heap: [] },
      ],
    },
    borrow: {
      title: "Shared borrows: many readers, no owner change",
      code: [
        "fn len_of(s: &String) -> usize { s.len() }",
        "fn main() {",
        "    let name = String::from(\"rattler\");",
        "    let r1 = &name;",
        "    let r2 = &name;",
        "    let n = len_of(r1);",
        "    println!(\"{r1} {r2} {n}\");",
        "}",
      ],
      steps: [
        { line: 3, note: "<code>name</code> owns the buffer.", stack: [S("name", "String", { owns: "h1" })], heap: [{ id: "h1", value: "\"rattler\"" }] },
        { line: 4, note: "<code>&amp;name</code> creates a <strong>shared reference</strong>: a pointer to <code>name</code> that may read but not change it. Ownership does not move.", stack: [S("name", "String", { owns: "h1" }), S("r1", "&String", { ref: "name" })], heap: [{ id: "h1", value: "\"rattler\"" }] },
        { line: 5, note: "Any number of shared references can exist at once.", stack: [S("name", "String", { owns: "h1" }), S("r1", "&String", { ref: "name" }), S("r2", "&String", { ref: "name" })], heap: [{ id: "h1", value: "\"rattler\"" }] },
        { line: 6, note: "Passing <code>r1</code> to <code>len_of</code> copies the reference (references are <code>Copy</code>). The function borrows; when it returns, its borrow ends.", stack: [S("name", "String", { owns: "h1" }), S("r1", "&String", { ref: "name" }), S("r2", "&String", { ref: "name" }), S("n", "usize", { value: "7" })], heap: [{ id: "h1", value: "\"rattler\"" }] },
        { line: 7, note: "Everyone can still read.", stack: [S("name", "String", { owns: "h1" }), S("r1", "&String", { ref: "name" }), S("r2", "&String", { ref: "name" }), S("n", "usize", { value: "7" })], heap: [{ id: "h1", value: "\"rattler\"" }] },
        { line: 8, note: "References are dropped without freeing anything; only the owner <code>name</code> frees the buffer.", stack: [S("name", "String", { state: "dropped" }), S("r1", "&String", { state: "dropped" }), S("r2", "&String", { state: "dropped" }), S("n", "usize", { state: "dropped" })], heap: [] },
      ],
    },
    "mut-borrow": {
      title: "A mutable borrow: one writer at a time",
      code: [
        "fn main() {",
        "    let mut deps = vec![\"python\"];",
        "    let w = &mut deps;",
        "    w.push(\"numpy\");",
        "    // let r = &deps; // error[E0502] while w is still used below",
        "    println!(\"{}\", w.len());",
        "    let r = &deps;",
        "    println!(\"{r:?}\");",
        "}",
      ],
      steps: [
        { line: 2, note: "<code>deps</code> is a <code>Vec</code>: pointer, length, capacity on the stack, elements on the heap.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true })], heap: [{ id: "h1", value: "[\"python\"]" }] },
        { line: 3, note: "<code>&amp;mut deps</code> is an <strong>exclusive</strong> reference. While it is in use, nothing else may read or write <code>deps</code>.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true }), S("w", "&mut Vec<&str>", { ref: "deps", mut: true })], heap: [{ id: "h1", value: "[\"python\"]" }] },
        { line: 4, note: "Writing through <code>w</code>. The push may reallocate the buffer; that is exactly why other references must not exist right now: they could be left pointing at freed memory.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true }), S("w", "&mut Vec<&str>", { ref: "deps", mut: true })], heap: [{ id: "h1", value: "[\"python\", \"numpy\"]" }] },
        { line: 5, note: "A shared borrow here would be rejected, because <code>w</code> is used again on the next line.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true }), S("w", "&mut Vec<&str>", { ref: "deps", mut: true })], heap: [{ id: "h1", value: "[\"python\", \"numpy\"]" }] },
        { line: 6, note: "This is the last use of <code>w</code>. A borrow lasts until its last use (non-lexical lifetimes), not until the closing brace.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true }), S("w", "&mut Vec<&str>", { ref: "deps", mut: true })], heap: [{ id: "h1", value: "[\"python\", \"numpy\"]" }] },
        { line: 7, note: "<code>w</code> is no longer live, so a shared borrow is fine now.", stack: [S("deps", "Vec<&str>", { owns: "h1", mut: true }), S("w", "&mut Vec<&str>", { state: "dropped" }), S("r", "&Vec<&str>", { ref: "deps" })], heap: [{ id: "h1", value: "[\"python\", \"numpy\"]" }] },
        { line: 9, note: "The owner frees the buffer at the end of scope.", stack: [S("deps", "Vec<&str>", { state: "dropped" }), S("w", "&mut Vec<&str>", { state: "dropped" }), S("r", "&Vec<&str>", { state: "dropped" })], heap: [] },
      ],
    },
    "fn-move": {
      title: "Passing by value moves into the function",
      code: [
        "fn consume(s: String) { println!(\"{s}\"); }",
        "fn main() {",
        "    let spec = String::from(\"numpy >=2\");",
        "    consume(spec);",
        "    // consume(spec); // error[E0382]: use of moved value",
        "}",
      ],
      steps: [
        { line: 3, note: "<code>spec</code> owns the buffer.", stack: [S("spec", "String", { owns: "h1", frame: "main" })], heap: [{ id: "h1", value: "\"numpy >=2\"" }] },
        { line: 1, note: "Calling <code>consume(spec)</code> moves ownership into the parameter <code>s</code>. <code>spec</code> in <code>main</code> is now moved-from.", stack: [S("spec", "String", { state: "moved", frame: "main" }), S("s", "String", { owns: "h1", frame: "consume" })], heap: [{ id: "h1", value: "\"numpy >=2\"" }] },
        { line: 4, note: "<code>consume</code> returns and <code>s</code> is dropped there, freeing the buffer. If the caller still needed the string, the function should have taken <code>&amp;str</code> instead.", stack: [S("spec", "String", { state: "moved", frame: "main" })], heap: [] },
        { line: 5, note: "Calling it again would not compile. The fix is to borrow (<code>fn consume(s: &amp;str)</code>) or to pass <code>spec.clone()</code>.", stack: [S("spec", "String", { state: "moved", frame: "main" })], heap: [] },
      ],
    },
  };

  registerViz("ownership", (el, config, api) => {
    const { h, s, esc, shell, player, select } = api;
    const custom = config.code && config.steps ? { title: config.title || "Ownership", code: config.code, steps: config.steps } : null;
    let current = custom || SCENARIOS[config.scenario] || SCENARIOS.move;
    const ui = shell(el, { title: config.title || current.title, kind: "Ownership" });
    const grid = h("div", { class: "own-grid" });
    const codeView = h("pre", { class: "own-code" });
    const svgHost = h("div", { class: "own-mem" });
    grid.append(codeView, svgHost);
    ui.stage.append(grid);
    ui.setLegend([["owner → heap", "var(--viz-1)"], ["borrow (&)", "var(--viz-2)"], ["mutable borrow (&mut)", "var(--viz-4)"], ["moved / dropped", "var(--fg-muted)"]]);
    if (!custom && !config.scenario) {
      ui.head.append(select(Object.keys(SCENARIOS).map((k) => [k, k]), "move", (v) => { current = SCENARIOS[v]; ui.head.querySelector(".w-title").textContent = current.title; play.load(current.steps); }));
    }
    const ROW = 46;
    const render = (step) => {
      codeView.innerHTML = current.code.map((l, i) => `<span class="own-line${i + 1 === step.line ? " on" : ""}">${String(i + 1).padStart(2, " ")}  ${esc(l)}</span>`).join("");
      const rows = Math.max(step.stack.length, step.heap.length, 1);
      const W = 460;
      const H = rows * ROW + 34;
      const svg = s("svg", { viewBox: `0 0 ${W} ${H}`, class: "diagram", role: "img", "aria-label": "Stack and heap" });
      svg.append(
        s("defs", {}, ...["1", "2", "4"].map((n) => s("marker", { id: `own-arrow-${n}-${el.dataset.vizId}`, viewBox: "0 0 10 10", refX: 9, refY: 5, markerWidth: 7, markerHeight: 7, orient: "auto-start-reverse" }, s("path", { d: "M0,0 L10,5 L0,10 z", fill: `var(--viz-${n})` })))),
        s("text", { x: 8, y: 16, class: "own-h", text: "Stack" }),
        s("text", { x: 300, y: 16, class: "own-h", text: "Heap" }),
      );
      const slotY = new Map();
      step.stack.forEach((slot, i) => {
        const y = 26 + i * ROW;
        slotY.set(slot.name, y);
        const dead = slot.state === "moved" || slot.state === "dropped";
        svg.append(
          s("rect", { x: 8, y, width: 200, height: ROW - 10, rx: 6, class: `own-slot${dead ? " dead" : ""}` }),
          s("text", { x: 16, y: y + 15, class: "own-name", text: `${slot.mut ? "mut " : ""}${slot.name}${slot.frame && slot.frame !== "main" ? `  (in ${slot.frame})` : ""}` }),
          s("text", { x: 16, y: y + 30, class: "own-type", text: `${slot.type}${slot.value ? " = " + slot.value : ""}${dead ? `  · ${slot.state}` : ""}` }),
        );
      });
      const heapY = new Map();
      step.heap.forEach((cell, i) => {
        const y = 26 + i * ROW;
        heapY.set(cell.id, y);
        svg.append(s("rect", { x: 300, y, width: 152, height: ROW - 10, rx: 6, class: "own-heap" }), s("text", { x: 310, y: y + 22, class: "own-val", text: cell.value }));
      });
      step.stack.forEach((slot) => {
        const y = slotY.get(slot.name) + (ROW - 10) / 2;
        if (slot.owns && heapY.has(slot.owns)) {
          const hy = heapY.get(slot.owns) + (ROW - 10) / 2;
          svg.append(s("path", { d: `M208,${y} C254,${y} 254,${hy} 298,${hy}`, fill: "none", stroke: "var(--viz-1)", "stroke-width": 2, "marker-end": `url(#own-arrow-1-${el.dataset.vizId})` }));
        }
        if (slot.ref && slotY.has(slot.ref) && slot.state !== "dropped") {
          const ty = slotY.get(slot.ref) + (ROW - 10) / 2;
          const n = slot.mut ? "4" : "2";
          const bend = 240 + (slotY.get(slot.name) - slotY.get(slot.ref)) / 6;
          svg.append(s("path", { d: `M208,${y} C${bend},${y} ${bend},${ty} 210,${ty}`, fill: "none", stroke: `var(--viz-${n})`, "stroke-width": 2, "stroke-dasharray": "5 4", "marker-end": `url(#own-arrow-${n}-${el.dataset.vizId})` }));
        }
      });
      svgHost.innerHTML = "";
      svgHost.append(svg);
      ui.setLog(step.note || "");
    };
    el.dataset.vizId = el.dataset.vizId || Math.random().toString(36).slice(2, 8);
    const play = player(ui.controls, render, { speed: 1400 });
    play.load(current.steps);
  });
})();
