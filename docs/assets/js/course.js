/* Rust Practicum course runtime: layout, progress, Rust via the Playground, exercises, quizzes. */
(() => {
  "use strict";

  const PYODIDE_URL = "https://cdn.jsdelivr.net/pyodide/v314.0.3/full/";
  const CDN = {
    prism: "https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/",
    codemirror: "https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.16/",
    mermaid: "https://cdnjs.cloudflare.com/ajax/libs/mermaid/10.9.1/mermaid.min.js",
  };
  const STORE_KEY = "rust-practicum:v1";
  const C = window.CURRICULUM || { parts: [], reference: [] };
  const body = document.body;
  const ROOT = body.dataset.root || "";
  const PAGE_ID = body.dataset.module || body.dataset.page || "";

  /* ---------------- utilities ---------------- */
  const $ = (sel, el = document) => el.querySelector(sel);
  const $$ = (sel, el = document) => Array.from(el.querySelectorAll(sel));
  function h(tag, attrs = {}, ...children) {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
      if (v === null || v === undefined || v === false) continue;
      if (k === "class") el.className = v;
      else if (k === "html") el.innerHTML = v;
      else if (k === "text") el.textContent = v;
      else if (k.startsWith("on") && typeof v === "function") el.addEventListener(k.slice(2), v);
      else el.setAttribute(k, v === true ? "" : v);
    }
    for (const c of children.flat()) {
      if (c === null || c === undefined || c === false) continue;
      el.append(c instanceof Node ? c : document.createTextNode(String(c)));
    }
    return el;
  }
  const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  function dedent(text) {
    const lines = text.replace(/\t/g, "    ").split("\n");
    while (lines.length && !lines[0].trim()) lines.shift();
    while (lines.length && !lines[lines.length - 1].trim()) lines.pop();
    const indents = lines.filter((l) => l.trim()).map((l) => l.match(/^ */)[0].length);
    const min = indents.length ? Math.min(...indents) : 0;
    return lines.map((l) => l.slice(min)).join("\n");
  }
  function loadScript(src) {
    return new Promise((resolve, reject) => {
      const existing = document.querySelector(`script[data-src="${src}"]`);
      if (existing) {
        if (existing.dataset.loaded) resolve();
        else existing.addEventListener("load", () => resolve(), { once: true });
        return;
      }
      const s = h("script", { src, "data-src": src });
      s.addEventListener("load", () => { s.dataset.loaded = "1"; resolve(); }, { once: true });
      s.addEventListener("error", () => reject(new Error(`failed to load ${src}`)), { once: true });
      document.head.append(s);
    });
  }
  function loadCss(href) {
    if (document.querySelector(`link[href="${href}"]`)) return;
    // Library styles go before the page's own so course.css wins ties (e.g. CodeMirror's white background in dark mode).
    const link = h("link", { rel: "stylesheet", href });
    const first = document.head.querySelector("link[rel='stylesheet'], style");
    if (first) first.before(link);
    else document.head.append(link);
  }
  const icon = {
    menu: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 6h16M4 12h16M4 18h16"/></svg>',
    sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
    moon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/></svg>',
    chev: '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M9 6l6 6-6 6"/></svg>',
    logo: '<svg class="logo" viewBox="0 0 32 32" aria-hidden="true"><g fill="#b7410e"><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(0 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(45 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(90 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(135 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(180 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(225 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(270 16 16)"/><rect x="14" y="1" width="4" height="7" rx="1" transform="rotate(315 16 16)"/></g><circle cx="16" cy="16" r="10" fill="#b7410e"/><circle cx="16" cy="16" r="4.5" fill="#fff"/></svg>',
  };

  /* ---------------- progress store ---------------- */
  const Store = {
    data: null,
    load() {
      if (this.data) return this.data;
      try { this.data = JSON.parse(localStorage.getItem(STORE_KEY)) || {}; } catch { this.data = {}; }
      for (const k of ["items", "quizzes", "exercises", "checklists", "notes", "visits"]) this.data[k] ||= {};
      return this.data;
    },
    save() {
      try { localStorage.setItem(STORE_KEY, JSON.stringify(this.data)); } catch { /* storage unavailable: progress is session-only */ }
      document.dispatchEvent(new CustomEvent("pp:progress"));
    },
    get(section, key) { return this.load()[section][key]; },
    set(section, key, value) { this.load()[section][key] = value; this.save(); },
  };
  window.PPStore = Store;

  const allItems = () => C.parts.flatMap((p) => [...p.modules, ...p.projects].map((m) => ({ ...m, part: p })));
  const isDone = (id) => !!Store.get("items", id)?.done;

  /* ---------------- theme ---------------- */
  function currentTheme() {
    const forced = document.documentElement.dataset.theme;
    if (forced) return forced;
    return matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  function applySavedTheme() {
    let saved = null;
    try { saved = localStorage.getItem(STORE_KEY + ":theme"); } catch { saved = null; }
    if (saved) document.documentElement.dataset.theme = saved;
  }
  function toggleTheme() {
    const next = currentTheme() === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    try { localStorage.setItem(STORE_KEY + ":theme", next); } catch { /* ignore */ }
    document.dispatchEvent(new CustomEvent("pp:theme", { detail: next }));
    updateThemeButton();
  }
  function updateThemeButton() {
    const b = $("#pp-theme");
    if (b) b.innerHTML = currentTheme() === "dark" ? icon.sun : icon.moon;
  }

  /* ---------------- shell: header, sidebar, toc, pager ---------------- */
  function progressText() {
    const items = allItems();
    return `${items.filter((m) => isDone(m.id)).length} / ${items.length} complete`;
  }
  function addFavicon() {
    if (document.querySelector("link[rel~='icon']")) return;
    const svg = icon.logo.replace('class="logo" ', 'xmlns="http://www.w3.org/2000/svg" ');
    document.head.append(h("link", { rel: "icon", href: "data:image/svg+xml," + encodeURIComponent(svg) }));
  }
  function buildShell() {
    addFavicon();
    const main = $("main") || h("main");
    const header = h("header", { class: "pp-header" },
      h("button", { class: "icon-btn pp-menu-btn", "aria-label": "Toggle navigation", html: icon.menu, onclick: () => body.classList.toggle("nav-open") }),
      h("a", { class: "pp-brand", href: ROOT + "index.html", html: `${icon.logo}<span>${esc(C.title)} <small>Rust ${esc(C.rust)}</small></span>` }),
      h("span", { class: "spacer" }),
      h("span", { class: "pp-progress-pill", id: "pp-progress-pill", text: progressText() }),
      h("button", { class: "icon-btn", id: "pp-theme", "aria-label": "Toggle dark mode", onclick: toggleTheme }),
    );
    const sidebar = h("nav", { class: "pp-sidebar", "aria-label": "Curriculum" });
    renderSidebar(sidebar);
    const contentWrap = h("div", { class: "pp-content-wrap" });
    const mainWrap = h("div", { class: "pp-main" }, contentWrap);
    main.replaceWith(h("div", { class: "pp-layout" }, sidebar, mainWrap));
    contentWrap.append(main);
    const toc = buildToc(main);
    if (toc) contentWrap.append(toc);
    body.prepend(header);
    updateThemeButton();
    document.addEventListener("pp:progress", () => {
      $("#pp-progress-pill").textContent = progressText();
      renderSidebar(sidebar);
    });
    mainWrap.addEventListener("click", () => body.classList.remove("nav-open"));
  }
  function renderSidebar(nav) {
    nav.innerHTML = "";
    const here = location.pathname.split("/").slice(-2).join("/");
    for (const part of C.parts) {
      const items = [...part.modules, ...part.projects];
      const containsCurrent = items.some((m) => m.id === PAGE_ID);
      const wrap = h("div", { class: "part" + (containsCurrent ? " open" : "") });
      const done = items.filter((m) => isDone(m.id)).length;
      wrap.append(h("button", {
        html: `<span class="num">${part.number}</span><span>${esc(part.title)} <span class="status">${done}/${items.length}</span></span><span class="chev">${icon.chev}</span>`,
        onclick: () => wrap.classList.toggle("open"),
      }));
      const ul = h("ul");
      for (const m of items) {
        const label = m.id.startsWith("m") ? m.id.slice(1) : m.id === "capstone" ? "★" : "P" + m.id.slice(1);
        ul.append(h("li", {}, h("a", {
          href: ROOT + m.href, class: m.id === PAGE_ID || m.href.endsWith(here) ? "current" : "",
          html: `<span class="mid">${esc(label)}</span><span>${esc(m.title.replace(/^Project \d+: /, ""))}</span>${isDone(m.id) ? '<span class="done">✓</span>' : ""}`,
        })));
      }
      wrap.append(ul);
      nav.append(wrap);
    }
    nav.append(h("div", { class: "side-title", text: "Reference" }));
    const ref = h("div", { class: "part open" });
    const ul = h("ul", { style: "padding-left:0" });
    for (const r of C.reference) {
      ul.append(h("li", {}, h("a", { href: ROOT + r.href, class: r.id === PAGE_ID ? "current" : "", html: `<span>${esc(r.title)}</span>` })));
    }
    ref.append(ul);
    nav.append(ref);
  }
  function buildToc(main) {
    const sections = $$(".lesson-section[id] > h2, .lesson-section[id] > header > h2", main);
    if (sections.length < 3) return null;
    const toc = h("aside", { class: "pp-toc" }, h("h4", { text: "On this page" }));
    const links = new Map();
    for (const h2 of sections) {
      const sec = h2.closest(".lesson-section");
      const a = h("a", { href: "#" + sec.id, text: h2.textContent });
      links.set(sec, a);
      toc.append(a);
    }
    const obs = new IntersectionObserver((entries) => {
      for (const e of entries) {
        if (e.isIntersecting) {
          links.forEach((a) => a.classList.remove("active"));
          links.get(e.target)?.classList.add("active");
        }
      }
    }, { rootMargin: "-20% 0px -70% 0px" });
    links.forEach((_, sec) => obs.observe(sec));
    return toc;
  }
  function buildModuleFooter(main) {
    const items = allItems();
    const idx = items.findIndex((m) => m.id === PAGE_ID);
    if (idx === -1) return;
    const item = items[idx];
    const footer = h("section", { class: "module-footer" });
    const box = h("div", { class: "complete-box" });
    const refresh = () => {
      const done = isDone(PAGE_ID);
      box.classList.toggle("is-done", done);
      const quizzes = $$(".quiz[data-id]", main).map((q) => Store.get("quizzes", q.dataset.id));
      const ex = $$(".exercise[data-id]", main);
      const solved = ex.filter((e) => Store.get("exercises", e.dataset.id)?.solved).length;
      const quizText = quizzes.length ? quizzes.map((q) => (q ? `${Math.round(q.best * 100)}%` : "not taken")).join(", ") : "none";
      box.innerHTML = "";
      box.append(
        h("div", { style: "flex:1;min-width:220px" },
          h("strong", { text: done ? "You completed this " + (item.id.startsWith("m") ? "module" : "project") + "." : "Finished? Mark it complete to track your progress." }),
          h("div", { class: "status", text: `Exercises solved: ${solved}/${ex.length} · Quiz best: ${quizText}` })),
        h("button", {
          class: "btn " + (done ? "" : "ok"), text: done ? "Mark as not complete" : "Mark complete",
          onclick: () => { Store.set("items", PAGE_ID, done ? { done: false } : { done: true, at: new Date().toISOString() }); refresh(); },
        }),
      );
    };
    refresh();
    document.addEventListener("pp:progress", refresh);
    const notesKey = PAGE_ID;
    const notes = h("textarea", { placeholder: "Write what you learned, what confused you, and questions to revisit. Saved in this browser." });
    notes.value = Store.get("notes", notesKey) || "";
    let t;
    notes.addEventListener("input", () => { clearTimeout(t); t = setTimeout(() => Store.set("notes", notesKey, notes.value), 400); });
    footer.append(box, h("div", { class: "notes-box" }, h("h3", { text: "My notes" }), notes));
    const pager = h("nav", { class: "pager", "aria-label": "Module navigation" });
    if (idx > 0) pager.append(h("a", { class: "prev", href: ROOT + items[idx - 1].href, html: `<small>Previous</small>${esc(items[idx - 1].title)}` }));
    if (idx < items.length - 1) pager.append(h("a", { class: "next", href: ROOT + items[idx + 1].href, html: `<small>Next</small>${esc(items[idx + 1].title)}` }));
    footer.append(pager);
    main.append(footer);
    const visits = Store.get("visits", PAGE_ID) || 0;
    Store.set("visits", PAGE_ID, visits + 1);
  }

  /* ---------------- syntax highlighting ---------------- */
  async function highlightAll(root = document) {
    const blocks = $$("pre > code[class*='language-']", root).filter((c) => !c.dataset.hl);
    addCopyButtons(root);
    if (!blocks.length) return;
    try {
      await loadScript(CDN.prism + "prism.min.js");
      const langs = new Set(blocks.map((b) => (b.className.match(/language-(\w+)/) || [])[1]));
      const extra = ["rust", "python", "bash", "toml", "yaml", "json", "ini", "diff"].filter((l) => langs.has(l));
      for (const l of extra) await loadScript(`${CDN.prism}components/prism-${l}.min.js`);
      for (const b of blocks) {
        if (b.className.includes("language-shell") || b.className.includes("language-sh")) b.className = "language-bash";
        window.Prism.highlightElement(b);
        b.dataset.hl = "1";
      }
    } catch { /* offline: plain code still readable */ }
  }
  function addCopyButtons(root) {
    for (const pre of $$("pre", root)) {
      if (pre.closest(".mermaid") || pre.classList.contains("output") || pre.classList.contains("mermaid") || $(".copy-btn", pre)) continue;
      const btn = h("button", { class: "copy-btn", text: "Copy", type: "button" });
      btn.addEventListener("click", async () => {
        const text = ($("code", pre) || pre).innerText;
        try { await navigator.clipboard.writeText(text); btn.textContent = "Copied"; } catch { btn.textContent = "Select & copy"; }
        setTimeout(() => (btn.textContent = "Copy"), 1400);
      });
      pre.append(btn);
    }
  }
  function highlightInto(el, code) {
    el.textContent = code;
    if (window.Prism?.languages?.rust) el.innerHTML = window.Prism.highlight(code, window.Prism.languages.rust, "rust");
  }

  /* ---------------- editor ---------------- */
  let cmReady = null;
  function ensureCodeMirror() {
    if (!cmReady) {
      loadCss(CDN.codemirror + "codemirror.min.css");
      cmReady = loadScript(CDN.codemirror + "codemirror.min.js")
        .then(() => loadScript(CDN.codemirror + "addon/mode/simple.min.js"))
        .then(() => Promise.all([
          loadScript(CDN.codemirror + "mode/rust/rust.min.js"),
          loadScript(CDN.codemirror + "addon/edit/matchbrackets.min.js"),
          loadScript(CDN.codemirror + "addon/selection/active-line.min.js"),
        ]))
        .then(() => true)
        .catch(() => false);
    }
    return cmReady;
  }
  function makeEditor(host, code, { onRun, readOnly = false } = {}) {
    const ta = h("textarea", { spellcheck: "false", autocapitalize: "off", autocomplete: "off" });
    ta.value = code;
    ta.rows = Math.min(Math.max(code.split("\n").length + 1, 4), 28);
    host.append(ta);
    const handlers = [];
    let cm = null;
    ta.addEventListener("input", () => handlers.forEach((fn) => fn()));
    ta.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") { e.preventDefault(); onRun?.(); }
      else if (e.key === "Tab" && !e.shiftKey) { e.preventDefault(); ta.setRangeText("    ", ta.selectionStart, ta.selectionEnd, "end"); }
    });
    ensureCodeMirror().then((ok) => {
      if (!ok || !window.CodeMirror || !ta.isConnected) return;
      cm = window.CodeMirror.fromTextArea(ta, {
        mode: "rust", lineNumbers: true, indentUnit: 4, matchBrackets: true, styleActiveLine: true,
        viewportMargin: Infinity, readOnly,
        extraKeys: {
          "Ctrl-Enter": () => onRun?.(), "Cmd-Enter": () => onRun?.(),
          Tab: (c) => (c.somethingSelected() ? c.indentSelection("add") : c.replaceSelection("    ", "end")),
          "Shift-Tab": (c) => c.indentSelection("subtract"),
        },
      });
      cm.on("change", () => handlers.forEach((fn) => fn()));
    });
    return {
      getValue: () => (cm ? cm.getValue() : ta.value),
      setValue: (v) => (cm ? cm.setValue(v) : (ta.value = v)),
      onChange: (fn) => handlers.push(fn),
    };
  }

  /* ---------------- Rust runtime (the official Rust Playground) ---------------- */
  // Exercise tests are wrapped exactly as practicum/exercises.py wraps them for `pixi run check-exercises`.
  const TEST_MODULE = "pp_checks";
  function wrapTests(code, tests) {
    const body = tests.split("\n").map((l) => (l.trim() ? "    " + l : l)).join("\n");
    return `${code.replace(/\s+$/, "")}\n\n#[cfg(test)]\n#[allow(unused_imports)]\nmod ${TEST_MODULE} {\n    use super::*;\n\n${body}\n}\n`;
  }
  const hasMain = (code) => /(^|\n)\s*(pub\s+)?(async\s+)?fn\s+main\s*\(/.test(code);
  const CARGO_NOISE = /^\s*(Compiling|Finished|Running|Doc-tests|Blocking|Updating|Downloading|Downloaded|Locking|Adding)\b/;
  function cleanStderr(text) {
    return (text || "").split("\n").filter((l) => !CARGO_NOISE.test(l)).join("\n").replace(/^\s+|\s+$/g, "");
  }
  /** Parse libtest output into [{name, ok, msg}]. Mirrors practicum/exercises.py:parse_test_output. */
  function parseTestOutput(stdout) {
    const results = [];
    const seen = new Map();
    for (const m of stdout.matchAll(/^test (\S+) \.\.\. (ok|FAILED|ignored)/gm)) {
      const name = m[1].replace(new RegExp(`^${TEST_MODULE}::`), "");
      if (m[2] === "ignored") continue;
      const r = { name, ok: m[2] === "ok", msg: "" };
      seen.set(m[1], r);
      results.push(r);
    }
    for (const m of stdout.matchAll(/^---- (\S+) stdout ----\n([\s\S]*?)(?=^---- |^failures:|^test result:)/gm)) {
      const r = seen.get(m[1]);
      if (!r) continue;
      const lines = m[2].split("\n").filter((l) => l.trim() && !l.startsWith("note: run with") && !/^thread '.*'( \(\d+\))? panicked at /.test(l));
      r.msg = lines.join(" ").replace(/\s+/g, " ").trim();
    }
    return results;
  }

  const Rust = {
    endpoint: "https://play.rust-lang.org/execute",
    edition: "2024",
    channel: "stable",
    status: "idle",
    running: 0,
    statusListeners: new Set(),
    setStatus(s) { this.status = s; this.statusListeners.forEach((fn) => fn(s)); },
    /** Run code on the Playground. Pass an AbortController to make it stoppable. Returns {success, stdout, stderr} or {error}. */
    async execute(code, { tests = false, controller = new AbortController() } = {}) {
      const timer = setTimeout(() => controller.abort("timeout"), 45000);
      this.running += 1;
      this.setStatus("running");
      try {
        const res = await fetch(this.endpoint, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          signal: controller.signal,
          body: JSON.stringify({
            channel: this.channel, mode: "debug", edition: this.edition,
            crateType: hasMain(code) ? "bin" : "lib", tests, code, backtrace: false,
          }),
        });
        if (!res.ok) throw new Error(`The Rust Playground answered HTTP ${res.status}.`);
        const data = await res.json();
        return { success: !!data.success, stdout: data.stdout || "", stderr: data.stderr || "" };
      } catch (e) {
        if (controller.signal.aborted) return { error: controller.signal.reason === "timeout" ? "Timed out after 45 seconds. Look for an infinite loop." : "Stopped." };
        this.failed = true;
        return { error: `Could not reach the Rust Playground (${e.message}). Run the code locally instead: see Module 00.` };
      } finally {
        clearTimeout(timer);
        this.running -= 1;
        this.setStatus(this.running ? "running" : this.failed ? "error" : "ready");
        this.failed = false;
      }
    },
  };
  window.PPRust = Rust;

  function runtimeBadge() {
    const badge = h("span", { class: "status" });
    const update = (s) => {
      badge.textContent = s === "running" ? "Compiling on the Rust Playground…" : s === "error" ? "Playground unavailable" : `Rust ${C.rust || "stable"} · edition ${Rust.edition}`;
    };
    update(Rust.status);
    Rust.statusListeners.add(update);
    return badge;
  }
  function appendOut(outEl, text, cls) {
    if (!text) return;
    const span = h("span", { class: cls || null, text });
    outEl.append(span);
    outEl.scrollTop = outEl.scrollHeight;
  }
  /** Show a Playground result: program output, then compiler diagnostics/panics. Returns stdout. */
  function showResult(out, res, { compileOnly = false } = {}) {
    out.innerHTML = "";
    if (res.error) { appendOut(out, res.error + "\n", "err"); return ""; }
    appendOut(out, res.stdout);
    const err = cleanStderr(res.stderr);
    if (err) appendOut(out, (res.stdout ? "\n" : "") + err + "\n", res.success ? "sys" : "err");
    if (res.success && compileOnly) appendOut(out, "Compiled with no errors. (This code has no fn main, so there is nothing to run.)\n", "sys");
    if (!out.textContent) appendOut(out, "(no output)", "sys");
    return res.stdout;
  }

  /* ---------------- runnable cells ---------------- */
  function localOnly(el, code) {
    const pre = h("pre", {}, h("code", { class: "language-rust" }));
    highlightInto($("code", pre), code);
    const reason = el.dataset.local || "This code needs crates or a local environment the Playground doesn't have.";
    return [h("div", { class: "editor-host" }, pre),
      h("div", { class: "local-only-note", html: `${esc(reason)} Paste it into <code>src/main.rs</code> of a Cargo project and run <code>pixi run cargo run</code> in your terminal.` })];
  }
  function mountRunnable(el) {
    const src = $("textarea", el);
    const original = dedent(src ? src.value : el.textContent);
    const isLocal = el.hasAttribute("data-local");
    const expectFail = el.hasAttribute("data-compile-fail");
    el.innerHTML = "";
    el.classList.add("widget");
    const head = h("div", { class: "widget-head" },
      h("span", { class: "w-kind", text: isLocal ? "Run locally" : expectFail ? "Compiler says no" : "Runnable" }),
      el.dataset.title ? h("span", { class: "w-title", text: el.dataset.title }) : null,
      h("span", { class: "spacer" }));
    el.append(head);
    if (isLocal) { el.append(...localOnly(el, original)); highlightAll(el); return; }
    const host = h("div", { class: "editor-host" });
    const out = h("pre", { class: "output", "aria-live": "polite" });
    const predict = el.hasAttribute("data-predict");
    let predictInput = null;
    let controller = null;
    const run = async () => {
      if (predict && predictInput && !predictInput.value.trim()) { predictInput.focus(); verdict.textContent = "Write your prediction first, then run."; verdict.className = "verdict bad"; return; }
      out.innerHTML = "";
      appendOut(out, "Compiling…\n", "sys");
      runBtn.disabled = true; stopBtn.hidden = false;
      const code = editor.getValue();
      controller = new AbortController();
      const res = await Rust.execute(code, { controller });
      const printed = showResult(out, res, { compileOnly: !hasMain(code) });
      runBtn.disabled = false; stopBtn.hidden = true;
      if (predict && predictInput) {
        const norm = (s) => s.replace(/\r/g, "").split("\n").map((l) => l.trimEnd()).join("\n").trim();
        const ok = !res.error && norm(predictInput.value) === norm(printed);
        verdict.textContent = ok ? "Your prediction matched exactly." : "Not quite. Compare your prediction with the real output and work out why.";
        verdict.className = "verdict " + (ok ? "ok" : "bad");
      }
    };
    const runBtn = h("button", { class: "btn primary small", text: predict ? "Run & compare" : expectFail ? "Compile ▶" : "Run ▶", title: "Ctrl/Cmd + Enter", onclick: run });
    const stopBtn = h("button", { class: "btn small", text: "Stop", hidden: true, onclick: () => controller?.abort("stopped") });
    const resetBtn = h("button", { class: "btn small", text: "Reset", onclick: () => { editor.setValue(original); out.innerHTML = ""; } });
    head.append(runtimeBadge(), runBtn, stopBtn, resetBtn);
    el.append(host);
    const editor = makeEditor(host, original, { onRun: run });
    const verdict = h("span", { class: "verdict" });
    if (predict) {
      predictInput = h("textarea", { rows: 3, placeholder: "Predict the exact output before you run it…" });
      el.append(h("div", { class: "predict-box" }, h("label", { text: "Your prediction" }), predictInput, verdict));
    }
    el.append(out);
    if (el.hasAttribute("data-autorun")) run();
  }

  /* ---------------- exercises ---------------- */
  function mountExercise(el) {
    const id = el.dataset.id;
    const prompt = $(".exercise-prompt", el);
    const starterEl = $("textarea.starter", el) || $("textarea", el);
    const starter = dedent(starterEl ? starterEl.value : "");
    const testsEl = $("script.tests", el);
    const tests = dedent(testsEl ? testsEl.textContent : "");
    const details = $$(":scope > details", el);
    const saved = Store.get("exercises", id);
    el.innerHTML = "";
    el.classList.add("widget");
    if (saved?.solved) el.classList.add("solved");
    const head = h("div", { class: "widget-head" },
      h("span", { class: "w-kind", text: "Exercise" }),
      h("span", { class: "w-title", text: el.dataset.title || "" }),
      h("span", { class: "spacer" }));
    const host = h("div", { class: "editor-host" });
    const out = h("pre", { class: "output" });
    const results = h("ul", { class: "test-results" });
    const editor = makeEditor(host, saved?.code ?? starter, { onRun: () => check() });
    let saveTimer;
    editor.onChange(() => { clearTimeout(saveTimer); saveTimer = setTimeout(() => Store.set("exercises", id, { ...(Store.get("exercises", id) || {}), code: editor.getValue() }), 500); });
    const runIt = async () => {
      results.innerHTML = ""; out.innerHTML = "";
      appendOut(out, "Compiling…\n", "sys");
      const code = editor.getValue();
      showResult(out, await Rust.execute(code), { compileOnly: !hasMain(code) });
    };
    const fail = (text) => results.append(h("li", { class: "fail" }, h("span", { text: "✗" }), h("span", { text })));
    const check = async () => {
      out.innerHTML = ""; results.innerHTML = "";
      appendOut(out, "Compiling and running the checks…\n", "sys");
      checkBtn.disabled = true;
      const res = await Rust.execute(wrapTests(editor.getValue(), tests), { tests: true });
      checkBtn.disabled = false;
      out.innerHTML = "";
      if (res.error) { fail(res.error); return; }
      const parsed = parseTestOutput(res.stdout);
      if (!parsed.length) {
        fail("Your code did not compile, so no checks ran. Read the compiler's message below; it usually says exactly what to change.");
        appendOut(out, cleanStderr(res.stderr) + "\n", "err");
        return;
      }
      const passed = parsed.filter((r) => r.ok).length;
      for (const r of parsed) {
        results.append(h("li", { class: r.ok ? "pass" : "fail" }, h("span", { text: r.ok ? "✓" : "✗" }), h("span", {}, r.name, r.msg ? h("span", { class: "msg", text: " — " + r.msg }) : null)));
      }
      const all = passed === parsed.length && passed > 0;
      results.append(h("li", { class: all ? "pass" : "fail" }, h("strong", { text: all ? `All ${passed} checks passed. Nice work.` : `${passed}/${parsed.length} checks passed.` })));
      if (all) {
        Store.set("exercises", id, { code: editor.getValue(), solved: true, at: new Date().toISOString() });
        el.classList.add("solved");
      }
    };
    const checkBtn = h("button", { class: "btn ok small", text: "Check ✓", title: "Ctrl/Cmd + Enter", onclick: check });
    head.append(runtimeBadge(),
      h("button", { class: "btn small", text: "Run", onclick: runIt }),
      checkBtn,
      h("button", { class: "btn small", text: "Reset", onclick: () => { editor.setValue(starter); out.innerHTML = ""; results.innerHTML = ""; } }));
    el.append(head);
    if (prompt) el.append(prompt);
    el.append(host, results, out, ...details);
    highlightAll(el);
  }

  /* ---------------- quizzes ---------------- */
  function mountQuiz(el) {
    const id = el.dataset.id;
    const dataEl = $("script[type='application/json']", el);
    let quiz;
    try { quiz = JSON.parse(dataEl.textContent); } catch (e) { el.textContent = "Quiz data is invalid: " + e.message; return; }
    const passScore = quiz.passScore ?? 0.7;
    el.innerHTML = "";
    el.classList.add("widget");
    const head = h("div", { class: "widget-head" },
      h("span", { class: "w-kind", text: "Quiz" }),
      h("span", { class: "w-title", text: quiz.title || "Check your understanding" }),
      h("span", { class: "spacer" }),
      h("span", { class: "status", text: `${quiz.questions.length} questions · pass at ${Math.round(passScore * 100)}%` }));
    el.append(head);
    const answered = new Map();
    const summary = h("div", { class: "quiz-summary" });
    const qEls = quiz.questions.map((q, i) => renderQuestion(q, i, (correct) => { answered.set(i, correct); updateSummary(); }));
    el.append(...qEls, summary);
    function updateSummary() {
      const correct = [...answered.values()].filter(Boolean).length;
      const total = quiz.questions.length;
      const pct = correct / total;
      summary.innerHTML = "";
      const ring = h("div", { class: "score-ring", style: `--p:${Math.round((answered.size ? correct / total : 0) * 100)}`, text: `${correct}/${total}` });
      const prev = Store.get("quizzes", id);
      let msg = `Answered ${answered.size} of ${total}.`;
      if (answered.size === total) {
        const passed = pct >= passScore;
        msg = passed ? `Passed with ${Math.round(pct * 100)}%. Review any you missed, then move on.` : `${Math.round(pct * 100)}%. Re-read the sections behind the questions you missed, then retake the quiz.`;
        Store.set("quizzes", id, { best: Math.max(pct, prev?.best ?? 0), last: pct, at: new Date().toISOString(), attempts: (prev?.attempts ?? 0) + 1 });
      }
      summary.append(ring, h("div", { style: "flex:1;min-width:200px" },
        h("div", { text: msg }),
        prev ? h("div", { class: "status", text: `Best so far: ${Math.round(prev.best * 100)}% over ${prev.attempts} attempt(s)` }) : null),
        h("button", { class: "btn small", text: "Retake quiz", onclick: () => { const fresh = h("div", { class: "quiz", "data-id": id }); fresh.append(h("script", { type: "application/json", text: JSON.stringify(quiz) })); el.replaceWith(fresh); mountQuiz(fresh); } }));
    }
    updateSummary();
    highlightAll(el);
  }
  function renderQuestion(q, i, onAnswer) {
    const type = q.type || "single";
    const wrap = h("div", { class: "quiz-q" },
      h("div", { class: "q-num", text: `Question ${i + 1}${type === "multi" ? " · select all that apply" : type === "output" ? " · type the exact output" : ""}` }),
      h("div", { class: "q-prompt", html: q.prompt }));
    if (q.code) wrap.append(h("pre", {}, h("code", { class: "language-rust", text: dedent(q.code) })));
    const feedback = h("div", { class: "q-feedback" });
    let getAnswer;
    let optionEls = [];
    if (type === "single" || type === "multi") {
      const group = `q${Math.random().toString(36).slice(2)}`;
      const opts = h("div", { class: "q-options" });
      optionEls = q.options.map((o, idx) => {
        const input = h("input", { type: type === "multi" ? "checkbox" : "radio", name: group, value: idx });
        const label = h("label", { class: "q-option" }, input, h("span", { html: o }));
        opts.append(label);
        return label;
      });
      wrap.append(opts);
      getAnswer = () => optionEls.map((l, idx) => ($("input", l).checked ? idx : -1)).filter((x) => x >= 0);
    } else {
      const input = type === "output" ? h("textarea", { class: "q-text-input", rows: 3, placeholder: "Exact output…" }) : h("input", { class: "q-text-input", type: "text", placeholder: "Your answer…" });
      wrap.append(input);
      getAnswer = () => input.value;
    }
    const checkBtn = h("button", { class: "btn small primary", text: "Check answer" });
    checkBtn.addEventListener("click", () => {
      const ans = getAnswer();
      if ((Array.isArray(ans) && !ans.length) || (!Array.isArray(ans) && !ans.trim())) { feedback.className = "q-feedback show bad"; feedback.innerHTML = "<strong>Choose or type an answer first.</strong>"; return; }
      let correct;
      if (type === "single" || type === "multi") {
        const expected = type === "multi" ? [...q.answer].sort() : [q.answer];
        correct = JSON.stringify([...ans].sort()) === JSON.stringify(expected);
        optionEls.forEach((l, idx) => {
          if (expected.includes(idx)) l.classList.add("correct");
          else if (ans.includes(idx)) l.classList.add("incorrect");
          $("input", l).disabled = true;
        });
      } else {
        const norm = (s) => s.replace(/\r/g, "").split("\n").map((l) => l.trimEnd()).join("\n").trim();
        const accepted = (Array.isArray(q.answer) ? q.answer : [q.answer]).map((a) => (type === "text" ? norm(a).toLowerCase() : norm(a)));
        const given = type === "text" ? norm(ans).toLowerCase() : norm(ans);
        correct = accepted.includes(given);
        $(".q-text-input", wrap).disabled = true;
      }
      checkBtn.disabled = true;
      feedback.className = "q-feedback show " + (correct ? "ok" : "bad");
      const reveal = type === "output" || type === "text" ? `<p>Expected: <code>${esc(Array.isArray(q.answer) ? q.answer[0] : q.answer).replace(/\n/g, "<br>")}</code></p>` : "";
      feedback.innerHTML = `<strong>${correct ? "Correct." : "Not quite."}</strong>${correct ? "" : reveal}${q.explanation}`;
      highlightAll(feedback);
      onAnswer(correct);
    });
    wrap.append(h("div", { class: "btn-row" }, checkBtn), feedback);
    return wrap;
  }

  /* ---------------- flashcards ---------------- */
  function mountFlashcards(el) {
    let cards;
    try { cards = JSON.parse($("script[type='application/json']", el).textContent); } catch { return; }
    if (!Array.isArray(cards)) cards = cards.cards || [];
    el.innerHTML = "";
    el.classList.add("widget");
    let i = 0;
    const card = h("div", { class: "flashcard", tabindex: "0", role: "button", "aria-label": "Flip card" });
    const counter = h("span", { class: "status" });
    const show = () => {
      card.classList.remove("flipped");
      card.innerHTML = `<div class="face front"><small>Prompt</small><div>${cards[i].front}</div></div><div class="face back"><small>Answer</small><div>${cards[i].back}</div></div>`;
      counter.textContent = `${i + 1} / ${cards.length}`;
    };
    card.addEventListener("click", () => card.classList.toggle("flipped"));
    card.addEventListener("keydown", (e) => { if (e.key === " " || e.key === "Enter") { e.preventDefault(); card.classList.toggle("flipped"); } });
    el.append(
      h("div", { class: "widget-head" }, h("span", { class: "w-kind", text: "Flashcards" }), h("span", { class: "w-title", text: el.dataset.title || "Review" }), h("span", { class: "spacer" }), counter),
      h("div", { class: "card-stage" }, card),
      h("div", { class: "viz-controls" },
        h("button", { class: "btn small", text: "← Prev", onclick: () => { i = (i - 1 + cards.length) % cards.length; show(); } }),
        h("button", { class: "btn small primary", text: "Flip", onclick: () => card.classList.toggle("flipped") }),
        h("button", { class: "btn small", text: "Next →", onclick: () => { i = (i + 1) % cards.length; show(); } }),
        h("button", { class: "btn small", text: "Shuffle", onclick: () => { cards.sort(() => Math.random() - 0.5); i = 0; show(); } })),
    );
    show();
  }

  /* ---------------- tabs, checklists, mermaid ---------------- */
  function mountTabs(el) {
    const tabs = $$(":scope > .tab", el);
    const bar = h("div", { class: "tab-bar", role: "tablist" });
    tabs.forEach((t, i) => {
      const b = h("button", { role: "tab", "aria-selected": i === 0 ? "true" : "false", text: t.dataset.label || `Tab ${i + 1}` });
      b.addEventListener("click", () => {
        tabs.forEach((x) => x.classList.remove("active"));
        $$("button", bar).forEach((x) => x.setAttribute("aria-selected", "false"));
        t.classList.add("active"); b.setAttribute("aria-selected", "true");
        t.querySelectorAll(".CodeMirror").forEach((cm) => cm.CodeMirror?.refresh());
      });
      bar.append(b);
      if (i === 0) t.classList.add("active");
    });
    el.prepend(bar);
  }
  function mountChecklist(el) {
    const id = el.dataset.id;
    const items = $$(":scope > li", el);
    const state = Store.get("checklists", id) || [];
    items.forEach((li, i) => {
      const box = h("input", { type: "checkbox" });
      box.checked = !!state[i];
      const span = h("span");
      while (li.firstChild) span.append(li.firstChild);
      li.append(box, span);
      li.classList.toggle("checked", box.checked);
      box.addEventListener("change", () => {
        const s = Store.get("checklists", id) || [];
        s[i] = box.checked;
        Store.set("checklists", id, s);
        li.classList.toggle("checked", box.checked);
      });
    });
  }
  async function renderMermaid(root = document) {
    const blocks = $$("pre.mermaid, div.mermaid", root);
    if (!blocks.length) return;
    blocks.forEach((b) => { b.dataset.mmd ||= b.textContent; });
    try {
      await loadScript(CDN.mermaid);
      window.mermaid.initialize({ startOnLoad: false, theme: currentTheme() === "dark" ? "dark" : "neutral", fontFamily: "Inter, system-ui, sans-serif", securityLevel: "strict" });
      for (const b of blocks) { b.removeAttribute("data-processed"); b.innerHTML = esc(b.dataset.mmd); }
      await window.mermaid.run({ nodes: blocks });
    } catch { /* diagrams fall back to their source text */ }
  }

  /* ---------------- home page ---------------- */
  function renderHome() {
    const map = $("#curriculum-map");
    if (!map) return;
    const draw = () => {
      map.innerHTML = "";
      for (const part of C.parts) {
        const items = [...part.modules, ...part.projects];
        const done = items.filter((m) => isDone(m.id)).length;
        const hours = Math.round(part.modules.reduce((a, m) => a + (m.minutes || 0), 0) / 60);
        map.append(h("article", { class: "part-card" },
          h("header", {}, h("span", { class: "tag", text: `Part ${part.number}` }), h("h3", { text: part.title }), h("span", { class: "level", text: `${part.level}${hours ? ` · ~${hours} h of lessons` : ""}` })),
          h("p", { style: "margin:0.4em 0 0;color:var(--fg-muted)", text: part.summary }),
          h("div", { class: "bar" }, h("i", { style: `width:${items.length ? (done / items.length) * 100 : 0}%` })),
          h("ul", { class: "module-list" }, items.map((m) => h("li", {}, h("a", { href: m.href, title: m.summary || "", html: `<span class="mid">${m.id === "capstone" ? "★" : m.id.toUpperCase()}</span>${esc(m.title)}${isDone(m.id) ? '<span class="done">✓</span>' : ""}` })))),
        ));
      }
      const all = allItems();
      const doneCount = all.filter((m) => isDone(m.id)).length;
      const stat = $("#progress-stats");
      if (stat) {
        const quizzes = Object.values(Store.load().quizzes);
        const avg = quizzes.length ? Math.round((quizzes.reduce((a, q) => a + q.best, 0) / quizzes.length) * 100) : 0;
        const solved = Object.values(Store.load().exercises).filter((e) => e.solved).length;
        const next = all.find((m) => !isDone(m.id));
        stat.innerHTML = `<div class="card"><h4>Completed</h4><div style="font-size:1.6rem;font-weight:800">${doneCount} / ${all.length}</div></div>
          <div class="card"><h4>Exercises solved</h4><div style="font-size:1.6rem;font-weight:800">${solved}</div></div>
          <div class="card"><h4>Average quiz best</h4><div style="font-size:1.6rem;font-weight:800">${quizzes.length ? avg + "%" : "—"}</div></div>
          <div class="card"><h4>Up next</h4>${next ? `<a href="${next.href}">${esc(next.title)}</a>` : "You finished everything."}</div>`;
      }
    };
    draw();
    document.addEventListener("pp:progress", draw);
    $("#export-progress")?.addEventListener("click", () => {
      const blob = new Blob([JSON.stringify(Store.load(), null, 2)], { type: "application/json" });
      const a = h("a", { href: URL.createObjectURL(blob), download: `rust-practicum-progress-${new Date().toISOString().slice(0, 10)}.json` });
      document.body.append(a); a.click(); a.remove();
    });
    $("#import-progress")?.addEventListener("change", async (e) => {
      const file = e.target.files[0];
      if (!file) return;
      try { Store.data = JSON.parse(await file.text()); Store.load(); Store.save(); alertBox("Progress imported."); } catch { alertBox("That file is not a progress export."); }
    });
  }
  function alertBox(text) {
    const el = $("#progress-message");
    if (el) { el.textContent = text; setTimeout(() => (el.textContent = ""), 4000); }
  }

  /* ---------------- boot ---------------- */
  function mountComponents(root = document) {
    $$(".runnable", root).forEach(mountRunnable);
    $$(".exercise[data-id]", root).forEach(mountExercise);
    $$(".quiz[data-id]", root).forEach(mountQuiz);
    $$(".flashcards", root).forEach(mountFlashcards);
    $$(".tabs", root).forEach(mountTabs);
    $$(".checklist[data-id]", root).forEach(mountChecklist);
    if (window.PPViz) $$(".viz[data-viz]", root).forEach((el) => { try { window.PPViz.mount(el); } catch (e) { el.textContent = `Visualizer failed: ${e.message}`; } });
  }
  // The single-file edition (body[data-engine="single"]) supplies its own shell and calls these directly.
  window.PPCourse = { Store, Rust, mount: mountComponents, highlightAll, renderMermaid, currentTheme, isDone, allItems };
  applySavedTheme();
  if (body.dataset.engine === "single") return;
  document.addEventListener("DOMContentLoaded", () => {
    const main = $("main");
    if (!main) return;
    buildShell();
    if (PAGE_ID === "home") renderHome();
    mountComponents();
    buildModuleFooter(main);
    highlightAll();
    renderMermaid();
    document.addEventListener("pp:theme", () => renderMermaid());
    if (location.hash) document.getElementById(location.hash.slice(1))?.scrollIntoView();
  });
})();
