# Luau static analysis (type check, autocomplete, hover, go-to-def, rename, lint) from a Rust app — state as of 2026-10-15

> Scope: PH2D runs user scripts through `mlua 0.12` (`crates/ph2d-script`, feature `luau`). Question: where does
> the *Analysis* half of Luau (Frontend / TypeChecker / Autocomplete / Linter) come from, and how do we drive it?
> Evidence = upstream repos/crates (via `gh api` and crates.io API on 2026-10-15) + **local reads and local
> measurements** (binaries downloaded to the session scratchpad, nothing installed into the repo).
> Measurement machine: 32 cores, `loadavg` between ~5 and ~7.5 during the timings (above the `~5` threshold of
> CLAUDE.md §5.0 ⇒ absolute ms are approximate; relative orders of magnitude are what count). All timings are the
> MINIMUM of 5 runs. Scripts: `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/af38a609-65da-48fc-a6db-15ebe72c095a/scratchpad/scripts/timeit.py` and `.../scratchpad/scripts/lspdrive.py` (scratchpad, ephemeral).

## Q1 — Does mlua 0.12 / `luau0-src` build or expose Luau's Analysis module?

### Takeaway
**No — definitively.** `luau0-src` ships and compiles only Ast, Bytecode, CodeGen (optional), Common, Compiler,
Config, Custom, Inliner, Require and VM; there is no `Analysis/` directory at all, and mlua's maintainer has
stated that Analysis is "outside of mlua scope" and would need a new crate. mlua gives us the VM + compiler only.

### Cited Findings
- Locally resolved versions for `ph2d-script`: `mlua v0.12.1 → mlua-sys v0.12.0`, build-dep `luau0-src 0.21.0+luau736` (i.e. Luau release 736) — `cargo tree -p ph2d-script` in the worktree; sources at `/home/enio/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/luau0-src-0.21.0+luau736/`. `ph2d-script` declares `mlua = { version = "0.12", features = ["luau"] }` — `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/crates/ph2d-script/Cargo.toml`.
- The vendored tree `.../luau0-src-0.21.0+luau736/luau/` contains exactly: `Ast Bytecode CodeGen Common Compiler Config Custom Inliner Require VM`. **No `Analysis`**. `find -iname '*TypeChecker*' -o -iname '*Frontend*' -o -iname '*Autocomplete*'` returns nothing; the only "analysis"-named files are CodeGen's `BytecodeAnalysis.cpp`/`IrAnalysis.cpp` (JIT internals) and `Config/src/LinterConfig.cpp` (lint *config parsing* only, no linter) — local `find`.
- The build script `.../luau0-src-0.21.0+luau736/src/lib.rs` compiles static libs `luauast`, `luaucodegen` (only if `enable_codegen`), `luaucommon`, `luaubytecode`, `luauinliner`, `luaucompiler`, `luauconfig`, `luaucustom`, `luaurequire`, `luauvm` — grep of `lib_name` in that file. No analysis library.
- Upstream `mlua-rs/luau-src-rs` main (as of 2026-10-15) has the same directory list (`Ast Bytecode CodeGen Common Compiler Config Custom Inliner Require VM`), and `src/lib.rs` contains zero occurrences of "analysis" — [luau-src-rs](https://github.com/mlua-rs/luau-src-rs) via `gh api repos/mlua-rs/luau-src-rs/contents/luau`.
- Latest releases: `mlua 0.12.2` (2026-10-03), `mlua-sys 0.13.0` (2026-10-03), `luau0-src 0.22.0+luau740` (2026-09-27); all MIT — [crates.io mlua](https://crates.io/crates/mlua), [crates.io luau0-src](https://crates.io/crates/luau0-src). Crate description: "Minimal sources of Luau and logic to build them." — local `Cargo.toml` of luau0-src.
- mlua's FFI surface for Luau (`mlua-sys-0.12.0/src/luau/`): `compat.rs lauxlib.rs luacodegen.rs luacode.rs lualib.rs luarequire.rs lua.rs` — VM, compiler (`luacode.h` C API), codegen and require only. The only "analysis" hit in mlua 0.12.1 is a doc comment about *heap memory analysis* (`mlua-0.12.1/src/luau/mod.rs:4`) — local grep.
- mlua issue #543 "Fragment autocomplete in luau" (2025-03-09, closed): asked whether mlua could expose Luau's fragment-autocomplete/analysis C++ APIs. Maintainer khvzak: *"This functionality is completely different/independent from the VM+Compiler implementation and outside of mlua scope. If something similar will be exposed in Rust, it likely requires a new crate and new set of libraries."* — [mlua#543](https://github.com/mlua-rs/mlua/issues/543).
- Note: `luau0-src` *does* compile the C++ **Ast** (parser) library, but it is a C++ API (`Luau/Parser.h`, `Ast.h`, `Cst.h`…) with no C wrapper and no Rust binding in mlua — local `ls .../luau/Ast/include/Luau/`; the only C APIs are `lua.h`/`lualib.h` and `luacode.h`.

### Inferences
- "mlua exposes the type checker" is false for every mlua release up to 0.12.2. The earlier research gap is closed: the risk is real, and Analysis must come from somewhere else (a separate C++ build, a child process, or a pure-Rust port).
- Because `luau0-src` already links Luau's Ast/Common/Config/VM statically into our binary, any *second* static copy of Luau (with Analysis) risks duplicate C++ symbols / ODR clashes if versions differ (see Q5).

### Gaps
- Did not check mlua's `master` branch for unreleased work after 0.12.2; issue search on `mlua-rs/mlua` for "analysis"/"type checker" found no open work item.

## Q2 — Rust crates that bind Luau Analysis, and pure-Rust Luau tooling (parse/lint/format/type-check)

### Takeaway
2026 produced a new crop: **one real FFI binding** (`luau-analyze` by cortesi: diagnostics + lint only, no
autocomplete/hover, macOS+Linux only, 0.0.1, 3 stars) and **four experimental pure-Rust ports** of Luau that
include the type checker (`luaur`, `ulua`, `ruau`, `luau-rs`/`luau-analysis`), none production-grade. The mature
pure-Rust Luau tools (full_moon, StyLua, selene, darklua) parse/format/lint **syntax** incl. type annotations but do
**not type-check**.

### Cited Findings
**FFI bindings to the official C++ Analysis**
- `luau-analyze` 0.0.1 (crates.io 2026-04-06, MIT; repo last push 2026-09-27, 3 stars): "In-process Luau type checking for Rust. Wraps the Luau `Analysis` frontend via a C shim so you can load host definitions, check Luau source, and get structured diagnostics without spawning an external process." Design: "Strict mode only, new solver only"; `require` resolution via `check_path`; virtual modules; per-check timeout + `CancellationToken`; "Supported platforms: macOS and Linux." — [cortesi/luau-analyze](https://github.com/cortesi/luau-analyze), [crates.io](https://crates.io/crates/luau-analyze).
- Its C shim (877 lines, `crates/luau-analyze/shim/analyze_shim.cpp`) exports `luau_checker_new/free`, `luau_checker_add_definitions` (calls `Frontend::loadDefinitionFile`), `luau_checker_check` (returns type errors + `lintResult.errors/warnings`), cancellation tokens, and `luau_extract_entrypoint_schema`; it constructs `Luau::Frontend` with `runLintChecks = true` and `setLuauSolverMode(SolverMode::New)`. Public Rust API: `Checker::{new, add_definitions, add_definitions_path, check, check_path, check_with_options}`, `Diagnostic`, `Severity`, `CheckResult`. **No autocomplete, hover, definition or rename** entry points — [shim source](https://github.com/cortesi/luau-analyze/blob/main/crates/luau-analyze/shim/analyze_shim.cpp), [lib.rs](https://github.com/cortesi/luau-analyze/blob/main/crates/luau-analyze/src/lib.rs).
- Its `build.rs` (repo HEAD) compiles Luau components `Common, Ast, VM, Compiler, Config, Analysis, Require` + 3 CLI files with `cc` (C++17) **"into a private shared library that the Rust crate loads at runtime. Keeping the analyzer in its own dynamic library isolates Luau symbols from other embedders such as `mlua`."** (uses `-fvisibility=hidden`) — [build.rs](https://github.com/cortesi/luau-analyze/blob/main/crates/luau-analyze/build.rs).
- `luau-analyzer-sys` 0.1.1 (crates.io 2026-05-24, MIT): "A high-performance, embedded Luau type-checking and analysis engine … bindings to the Luau analyzer" — but its declared repo `github.com/luks-luau/luau-analyzer-sys` returns **404** (2026-10-15); 158 downloads — [crates.io](https://crates.io/crates/luau-analyzer-sys). Unverifiable ⇒ not a candidate.

**Pure-Rust ports of Luau that include a type checker (all MIT, all experimental)**
- `luaur` 0.1.8 (2026-07-06; 43 stars; repo pushed 2026-09-24): "A faithful, line-for-line translation of Luau … from C++17 to Rust … compiler, virtual machine, and type checker … **5,347 ported unit tests pass, and all 293 upstream conformance scripts run byte-identically**" (against Luau commit `8f33df9`); ~205k lines C++ → ~420k lines Rust; runs on `wasm32-unknown-unknown`; crates include `luaur-analysis`, `luaur-analyze-cli`, `luaur-rt` (mlua-style API). LICENSE file: MIT (Jankiewicz + Roblox + Lua.org) — [pjankiewicz/luaur](https://github.com/pjankiewicz/luaur).
- `ulua` 0.1.7 (2026-10-08; 11 stars): idiomatic refactor of luaur ("Removed all `allow` attributes … Reduced `unsafe`"), "compiler, register VM, and type checker", WASM playground. crates.io says MIT but the GitHub repo has **no LICENSE file** (GitHub licence = null) — [webc-site/ulua](https://github.com/webc-site/ulua), [crates.io](https://crates.io/crates/ulua).
- `ruau` 0.4.0 (2026-08-17; 5 stars, MIT): "experimental pure Rust implementation of Luau … compatible parsing, type-checking, and a byte-code compatible VM. It does not include the legacy old type solver …"; tracks Luau `0.734`; crates include `ruau-typecheck`, `ruau-syntax`, `ruau-declaration` — [cortesi/ruau](https://github.com/cortesi/ruau).
- `luau-rs` (`luau` 0.739.0 + `luau-analysis` 0.739.0, 2026-09-22, MIT, 4 stars): "Rust implementation of Luau, including the parser, bytecode compiler … virtual machine …"; `luau-analysis` = "Luau type checking, linting, autocomplete, and analysis tooling" (src has `autocomplete/ checker/ lint/ solver/ frontend/ documentation.rs semantic.rs`). README: **"> [!CAUTION] luau-rs is highly experimental. It is not ready for production use … For production use, choose the official Luau implementation or, for Rust applications, mlua with its `luau` feature."** 75 downloads of `luau-analysis` — [luau-rs/luau](https://github.com/luau-rs/luau), [crates.io luau-analysis](https://crates.io/crates/luau-analysis).

**Mature pure-Rust syntax tooling (no type checking)**
- `full_moon` 3.0.0 (2026-10-01, MPL-2.0): Lua/Luau lossless parser; Cargo features `luau = ["roblox"]`, `lua52/53/54`, `luajit`, `cfxlua` — [Kampfkarren/full-moon Cargo.toml](https://github.com/Kampfkarren/full-moon/blob/main/full-moon/Cargo.toml), [crates.io](https://crates.io/crates/full_moon).
- `StyLua` v2.6.0 (2026-10-10, MPL-2.0) formatter, prebuilt for linux/macos/windows — [releases](https://github.com/JohnnyMorganz/StyLua/releases). `selene` 0.32.0 (2026-10-01, MPL-2.0; `selene-lib` 0.31.0 on crates.io) linter — [releases](https://github.com/Kampfkarren/selene/releases). `darklua` 0.19.0 (2026-06-10, MIT) code transformer — [crates.io](https://crates.io/crates/darklua).
- `luau-parser` 0.2.68 (msix29, 2025-05-22, MIT, 4 stars): lossless CST with error recovery; "does not guarantee that the code itself is error free" — [msix29/luau-parser](https://github.com/msix29/luau-parser). `luau-ast-rs` 0.1.29 (last 2023-08-02) — [crates.io](https://crates.io/crates/luau-ast-rs).

### Inferences
- **MPL-2.0** (full_moon/StyLua/selene) is file-level copyleft: usable as a dependency in a proprietary/commercial app as long as modifications to *those files* are published — check against PH2D's licence policy (SKILL_Stack HR rules) before taking it; MIT options exist for everything except a formatter.
- `luau-analyze` (cortesi) is the best *template* (≈900 lines of shim, symbol-isolation trick) but not a drop-in: it lacks every IDE feature beyond diagnostics, has no Windows support, and is a one-person 0.0.1 crate.
- The pure-Rust ports are interesting long-term (no C++ toolchain, WASM, one language), but none claims production readiness except luaur's test-suite claim, and they lag upstream (ruau tracks 734, luau-rs 739, luaur a fixed commit). Under PH2D §0.9 they could serve as *candidates measured against the oracle* (official `luau-analyze`/luau-lsp), not as the ground truth.

### Gaps
- Did not compile/run any of the pure-Rust ports; their autocomplete/hover completeness, latency and memory are unmeasured. luaur's 5,347-test claim is self-reported.
- The selene/full_moon Luau type-syntax coverage for the very newest syntax (e.g. `declare extern type`, user-defined type functions) was not verified.

## Q3 — luau-lsp: version, non-Roblox hosts, definitions files, latency, memory, transport, library vs process

### Takeaway
**luau-lsp 1.70.1 (2026-09-27, MIT)** is the only complete Luau IDE engine (diagnostics, completion, hover,
signature help, go-to-def, references, rename, semantic tokens, inlay hints, code actions…), ships binaries for
Linux x86_64/arm64, macOS and Windows, works for non-Roblox hosts with `--platform=standard` + a definitions file,
and — **measured locally** — answers completion/hover/definition/rename in ≤1 ms and re-checks a 7.5k-line file in
~30 ms with 15–62 MB RSS on the default (old) solver. Driving it as a stdio child process is the low-risk path.

### Cited Findings
- Releases: 1.70.1 (2026-09-27) and 1.70.0 (2026-09-20); assets `luau-lsp-linux-arm64.zip, luau-lsp-linux-x86_64.zip, luau-lsp-macos.zip, luau-lsp-win64.zip` (+ `Luau.rbxm`). Licence MIT; 551 stars; pushed 2026-10-15 — [luau-lsp releases](https://github.com/JohnnyMorganz/luau-lsp/releases). Release cadence ~monthly through 2026 (1.60.0 2026-01-04 … 1.70.1) — [CHANGELOG](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CHANGELOG.md).
- Supported features: Diagnostics (incl. type errors), Autocompletion, Hover, Signature Help, Go To Definition, Go To Type Definition, Find References, Document Link, Document Symbol, Color Provider, Rename, Semantic Tokens, Inlay Hints, Documentation Comments (Moonwave style), Code Actions, Workspace Symbols, Folding Range, Call Hierarchy — [README](https://github.com/JohnnyMorganz/luau-lsp#supported-features).
- Non-Roblox use: "The language server will start working immediately for general Luau code … To provide global type definitions for a custom environment, specify `luau-lsp.types.definitionFiles`. Corresponding documentation is configured using `luau-lsp.types.documentationFiles`." `luau-lsp.platform.type` enum `["standard","roblox"]`, default `"roblox"` — [README](https://github.com/JohnnyMorganz/luau-lsp), [editors/code/package.json](https://github.com/JohnnyMorganz/luau-lsp/blob/main/editors/code/package.json). Settings description: *"Note that definition file syntax is currently unstable and may change at any time"*.
- Maintainer on a custom-engine request (issue #1396, 2026-03-13): *"there is no single source of truth about how definition files work in Luau, apart from the source code itself. Definition files are technically still an 'unstable' part of the language and could change."*; points to `scripts/globalTypes.None.d.luau` as the example — [luau-lsp#1396](https://github.com/JohnnyMorganz/luau-lsp/issues/1396).
- CLI (local `luau-lsp --help`, 1.70.1): subcommands `analyze`, `lsp`, `require-graph`; `analyze` takes `--definitions @NAME=PATH` (also `--defs`, syntax `--definitions:@name=path`), `--platform`, `--settings PATH`, `--formatter`, `--annotate`, `--base-luaurc`, `--flag:KEY=VALUE`, `--no-flags-enabled` ("do not enable all Luau FFlags by default"), `--sourcemap`, `--ignore`.
- **Local measurement — definitions syntax changed**: a definitions file using `declare class Entity … end` is **rejected** by luau-lsp 1.70.1 ("SyntaxError: Expected ':' when parsing global variable declaration, got 'Entity'" → "Failed to read definitions file … Extended types will not be provided"); the same file rewritten as `declare extern type Entity with … end` loads and produces correct diagnostics (`Expected this to be 'number', but got 'string'`, `Key 'nmae' not found in external type 'Entity'`, `LocalUnused`). Upstream Luau's own sources/tests use `declare extern type` (`Analysis/src/EmbeddedBuiltinDefinitions.cpp`, `tests/TypeInfer.definitions.test.cpp`) and luau-lang PRs of 2026-09-08 keep evolving "extern types in definition files" (#2903, #2954) — [luau-lang/luau](https://github.com/luau-lang/luau). Test files in scratchpad `.../scratchpad/work/ph2d.d.luau`, `ph2d2.d.luau`, `small.luau`.
- Transport: stdio JSON-RPC (default); `--pipe=FILE` also mentioned in the changelog — [CHANGELOG](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CHANGELOG.md). Supports pull diagnostics (`textDocument/diagnostic`) and asks the client `workspace/configuration` — local driver.
- **Local measurement — LSP over stdio** (luau-lsp 1.70.1, `--definitions:@ph2d=…`, client answers `platform.type=standard`; driver `.../scratchpad/scripts/lspdrive.py`):

  | request | latency | notes |
  |---|---|---|
  | `initialize` | 1–2 ms | serverInfo Luau 1.70.1 |
  | open `small.luau` (6 lines) + pull diagnostics | 5–6 ms | 4 correct diagnostics; RSS 15.5 MB |
  | open `big.luau` (7 502 lines, 1 500 fns) + diagnostics | 27–29 ms | RSS 33 MB |
  | cached re-pull of diagnostics | 0 ms | |
  | completion after `ph2d.` | 0 ms | `vec2, time, spawn` (from definitions) |
  | completion after typing `e:` (didChange) | 0–1 ms | `translate, get_component` |
  | hover on `e` | 0 ms | ```local e: Entity``` |
  | go-to-definition | 0 ms | correct range |
  | rename | 0 ms | WorkspaceEdit |
  | edit `big.luau` → diagnostics | 27–32 ms per edit | full-text sync |
  | completion in `big.luau` after edit | 22 ms first, 3 ms after | fragment/incremental path |
  | final RSS | 62 MB | after all of the above |

- **Local measurement — `luau-lsp analyze` one-shot** (process start included): small file 4 ms / 12 MB; 7.5k-line file 28–31 ms / 31 MB (old solver).
- Memory complaints exist on big Roblox workspaces (#1220 "eating up all my memory", 2025-10-02; #1320, 2025-12-29, answered "quite an old version … (1.34.0) … latest is 1.59.0") — [#1220](https://github.com/JohnnyMorganz/luau-lsp/issues/1220), [#1320](https://github.com/JohnnyMorganz/luau-lsp/issues/1320). Changelog cites "Significant performance improvements to initial workspace indexing time (~3m to ~20s on 30,000 files)" — [CHANGELOG](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CHANGELOG.md).
- New since 1.65.0 (2026-04-06): "experimental Luau-based plugin system to allow you to perform source code transformations before Luau processes your code … custom require resolution" — [CHANGELOG](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CHANGELOG.md), [Plugin README](https://github.com/JohnnyMorganz/luau-lsp/blob/main/src/Plugin/README.md).
- In-process feasibility: the CMake build already splits a **static library target `Luau.LanguageServer`** from the executable `Luau.LanguageServer.CLI` (C++17, `LSP_VERSION "1.70.1"`); headers under `src/include/LSP/` include `LanguageServer.hpp, Workspace.hpp, Completion.hpp, Diagnostics.hpp, Refactoring.hpp, SemanticTokens.hpp, TextDocument.hpp, WorkspaceFileResolver.hpp, JsonRpc.hpp, Transport/` — [CMakeLists.txt](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CMakeLists.txt). Luau itself is a git submodule (`url = https://github.com/Roblox/luau.git`) — [.gitmodules](https://github.com/JohnnyMorganz/luau-lsp/blob/main/.gitmodules).

### Inferences
- The library target's API is the LSP protocol (JSON types, `Workspace` keyed by URIs) — embedding it in-process means C++ glue + `cxx`/C shim around an API that is internal and moves monthly. The stdio child process gives the same features with a stable, versioned protocol (LSP), crash isolation (it has had use-after-free crashes fixed in 2026, e.g. #1331), and zero symbol clash with mlua's Luau. Sub-millisecond request latency measured above means process IPC overhead is irrelevant for an editor.
- Our in-app editor can speak LSP to a bundled `luau-lsp` (per-OS binary, ~12.7 MB unpacked on Linux) with an in-memory "workspace": send `didOpen/didChange` with buffer text, answer `workspace/configuration` with `platform.type=standard`, and feed a PH2D-generated `ph2d.d.luau` via `--definitions`.
- Because definition-file syntax is officially unstable (and `declare class` already broke in 2026), the PH2D definitions file **must be generated from the Rust API registry and gated** (a test that runs the pinned luau-lsp/luau-analyze on it), and the luau-lsp version must be pinned.

### Gaps
- Startup latency/memory with a *large* definitions file (hundreds of PH2D API types) not measured. Windows/macOS behaviour not tested locally.
- No benchmark data found from upstream for per-keystroke latency on big projects; only issue anecdotes.

## Q4 — The official `luau-analyze` CLI: output format, modes, speed

### Takeaway
`luau-analyze` 0.742 (2026-10-10) type-checks + lints files with `--formatter=plain|gnu`, `--mode=strict|nonstrict`,
`--solver=new|old` (**default new** since ≤ 0.721), `--annotate`; but it has **no option to load a host definitions
file**, so it cannot know PH2D globals — usable only as an oracle/CI tool, not as the editor engine.

### Cited Findings
- Latest releases: 0.742 (2026-10-10), 0.741 (2026-10-02); assets `luau-macos.zip, luau-ubuntu.zip, luau-windows.zip, Luau.Web.js`; MIT — [luau releases](https://github.com/luau-lang/luau/releases).
- Help text (local binary 0.742 and [CLI/src/Analyze.cpp](https://github.com/luau-lang/luau/blob/master/CLI/src/Analyze.cpp)): "omitted: typecheck and lint input files"; "--annotate: typecheck input files and output source with type annotations"; "--formatter=plain: report analysis errors in Luacheck-compatible format"; "--formatter=gnu: report analysis errors in GNU-compatible format"; "--mode={strict|nonstrict} … (default: `nonstrict`)"; "--solver={new|old}: selects which typechecker to use (default: `new`)"; "--timetrace"; also `--fflags=` and `--logbase=`. `grep -i definition Analyze.cpp` only hits `#include "Luau/BuiltinDefinitions.h"` — no user-definitions option.
- The `--solver` option text "(defaults to the new solver)" and `SolverMode solverMode = Luau::SolverMode::New` are present at tag 0.721 (2026-05-15) and absent (pattern not found) at tags 0.693–0.716 — `gh api contents/CLI/src/Analyze.cpp?ref=<tag>`. Recent CLI commits: "Reject unknown options in luau-analyze (#2698)" 2026-08-26; "Report input files that could not be opened (#2723)" 2026-09-08 — [commits](https://github.com/luau-lang/luau/commits/master/CLI/src/Analyze.cpp).
- Output sample, `--formatter=gnu` (local): `./small.luau:2.11-2.14: TypeError: Unknown global 'ph2d'; consider assigning to it first` / `./small.luau:5.7-5.7: LocalUnused: Variable 's' is never used; prefix with '_' to silence`. Default format: `./big.luau(5001,5): TypeError: …`.
- **Local speed** (0.742, min of 5): 6-line file 3 ms / 12 MB; 7 502-line single module, new solver (default) **967 ms / 476 MB** and it reports `TypeError: Code is too complex to typecheck!`; same file `--solver=old` **28 ms / 27 MB**.
- The Analysis module upstream: 77 `.cpp` files, ~2.84 MB of C++ in `Analysis/src`; headers include `Frontend.h, Autocomplete.h, FragmentAutocomplete.h, Linter.h, TypeInfer.h, TypeChecker2.h, ConstraintSolver.h, FileResolver.h, ConfigResolver.h, Documentation.h, AstQuery.h` — [Analysis/include/Luau](https://github.com/luau-lang/luau/tree/master/Analysis/include/Luau).

### Inferences
- `luau-analyze` is the right **oracle** (PH2D §0.9): run it in CI over sample scripts and compare with whatever engine the editor uses. For host-aware checking in CI, `luau-lsp analyze --definitions` is the equivalent that accepts definitions.

### Gaps
- Exact release where luau-analyze's default switched to the new solver could be anywhere in 0.717–0.721 (only tags 0.716 and 0.721 were sampled); the grep pattern miss at older tags does not prove the old default.

## Q5 — Luau's new type solver: status in open-source Luau and in luau-lsp; latency implications

### Takeaway
The new solver is in open source and is the **default in `luau-analyze`**; on Roblox it went GA on 2025-11-20 for
nocheck/nonstrict (strict still opt-in), with the old solver kept "through 2026". luau-lsp still defaults to the
**old** solver (`enableNewSolver: false`). Measured locally, the new solver is ~5–30× slower and ~4–15× more
memory-hungry on larger single modules, and hits "too complex" on a 7.5k-line synthetic module.

### Cited Findings
- Roblox DevForum "General Release: Luau's New Type Solver" (2025-11-20): the solver leaves Studio Beta (beta toggle removed 2026-01-07); "enabled by default for users of nocheck and nonstrict modes"; strict-mode projects stay on the old solver by default and "can continue to opt in voluntarily"; *"we're committing to keep the old type inference engine available through 2026"*; the new solver *"is still slower than we'd like in some cases"* — [DevForum 4084991](https://devforum.roblox.com/t/general-release-luaus-new-type-solver/4084991).
- luau-lsp setting `luau-lsp.fflags.enableNewSolver` default **false** ("Enables the flags required for Luau's new type solver"); `luau-lsp.fflags.sync` default true (syncs Roblox's published `Luau*` FFlags) — [package.json](https://github.com/JohnnyMorganz/luau-lsp/blob/main/editors/code/package.json). 1.60.x: "`luau-lsp analyze` will now respect `luau-lsp.fflags.enableNewSolver` if enabled in the provided `--settings` file" — [CHANGELOG](https://github.com/JohnnyMorganz/luau-lsp/blob/main/CHANGELOG.md).
- cortesi/luau-analyze and ruau target the new solver only — [luau-analyze](https://github.com/cortesi/luau-analyze), [ruau](https://github.com/cortesi/ruau).
- **Local scaling measurement**, `luau-lsp analyze` (one-shot, includes process start), synthetic modules of N functions on one table, definitions loaded (old solver = `{"luau-lsp.platform.type":"standard"}`, new = `+ "luau-lsp.fflags.enableNewSolver": true`):

  | module | old solver | new solver |
  |---|---|---|
  | 40 fns / 202 lines | 5 ms / 11 MB | 7 ms / 13 MB |
  | 200 fns / 1 002 lines | 7 ms / 13 MB | 34 ms / 32 MB |
  | 600 fns / 3 002 lines | 15 ms / 18 MB | 218 ms / 116 MB |
  | 1 500 fns / 7 502 lines | 31 ms / 31 MB | 998 ms / 482 MB + "Code is too complex to typecheck!" |

  Diagnostics on the small test file were identical in content between solvers.
- Beta-period reports of high memory: DevForum thread "New Luau Type Solver causing extremely high memory usage" — [DevForum 4110630](https://devforum.roblox.com/t/new-luau-type-solver-causing-extremely-high-memory-usage/4110630) (title only; not read in full).

### Inferences
- For typical PH2D user scripts (tens to a few hundred lines) both solvers are interactive (< 10 ms). The new solver's superlinear cost matters only for very large single modules; keep the solver choice a runtime setting (it is an FFlag/setting in both luau-lsp and the C++ Frontend), default to what luau-lsp defaults to, and measure on real PH2D scripts before switching. Since the old solver is slated for removal after 2026, plan for the new one.
- The synthetic module (1 500 methods on one table) is a stress case, not representative; it shows the slope, not the typical cost.

### Gaps
- No official Luau statement found on when the old solver code will be deleted from the open-source repo.
- Did not measure fragment-autocomplete latency under the new solver specifically.

## Q6 — Building Luau Analysis ourselves (cc/cxx): API surface, effort, compile time, symbol clash with mlua

### Takeaway
Feasible and measured: a `cc`-based build of Common+Ast+VM+Compiler+Config+Analysis took **28.7 s** (release,
clean, 16-core cap) and produced a 47 MB rlib. The C++ surface needed is `Frontend` + `FileResolver` +
`ConfigResolver` + `loadDefinitionFile` + `autocomplete`/`FragmentAutocomplete` + `Linter` + `AstQuery`. The hard
part is not compiling, it is (a) re-implementing what luau-lsp already does for hover/definition/rename/docs and
(b) avoiding a second copy of Luau symbols next to mlua's `luau0-src`.

### Cited Findings
- Local build: crates.io `luau-analyze-0.0.1` (bundled Luau sources: `Analysis Ast Common Compiler Config VM`, 74 files in `Analysis/src`) built with `cargo build --release` via `scripts/ph2d-run.sh` (CPU ≤ 1600 % of 32 cores): **"Finished `release` profile [optimized] target(s) in 28.74s"**; `libluau_analyze.rlib` = 46 759 348 bytes; loadavg 4.6 → 12.8 during build. Build dir `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/af38a609-65da-48fc-a6db-15ebe72c095a/scratchpad/la_build/`.
- The C API a shim needs to expose is small for diagnostics (cortesi's shim: ~877 lines for checker/definitions/check/cancellation) — [analyze_shim.cpp](https://github.com/cortesi/luau-analyze/blob/main/crates/luau-analyze/shim/analyze_shim.cpp). Upstream headers for the IDE features exist: `Autocomplete.h`, `FragmentAutocomplete.h`, `AstQuery.h` (find node/binding at position), `Documentation.h`, `Linter.h`, `FileResolver.h`, `ConfigResolver.h`, `Frontend.h` — [Analysis/include/Luau](https://github.com/luau-lang/luau/tree/master/Analysis/include/Luau).
- Symbol isolation: cortesi moved from a static link (0.0.1 on crates.io produces an rlib) to "a private shared library … isolates Luau symbols from other embedders such as `mlua`" with `-fvisibility=hidden` — [build.rs](https://github.com/cortesi/luau-analyze/blob/main/crates/luau-analyze/build.rs).
- Analysis depends on VM too (cortesi's component list includes `VM`), and luau0-src already links `luauast`, `luaucommon`, `luauconfig`, `luauvm`… statically into PH2D — local `luau0-src-0.21.0+luau736/src/lib.rs`.
- Upstream Luau syncs roughly weekly (0.741 2026-10-02, 0.742 2026-10-10) and `luau0-src` follows a few versions behind (luau736 → luau740) — [luau releases](https://github.com/luau-lang/luau/releases), [crates.io luau0-src](https://crates.io/crates/luau0-src).
- How Lune (Rust, mlua-based Luau runtime, v0.11.0 2026-10-11, MPL-2.0) handles intelligence: it does not embed Analysis; its README advertises documentation "directly in your editor", i.e. via luau-lsp type definitions — [lune-org/lune](https://github.com/lune-org/lune). (Interpretation of the README line; Lune's exact mechanism not re-verified in source.)

### Inferences
- Two clean ways to avoid the ODR/duplicate-symbol hazard: (1) build Analysis into its own `cdylib`/shared object with hidden visibility (cortesi's approach; costs a per-OS shared library to ship), or (2) vendor *only* `Analysis/` from the **exact same Luau tag** as `luau0-src` (currently 736/740) and link it against `luau0-src`'s already-built `luauast/luaucommon/luauconfig/luauvm` — no duplicates, but every mlua bump forces an Analysis bump in lock-step. Mixing two different Luau versions statically is the one thing not to do.
- Effort estimate (inference, not sourced): diagnostics-only shim ≈ cortesi's ~900 lines (days); full parity with luau-lsp's completion item shaping, hover markdown, documentation comments, rename, go-to-def across requires, semantic tokens ≈ re-writing a large part of luau-lsp's `Completion/Refactoring/Workspace` code (weeks–months), and then tracking upstream's monthly changes. The child-process route gets all of it for free.
- 28.7 s of extra clean build (C++, cached by cargo afterwards) is acceptable for PH2D, but it lands in every clean CI build of the crate that owns it.

### Gaps
- Did not try linking Analysis next to mlua in one binary to observe the actual clash (link error vs silent ODR). Windows (MSVC) build of the Analysis shim not tried.
- No public write-up found from a project that embedded the full Luau IDE stack (autocomplete+hover) in-process in Rust.

## Q7 — tree-sitter grammars for Luau

### Takeaway
Use `tree-sitter-grammars/tree-sitter-luau` (MIT, queries for highlights/locals/folds/indents/injections, repo
active 2026-09) for instant syntax highlighting/folding independent of the analyzer; the crates.io release is old
(1.2.0, 2024-12-22) so pin a git rev if newer syntax matters.

### Cited Findings
- `tree-sitter-grammars/tree-sitter-luau` (same repo as `amaanq/tree-sitter-luau`): MIT, pushed 2026-09-13, `queries/` = `folds.scm, highlights.scm, indents.scm, injections.scm, locals.scm` — [repo](https://github.com/tree-sitter-grammars/tree-sitter-luau). Crate `tree-sitter-luau` 1.2.0 (2024-12-22, MIT, 57 682 downloads) — [crates.io](https://crates.io/crates/tree-sitter-luau).
- `polychromatist/tree-sitter-luau`: MIT, 21 stars, last push 2025-12-08 — [repo](https://github.com/polychromatist/tree-sitter-luau).
- `tree-sitter-luau-fork` 0.1.3 (2024-07-28) and `luau-ast-rs-grammar` 0.0.2 (2023) are stale — crates.io search.

### Inferences
- Layering: tree-sitter (every keystroke, sub-ms, error-tolerant) for colour/folds/indent; Luau Analysis (luau-lsp) for semantics (types, completion, hover, rename, semantic tokens overriding tree-sitter colours).

### Gaps
- Coverage of the newest Luau syntax (`declare extern type`, user-defined type functions, attributes) by the grammar not verified.

## Q8 — Options ranked and recommended path

### Takeaway
**Recommended: bundle the official `luau-lsp` binary (pinned version, MIT) and drive it from Rust as an LSP stdio
child process with a generated `ph2d.d.luau` definitions file; tree-sitter-luau for syntax; official
`luau-analyze`/`luau-lsp analyze` as the CI oracle.** Keep an in-process C++ Analysis shim as plan B and the
pure-Rust ports as a watch item.

### Cited Findings
- Full feature set, cross-platform binaries and measured ≤1 ms request latency / 15–62 MB RSS for luau-lsp — see Q3 (local measurements; [luau-lsp](https://github.com/JohnnyMorganz/luau-lsp)).
- mlua cannot supply Analysis — Q1 ([mlua#543](https://github.com/mlua-rs/mlua/issues/543)).
- The only FFI crate is diagnostics-only, macOS/Linux-only — Q2 ([cortesi/luau-analyze](https://github.com/cortesi/luau-analyze)).
- Pure-Rust ports self-describe as experimental — Q2 ([luau-rs](https://github.com/luau-rs/luau), [ruau](https://github.com/cortesi/ruau)).

### Inferences (ranking)
1. **luau-lsp as child process (stdio LSP)** — MIT; mature (monthly releases, 551 stars, full LSP feature list); latency measured sub-ms per request, ~30 ms re-check for 7.5k lines; zero symbol clash with mlua; crash-isolated; costs: shipping a ~12–13 MB per-OS binary, an LSP client in Rust (e.g. `lsp-types` + JSON-RPC over stdio), unstable definitions syntax (generate + gate it), Roblox-flavoured defaults to override (`platform.type=standard`, consider `fflags.sync=false` for reproducibility). **Best now.**
2. **In-process C++ Analysis via own shim (cc + C ABI, hidden-visibility shared lib or same-tag link with luau0-src)** — MIT; 28.7 s build; full control and no extra process; but diagnostics are easy and everything else (completion shaping, hover, rename, docs) must be re-written and kept in lock-step with upstream. **Plan B / later optimisation**, only if a product need (e.g. type info inside node graphs, offline WASM) justifies it.
3. **`luau-analyze` crate (cortesi)** — MIT; good for a "check on save" gate in-process, but no IDE features, no Windows, 0.0.1. **Template, not dependency.**
4. **Pure-Rust ports (luaur / ulua / ruau / luau-rs)** — MIT (ulua repo lacks LICENSE file); no C++; WASM-capable; experimental, lagging upstream, unmeasured. **Watch; re-evaluate with an oracle comparison against luau-lsp in 6–12 months.**
5. **full_moon / selene / StyLua** — MPL-2.0; syntax-level only (format, lint w/o types). Useful for a formatter (StyLua) if MPL is acceptable; not an answer to type intelligence.
6. **`luau-analyze` CLI** — no host definitions; oracle/CI only.

### Gaps
- Product decision needed (owner): is shipping an extra executable per OS acceptable for the editor? (affects option 1 vs 2).
- Not verified: luau-lsp behaviour inside a sandboxed/no-filesystem setup with purely in-memory documents for `require` between user scripts (likely needs files on disk or the plugin system for custom require resolution).
