# Rust code-editor architecture: state of the art (October 2026)

> Research notes for the PH2D integrated script/code editor (Lua via mlua 0.12; Vello + Parley UI; bevy_ecs).
> Collected 2026-10-15. Crate versions, dates and licences come from the crates.io API (`/api/v1/crates/<name>`,
> queried 2026-10-15) and from GitHub `Cargo.toml` and the repository API, unless another source is given. Anything
> not verified this session is marked **[unverified]** and also appears under Gaps.

## Q1. Text buffer: rope vs piece table vs gap buffer, and the Rust crates

### Takeaway
A B-tree rope with chunked UTF-8 leaves, `Arc`-shared nodes (O(1) snapshots for background threads) and summarized
secondary metrics (lines, UTF-16, chars) is the consensus design. Zed, Helix, Lapce/xi and crop all use it. VS Code's
piece tree solves the same problem in a GC'd language. For PH2D the choice is between **ropey** (1.6.1 stable;
2.0.0-beta.1 is byte-indexed and has had no final release since 2025-08) and **crop** (byte-indexed, fastest in published
benchmarks, MIT). Zed's `sum_tree` is Apache-2.0, but Zed's `rope` crate is **GPL-3.0**.

### Cited Findings
- **Zed rope = `SumTree<Chunk>`**, a B+ tree. Each leaf `Chunk` is an inline `ArrayString` of up to 2×`CHUNK_BASE` bytes
  (`CHUNK_BASE` = 64 in release, so up to 128). Internal nodes store the sum of their children's summaries. —
  [Zed decoded: Rope & SumTree](https://zed.dev/blog/zed-decoded-rope-sumtree)
- Zed's `TextSummary` aggregates: `len` (UTF-8 bytes), `len_utf16`, `lines` (row/column `Point`), first and last line
  char counts, last-line UTF-16 length, `longest_row` and `longest_row_chars`. A cursor seeks along one dimension and
  reads another in O(log n): offset↔point, and UTF-8↔UTF-16 for LSP. — [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)
- Zed nodes are `Arc`-shared. A snapshot only bumps reference counts, and snapshots go to a background thread that
  re-parses with tree-sitter **on every edit**. Concurrent access is "a hard requirement" ("it matters more than any of
  that other stuff"). — [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)
- Zed uses SumTree in more than 20 places: the DisplayMap layers (folds, inlays, wrapping, blocks), file lists, git
  blame and diagnostics. The rope is one instance of a general "summarized sequence" structure. —
  [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)
- Gap buffers win on local edits and degrade on non-local ones. The ropey author calls ropes a "jack of all trades"
  that are "always solidly good" at O(log N). — [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)
- **Licences in Zed's tree:** `crates/sum_tree` is **Apache-2.0** (`publish = false`). `crates/rope`, `crates/text`,
  `crates/editor`, `crates/language` and `crates/multi_buffer` are **GPL-3.0-or-later**. `crates/gpui` is **Apache-2.0**
  (published to crates.io as 0.2.2, 2025-10-22). The repository as a whole is "GPL-3.0-or-later" with Apache-2.0
  components "where marked". — [zed Cargo.toml files](https://github.com/zed-industries/zed/tree/main/crates),
  [zed README](https://github.com/zed-industries/zed), [crates.io gpui](https://crates.io/crates/gpui)
- **ropey**: stable 1.6.1 (2023-10-18), latest pre-release **2.0.0-beta.1 (2025-08-02)**, MIT OR Apache-2.0, about
  3.76 M recent downloads. 1.x indexes by `char`, converts to and from UTF-16, tracks all Unicode line endings including
  CRLF, and has had `byte_slice()` since 1.4. — [crates.io ropey](https://crates.io/crates/ropey),
  [ropey CHANGELOG](https://github.com/cessen/ropey/blob/master/CHANGELOG.md), [ropey README](https://github.com/cessen/ropey)
- **Ropey 2.0 beta is byte-indexed** ("uses byte indices into that utf8 data", like `String`). Lines are line *breaks*
  (`"Hello\nworld\n"` has 3 lines), and line APIs take a `LineType`. Char, UTF-16 (`metric_utf16`) and LF / LF_CR /
  Unicode line metrics are opt-in features. `simd` is on by default. The docs give no date for 2.0 final. —
  [docs.rs ropey 2.0.0-beta.1](https://docs.rs/ropey/2.0.0-beta.1/ropey/)
- Ropey's claims: edits take "single-digit microseconds" even on multi-GB texts. It sustains more than 1.8 M incoherent
  and more than 3.3 M coherent insertions per second while building a ~100 MB text. A 100 MB file uses ~110 MB in memory.
  Clones are O(1) (8 bytes). — [ropey README](https://github.com/cessen/ropey)
- **crop** 0.4.3 (2025-04-25), MIT. B-tree rope with UTF-8 **byte** offsets; only LF and CRLF are line breaks. Clones
  are thread-safe CoW (16 bytes), designed for "snapshotting a rope to a background thread". —
  [crop README](https://github.com/nomad/crop), [crates.io crop](https://crates.io/crates/crop)
- **crop benchmark** (crdt-benchmarks editing traces, 2018 MBP i7, times in ms):

  | Trace | crop | jumprope | ropey | String |
  |---|---|---|---|---|
  | automerge-paper | 12.39 | 12.52 | 44.14 | 108.57 |
  | rustcode | 2.67 | 2.86 | 7.96 | 13.40 |
  | sveltecomponent | 0.95 | 1.08 | 3.65 | 1.22 |
  | seph-blog1 | 6.47 | 6.94 | 23.46 | 22.26 |

  — [crop README](https://github.com/nomad/crop). These are ropey **1.x** numbers. Ropey 2.0 is not in the table.
- **jumprope** 1.1.2 (2023-05-15), ISC OR Apache-2.0. Char offsets, and it reallocates the text on clone, so snapshots
  are not cheap. — [crates.io jumprope](https://crates.io/crates/jumprope), [crop README](https://github.com/nomad/crop)
- **xi-rope** 0.3.0 (2019-06-29, unmaintained), Apache-2.0. The Lapce fork **lapce-xi-rope** 0.4.0 (2025-12-19) is
  Apache-2.0. — [crates.io xi-rope](https://crates.io/crates/xi-rope), [crates.io lapce-xi-rope](https://crates.io/crates/lapce-xi-rope)
- **VS Code piece tree** (2018): the old line array needed ~600 MB for a 35 MB / 13.7 M-line file ("roughly 20 times
  the initial file size"). The replacement is a piece table plus a red-black tree with subtree lengths and line-break
  counts in each node, giving O(log n) offset and line lookup. It uses 64 KB chunk buffers (V8's 256 MB string limit
  broke a 500 MB file). Memory after load is "very close to the original file size". `getLineContent` is the
  "Achilles heel". Referencing append-only buffers instead of copying made operations "three times faster". A C++ port
  was abandoned because of JS↔C++ crossings. —
  [VS Code: Text Buffer Reimplementation](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation)
- xi retrospective: the rope "has excellent worst-case performance". Its complexity is contained behind a clean
  interface, and it is "a fairly niche data structure" for large texts with many small edits. —
  [Raph Levien, xi-editor retrospective (2020)](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)

### Inferences
- **Indexing costs.** Use **UTF-8 byte offsets** as the canonical position, because tree-sitter, parley (`raw_selection`
  and `raw_compose` are byte indices) and Rust `str` all speak bytes. Keep **line** and **UTF-16** as summarized metrics
  (needed for LSP / emmylua positions and AccessKit). **Graphemes** cannot be summarized, since segmentation needs local
  context; compute them on demand at cursor-motion time over a small window (parley 0.12 already refuses to place the
  caret inside a cluster). This matches Zed's summary set and ropey 2.0's byte-first model.
- For PH2D the realistic picks are:
  1. **crop**: byte-indexed, fastest in published traces, MIT, cheap snapshots. Risk: one maintainer, last release
     2025-04, and no UTF-16 metric that I verified.
  2. **ropey 1.6.1**: stable and in production in Helix, but char-indexed. Ropey 2.0 matches the target model, but a
     beta that has been unreleased for 14 months is a dependency risk.
  3. **Own SumTree-style rope** modelled on Zed's (Apache-2.0 `sum_tree` can be studied or vendored; the GPL `rope`
     crate cannot). This gives one generic structure to reuse for fold maps, inlay maps, wrap maps and diagnostics, as
     Zed does.

  The "best engine underneath" reading of the brief favours option 3, a generic summarized B-tree, because PH2D will
  need the same structure for display maps anyway. Option 1 is the fast path.
- A piece table only makes sense in a GC'd runtime or for undo-by-append. A rope with Arc snapshots plus a separate undo
  log gives the same benefits in Rust.

### Gaps
- No independent benchmark of **ropey 2.0 beta vs crop** was found. crop's table uses ropey 1.x.
- Zed's `TREE_BASE` value and any published Zed rope micro-benchmarks: the blog gives none, and its `rope_benchmark`
  crate exists but I did not read its results.
- Whether crop has a UTF-16 metric was not verified.
- Ropey 2.0 final ETA: not stated anywhere I looked.

## Q2. Incremental parsing: tree-sitter vs red-green trees (rowan) vs Lezer; error recovery

### Takeaway
Use **tree-sitter (0.27.1, MIT)** with **tree-sitter-lua (0.5.0, MIT)** for highlighting, folding, textobjects,
indentation and injections: reparsing is incremental and trees are cheap to copy across threads. Semantic work
(rename, refactor, type-aware completion) belongs to a lossless red-green tree (rowan-style), which is what
rust-analyzer and emmylua do. The two layers coexist, as in Zed (tree-sitter for syntax, LSP for semantics).

### Cited Findings
- **tree-sitter** 0.27.1 (2026-10-08), MIT, ~17.9 M recent downloads. tree-sitter-highlight 0.27.1 and
  tree-sitter-language 0.1.9 are both MIT. — [crates.io tree-sitter](https://crates.io/crates/tree-sitter),
  [crates.io tree-sitter-highlight](https://crates.io/crates/tree-sitter-highlight)
- **tree-sitter-lua** 0.5.0 (2026-02-26), MIT, maintained in the `tree-sitter-grammars` org. —
  [crates.io tree-sitter-lua](https://crates.io/crates/tree-sitter-lua)
- **Incremental edit protocol:** call `ts_tree_edit(InputEdit{start, old_end, new_end, in bytes and points})`, then
  parse again with the old tree. The new tree shares structure with the old one. Held nodes need `ts_node_edit`. —
  [tree-sitter: Advanced Parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- **Injections / multi-language:** `set_included_ranges` parses disjoint ranges with another grammar (for example
  Lua-in-something). Tree-sitter leaves the interplay between languages to the application. —
  [tree-sitter docs](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- **Concurrency:** `ts_tree_copy` is an atomic ref-count bump. Copies can be queried or edited on another thread, but a
  single `TSTree` is not thread-safe. — [tree-sitter docs](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- Zed snapshots the rope and reparses with tree-sitter on a background thread on every edit. —
  [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)
- Helix's `Syntax` wraps tree-sitter for highlighting and related features. —
  [Helix architecture.md](https://github.com/helix-editor/helix/blob/master/docs/architecture.md)
- **xi retrospective:** regex/TextMate highlighting is slow and imprecise (syntect is ~2,500× slower than pulldown-cmark
  on Markdown). xi highlighted asynchronously and incrementally, with OT-style updates so stale results never colour the
  wrong span. Raph recommended Lezer for future work. —
  [xi retrospective](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)
- **syntect** 5.3.0 (2025-09-27), MIT. It is the TextMate-grammar (regex) alternative. — [crates.io syntect](https://crates.io/crates/syntect)
- **rust-analyzer's parser** is hand-written recursive descent. It emits a flat event stream assembled into `rowan`
  trees, and "parsing never fails": it returns `(T, Vec<Error>)` and the trees are incomplete by design. The `syntax`
  crate is independent of salsa and LSP, and the tree "is a value type". —
  [rust-analyzer architecture.md](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- **Red-green tree crates:** `rowan` 0.18.0-alpha.1 (2026-09-14; stable 0.16.x line), MIT OR Apache-2.0, ~5.5 M recent
  downloads. `cstree` 0.14.0 (2026-04-22), MIT OR Apache-2.0, a thread-safe rowan derivative. `biome_rowan` 0.7.0
  (2026-09-19), MIT OR Apache-2.0. `ra_ap_syntax` 0.0.358 (2026-10-12), MIT OR Apache-2.0. —
  [rowan](https://crates.io/crates/rowan), [cstree](https://crates.io/crates/cstree),
  [biome_rowan](https://crates.io/crates/biome_rowan), [ra_ap_syntax](https://crates.io/crates/ra_ap_syntax)
- **Lua-specific full-fidelity parsers:** `full_moon` 3.0.0 (2026-10-01), lossless Lua AST, **MPL-2.0**. `emmylua_parser`
  0.29.1 (2026-08-14), **MIT**. — [crates.io full_moon](https://crates.io/crates/full_moon),
  [crates.io emmylua_parser](https://crates.io/crates/emmylua_parser)
- **Formatters:** `topiary-core` 0.8.0 (2026-10-03, MIT), query-driven on tree-sitter. `stylua` 2.6.0 (2026-10-10,
  **MPL-2.0**), the de facto Lua formatter, built on full_moon. emmylua ships `emmylua_formatter` (`luafmt`).
  `arborium` 2.18.2 (MIT OR Apache-2.0) bundles tree-sitter grammars. — [topiary-core](https://crates.io/crates/topiary-core),
  [stylua](https://crates.io/crates/stylua), [emmylua repo](https://github.com/EmmyLuaLs/emmylua-analyzer-rust),
  [arborium](https://crates.io/crates/arborium)

### Inferences
- **Two-tree design:** tree-sitter on a background thread for anything visual (highlight, fold, bracket match,
  structural selection, textobjects via `textobjects.scm`, indent queries), and an error-tolerant lossless red-green
  tree for analysis and refactoring. Do not try to make one serve both: tree-sitter CSTs are not designed for
  semantic rewrites, and red-green trees are not query-driven highlighters.
- **Error recovery for incomplete Lua while typing:** tree-sitter produces ERROR/MISSING nodes and keeps highlighting
  stable. A rowan-style parser following rust-analyzer's "never fails, returns errors on the side" rule gives
  full-fidelity trees for half-typed code.
- **Licence:** full_moon and stylua are **MPL-2.0**, which is file-level copyleft. Using them unmodified as dependencies
  is fine. Modifying their files means publishing those files. The emmylua crates are MIT, which is easier.

### Gaps
- No published numbers on tree-sitter incremental reparse cost were found in the docs fetched. Zed's
  reparse-on-every-edit is qualitative evidence only.
- The quality of tree-sitter-lua's `highlights.scm` / `textobjects.scm` and its coverage of Lua 5.4/5.5 were not checked.
- Whether `emmylua_parser` is rowan-based or lossless is **[unverified]**: the README does not say.
- Lezer is JavaScript only. I found no maintained Rust port (the crates.io lookup for `lezer-rs` returned nothing).

## Q3. Semantic analysis: salsa, LSP in-process or out, Lua analyzers

### Takeaway
rust-analyzer's architecture is the model to follow. A query-based incremental core (salsa 0.28.5, MIT/Apache-2.0)
exposes a **plain Rust API of POD types plus immutable snapshots plus cancellation**, and LSP is only a thin adapter
crate. For PH2D's embedded Lua editor, call the analysis engine **in-process through its Rust API**, not via LSP
JSON-RPC. The leading candidate is **emmylua-analyzer-rust (MIT, Lua 5.1–5.5 + LuaJIT)**. Its crates are on crates.io,
but no embedding-API stability is promised.

### Cited Findings
- **salsa** 0.28.5 (2026-09-24), Apache-2.0 OR MIT, ~2.66 M recent downloads. Model: inputs plus memoized pure
  functions, with salsa deciding when a memoized result can be reused. — [crates.io salsa](https://crates.io/crates/salsa),
  [salsa-rs/salsa](https://github.com/salsa-rs/salsa)
- **rust-analyzer layering:** `syntax` (rowan) → `base-db` (salsa inputs, opaque `FileId`s, no build-system knowledge) →
  `hir-*` (macro expansion, name resolution, types) → `ide` (features) → `rust-analyzer` ("the only crate that knows
  about LSP and JSON"). —
  [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- **`ide` is the API boundary:** "the `ide` crate's API is build out of POD types with public fields", in editor terms
  (offsets, labels). `AnalysisHost::apply_change` is transactional, and `Analysis` is an immutable snapshot. —
  [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- **Cancellation:** a change bumps salsa's global revision. Readers that notice it unwind with a special panic, which
  `ide` turns into `Result<T, Cancelled>`. This needs `panic = "unwind"`. —
  [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- **Core invariant:** "typing inside a function's body never invalidates global derived data". Core crates do no I/O. —
  [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- **emmylua-analyzer-rust:** MIT, ~520 stars. Crates: `emmylua_parser`, `emmylua_code_analysis` (the analysis library
  behind the LS), `emmylua_ls`, `emmylua_check` (50+ diagnostics; SARIF/JSON output), `emmylua_formatter` (`luafmt`),
  `emmylua_doc_cli`. Supports Lua 5.1–5.5, LuaJIT, EmmyLua and LuaCATS annotations. Claims "Fast" and incremental, with
  no benchmarks published. — [GitHub EmmyLuaLs/emmylua-analyzer-rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust)
- Crate versions: `emmylua_ls` 0.25.1, `emmylua_code_analysis` 0.25.1 and `emmylua_parser` 0.29.1, all 2026-08-14 and
  MIT. Homebrew ships 0.23.0, Alpine edge 0.21.0 (build 2026-03-09). Pre-1.0 means the API is not stable. —
  [crates.io emmylua_code_analysis](https://crates.io/crates/emmylua_code_analysis),
  [web search summary](https://pkgs.alpinelinux.org/package/edge/testing/x86/emmylua-ls)
- LSP plumbing: `lsp-types` 0.97.0 (2024-06-04, MIT). `tower-lsp` 0.20.0 (2023, stale); its community fork
  `tower-lsp-server` is 0.24.0-rc.1 (2026-09-11). — [lsp-types](https://crates.io/crates/lsp-types),
  [tower-lsp-server](https://crates.io/crates/tower-lsp-server)
- xi's plugin and RPC lesson: process separation between front-end and core "was not a good idea". JSON-RPC needed
  error handling and version negotiation, and serde bloated the core (9.3 MB). Process separation *for plug-ins* "remains
  workable, similar to LSP". — [xi retrospective](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)
- Zed converts UTF-8 points to UTF-16 via its rope summaries **specifically for LSP**, because LSP positions are UTF-16
  by default. — [Zed blog](https://zed.dev/blog/zed-decoded-rope-sumtree)

### Inferences
- **In-process wins for PH2D.** The editor only targets PH2D's Lua API, the analysis must know PH2D-specific globals
  (ECS components exposed through mlua), and ADR-0075 forbids runtime plugins. A direct Rust call to an
  `AnalysisHost`/`Analysis`-style API removes JSON serialization, UTF-16 conversions and process management. Keep the API
  LSP-*shaped* (POD types, offsets) so an optional LSP adapter crate can be added later for external editors. This is
  exactly rust-analyzer's split.
- **Two paths with emmylua:**
  1. Depend on `emmylua_code_analysis` in-process. MIT allows it, but pre-1.0 means API churn on every bump. Pin the
     version and wrap it behind our own trait.
  2. Run `emmylua_ls` as a child-process LSP. This is stable at the protocol level, but contradicts "invisible engine"
     and adds latency.

  A third path, our own salsa-based Lua analyzer, is the "best possible" option and the largest effort. Lua is small,
  so a salsa + rowan analyzer for PH2D's dialect plus generated type stubs from the ECS registry is feasible.
- The PH2D API surface should be emitted as **EmmyLua/LuaCATS annotation stubs** (`---@class`, `---@field`), generated
  from the component registry. emmylua then types user scripts against PH2D components with no custom analyzer code.
- lua-language-server (sumneko, written in Lua, MIT) and luau-lsp (C++, Luau dialect) do not fit an in-process Rust
  embedding. **[unverified this session: licences and versions]**

### Gaps
- The salsa changelog was not read, so 0.2x vs "salsa 2022 / 3.0" naming and current features (tracked structs,
  durability, parallel queries) are unconfirmed. ty/ruff adoption of salsa is also unconfirmed from fetched pages.
- No benchmarks of emmylua vs lua-language-server were found.
- emmylua's embedding API stability and whether it uses salsa internally are unknown.

## Q4. Rendering and input latency: GPUI, glyph atlases, parley vs cosmic-text, damage, < 10 ms typing

### Takeaway
The proven recipe, from GPUI: a small set of instanced primitives, **shaped-run caches keyed by (text, font)** with
only changed lines reshaped, an alpha-only glyph atlas with subpixel variants, everything drawn in one instanced call, and
an 8.33 ms budget at 120 Hz. In our stack, Vello already does GPU 2D. The editor-specific work is **per-line layout
caching** (one `parley::Layout` per visible line, never one per document), virtualized rendering of visible lines only,
and keeping highlighting and analysis off the keystroke path. Measured "good" typing latency is 1–5 ms in the
editor's own share (Pavel Fatin, Typometer).

### Cited Findings
- **GPUI:** the frame budget is "8.33ms per frame" at 120 Hz. It draws only rectangles (SDF), shadows (Evan Wallace's
  erf approximation), glyphs, icons and images, each with a dedicated shader and per-primitive instance data. —
  [Zed: Leveraging Rust and the GPU to render UIs at 120 FPS](https://zed.dev/blog/videogame)
- **GPUI text:** shaping is done by the OS (CoreText on macOS at the time). **Shaped glyphs are cached by text–font
  pair; only changed text is reshaped.** Glyphs are rasterized alpha-only into an atlas packed with `etagere`, with up
  to 16 variants per glyph for subpixel positioning. All glyphs go in one instanced draw, and colour is applied by
  multiplication. Atlas composition "approximates the bandwidth of the GPU". — [Zed blog](https://zed.dev/blog/videogame)
- **GPUI element model:** layout pushes constraints down and sizes up (Flutter-like). Paint pushes primitives into a
  layered `Scene`. — [Zed blog](https://zed.dev/blog/videogame)
- **Latency thresholds and measurements** (Typometer). A typical keyboard plus monitor adds ~26 ms (14 in, 12 out). The
  editor is "the weakest link". Windows averages: GVim 0.9 ms, IDEA zero-latency 2.9, Notepad++ 4.3, Sublime 8.2, Atom
  49.4. If latency grows with window size, the editor is "repainting more than the newly inserted symbol". Process
  highlighting, autocomplete and folding asynchronously. The suggested target is "1-5 ms". —
  [Pavel Fatin, Typing with pleasure](https://pavelfatin.com/typing-with-pleasure/)
- **Zed vs VS Code latency (2026):** only secondary blog numbers exist. One May 2026 test on an M3 MBP reports ~2 ms for
  Zed vs ~12 ms for VS Code/Cursor, and other posts disagree. No first-party Zed measurement was found. —
  [vantaige.io benchmark](https://vantaige.io/blog/zed-1-vs-cursor-vs-vscode-real-benchmark-2026),
  [Typometer](https://github.com/frarees/typometer)
- **parley** is at **0.12.0 (2026-10-09)**. PH2D's gate is 0.11 (0.11.0 on 2026-06-24, 0.11.1 on 2026-08-16).
  `parley::editing` has `PlainEditor` ("basic plain text editor with a single style applied to the entire text"),
  `PlainEditorDriver`, `Cursor`, `Selection`, `Generation` and `SplitString`. — [parley CHANGELOG](https://github.com/linebender/parley/blob/main/CHANGELOG.md),
  [docs.rs parley::editing](https://docs.rs/parley/0.12.0/parley/editing/index.html)
- **PlainEditor features** (0.12 docs): IME preedit via `is_composing`, `raw_compose` (byte range) and
  `ime_cursor_area` (candidate box). Selection geometry via `selection_geometry`, plus `cursor_geometry`. `generation()`
  serves as a redraw token. Text is exposed as `SplitString` excluding preedit, or `raw_text()` including it. The layout
  is updated lazily when dirty (`layout`, `try_layout`, `refresh_layout`), and `set_quantize` gives pixel-aligned
  coordinates. — [docs.rs PlainEditor](https://docs.rs/parley/0.12.0/parley/editing/struct.PlainEditor.html)
- **Parley layouts are immutable in content:** "if the text content or the styles applied to that content change then a
  new `Layout` must be created". Only re-line-breaking and alignment are cheap to redo. —
  [docs.rs parley](https://docs.rs/parley/latest/parley/)
- Parley 0.12 added `is_cursor_visible`, `PlainEditorDriver::set_selection`, `set_base_direction`, grapheme-safe caret
  placement and `Cursor::upstream_cluster`/`downstream_cluster`. 0.11 added `set_line_break_override`. 0.10 added the
  `complex-scripts` feature (dictionary breaking for CJK, Thai and others). —
  [parley CHANGELOG](https://github.com/linebender/parley/blob/main/CHANGELOG.md)
- **cosmic-text** 0.19.0 (2026-04-22), MIT OR Apache-2.0. **glyphon** 0.12.0 (2026-07-09), MIT OR Apache-2.0 OR Zlib, a
  wgpu text renderer on cosmic-text. — [cosmic-text](https://crates.io/crates/cosmic-text), [glyphon](https://crates.io/crates/glyphon)
- xi: native toolkits were slow at simple text, and GPU acceleration is "essentially required". Word wrap during live
  resize under async caused races and tearing, and scrolling "took months to get right". —
  [xi retrospective](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)

### Inferences
- **Do not use `PlainEditor` as the code editor's document model.** It holds the whole text in one string buffer with
  one style and one `Layout`, and any content change means a new `Layout`. That is O(document) per keystroke. Use it for
  single-line fields (search box, rename box) only.
- **For the code editor, the rope is the source of truth.** Build a **per-line (or per-wrapped-paragraph) layout cache**
  keyed by (line content hash, style spans, wrap width, font generation). Only edited lines are reshaped, which is the
  GPUI rule. Reuse parley's `Cursor` and `Selection` hit-testing logic per line, or reimplement it on top of the cached
  line layouts. Monospace code allows fast paths (column × advance) for non-ligature ASCII lines. Keep parley for
  shaping so ligatures, fallback, bidi and emoji stay correct.
- **Vello vs a glyph atlas.** Vello renders glyph outlines on the GPU (no atlas) and re-encodes the scene each frame. For
  a code view of ~100 visible lines × ~100 glyphs, encoding cost matters. Cache encoded `vello::Scene` fragments per
  line and append them, and redraw only when `generation`, the scroll offset or the caret blink changes. Measure the
  per-frame encode time (§0.0 of CLAUDE.md: measure before limiting).
- **Latency budget (< 10 ms target, 1–5 ms editor share):** on keypress, (1) apply the edit to the rope, (2) relayout
  only the touched line(s), (3) draw on the same frame. Tree-sitter reparse, highlight queries, diagnostics and
  completion run on background snapshots and arrive one frame later, keeping the old highlight spans mapped through the
  edit (xi's OT-style rebasing of stale results). Use winit's redraw request immediately after the input event, with no
  extra frame of buffering (Fatin: avoid redundant buffering).
- **Damage / partial redraw.** With Vello the whole surface is re-rasterized each frame, so "partial redraw" means
  partial re-*encode* (per-line scene caching) and skipping frames when nothing changed. True damage-rect presentation
  would need wgpu-level scissoring. Measure first.
- **Smooth scrolling:** render with fractional line offsets and pre-shape one screen above and below the viewport.

### Gaps
- No public measurement of Vello's per-frame encode cost for dense text (thousands of glyphs) was gathered this session.
- No first-party Zed latency numbers were found; the 2026 blog numbers are secondary and inconsistent.
- Whether cosmic-text's `Editor`, Lapce/floem's editor view or iced's `text_editor` keep per-line layout caches was
  **[unverified]** (background knowledge: cosmic-text's `Buffer` keeps a `BufferLine` per line with cached shape and
  layout, and iced's `text_editor` uses cosmic-text).

## Q5. Multi-cursor/selection model, undo/redo, transactions, local history

### Takeaway
Use the CodeMirror 6 / Helix model: `Selection = Vec<Range{anchor, head}>` plus a primary index. A single cursor is a
collapsed range. Every edit is an invertible `Transaction` (a ChangeSet plus a new selection) that maps selections and
marks through it. Undo stores inverted transactions. Zed's CRDT gives per-user undo as odd/even counters, which matters
only if collaboration is in scope.

### Cited Findings
- **Helix-core** is "heavily based on CodeMirror 6". It is functional (operations return new values), with Rope = ropey.
  `Selection` holds `Range`s (movable head, fixed anchor), and a single cursor is a range with head = anchor. Multiple
  selections are a core primitive. Edits go through an OT-like `Transaction` applied to the rope and **inverted to
  produce an undo**. Selections and marks are mapped across transactions. `Document` keeps selections **per view**. —
  [Helix architecture.md](https://github.com/helix-editor/helix/blob/master/docs/architecture.md)
- Helix is **MPL-2.0** (version 25.7.1 in Cargo.toml; last push 2026-09-29; ~46.6 k stars). `helix-core` is **not
  published** on crates.io (only a 0.0.0 placeholder from 2021). — [GitHub helix](https://github.com/helix-editor/helix),
  [crates.io helix-core](https://crates.io/crates/helix-core)
- **Zed undo in the CRDT:** each operation ID maps to a counter, and it is undone if the count is odd. Each user has
  their own undo history, and concurrent undos of the same operation converge. Text is never mutated: fragments point
  to immutable insertions, and deletions are tombstones with version vectors. **Anchors** = (insertion ID, offset) and
  stay valid across any edit. — [Zed: How CRDTs make multiplayer text editing part of Zed's DNA](https://zed.dev/blog/crdts)
- xi: CRDTs fit poorly with history-dependent operations such as auto-indent. —
  [xi retrospective](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html)

### Inferences
- **Recommended undo model:** a linear undo stack of transactions with **grouping** (merge consecutive insertions in
  the same word or within ~X ms, and treat a multi-cursor edit as one transaction). Restore selections on undo.
  Optionally offer an undo *tree* (Helix and Vim have one **[unverified detail]**) behind a "history" panel. Persist
  history to disk per script (local history) as the serialized transaction log plus periodic rope snapshots.
- **Integration with PH2D's global undo** (CLAUDE.md §5.1: one undo queue by DIFF, `ProjectState`): the script buffer's
  transaction log should be the diff that the project undo stores, and should not live in a parallel stack. That avoids
  two undo histories fighting.
- **Anchors** (positions that survive edits) are needed for diagnostics, breakpoints, bookmarks and inlay hints.
  Without a CRDT, implement them as (offset, bias) and map them through each ChangeSet, as Helix does with selections.
- **Multi-cursor ops** (Sublime/Zed style: add next occurrence, column select, split selection into lines) are all
  pure functions `Selection → Selection` over the rope, which is easy to test.

### Gaps
- The Helix `History` (undo-tree) implementation and Kakoune's model were not read this session.
- Sublime Text internals are closed-source. No primary source was found.

## Q6. Collaboration / CRDT

### Takeaway
If scripts are ever co-edited live over PH2D's LAN board, choose **loro** (1.16.2, MIT; rich text and movable lists,
actively released) or **yrs** (0.28.0, MIT; Yjs-wire-compatible), or Zed's design (commutative fragments, tombstones,
per-user undo counters) reimplemented over our own SumTree. diamond-types is the fastest by its author's claims, but
its crates.io release is stale (2022) and the author calls it WIP.

### Cited Findings
- **Zed's CRDT:** concurrent operations are made "inherently commutative" instead of transformed (no OT). Lamport
  timestamps order concurrent inserts, and deletions carry version vectors so they hide only text the deleter had seen.
  Undo is a per-operation counter. — [Zed CRDT blog](https://zed.dev/blog/crdts)
- **diamond-types:** ISC. crates.io 1.0.0 is from 2022-08-25, and the README says "the package published to cargo is
  quite out of date, both in terms of API and performance". Master's Cargo.toml says 2.0.0. It is plain text only, with
  more types on a branch. Self-described as "The world's fastest CRDT. WIP." Last push 2026-09-02. —
  [GitHub diamond-types](https://github.com/josephg/diamond-types), [crates.io](https://crates.io/crates/diamond-types)
- **automerge** 0.12.0 (2026-09-16, MIT). **yrs** 0.28.0 (2026-09-17, MIT). **loro** 1.16.2 (2026-09-21, MIT). —
  [automerge](https://crates.io/crates/automerge), [yrs](https://crates.io/crates/yrs), [loro](https://crates.io/crates/loro)
- crop's benchmarks use the `crdt-benchmarks` editing traces (automerge-paper and others), which are useful as replay
  workloads for our own buffer tests. — [crop README](https://github.com/nomad/crop)

### Inferences
- For a **script editor**, live co-editing is a nice-to-have. The architecture should not *preclude* it: use anchors
  rather than raw offsets for every persistent position, and route edits as transactions (ChangeSets) that could be
  converted to CRDT operations. Adding a CRDT later is then a backend swap.
- If co-editing is built, a text CRDT that stores its own document (loro or yrs) duplicates the rope. Zed avoids that by
  making the CRDT *be* the buffer (a fragment SumTree). The "best engine" path is Zed's design, reimplemented clean-room
  from the blog over our SumTree (Zed's `text` crate is GPL-3.0 and must not be copied).

### Gaps
- No head-to-head 2026 benchmark of loro, yrs and diamond-types text performance was fetched.
- Whether PH2D's MiroClone LAN sync layer already uses a CRDT (and which one) should be checked in-repo
  (`crates/ph2d-board-*`).

## Q7. Large files, background work, file watching

### Takeaway
Rope plus Arc snapshots gives large-file editing for free. Expensive features (tree-sitter, analysis) must degrade
above a size threshold, set by measurement. Background work runs on snapshots: rayon or a dedicated thread pool for
CPU-bound parsing and analysis, no tokio needed in-process. File watching uses **notify** (9.0.0-rc.5; stable 8.x; CC0).

### Cited Findings
- Ropey: edits stay in "single-digit microseconds" on multi-GB text, and memory overhead is ~10% (100 MB → 110 MB). —
  [ropey README](https://github.com/cessen/ropey)
- VS Code: per-line arrays exploded memory at 13.7 M lines (600 MB for 35 MB), and the piece tree stays near file size.
  — [VS Code blog](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation)
- Snapshot-to-background-thread is the shared pattern (crop, Zed, tree-sitter's cheap tree copy). —
  [crop](https://github.com/nomad/crop), [Zed](https://zed.dev/blog/zed-decoded-rope-sumtree),
  [tree-sitter](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- rust-analyzer: cancellation by revision bump means stale background work aborts as soon as the user types. —
  [rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
- Helix `helix-event` provides an `AsyncHook` for debounced, cancellable tasks. —
  [Helix architecture.md](https://github.com/helix-editor/helix/blob/master/docs/architecture.md)
- **notify** max version 9.0.0-rc.5 (2026-08-30), licence **CC0-1.0**, ~44.8 M recent downloads. —
  [crates.io notify](https://crates.io/crates/notify)
- Diffing for gutter and local history: `imara-diff` 0.2.0 (Apache-2.0; used by Helix and gitoxide **[unverified]**) and
  `similar` 3.2.0 (Apache-2.0). — [imara-diff](https://crates.io/crates/imara-diff), [similar](https://crates.io/crates/similar)

### Inferences
- In PH2D's ECS, the editor buffer is a component, and edits are events. A system copies a rope snapshot plus a
  revision into a background job (rayon or bevy's `AsyncComputeTaskPool`). Results (highlight spans, diagnostics) come
  back as events tagged with the revision and are rebased through intervening ChangeSets or dropped. This keeps ADR-0075
  (systems don't call each other).
- Scripts are usually small, so large-file thresholds matter mostly for viewing logs and data. Set them by
  measurement (CLAUDE.md §0.0), not by guess.

### Gaps
- No tokio requirement was found for an in-process analyzer. emmylua_ls's runtime (tokio?) matters only if run as an LSP
  process **[unverified]**.

## Q8. Embeddable Rust editor components and their licences

### Takeaway
There is **no permissively licensed, production-grade, embeddable code-editor component** that fits a Vello/Parley
retained UI. Zed's editor is GPL-3.0. Helix is MPL-2.0 and unpublished. Lapce is Apache-2.0 but tied to floem. The
others are toolkit-bound (egui, iced, cosmic-text). The realistic path is to **build our own editor crate** from
permissive parts (crop or ropey, or our own SumTree; tree-sitter; parley; emmylua) and use Zed and Helix as design
references, learning from the blogs and docs. Treat GPL code as restricted (`/pd-cleanroom`).

### Cited Findings (licences verified 2026-10-15)

| Component | Licence | Version / activity | Notes |
|---|---|---|---|
| Zed `editor`, `text`, `rope`, `language`, `multi_buffer` | **GPL-3.0-or-later** | unpublished | Restricted. Read the blogs, not the code. [zed crates](https://github.com/zed-industries/zed/tree/main/crates) |
| Zed `sum_tree` | **Apache-2.0** | unpublished (`publish=false`) | Can be vendored with attribution. [zed sum_tree](https://github.com/zed-industries/zed/tree/main/crates/sum_tree) |
| Zed `gpui` | **Apache-2.0** | 0.2.2 (2025-10-22) | Its own UI framework; does not fit Vello. [crates.io](https://crates.io/crates/gpui) |
| Helix (`helix-core` etc.) | **MPL-2.0** | 25.7.1; pushed 2026-09-29 | Not on crates.io. File-level copyleft. [GitHub](https://github.com/helix-editor/helix) |
| Lapce | **Apache-2.0** | pushed 2026-10-09; 38.9 k stars | Editor logic is tied to floem. [GitHub](https://github.com/lapce/lapce) |
| floem | **MIT** | crates.io 0.2.0 (2024-11); repo active 2026-09 | Lapce's UI toolkit. [GitHub](https://github.com/lapce/floem) |
| lapce-xi-rope | Apache-2.0 | 0.4.0 (2025-12-19) | [crates.io](https://crates.io/crates/lapce-xi-rope) |
| xi-editor | **Apache-2.0** | last push 2024-03 (dormant) | [GitHub](https://github.com/xi-editor/xi-editor) |
| cosmic-text | MIT OR Apache-2.0 | 0.19.0 (2026-04-22) | Has a `Buffer`/`Editor` **[unverified detail]**. [crates.io](https://crates.io/crates/cosmic-text) |
| glyphon | MIT OR Apache-2.0 OR Zlib | 0.12.0 (2026-07-09) | wgpu renderer on cosmic-text. [crates.io](https://crates.io/crates/glyphon) |
| iced (`text_editor` widget) | **MIT** | 0.15.0-dev on master | [GitHub](https://github.com/iced-rs/iced) |
| egui | MIT OR Apache-2.0 | 0.37.0 | [GitHub](https://github.com/emilk/egui) |
| egui_code_editor | MIT | 0.4.2 (2026-10-07) | egui-bound, simple. [crates.io](https://crates.io/crates/egui_code_editor) |
| parley `PlainEditor` | Apache-2.0 OR MIT | 0.12.0 (2026-10-09) | Single-style, single buffer. [docs.rs](https://docs.rs/parley/0.12.0/parley/editing/index.html) |

### Inferences
- **MPL-2.0 (Helix, full_moon, stylua)** allows linking into a proprietary or permissive codebase. Only modified MPL
  *files* must stay MPL and be published. Copying Helix's selection or transaction code into our files would make those
  files MPL. Depending on an unmodified crate is fine. helix-core is not on crates.io, so it would be a git dependency
  or vendored.
- The **Zed `sum_tree` crate (Apache-2.0)** is the single most valuable permissive artefact. It is the generic
  structure Zed builds its rope, display maps and CRDT on. Vendoring it needs the Apache NOTICE and attribution; it is
  not on crates.io.

### Gaps
- Lapce's crate-level structure (`lapce-core`, floem's `views::editor`) and how separable it is from floem was not
  verified this session.
- cosmic-text `Editor`/`ViEditor` maturity for code (syntax highlighting via syntect in `cosmic-text`'s examples) is
  **[unverified]**.

## Q9. Accessibility (AccessKit) for a custom-rendered editor

### Takeaway
AccessKit supports text editing via `TextRun` nodes (per-run character lengths, word starts and positions) and a text
selection. egui has a reference implementation. Parley **0.11** had built-in AccessKit support for `PlainEditor`
(`LayoutAccessibility`). Parley **0.12 removed it** and moved it into an example, so PH2D will own this code either way.

### Cited Findings
- **accesskit** is at 0.25.1 (2026-09-25), MIT OR Apache-2.0. PH2D's gate is 0.24. — [crates.io accesskit](https://crates.io/crates/accesskit)
- **Parley 0.12:** "The `accesskit` feature and its integration were removed. The integration now lives in the `editor`
  example, and `LayoutAccessibility` and related `PlainEditor` accessibility methods were dropped." `ClusterPath::new`
  and `Style::locale` were made public so integrations can be built outside parley. —
  [parley CHANGELOG](https://github.com/linebender/parley/blob/main/CHANGELOG.md)
- Parley 0.12 `PlainEditor::is_cursor_visible()`: AT consumers should hide the selection when the IME hides the caret.
  — [docs.rs PlainEditor](https://docs.rs/parley/0.12.0/parley/editing/struct.PlainEditor.html)
- **egui's `accesskit_text`** module maps cursor positions to text-run nodes (a row may be split into several runs).
  AccessKit `word_starts` uses `u8` indices, so **text runs have a maximum length**. —
  [egui accesskit_text (servo docs)](https://doc.servo.org/egui/text_selection/accesskit_text/index.html)
- AccessKit PR #145 added Windows screen-reader feedback for text editing. Rich-text formatting info was the main
  missing piece at the time, and a caret issue with Braille routing buttons was flagged. —
  [AccessKit PR #145](https://github.com/AccessKit/accesskit/pull/145)
- Godot's AccessKit integration notes that custom editor canvases are hard to make accessible. —
  [Godot PR #76829](https://github.com/godotengine/godot/pull/76829)

### Inferences
- **Virtualize the accessibility tree like the rendering:** one `TextRun` node per visible line (split at 255 chars for
  the `u8` limit), plus a document-level node exposing the selection as (node, character index) pairs. Lines outside
  the viewport are omitted or summarized. Rebuild only the nodes of changed lines, keyed by the same per-line cache as
  layout.
- **Upgrade note for the stack:** PH2D gates parley 0.11, which still has `LayoutAccessibility`. Moving to 0.12 drops it.
  If the code editor builds its own AccessKit mapping from per-line layouts (recommended anyway, since PlainEditor is
  not the document model), the upgrade becomes neutral.

### Gaps
- AccessKit 0.24 → 0.25 changes to text-related APIs were not checked.
- Screen-reader behaviour on Linux (AT-SPI via accesskit_unix) for large multi-line editors: no source found.

---

### Cross-cutting summary of what the best editors do

| Concern | Zed | Helix | Lapce / xi | VS Code | rust-analyzer |
|---|---|---|---|---|---|
| Buffer | SumTree rope, Arc snapshots | ropey | xi-rope (lapce fork) | piece tree (RB-tree) | — (consumes text) |
| Syntax | tree-sitter, background on every edit | tree-sitter | tree-sitter (xi: syntect async) | TextMate + tree-sitter **[unverified]** | rowan, lossless and error-tolerant |
| Semantics | LSP | LSP client | LSP | LSP | salsa; `ide` POD API, LSP is a thin shell |
| Edits/undo | CRDT fragments, per-user undo counters | CM6-style Transactions, inverted for undo | CRDT (xi) | — | — |
| Rendering | GPUI: instanced primitives, glyph atlas, cached shaping, 120 fps | terminal | floem/wgpu | DOM/Electron | — |

Sources: [Zed rope](https://zed.dev/blog/zed-decoded-rope-sumtree) · [Zed GPUI](https://zed.dev/blog/videogame) ·
[Zed CRDT](https://zed.dev/blog/crdts) · [Helix arch](https://github.com/helix-editor/helix/blob/master/docs/architecture.md) ·
[xi retrospective](https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html) ·
[VS Code piece tree](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation) ·
[rust-analyzer arch](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md)
