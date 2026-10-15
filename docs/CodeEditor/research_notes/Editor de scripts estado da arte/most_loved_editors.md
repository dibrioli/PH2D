# Most-loved code editors (2024–2026): who leads, why, and what developers hate

> Research notes for the PH2D integrated script editor (reference: Godot's script editor). Compiled
> 2026-10-15. Every number is dated. Survey tables for SO 2024/2025 were extracted from the raw HTML of
> survey.stackoverflow.co; SO 2026 from its official Markdown export (`/2026/technology/data.md`).
> Items I know but could not verify against a fetched source this session are kept in **Gaps**, never in
> Cited Findings.

## 1. Survey data — who leads USAGE vs who leads ADMIRATION (2024 → 2025 → 2026)

### Takeaway
VS Code dominates usage every year (73.6% → 75.9% → 70.9%) but is only mid-pack on admiration (77.2% → 62.6% → 55.7%). Admiration is led by small, fast, low-config, keyboard-first editors: **Neovim** (82.7% in 2024, top), and in 2026 **Helix** (88.5%, top of named mainstream editors), Neovim 69.9%, Emacs 61.0%, Zed 57.3%. AI-first forks grew in usage but their admiration fell sharply in 2026 (Cursor 33.0%, Windsurf 22.7%).

### Cited Findings

**Stack Overflow 2026** (n = 13,193 for the IDE question; question `DevIDE`, v2026.1) — [SO 2026 technology data (md)](https://survey.stackoverflow.co/2026/technology/data.md) · [per-group breakdown](https://survey.stackoverflow.co/2026/technology/data/dev-ide.md)
- Headline: "VS Code is still the most used IDE despite AI disruptors; usage has remained at or above 71% since 2021" and for admiration: "IYKYK: Helix users want to keep using it as their IDE more than any other IDE this year." — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)
- Usage 2026 (all respondents): VS Code 70.9% · Notepad++ 27.8% · Visual Studio 27.6% · Vim 27.2% · IntelliJ IDEA 24.3% · Cursor 14.9% · Neovim 14.7% · Android Studio 14.2% · PyCharm 13.5% · Nano 12.1% · **Zed 11.4%** · Jupyter 11.4% · Xcode 10.0% · Sublime Text 9.8% · VSCodium 7.7% · Rider 7.0% · Eclipse 6.9% · WebStorm 6.7% · GitHub Codespaces 6.6% · PhpStorm 5.6% · Emacs 5.3% · CLion 4.8% · RustRover 3.5% · GoLand 3.4% · Windsurf 3.2% · Kiro 2.8% · Replit 2.1% · NetBeans 2.1% · **Helix 1.4%** · RubyMine 1.4% · Trae 0.7% · Antigravity 0.4% · Kate 0.3% · BBEdit 0.2% — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)
- Admired / Desired 2026 (Admired = used AND want to keep using; Desired = want to use): **Helix 88.5%** / 1.4% · BBEdit 82.8% (tiny n=29) · Kate 81.8% (n=44) · R Studio 75.0% · **Neovim 69.9%** / 12.9% · Antigravity 62.1% (n=58) · **Emacs 61.0%** / 4.6% · **Zed 57.3%** / 9.7% · Rider 56.9% · RustRover 56.7% · VS Code 55.7% / 40.6% · Vim 55.7% / 16.4% · WebStorm 55.2% · IntelliJ IDEA 53.3% / 14.5% · GoLand 52.7% · Notepad++ 52.5% · PhpStorm 51.5% · VSCodium 51.4% · CLion 51.1% · RubyMine 51.1% · PyCharm 49.7% · Sublime Text 49.6% · Visual Studio 47.8% · Nano 46.8% · Jupyter 44.7% · Android Studio 39.4% · Xcode 38.7% · Trae 35.6% · Codespaces 34.2% · **Cursor 33.0%** / 7.9% · Eclipse 32.6% · Kiro 31.2% · NetBeans 30.4% · Replit 24.4% · **Windsurf 22.7%** — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)
- Early-career skew 2026 (1–5 yrs work experience, n=2,881): Neovim usage 23.4% (vs 14.7% overall), Zed 15.9% (vs 11.4%), VS Code 76.6% — [SO 2026 dev-ide.md](https://survey.stackoverflow.co/2026/technology/data/dev-ide.md)
- 2026 respondents: 76.5% "used a development environment (IDE) to write code or interact with agent-assisted developer tools" (12,423) — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)
- Note: in 2026 Claude Code was no longer in the IDE option list (coding agents are a separate category in 2026 per the chapter intro: "cloud providers, coding agents, LLMs, and security ops") — [SO 2026 technology.md](https://survey.stackoverflow.co/2026/technology.md)

**Stack Overflow 2025** (IDE question n = 26,143) — [SO 2025 technology](https://survey.stackoverflow.co/2025/technology)
- Headline: "Not only is Visual Studio Code the most used IDE for five years in a row, it is also consistently ranked the most desired IDE." — [SO 2025](https://survey.stackoverflow.co/2025/technology)
- Usage 2025 (all / professional): VS Code 75.9/76.2 · Visual Studio 29.0/29.7 · Notepad++ 27.4 · IntelliJ 27.1/28.4 · Vim 24.3 · **Cursor 17.9/19.3** · PyCharm 15.0 · Android Studio 15.0 · Jupyter 14.1 · Neovim 14.0 · Nano 12.2 · Sublime 10.5 · Xcode 10.0 · Claude Code 9.7 · WebStorm 7.6 · **Zed 7.3** · Rider 7.1 · Eclipse 7.1 · VSCodium 6.2 · PhpStorm 5.8 · Windsurf 4.9 · RustRover 3.2 — [SO 2025](https://survey.stackoverflow.co/2025/technology)
- Desired / Admired 2025: VS Code 48.9/62.6 · IntelliJ 17.5/58.2 · Visual Studio 16.0/51.8 · Vim 15.7/59.3 · Notepad++ 15.5/54.7 · Cursor 14.6/46.7 · **Neovim 13.3/74.4 (highest admired of the list)** · Claude Code 10.3/52.5 · PyCharm 9.6/52.9 · Zed 7.5/57.0 · Xcode 6.1/41.3 · Sublime 5.9/50.1 · WebStorm 5.7/59.1 · Rider 5.6/60.8 · Windsurf 4.3/41.8 · RustRover 3.7/62.6 · Eclipse 3.0/32.9 — [SO 2025](https://survey.stackoverflow.co/2025/technology)
- Helix and Emacs were NOT in the 2025 option list (so 2025 can't rank them) — [SO 2025](https://survey.stackoverflow.co/2025/technology)

**Stack Overflow 2024** — [SO 2024 technology](https://survey.stackoverflow.co/2024/technology)
- Headline: "Visual Studio Code is the most-desired (and used) IDE tool (59%) for developers but the most-admired is Neovim (83%)." — [SO 2024](https://survey.stackoverflow.co/2024/technology)
- Usage 2024: VS Code 73.6 · Visual Studio 29.3 · IntelliJ 26.8 · Notepad++ 23.9 · Vim 21.6 · Android Studio 16.1 · PyCharm 15.1 · Jupyter 12.8 · Neovim 12.5 · Sublime 10.9 · Eclipse 9.4 · Xcode 9.3 · Nano 9.2 · WebStorm 7.5 · PhpStorm 5.9 · Rider 5.7 · VSCodium 4.8 · CLion 4.6 · Emacs 4.2 · GoLand 3.3 · Qt Creator 2.7 · Kate 1.8 · **Fleet 1.7 · Helix 1.7** — [SO 2024](https://survey.stackoverflow.co/2024/technology)
- Desired / Admired 2024: VS Code 58.7/77.2 · IntelliJ 20.3/68.4 · Visual Studio 19.2/61.3 · Vim 16.6/69.2 · Notepad++ 15.4/62.8 · **Neovim 13.9/82.7** · PyCharm 11.3/63.8 · Rider 4.5/73.4 · DataGrip 4.3/73.0 · Goland 3.4/69.3 · **Helix 1.9/68.6** · Emacs 3.7/67.3 · Sublime 6.5/54.9 · Xcode 6.8/49.5 · Kate 1.4/64.3 · **Fleet 1.7/43.4** · Eclipse 3.4/30.1 · Code::Blocks 0.7/21.3 — [SO 2024](https://survey.stackoverflow.co/2024/technology)

**JetBrains State of Developer Ecosystem 2025**
- 24,534 developers, 194 countries, fieldwork April–June 2025, published Oct 2025; weighted for JetBrains-user bias but "some bias is likely present"; shows higher JetBrains IDE share than SO — [JetBrains Research blog](https://blog.jetbrains.com/research/2025/10/state-of-developer-ecosystem-2025/) (via search summary); [The Register, 2025-10-21](https://www.theregister.com/2025/10/21/massive_jetbrains_dev_survey)
- Cursor raw (unweighted) responses: 135 (2024) → 2,300+ (2025); 85% use AI coding tools; 23% worry about low-quality generated code — [The Register, 2025-10-21](https://www.theregister.com/2025/10/21/massive_jetbrains_dev_survey)
- Java devs' IDEs 2025 (secondary, unverified against JetBrains): IntelliJ 84%, VS Code 31%, Eclipse 28% — [search summary of lp.jetbrains.com/the-state-of-java-2025](https://lp.jetbrains.com/the-state-of-java-2025)

**Fleet** — JetBrains discontinued Fleet; no downloads/updates from 2025-12-22; reason: could not justify two general-purpose IDEs; Fleet platform reused for agentic tool "Air" — [InfoWorld](https://www.infoworld.com/article/4104991/jetbrains-discontinues-fleet-ide.html); [DevClass, 2025-12-09](https://devclass.com/2025/12/09/jetbrains-abandons-fleet-ide-pins-hopes-on-forthcoming-air-agentic-development-tool/); [AlternativeTo](https://alternativeto.net/news/2025/12/jetbrains-will-discontinue-its-fleet-ide-on-december-22-2025-due-to-overlapping-purposes/)

### Inferences
- **Ranked by admiration among editors with meaningful n (SO 2026):** Helix (88.5) > Neovim (69.9) > Emacs (61.0) > Zed (57.3) ≈ Rider/RustRover (~57) > VS Code = Vim (55.7) > IntelliJ (53.3) > Sublime (49.6) > Visual Studio (47.8) > Xcode (38.7) > Cursor (33.0) > Windsurf (22.7). Usage ranking: VS Code ≫ Notepad++ ≈ Visual Studio ≈ Vim > IntelliJ > Cursor ≈ Neovim > … > Zed (11.4) > … > Helix (1.4).
- The admired leaders share three traits: **speed**, **keyboard-first composable editing (modal or selection-first)**, and **built-in batteries without plugin config** (Helix, Zed). Fleet — a "lighter JetBrains" that never left preview — is the cautionary tale: 43.4% admired (2024), then discontinued.
- Zed is the fastest-growing editor in usage (7.3% → 11.4%, 2025→2026) while holding admiration ~57%. Cursor's admiration collapsed (46.7% → 33.0%) as usage dipped (17.9% → 14.9%): AI-first value alone does not create attachment.
- VS Code's admiration declining every year (77 → 63 → 56) while staying #1 in usage suggests "default choice, not loved choice"; signals of extension/AI-clutter fatigue (see §4–§6).
- Survey sample shrank (2025 IDE n=26,143 → 2026 n=13,193), so year-over-year deltas of a few points are noisy.

### Gaps
- JetBrains DevEco 2025 per-IDE share table not retrieved (data lives in its interactive "Data Playground"). No JetBrains 2026 report found yet (usually published ~Oct–Dec).
- Admiration numbers for Kakoune, Lapce, Fleet (post-2024) and Sublime's reasons not in any survey list.
- Sub-population admiration (e.g., professional-only 2026) not extracted; available under `da-dev-envs.md`.

## 2. Signature features per editor — why each is loved

### Takeaway
Each loved editor is known for 3–6 "signature" moves: Sublime = speed + Goto Anything + multiple selections; VS Code = command palette + Quick Open + LSP-in-core + breadcrumbs/sticky scroll/minimap + extensions; JetBrains = intention actions (Alt+Enter) + inspections + deep refactoring; Helix = selection→action, multi-selection as primitive, tree-sitter syntax selection, LSP with zero config; Zed = GPU rendering at 120 fps, multibuffers, edit prediction that stays out of the way; Lapce = Rust/wgpu, built-in LSP and modal toggle; Cursor = agentic edits + Tab prediction.

### Cited Findings

**VS Code**
- Command Palette (Ctrl+Shift+P) "gives access to all VS Code functionality"; Quick Open (Ctrl+P) "navigate to any file or symbol by typing its name"; typing `?` lists available prefixes — [VS Code UI docs](https://code.visualstudio.com/docs/getstarted/userinterface)
- Breadcrumbs: file path + symbol path to cursor, navigable; Sticky Scroll: "shows the starting lines of currently visible nested scopes at the top of the editor"; Minimap: high-level overview, click/drag to jump; unlimited split editors; Zen mode hides all UI except editor (Ctrl+K Z); preview tabs (italic, reused on single-click); indent guides — [VS Code UI docs](https://code.visualstudio.com/docs/getstarted/userinterface)
- Bracket pair colorization moved from extension into core (VS Code 1.60, 2021-09-29): an update on 42k-line `checker.ts` took ~10 s via the extension vs <1 ms in core (">10,000× faster"); complexity O(N+E) → O(log³N+E); reason: the extension API is non-incremental, lacks token access and is async — "This limit cannot be overcome" — [VS Code blog, Dieterichs 2021](https://code.visualstudio.com/blogs/2021/09/29/bracket-pair-colorization)

**JetBrains (IntelliJ / Rider / RustRover)**
- Intention actions + quick-fixes = "context actions"; **Alt+Enter** opens them at the caret; yellow bulb = intention (improvement), red bulb with "!" = quick-fix for a detected problem; Ctrl+Q previews the change before applying; inspections flag problems against standards/performance/best practices — [IntelliJ docs: Intention actions](https://www.jetbrains.com/help/idea/intention-actions.html)
- JetBrains IDEs rank high on admiration among heavy IDEs: Rider 56.9%, RustRover 56.7%, IntelliJ 53.3% (2026); Rider 73.4% (2024) — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md); [SO 2024](https://survey.stackoverflow.co/2024/technology)
- IntelliJ "zero-latency typing" mode measured 1.7 ms avg on Linux vs 69.2 ms in default mode (2015) — shows the vendor took typing latency seriously — [Fatin 2015](https://pavelfatin.com/typing-with-pleasure/)

**Helix**
- "Multiple cursors as a core editing primitive, inspired by Kakoune"; tree-sitter "error tolerant and robust syntax trees" for highlighting, indentation, navigation; "select syntax tree nodes instead of plain text"; LSP "auto completion, goto definition, documentation, diagnostics" with "no additional configuration"; built-in fuzzy finder for files/symbols + project search; surround; "No Electron. No VimScript. No JavaScript."; FAQ: "much less fiddling with config files" than Vim — [helix-editor.com](https://helix-editor.com/)
- "`selection → action`" model: select first, then act; "A cursor is simply a single width selection"; multiple selections are "a core mode of interaction" (select all instances, `c` to change all); built-in tutor `hx --tutor` — [Helix docs: Usage](https://docs.helix-editor.com/usage.html)
- Admired 88.5% (SO 2026), the highest of any editor with n>100 — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)

**Zed**
- Custom GPU UI framework GPUI written in Rust "to render user interfaces at 120 FPS"; budget ≈ 8.33 ms per frame at 120 Hz covering state update + layout + framebuffer; motivated by Atom/Electron GC pauses and DOM relayout causing dropped frames; only 5 primitives (rects via SDF, shadows, text, icons, images), each with a dedicated shader; glyph atlas with up to 16 subpixel variants; OS text shaping so text looks native; Flutter-style layout — [Zed blog, Scandurra, 2023-03-07](https://zed.dev/blog/videogame)
- Edit prediction via open model **Zeta** (Feb 2025): accept with Tab, then repeated Tab for follow-up edits; when LSP completions are visible, prediction isn't previewed until Alt/Option is held (Linux default alt-l) — [Zed blog: edit prediction](https://zed.dev/blog/edit-prediction) (via search summary); Zeta derived from Qwen2.5-Coder-7B with open dataset — [Phoronix](https://www.phoronix.com/news/Zed-Zeta-AI-Model-Edit-Predict)
- "Subtle mode" (2025-03-19): users found inline predictions "too in-your-face"; subtle mode hides predictions unless a modifier is held; principle: AI "should never show a prediction unless the user asks for one" — [Zed blog: out-of-your-face AI](https://zed.dev/blog/out-of-your-face-ai)
- Multibuffer: compose excerpts from many files into one editable surface (cross-file refactors, reviewing agent changes) — secondary sources only: [tooljunction](https://www.tooljunction.io/ai-tools/zed)

**Sublime Text**
- Goto Anything; Multiple Selections ("rename variables quickly", Ctrl/⌘+D selects next occurrence); split editing / tab multi-select; GPU rendering on Linux/Mac/Windows keeping UI fluid "up to 8K" — [sublimetext.com](https://www.sublimetext.com/)
- Sublime measured 8.2 ms avg typing latency on Windows (2015), vs Atom 49.4 ms — [Fatin 2015](https://pavelfatin.com/typing-with-pleasure/)

**Lapce**
- Pure Rust, Floem UI, wgpu rendering; Vim-like modal editing that can be toggled on/off; built-in LSP (completion, diagnostics, code actions); built-in remote development modelled on VS Code Remote; plugins via WASI (C, Rust, AssemblyScript); repo active (3,653 commits, 2026 footer) but no tagged release shown — [github.com/lapce/lapce](https://github.com/lapce/lapce)

**Cursor / AI-first forks**
- Cursor 2026 positioning is agentic: background/cloud agents, parallel agent fleets, codebase indexing and semantic search, multi-model choice, Cmd+K targeted edits (Karpathy quote: "you can do Cmd+K for targeted edits") — [cursor.com/features](https://cursor.com/features)
- Adoption vs love: Cursor usage 17.9% (2025) → 14.9% (2026); admired 46.7% → 33.0%; Windsurf admired 41.8% → 22.7% — [SO 2025](https://survey.stackoverflow.co/2025/technology); [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)

**Fleet** — discontinued 2025-12-22 (see §1).

### Inferences
- For PH2D the **portable, high-ROI signatures** are: (1) one fuzzy entry point (Sublime Goto Anything / VS Code Quick Open with prefixes `@` symbol, `:` line, `>` command); (2) Alt+Enter-style context actions with a **preview before apply** (JetBrains) — directly serves "limit human error"; (3) multi-selection as a primitive + "select syntax node" expand/shrink (Helix/tree-sitter) — powerful but needs no mode; (4) GPU-rendered, frame-budgeted editor (Zed/Sublime) — PH2D already renders with Vello/wgpu; (5) core-integrated features rather than plugins (VS Code's bracket-colorization lesson: the extension boundary made it 10,000× slower).
- Helix's modal "selection→action" is loved by its users but is a learning wall; a creative-tool audience is closer to Godot/Sublime users. Borrow the *primitives* (multi-selection, syntax-node selection, built-in pickers, zero config), not the modal grammar — or make modal an opt-in like Lapce.

### Gaps (prior knowledge, not verified against a fetched source this session — verify before citing)
- JetBrains Local History (automatic, VCS-independent file snapshots), "Search Everywhere" (double Shift), "Find Action" (Ctrl+Shift+A), safe rename/extract refactorings — docs at jetbrains.com/help/idea/local-history.html and /searching-everywhere.html.
- Neovim: built-in LSP client + tree-sitter, Lua config, composable operator+motion grammar (`d` + `iw`), which-key style plugins.
- Emacs: Magit, Org-mode, extensibility via Elisp.
- Xcode: SwiftUI Previews / Playgrounds (live result next to code) — highly relevant to a game editor ("see the effect immediately").
- Visual Studio: debugger (Edit and Continue, hot reload), IntelliSense.
- Kakoune: originator of selection-first + multiple selections; client/server; "orthogonal" design.
- Zed multiplayer/collab and multibuffer: only secondary sources retrieved; check zed.dev/docs.
- Windsurf "Cascade", Cursor "Tab" next-edit prediction details not on the fetched features page.

## 3. Cross-cutting "can't live without" features

### Takeaway
The features that recur across loved editors cluster into: **navigation** (command palette, fuzzy file finder, go to symbol, breadcrumbs, sticky scroll), **editing power** (multi-cursor, rename symbol, syntax-node selection, snippets), **feedback** (inline diagnostics, semantic/bracket colorization, inlay hints), and **safety** (local history, preview of fixes, format on save, git gutter). Most were born as plugins and migrated into core.

### Cited Findings
- Command palette + Quick Open + breadcrumbs + sticky scroll + minimap + split editors + zen mode + indent guides are all core VS Code UI — [VS Code UI docs](https://code.visualstudio.com/docs/getstarted/userinterface)
- Multiple selections and Goto Anything are Sublime's headline features — [sublimetext.com](https://www.sublimetext.com/)
- Helix ships fuzzy finder (files + symbols), project-wide search, LSP and tree-sitter built in, with no config — [helix-editor.com](https://helix-editor.com/)
- Context actions with preview (Alt+Enter, Ctrl+Q preview) — [IntelliJ docs](https://www.jetbrains.com/help/idea/intention-actions.html)
- Bracket pair colorization in core, incremental, <1 ms per edit on 42k lines — [VS Code blog 2021](https://code.visualstudio.com/blogs/2021/09/29/bracket-pair-colorization)
- Lapce ships LSP code actions, diagnostics, completion in core — [lapce GitHub](https://github.com/lapce/lapce)

### Inferences
- **Minimum viable "loved" surface for PH2D's script editor** (simplicity criterion): one fuzzy palette (files, symbols, commands, line — prefix-switched), go-to-definition/rename via LSP, inline diagnostics, multi-cursor (Ctrl+D next occurrence), syntax-node expand selection, bracket colorization + indent guides, sticky scroll/breadcrumb (one of the two), format on save, local history, context actions with preview. Minimap, zen mode, code lens are lower priority.
- "Born a plugin, moved to core" (bracket colorization, LSP in Neovim/Helix, sticky scroll) is a recurring pattern: features that need the token/syntax tree + per-keystroke incremental update belong in the core.

### Gaps
- No quantitative ranking (survey) of individual features was found; "can't live without" evidence is qualitative (vendor docs + positioning). Error-lens style inline diagnostics, inlay hints, code lens, git blame/gutter, snippets usage data not found.

## 4. Latency and typing feel — what numbers matter

### Takeaway
The editor is the weakest link in keystroke-to-photon latency. Hardware alone costs ~26 ms on a typical 60 Hz setup; editors in 2015 ranged from ~1 ms (GVim, IntelliJ zero-latency) to 50–200+ ms (Atom, IntelliJ default, under load). Targets: editor paint **1–5 ms** (Fatin), whole frame ≤ **8.33 ms** at 120 Hz (Zed); worst-case (p99/max) matters more than average.

### Cited Findings
- Fatin, "Typing with Pleasure" (2015-12-20): proposed editor painting target **1–5 ms**; typical keyboard ~14 ms avg (debounce 7–12 ms) + LCD ~12 ms ⇒ ~26 ms hardware baseline; ideal hardware ~3 ms; V-Sync adds ~8 ms avg, up to ~17 ms at 60 Hz; latency affects speed, errors, strain and attention "even when unnoticed"; maximum values matter, not only averages — [pavelfatin.com](https://pavelfatin.com/typing-with-pleasure/)
- Fatin measurements (Typometer, Windows text file, avg ms): GVim 0.9 · IntelliJ zero-latency 2.9 · Notepad++ 4.3 · Emacs 5.3 · Sublime 8.2 · Eclipse 10.1 · NetBeans 11.8 · IntelliJ default 24.7 · Atom 49.4; Linux XML: IntelliJ zero-latency 1.7 vs default 69.2; worst case IntelliJ default up to 544.6 ms (>0.5 s) on battery+VM — [pavelfatin.com](https://pavelfatin.com/typing-with-pleasure/)
- Dan Luu (~2017, terminals): MSR touch demo — 100 ms "terrible", 10 ms noticeable but ok, <1 ms "as good as pen and paper"; Carmack VR: ~20 ms fine, 50 ms laggy, 150 ms unbearable; at ~10 chars/s a 99.9th-percentile event hits every ~100 s, so tail latency is felt; every terminal measured had perceptible tail latency — [danluu.com/term-latency](https://danluu.com/term-latency/)
- Zed: 8.33 ms frame budget at 120 Hz; GC pauses and relayout in Electron/Atom caused missed frames; no measured keystroke-latency numbers given in that post — [Zed blog 2023](https://zed.dev/blog/videogame)
- Sublime: GPU rendering keeps UI fluid up to 8K — [sublimetext.com](https://www.sublimetext.com/)

### Inferences
- Budget for PH2D: keystroke → glyph painted within **one frame at the display rate**, editor-side work **≤ 2–5 ms**, and **p99 measured**, not average. Never block typing on LSP/analysis/AI (all async; IntelliJ's "zero-latency" mode = paint the char before running analysis).
- Since PH2D shares a process with a live 2D scene/game viewport, the script editor must not be starved by scene rendering (or vice versa): measure keystroke latency with the scene running ("under load" column in Fatin/Luu).
- Typometer-style measurement (synthetic key events + screen capture) is a reproducible oracle we can run against our own editor (CLAUDE.md §0.9).

### Gaps
- No independent 2024–2026 keystroke-latency benchmark of Zed / VS Code / Helix / Neovim found this session (only vendor claims). A modern Typometer run would close this.

## 5. Simplicity vs power — zero-config, config fatigue, discoverability

### Takeaway
The most-admired editors win either by being *complete out of the box* (Helix: LSP + tree-sitter + pickers with "no additional configuration"; Zed) or by being *deeply composable* (Neovim/Emacs) at the cost of config. Discoverability is solved by a searchable command palette that shows keybindings (VS Code, JetBrains Find Action), and inline key-hint popups (Helix infobox / which-key). Fleet (a "light" IDE that tried to do both) failed.

### Cited Findings
- Helix: LSP with "no additional configuration"; "much less fiddling with config files" than Vim; tutor built in — [helix-editor.com](https://helix-editor.com/); [Helix usage docs](https://docs.helix-editor.com/usage.html)
- Helix 88.5% admired (2026) vs 1.4% usage — enthusiastic niche — [SO 2026](https://survey.stackoverflow.co/2026/technology/data.md)
- VS Code moved bracket colorization into core because the extension architecture made it fundamentally slow — evidence that extension-heavy design has a performance ceiling — [VS Code blog](https://code.visualstudio.com/blogs/2021/09/29/bracket-pair-colorization)
- VS Code Command Palette exposes "all functionality"; `?` in Quick Open lists modes — [VS Code UI docs](https://code.visualstudio.com/docs/getstarted/userinterface)
- Fleet: discontinued Dec 2025; admired only 43.4% in 2024 — [InfoWorld](https://www.infoworld.com/article/4104991/jetbrains-discontinues-fleet-ide.html); [SO 2024](https://survey.stackoverflow.co/2024/technology)
- Disabling AI suggestions is scattered across several settings in VS Code, JetBrains and Cursor; users report suggestions persisting after toggling one off (anecdotal forum posts) — [Cursor forum](https://forum.cursor.com/t/cant-disable-inline-suggestion-again/98698); [JetBrains forum](https://intellij-support.jetbrains.com/hc/en-us/community/posts/33491152873490/comments/33602041567762); [TMS Outsource](https://tms-outsource.com/blog/posts/how-to-turn-off-autocomplete-in-vs-code/)

### Inferences
- PH2D should be **zero-config by construction** (like Helix/Zed): GDScript-equivalent/Rust-script LSP, formatter, tree-sitter grammar, palette, all on by default; settings only for taste (font, theme, keymap preset).
- Discoverability recipe: every command lives in the palette **with its shortcut shown**; a which-key style hint after a prefix key; context-action bulb. One switch per feature family (e.g., one "AI suggestions: off/subtle/on"), never scattered toggles.

### Gaps
- No quantitative study of "config fatigue" found; evidence is survey admiration + vendor positioning + anecdotal forum posts. HN thread-level evidence not retrieved.
- Helix's infobox/which-key popup and JetBrains Find Action not verified against fetched docs this session.

## 6. Ergonomics, accessibility, and the most HATED behaviours

### Takeaway
Hated behaviours that cause errors: **intrusive/aggressive completions** (AI ghost text, full-line completion inserting wrong code), toggles that don't really turn features off, and latency/jank (Electron GC pauses, IntelliJ default-mode spikes >0.5 s). Accessibility is a real risk for custom-GPU editors: Zed's GPUI does not inherit OS accessibility.

### Cited Findings
- Zed users called inline predictions "too in-your-face" → subtle mode; principle "never show a prediction unless the user asks" — [Zed blog 2025-03-19](https://zed.dev/blog/out-of-your-face-ai)
- JetBrains users: AI suggestions "mostly noise"; full-line completions broke concentration and were hard to disable; Rider completion "too aggressive/irrelevant" thread — [JetBrains forum 1](https://intellij-support.jetbrains.com/hc/en-us/community/posts/29047175298578/comments/29074544655506); [JetBrains forum 2](https://intellij-support.jetbrains.com/hc/en-us/community/posts/33491152873490/comments/33602041567762); [Rider thread](https://intellij-support.jetbrains.com/hc/en-us/community/posts/28310832513170/comments/28333233650322)
- Cursor: suggestions kept appearing after disabling every inline setting; user asked for a status-bar toggle — [Cursor forum](https://forum.cursor.com/t/cant-disable-inline-suggestion-again/98698)
- JetBrains DevEco 2025: 23% of developers worry about low-quality AI-generated code — [The Register](https://www.theregister.com/2025/10/21/massive_jetbrains_dev_survey)
- Latency/jank as a hated trait: Atom/Electron GC pauses and relayouts caused missed frames (Zed's founders' own Atom experience) — [Zed blog 2023](https://zed.dev/blog/videogame); IntelliJ default mode worst case 544.6 ms — [Fatin 2015](https://pavelfatin.com/typing-with-pleasure/)
- Accessibility: Zed uses its own GUI toolkit (GPUI); an undated fediverse post says no accessibility work had been done yet — [friends.grishka.me](https://friends.grishka.me/posts/978463) (weak, undated source)

### Inferences
- For "limit human error": completions must never auto-insert on ambiguous triggers (accept only on explicit Tab/Enter, show a diff/preview for multi-line or AI edits — Zed subtle mode + JetBrains preview); every automated edit must be undoable as one step and recoverable through local history.
- PH2D renders UI with Vello (custom GPU stack, same risk class as GPUI): screen-reader support (e.g., AccessKit) must be designed in, not retrofitted.

### Gaps
- No verified sources this session on: font/ligature readability research (JetBrains Mono design rationale), autosave surprises, slow-startup complaints with numbers, VS Code screen-reader support level, and current (2026) Zed accessibility status (check Zed GitHub issues for "screen reader"/AccessKit).
