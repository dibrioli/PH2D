# Script languages for game engines / creative tools, and where language design is going (as of 2026-10-15)

> Scope: which language a PH2D user-script editor should be built around, judged on editor tooling,
> error prevention, performance, Rust embeddability, hot reload and learnability, plus the design
> trends that shape what programmers expect. Researched 2026-10-15. Version and date numbers for Rust
> crates come from the crates.io API (`/api/v1/crates/<name>`), queried that day. "Updated" is the
> publish date of the newest version.

## Q1. GDScript: why it works for Godot users, and its limits

### Takeaway
GDScript succeeds because it is easy (Python-like), built into the engine (editor, autocompletion, node
paths) and gradually typed: you can leave types out, let `:=` infer them, or switch on warnings that
demand them. Types buy three things: errors before running, autocompletion after the dot, and faster
code (typed opcodes). The limits are a type system with holes (collections, `as` → `null`) and lock-in
to one engine.

### Cited Findings
- Typed code catches more mistakes before it runs. Explicit types let the editor list a value's methods, properties and constants. "Dynamically typed values often produce no suggestions after a dot" (paraphrase of the docs). — [Godot docs: Static typing in GDScript](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Performance: "typed GDScript improves performance by using optimized opcodes when operand/argument types are known at compile time"; "Static types improve GDScript performance and more optimizations are planned for the future." — [Godot docs: Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Inference without ceremony: `var damage := 10.5` infers the type. The opt-in warnings `UNTYPED_DECLARATION`, `INFERRED_DECLARATION` and `UNSAFE_*` let a project choose how strict to be. They are **off by default**. — [Godot docs: Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Limitations: methods such as `push_back` and comparisons such as `==` on typed Array/Dictionary stay untyped. Nested typed collections (an array of typed arrays) are not supported. On a runtime type mismatch `as` silently yields `null`. — [Godot docs: Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Learnability evidence: when Godot dropped VisualScript, many users said GDScript was easy enough that they preferred it, and for some "Godot became a way to learn programming". — [Godot blog, 2022-08-23](https://godotengine.org/article/godot-4-will-discontinue-visual-scripting)
- Usage: GDScript is used by 3.3 % of all respondents in the Stack Overflow 2025 survey, against Lua at 9.2 %. — [SO Survey 2025, Technology](https://survey.stackoverflow.co/2025/technology/)

### Inferences
- The GDScript lesson for PH2D: **gradual typing plus engine-aware tooling** (types of nodes and assets known to the editor) is what makes the editor "smart", more than the syntax. The editor's intelligence is limited by how much of the program the type checker can see.
- Making typing warnings **opt-in** keeps beginners happy, but it means most code stays untyped and the editor stays blind. A tool for artists **and** programmers might rather *infer by default and show the inferred types* (inlay hints) than ask for annotations.
- Building a bespoke language (as GDScript is) costs a lot: parser, type checker, LSP, debugger and docs, all maintained in-house. Godot can afford that; for PH2D it argues for reusing a language whose tooling already exists.

### Gaps
- No recent (2025–26) benchmark of typed vs untyped GDScript in Godot 4.x was found. The docs give no numbers.
- C# usage in Godot (the 2nd official language) and its tooling story were not researched in depth.

## Q2. Luau (Roblox): gradual typing, the new type solver, performance, luau-lsp, Rust embedding

### Takeaway
Luau is the strongest ready-made candidate for PH2D. It is gradually typed, and its new type solver
became the default at Roblox on 2025-11-20. It has a full-featured open-source LSP (luau-lsp, MIT), a
fast interpreter with optional native code generation that uses type annotations, a sandbox mode built
for untrusted scripts, and it is **already reachable from the stack**: mlua 0.12.2 (MIT, 2026-10-03)
has `luau`, `luau-jit` and `luau-vector4` features plus `Lua::sandbox`. Weak points: strict mode on the
new solver was still being polished into 2026, and some error messages are verbose.

### Cited Findings
- **New type solver reached general release on 2025-11-20.** It "dramatically improves type inference to support more accurate autocomplete". It adds read-only table properties, better refinements, **type functions** (a power-user feature for expressive signatures), better singleton handling and more permissive casts. — [Roblox DevForum: [General Release] Luau's New Type Solver](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991)
- A redesigned **non-strict mode is the default**. It focuses on "reporting only definite runtime errors" plus lint-style warnings (for example unknown globals). Code that *might* work at runtime is not flagged. Behind the modes is "one shared type inference pass between each mode", with a separate error reporter per mode. — [same post](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991)
- Known issues at release: in strict mode some errors are "larger and more verbose, or otherwise less clear"; the solver is sometimes slow or memory-hungry in Studio; there are some "blocked type" or "inference failed" errors; strict mode is sometimes too strict. Strict-mode projects stayed on the old solver by default, which remains available through 2026. The type checker runs only at edit time, so "None of these issues can affect your experience in production". — [same post](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991)
- The Luau team says the motivation is developer experience, not performance ("It's not performance motivated, but rather developer experience motivated"). — [same thread](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991)
- Adoption signal: over 325K sessions in the first week, per Roblox CEO David Baszucki. — [X post](https://x.com/DavidBaszucki/status/1993495887017132355)
- Beta history: beta announced September 2024. — [DevForum New Type Solver [Beta]](https://devforum.roblox.com/t/new-type-solver-beta/3155804)
- **Performance:** the Luau interpreter is "noticeably faster than Lua 5.x". "On some workloads it can match the performance of LuaJIT interpreter which is written in highly specialized assembly." — [luau.org/performance](https://luau.org/performance)
- **Native codegen:** optional, x64 and arm64, compiles **only selected functions** (functions or modules the user marks), with no tracing or automatic hot-spot JIT. "Luau JIT takes into account the type annotations present in the source code to specialize code paths." The VM has a native 3-component float `vector` type with first-class math ("effectively native 3-wide SIMD"). — [luau.org/performance](https://luau.org/performance)
- **Sandboxing:** compiler optimizations assume a sandboxed global environment. Even under deliberate type-safety violations by global injection, "the code will still be safely sandboxed". `loadstring`, `getfenv` and `setfenv` deoptimize the environments they touch. — [luau.org/performance](https://luau.org/performance)
- **luau-lsp** (MIT, active, nightly builds): diagnostics (type errors and lints), autocomplete, hover, signature help, go to definition and type definition, find references, rename, code actions, semantic tokens, **inlay hints**, call hierarchy, workspace symbols. For non-Roblox hosts you "provide global type definitions for a custom environment" through `luau-lsp.types.definitionFiles`. — [github.com/JohnnyMorganz/luau-lsp](https://github.com/JohnnyMorganz/luau-lsp)
- **mlua 0.12.2** (MIT, 2026-10-03) features: `lua55`, `lua54`, `luajit`, `luajit52`, `luau`, `luau-jit`, `luau-vector4`, `vendored`, `async`, `send`, `serde`, `macros`, `anyhow`, … `Lua::sandbox` is **Luau-only**. Luau is "built automatically in vendored mode". WASM works through `wasm32-unknown-emscripten` for all versions except JIT builds. Caveat from the README: "mlua does not provide absolute safety even without using `unsafe`". — [github.com/mlua-rs/mlua](https://github.com/mlua-rs/mlua); [docs.rs/mlua](https://docs.rs/mlua/latest/mlua/); crates.io API
- mlua is by far the most downloaded Lua binding: 7.8M downloads in total, 2.2M recent. — crates.io API, 2026-10-15

### Inferences
- **Lua 5.4/5.5 vs LuaJIT vs Luau for PH2D:**
  - Lua 5.4/5.5 is the reference language, with no type system.
  - LuaJIT is the fastest dynamic option, but it has no types, its language is frozen at about 5.1 plus extensions, and JIT is unavailable on some platforms. A background observation, not verified in this session: LuaJIT development activity is low.
  - Luau gives Lua 5.1 compatibility plus **types, a sandbox, an LSP, vectors and codegen guided by annotations**.
  - For an editor whose power depends on analysis, Luau dominates the other two. The switch is a Cargo feature in a dependency PH2D already has.
- The "one inference pass, several reporters" design maps well onto a dual audience: **artists get non-strict** (only definite errors), **programmers turn on `--!strict`**, and both get the same autocomplete and inlay hints.
- The "editor intelligence" work for PH2D shrinks to **generating Luau definition files (`.d.luau`) from the ECS/reflection registry** (components, resources, events) and hosting luau-lsp, or the Luau analysis library directly, inside the editor.
- Native codegen that specializes on annotations creates an incentive: typed code runs faster. GDScript has the same incentive (the Q1 typed opcodes).

### Gaps
- No independent 2025–26 benchmark comparing Luau (interpreter or codegen), LuaJIT and Lua 5.4 inside mlua was found. luau.org gives only qualitative claims.
- Not verified: whether mlua exposes Luau's **type checker / Analysis library** (as opposed to the VM and compiler). The README lists none. PH2D might need luau-lsp as an external process, or its own FFI to `Luau.Analysis`.
- The status of strict mode on the new solver after January 2026 (shared self types for `:` methods were planned for "early months" of 2026, per the forum) was not confirmed.

## Q3. Other Rust-embeddable scripting languages: maturity, tooling, performance, licence (Oct 2026)

### Takeaway
Only **Rhai** has broad adoption among pure-Rust options (13M downloads, MIT/Apache), and it is
**dynamically typed and slow-ish**, with no first-party LSP of note. Rune, Koto, Steel and Gluon are
alive but niche. Mun (statically typed with hot reload, the closest to the ideal on paper) **has not
released since 2022**. Piccolo is experimental. None of them matches Luau's combination of gradual types,
an LSP, a sandbox and speed.

### Cited Findings (versions and dates from the crates.io API, 2026-10-15)
| Crate | Latest | Published | Licence | Downloads total / recent | Notes |
|---|---|---|---|---|---|
| `rhai` | 1.26.1 | 2026-09-10 | MIT OR Apache-2.0 | 13.0M / 4.4M | Dynamic; JS/Rust-like syntax |
| `rune` | 0.14.2 | 2026-05-22 | MIT OR Apache-2.0 | 179k / 23k | Dynamic, async-first |
| `koto` | 0.16.1 | 2026-01-05 | MIT | 34k / 1.4k | Lightweight, built to embed in Rust apps |
| `mun_runtime` | 0.4.0 | **2022-12-12** | MIT OR Apache-2.0 | 11k / 0.3k | Static types, hot reload, LLVM |
| `steel-core` | 0.8.3 | 2026-08-20 | MIT OR Apache-2.0 | 48k / 20k | Scheme |
| `piccolo` | 0.3.3 | 2024-06-16 | MIT (repo also says CC0 option) | 95k / 62k | Stackless Lua VM in pure Rust |
| `gluon` | 0.18.4 | 2026-08-06 | MIT | 81k / 2k | Statically typed functional language |
| `dyon` | 0.51.3 | 2026-09-08 | Apache-2.0 OR MIT | 149k / 1k | Piston's game scripting language |
| `rune-languageserver` | 0.14.1 | 2025-09-14 | — | 21k / 31 | Rune LSP |
| `koto-ls` | 0.16.1 | 2025-07-31 | — | 1.6k / 12 | Koto LSP |

- **Rhai:** dynamic typing; tight integration with Rust functions, types, getters/setters and indexers; custom operators and syntax for DSLs; "Any panic is a bug. It never panics the host system"; limits on stack, data size and runaway scripts; passes Miri; no-std and WASM. Benchmark claim: "1 million iterations in 0.14 sec on a single-core, 2.6 GHz Linux VM". An experimental bytecode VM ("Rhai Grain") is said to be about 1.8–2.2× faster. — [rhai.rs book: Features](https://rhai.rs/book/about/features.html)
- Rhai drawback cited by users: one Hacker News commenter called it "very slow – even slower than Python". This is an anecdote, not a benchmark. — [HN thread](https://news.ycombinator.com/item?id=42738753)
- **Rune:** "An embeddable dynamic programming language for Rust". Structs and enums with data, pattern matching, try operators, macros, first-class async and generators, **hot reloading listed as a highlight**, a stack-based VM, reference counting. — [github.com/rune-rs/rune](https://github.com/rune-rs/rune)
- **Mun:** statically typed and AOT-compiled with LLVM; "First class hot-reloading"; has an LSP (`mun language-server`) and a VS Code extension; needs LLVM 14 to build; milestone releases are "unsupported". — [github.com/mun-lang/mun](https://github.com/mun-lang/mun). Last crates.io release 2022-12-12 (crates.io API).
- **Steel:** an embeddable Scheme on a bytecode VM, mostly R5RS; Racket-like modules; `syntax-rules` and `syntax-case` macros; **higher-order contracts** (no static types); ships the `forge` package manager and a language server; the API may change before 1.0. — [github.com/mattwparas/steel](https://github.com/mattwparas/steel)
- **Piccolo:** "experimental" stackless Lua VM in pure Rust; garbage collection through `gc-arena`; "fuel" bounds each step; warns of "frequent pre-1.0 API breakage"; much of the stdlib is missing; sandboxing of untrusted scripts is a goal "still ongoing". — [github.com/kyren/piccolo](https://github.com/kyren/piccolo)
- **Koto:** "A lightweight scripting language for Rust applications", docs at v0.16. — [koto.dev](https://koto.dev/)
- **Dyon** was designed for game engines, by the Piston developers. — [boringcactus survey (2020, dated)](https://www.boringcactus.com/2020/09/16/survey-of-rust-embeddable-scripting-languages.html)
- **bevy_mod_scripting 0.23.0** (2026-10-14, MIT/Apache) supports Lua 5.1–5.4, LuaJIT, **Luau** and Rhai. Rune is "temporarily on hold" after a rewrite. It calls itself "a work in progress and not yet optimized or complete"; has "Hot loading"; no WASM. — [github.com/makspll/bevy_mod_scripting](https://github.com/makspll/bevy_mod_scripting); crates.io API

### Inferences
- For PH2D's criteria (editor analyzability first), the **only pure-Rust candidate with real static types is Gluon** (niche, about 2k recent downloads) **or Mun** (dormant). Neither is a safe bet for a product the owner wants to finish.
- Rhai is the best *pure-Rust* choice when "no C toolchain" or WASM matters, but **dynamic typing leaves the editor nearly blind**. Its value is as a **config/DSL language**, not as the main game-scripting language.
- The Bevy ecosystem's leading scripting crate already treats Luau as first-class. Choosing Luau keeps PH2D aligned with the ecosystem PH2D's `bevy_ecs` comes from.
- Mojo, Zig and Nelua: not relevant to embedded user scripting in a Rust editor (not researched further, by design).

### Gaps
- No up-to-date (2025–26) cross-language benchmark of Rhai, Rune, Koto, Luau and Lua inside Rust was found. The boringcactus survey is from 2020.
- Not checked: Rhai third-party LSP status. A `rhai-lsp` crate does **not** exist on crates.io; the LSP may live elsewhere.
- Steel's rumoured use as the Helix editor's plugin language was **not confirmed** in this session.
- Gluon's LSP status was not checked.

## Q4. What engines chose, and the Rust hot-reload state of the art

### Takeaway
The big engines split into **one easy, integrated language plus one "serious" language**: GDScript plus
C# (Godot), Blueprints plus C++ (Unreal), Luau (Roblox), Lua (Defold), C# (Unity). Rust engines either
script in Rust and hot-reload it (Fyrox dylib reload, Dioxus Subsecond hot-patching) or bolt on Lua or
Rhai (bevy_mod_scripting). Rust hot-reload exists but is **experimental everywhere**.

### Cited Findings
- **Godot 4 discontinued VisualScript** (announced 2022-08-23): it "never gained traction", the path to improve it "was never clear", "the approach we took from the start was simply not the right one". Only **0.5 %** of more than 5,000 poll respondents used it as their main language. Most requesters were "yet to be Godot users". Engines whose visual scripting works (Unreal, GameMaker, Construct) "offer high level game features packaged together with the visual scripting solution", and Godot is general-purpose. **Visual shaders were not affected.** The code moved to `godotengine/godot-visual-script`. — [Godot blog](https://godotengine.org/article/godot-4-will-discontinue-visual-scripting); [GameFromScratch](https://gamefromscratch.com/visualscript-removed-from-godot-4/)
- **Verse (UEFN):** failure is control flow. "A failure context is a context where it is allowable to execute failable expressions"; "Any failure within a failure context will cause the entire context to fail"; "it is currently also necessary to add `<transacts>` when using `<decides>`". — [Epic: Failure in Verse](https://dev.epicgames.com/documentation/en-us/uefn/failure-in-verse)
- Verse speculative execution: "Verse takes advantage of failure context to let you try out actions without committing them"; "When an expression succeeds, the effect of the expression is committed…"; "If the expression fails, the effects of the expression are rolled back, as though the expression never happened." — [Epic: Speculative execution](https://dev.epicgames.com/documentation/fortnite/speculative-execution)
- **Fyrox** code hot reloading compiles the game plugin as a dylib (`-C prefer-dynamic=yes`, `dev-hot-reload` profile). Plugin state is serialized through the `Visit` trait before unload and restored after. Trait-object vtables can dangle. The book calls it "very new and experimental", relying on unsafe functionality, and advises falling back to static linking if behaviour looks wrong after a reload. — [Fyrox book: Hot reloading](https://fyrox-book.github.io/beginning/hot_reloading.html)
- **Subsecond** (Dioxus): Rust hot-patching that arrived in Dioxus 0.7 (2025-09-08). Calls go through a jump table of pointers to the newest function versions. It needs `dx serve --hotpatch` while experimental. It currently tracks only the "tip" crate: edits in dependencies, including sibling workspace crates, are not picked up. It works on web (WASM), desktop and mobile. The docs say **Bevy and Iced already have Subsecond integrations**. — [Dioxus 0.7 release](https://dioxuslabs.com/blog/release-070); [Dioxus hot-reload docs](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload); [docs.rs/subsecond](https://docs.rs/subsecond/)
- `subsecond` newest version 0.8.0-alpha.2 (2026-10-14), MIT OR Apache-2.0, 1.2M recent downloads. `hot-lib-reloader` 0.8.2 (2025-08-11, MIT). — crates.io API
- Roblox: Luau (Q2). Bevy: no official scripting language; the community crate bevy_mod_scripting is a WIP (Q3).

### Inferences
- Verse's failure contexts with transactional rollback are the most original 2020s answer to "how do game scripts avoid half-applied state": a `[]` call fails, and the effects roll back. PH2D can get a cheap analogue: **run each script call against the ECS as a transaction (command buffer), applied only on success**, plus `Option`/`Result`-shaped APIs. That gives most of the safety without inventing a language.
- Two hot-reload tiers fit PH2D: (a) **Luau scripts reload instantly** (just recompile the chunk); (b) Rust feature crates could use **Subsecond** for developers. Subsecond's "tip crate only" limit clashes with PH2D's many-crate workspace, so it is a dev convenience, not a user feature.
- Background knowledge, **not verified in this session**: Unity scripts in C#; Defold scripts in Lua on LuaJIT; Unreal Blueprints sit next to C++ and Verse is UEFN-only.

### Gaps
- No primary Epic design paper on Verse ("The Verse Calculus", 2023) was fetched. Only the user docs were read.
- Unity's 2025–26 CoreCLR/.NET modernization status and Godot C# tooling were not researched.
- No data was found on Blueprint usage share in Unreal projects.

## Q5. Language-design trends toward error prevention, and survey evidence

### Takeaway
The measurable trend is **toward types, especially types you do not have to write**. In 2025 TypeScript
became GitHub's #1 language by contributors (GitHub ties this partly to AI-generated code being checked
by types), 86 % of Python typing-survey respondents use hints "always/often", and Rust stays the most
admired language. Programmers increasingly expect the compiler and editor to *assist*: inference with
inlay hints, exhaustive matching, Option/Result instead of null or exceptions, and actionable error
messages.

### Cited Findings
- SO Survey 2025: **Rust is "yet again" the most admired language (72 %)**, followed by Gleam (70 %), Elixir (66 %) and Zig (64 %). — [SO Survey 2025, Technology](https://survey.stackoverflow.co/2025/technology/) (via search snippet; the fetched page was truncated before the Admired section)
- SO Survey 2025, most used (all respondents): Python 57.9 %, TypeScript 43.6 %, C# 27.8 %, Rust 14.8 %, Lua 9.2 %, GDScript 3.3 %. — [SO Survey 2025, Technology](https://survey.stackoverflow.co/2025/technology/)
- **GitHub Octoverse 2025:** TypeScript became the #1 language on GitHub by monthly contributors in August 2025 (about 2.64M), up 66 % year on year and ahead of Python by about 42k. GitHub ties this to AI-assisted coding: typed systems "help identify LLM-generated compile errors earlier". — [Visual Studio Magazine, 2025-10-31](https://visualstudiomagazine.com/articles/2025/10/31/typescript-tops-github-octoverse-as-ai-era-reshapes-language-choices.aspx); [Linuxiac](https://linuxiac.com/typescript-is-github-top-programming-language-for-2025/) (secondary coverage; the original report was not opened)
- **Python typing survey 2025** (JetBrains, Meta and the community; 1,241 responses): **86 %** use type hints "always" or "often". The share is highest at 5–10 years of experience (93 %) and lower for juniors (83 %) and 10+ years (80 %). Top motivations are code quality and flexibility. Selection bias is possible. — [Meta Engineering, 2025-12-22](https://engineering.fb.com/2025/12/22/developer-tools/python-typing-survey-2025-code-quality-flexibility-typing-adoption/); [InfoQ](https://infoq.com/news/2026/01/facebook-typed-python-survey/). Conflict: Meta's 2024 post said 88 %, while 2025 summaries call 86 % "unchanged". — [Meta 2024](https://engineering.fb.com/2024/12/09/developer-tools/typed-python-2024-survey-meta/)
- Unverified secondary claim: Rust-based Python type checkers (Pyrefly, ty, Zuban) together reach about 20 % of respondents, and mypy fell from 61 % to 58 %. — [ascii.co.uk summary](https://ascii.co.uk/news/article/news-20251224-b921a428/2025-python-typing-survey-86-adoption-rust-checkers-gain-gro)
- Gradual typing in game scripting: Luau (non-strict by default, strict opt-in, one shared inference) and GDScript (`:=` inference, opt-in warnings) both take the "optional types, inferred where possible" route. — [Roblox DevForum](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991); [Godot docs](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Verse treats failure as a typed effect (`<decides>`, `<transacts>`) with rollback, a concrete example of **effect annotations** in a mainstream game language. — [Epic: Failure in Verse](https://dev.epicgames.com/documentation/en-us/uefn/failure-in-verse)
- Steel offers higher-order **contracts** instead of static types (another route to error prevention). — [Steel](https://github.com/mattwparas/steel)

### Inferences
- The convergent "future programmer" expectations:
  1. **Types without ceremony:** inference plus inlay hints, not mandatory annotations.
  2. **Instant feedback:** diagnostics as you type, hot reload with state kept, a REPL or live values.
  3. **No null or exception surprises:** Option/Result or Verse-style failure; exhaustive matching on enums.
  4. **Safe refactors:** rename and find-references that are type-aware.
  5. **Error messages as advice.** This is the Rust/Elm tradition; background knowledge, not fetched in this session.
  6. **AI pair programming that is checked by types**, which is the Octoverse thesis.
- Luau gives 1, 2 (with the editor's help), 4 and partly 3 (optional types with `?`, refinements; **no exhaustive-match construct**, since Luau has no `match`). PH2D can make up some of 3 and 5 with an **API design** that returns `T?` and **editor-side lints** (for example "unhandled nil").
- Units of measure, refinement types and ownership are present in research and in Rust, but no mainstream *game scripting* language exposes them. They are not a requirement for PH2D's script language. PH2D could approximate units through distinct opaque types (`Seconds`, `Pixels`) in the generated definition files.

### Gaps
- Exact "Rust most admired N years running" count: the SO page excerpt said "yet again" without a year count; the 2025 admired table was not fetched directly.
- No primary sources were fetched on Elm/Rust error-message design, Ruby RBS adoption, effect systems (Koka, OCaml 5) or refinement types. These trends are noted from background knowledge only.

## Q6. Visual vs text scripting, and the recommendation inputs for PH2D

### Takeaway
Visual scripting pays off only when it **ships with high-level, domain-specific building blocks** (Unreal
Blueprints, visual shaders). A general-purpose visual language loses to an easy text language (Godot's
0.5 %). For PH2D, which already has node graphs (Motion Nodes), the evidence supports **domain node
graphs plus one text language**. On the evidence gathered, that language is **Luau via mlua**: the best
balance of editor intelligence (gradual types, luau-lsp, inlay hints), low error rate (non-strict mode
reports only definite errors, strict for programmers), simplicity (Lua syntax), speed (an interpreter
near LuaJIT, typed codegen), sandboxing, and zero new dependencies.

### Cited Findings
- Godot's VisualScript failed while visual shaders stayed. Visual scripting works when it comes with "high level game features packaged together". — [Godot blog](https://godotengine.org/article/godot-4-will-discontinue-visual-scripting)
- Luau: types, LSP, sandbox, performance (see Q2). — [luau.org/performance](https://luau.org/performance); [luau-lsp](https://github.com/JohnnyMorganz/luau-lsp); [DevForum GA](https://devforum.roblox.com/t/general-release-luau%E2%80%99s-new-type-solver/4084991)
- mlua already supports Luau and the sandbox: [mlua README](https://github.com/mlua-rs/mlua)

### Inferences (recommendation inputs)
| Criterion | Luau (mlua) | Lua 5.4/5.5 / LuaJIT (mlua) | Rhai | Rune | Mun | Rust + Subsecond |
|---|---|---|---|---|---|---|
| Editor analyzability | **High** (gradual types, type functions, luau-lsp) | Low (needs LuaLS-style annotations) | Low (dynamic) | Low–medium (dynamic, has LSP) | High (static) | Highest (rust-analyzer) |
| Error prevention | Non-strict = definite errors only; strict opt-in | Runtime only | Runtime; never panics host | Runtime | Compile-time | Compile-time |
| Performance | Interpreter near LuaJIT interp; selective native codegen | LuaJIT fastest; 5.4 slower | Slow-ish (AST walker) | VM, mid | Native (LLVM) | Native |
| Rust embed | Already in stack (feature flag) | Already in stack | Pure Rust, no C | Pure Rust | LLVM 14 dependency | n/a (is Rust) |
| Hot reload | Trivial (re-run chunk) | Trivial | Trivial | Built-in | First-class | Experimental, tip crate only |
| Sandbox | **`Lua::sandbox`** (Luau-only) | Manual | Strong limits | Some | No | None |
| Learnability (artists) | High | High | High | Medium (Rust-like) | Medium | Low |
| Maturity / licence | Roblox-scale; MIT | Mature; MIT | 1.26; MIT/Apache | 0.14; MIT/Apache | **Dormant since 2022** | Experimental; MIT/Apache |

- Proposed shape: **Luau for game and user scripts** (non-strict by default for artists, `--!strict` for programmers); **generated `.d.luau` definitions from PH2D's ECS reflection** so the editor knows every component and asset; **ECS-transactional script calls** (apply on success, a Verse-inspired idea) plus nil-returning APIs; **domain node graphs** (Motion Nodes) for visual users instead of a general visual language; **Rust plus Subsecond** only as an internal dev accelerator.
- Risk to flag: the editor's analysis likely needs the Luau *Analysis* library (C++) or luau-lsp as a sidecar. mlua exposes the VM and compiler, and **may not expose the type checker** (unverified, see Q2 gaps). This is the main integration question for the CodeEditor line.

### Gaps
- No user study comparing artist learnability of Lua vs GDScript vs visual graphs was found.
- No measurement exists yet of hosting luau-lsp, or embedding Luau.Analysis, inside a wgpu/vello editor. It needs a spike.
