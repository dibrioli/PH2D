# Wave 11 — Carry-overs from Wave 10 closure

**Status:** Open (drafted 2026-05-24, immediately post Wave 10 close).
**Closes:** [ADR-0042 §6 carry-over list](../architecture/decisions/0042-wave-10-closure.md#6-wave-11-carry-over-recommendations) + 2 BgRemoval regressions logged in [regressions.md](../Testes/regressions.md).
**Maintainer:** Coord-A.

---

## 0. Intent

Wave 10 closed with 7 documented carry-overs (architectural follow-ups) + 2 user-reported BgRemoval regressions. This doc converts that list into **scoped, executable plans** so any session can pick the highest-leverage item without re-reading Wave 10.

Priority ordering reflects (a) user-blocking severity, (b) unblocking value (does it free other carry-overs?), (c) effort-to-payoff ratio.

---

## 1. Priority 1 — Unblocked, high-leverage

### 1.1 BgRemoval R-1 — pickcolor + slider perf

**Source:** `docs/Testes/regressions.md` R-1 (Enio smoke 2026-05-24).
**Symptom:** every `add_extra_color` (eyedropper click) or slider tick sets `params_dirty`, triggering a ~50ms pipeline cook on the 512² canvas preview. At 60 Hz drag, the cook starves the frame budget — sliders feel sluggish.
**Root cause:** preview cook runs SYNCHRONOUSLY on the main thread in `current_preview()`.

**Fix paths (pick one, not both):**

- **Path A — debounce on drag (1 day, simple, partial relief).**
  Add a "drag in progress" flag set by the dispatch when the user starts dragging any slider. While the flag is on, `current_preview()` skips the full cook and uses a smaller cap (256² instead of 512²). Drag end clears the flag and triggers one full-res preview. Net: drag becomes smooth, final preview unchanged.

- **Path B — async cook with double-buffer (3-5 days, padrão-ouro, full relief).**
  Spawn the cook on a worker thread. `current_preview()` returns the LAST-good cached frame immediately; the worker pushes the new frame into a swap slot when ready. Drag becomes 60 Hz smooth regardless of cook cost. Requires careful borrow-management to keep the `BgRemovalParams` snapshot consistent between main + worker.

**Recommendation:** Path A first (ships in 1 day), Path B as Wave 12+. Path A covers 80% of the symptom with 20% of the work; Path B is the architecturally correct version but its complexity warrants its own ADR.

**Acceptance:** Eyedropper click + drag any slider at 60 Hz; no perceived stutter.

---

### 1.2 BgRemoval R-2 — preview-vs-final mask divergence

**Source:** `docs/Testes/regressions.md` R-2 (Enio smoke 2026-05-24, updated observation 2026-05-24 evening — divergence may be illusory, the algorithm produces irregular mask on both surfaces).
**Pre-investigation needed:** before committing to a fix, confirm whether (a) preview and full produce visibly different shapes (original theory) OR (b) both produce the same blobby mask and the apparent divergence was the user observing different overlays (protect-stroke vs final alpha). The Enio's follow-up message implies (b) is the actual state.

**If (a) — divergence is real:** scale `params.grow_px` and `params.min_island_pixels` proportionally to the preview-vs-source ratio in `run_canvas_preview`. Existing constants `GROW_FULL_SCALE` and `MIN_ISLAND_PIXELS_FULL_SCALE` already exist; multiply by `preview_dim / source_dim` for the preview cook.

**If (b) — algorithm just produces irregular mask:** revisit defaults. Likely the `min_island_pixels` default is too low (small noise islands survive). Try doubling the default and confirm visual improvement.

**Acceptance:** preview overlay and post-Apply final image agree on the mask shape within ±2 px at the edge.

---

### 1.3 `ph2d-color` migration sweep — 10 BASELINE files

**Source:** ADR-0042 §6 #2 + `arch_color_space_typed` BASELINE.
**Setup done in Wave 10 post-ship polish:** `SrgbRgba` now declares `#[repr(transparent)]` over `[u8; 4]` + ships `iter_byte_slice` safe-code iterator. Zero-copy `bytemuck::cast_slice` is possible when a caller adds the dep.

**Sweep order (smallest first → minimize risk of cascading breakage):**

1. `crates/ph2d-render/src/premul.rs` (269 LOC) — foundational, touches every consumer. Migrate LAST despite being smallest, because a regression here propagates everywhere.
2. `crates/ph2d-tool-padding/src/algorithm.rs` (300 LOC) — leaf tool, pure memcpy.
3. `crates/ph2d-tool-make-square/src/algorithm.rs` (464 LOC) — leaf tool.
4. `crates/ph2d-tool-trim-transparency/src/algorithm.rs` (523 LOC) — leaf tool, scans alpha.
5. `crates/ph2d-tool-upscale/src/algorithm.rs` (685 LOC) — leaf tool, three kernels.
6. `crates/ph2d-tool-upscale/src/tool.rs` — set_source_snapshot signature.
7. `crates/ph2d-tool-bgremoval/src/tool.rs` — set_source_snapshot.
8. `crates/ph2d-tool-color-equalization/src/algorithm.rs` (large) — compute_histogram + resize_bilinear_rgba.
9. `crates/ph2d-tool-color-equalization/src/tool.rs` — set_source_snapshot.
10. `crates/ph2d-tool-equalize-sizes/src/algorithm.rs` — lanczos3_resample + mitchell_resample.
11. `crates/ph2d-render/src/premul.rs` — last, foundational.

**Migration pattern per file:**

```rust
// Before:
pub fn add_padding(rgba: &[u8], w: u32, h: u32, spec: PaddingSpec) -> PaddingResult { ... }

// After:
pub fn add_padding(pixels: &[SrgbRgba], w: u32, h: u32, spec: PaddingSpec) -> PaddingResult { ... }
// Where PaddingResult { pixels: Vec<SrgbRgba>, ... }
```

Callers convert at the IO boundary using `bytemuck::cast_slice` (add `bytemuck = "1.x"` as an optional/leaf-only dep — DO NOT add to ph2d-color itself; the leaf tool's Cargo.toml absorbs it).

**Acceptance:**
- `arch_color_space_typed` BASELINE empties (every entry removed).
- All workspace tests still pass — no behavioral change.
- Visual smoke: each tool still produces identical output on a reference sprite.

**Effort estimate:** 1-2 weeks per ADR-0042. POC done in Wave 10 post-ship (the repr-transparent setup); full sweep is mechanical-but-careful.

---

## 2. Priority 2 — Unblocked, smaller leverage

### 2.1 GH Action wiring of `auto-merge-eligibility.sh`

**Source:** ADR-0042 §6 #5.
**Action:** Add `.github/workflows/auto-merge.yml` that runs on every PR open/synchronize event. The workflow:

1. Checks out the PR base + head.
2. Runs `scripts/auto-merge-eligibility.sh "${{ github.base_ref }}" "${{ github.head_ref }}"`.
3. If exit 0 (eligible): comments on the PR with the "eligible — pending CI green" label.
4. If exit 1: comments "coord review required" + posts the script's stderr explanation.

When CI also goes green AND the PR has the "eligible" label, a follow-up workflow auto-merges.

**Effort:** ~1 day. Single-file workflow + permissions config.

**Acceptance:** Open a test PR touching only `crates/ph2d-panel-test/`; expect auto-merge after CI green.

---

### 2.2 Long-paint fns split (3 fns, recipe known)

**Source:** ADR-0042 §6 #3 + `FN_OVERAGE_OK` in `architecture_panel_loc_cap.rs`.

| File | Fn | Current LOC | Target | Recipe |
|---|---|---:|---:|---|
| `ph2d-panel-bgremoval/src/paint.rs` | `paint` | 401 | ≤ 200 | Same CEQ split (Etapa 5.2) — extract per-section helpers into a sibling `paint_sections.rs`. |
| `ph2d-panel-grid-snap/src/paint.rs` | `paint_body` | 301 | ≤ 200 | Split into kind / target / display section helpers. |
| `ph2d-panel-grid-snap/src/populate.rs` | `populate` | 214 | ≤ 200 | Split into per-section store init helpers. |

**Pattern (from CEQ split):** each helper takes `&mut PaintCtx` (or `scene + text + store + hit_index + theme + y_in`) and returns the `y_out` cursor.

**Effort:** 1-2 days per panel (smoke-validated). Total ~5 days.

**Acceptance:** After each split:
- `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` passes WITHOUT the file's `FN_OVERAGE_OK` entry.
- Visual smoke: the panel renders identically (CEQ split was byte-exact; replicate that bar).

---

### 2.3 `docs_bugs_have_gates` backfill (90 entries)

**Source:** ADR-0042 §6 #6.
**Action:** Walk `docs/UI_Bugs/README.md` (77 entries) + `docs/Image Tools Bugs/README.md` (13 entries). For each `### N.M Title` heading, find or write a corresponding gate (in `crates/*/tests/`) AND append a `**Gate:** crates/.../tests/foo.rs::test_name` line under the heading. If no gate is appropriate (e.g. the bug is a one-off design fix), append `**Gate:** gate-deferred: <reason>` instead.

Then enable the previously-deferred gate `tests/docs_bugs_have_gates.rs` that walks each `### ` entry and asserts the `**Gate:**` line exists.

**Effort:** 1 day (backfill is mechanical; gate-writing only for entries that genuinely warrant a new gate — most map to existing arch gates).

---

## 3. Priority 3 — Blocked / needs Enio

### 3.1 Golden-image SSIM gate

**Source:** ADR-0042 §6 #1.
**Blocked on:** Enio approving ~17 baseline PNGs (widgets + 9 panels).
**Plan when unblocked:**
1. Wire `vello` headless renderer into a test harness.
2. For each blessed widget / panel, render to PNG → compare with `image::imageops::ssim` (or a custom SSIM impl).
3. CI gate: fail if SSIM < 0.995 for any baseline.
4. Rebaseline flag: `--update-baselines` in tests, generates new PNGs (PR review confirms visual match).

**Estimated reduction in Enio smoke time:** ~70% per Wave 10 plan estimate.

### 3.2 `panel-canonical-template` AST gate

**Source:** ADR-0042 §6 #4.
**Blocked on:** Coord-A deciding the canonical template structure (which section of `pre_populate.rs` is the source-of-truth shape).
**Plan when unblocked:**
1. Codegen `__template__.rs` from the chosen blessed source.
2. `syn`-based AST visitor checks each `panel-*/src/populate.rs` against the template: link_slider_number coverage, slider↔chip storage parity, no `set_slider_value` in event.rs.

### 3.3 `no_tofu_glyphs` amplified (U+2000–FFFF coverage)

**Source:** ADR-0042 §6 #7.
**Trigger to resume:** a real tofu bug ships outside the existing arrows/cmd block. The basic gate (arrows U+2190..21FF + cmd block U+2300..23FF) covered ≥80% of historical incidents.
**Plan:** load Inter's glyph coverage table (via `ttf-parser` reading the bundled font), build a runtime allowlist, gate scans all UI strings against `c.is_in_inter_coverage()`.

---

## 4. Sequencing recommendation

**Week 1:** R-1 Path A (BgRemoval debounce) + R-2 pre-investigation. Both contained, both visible to Enio.

**Week 2-3:** ph2d-color migration sweep (10 files in order above). The biggest payoff is unlocking `arch_color_space_typed` BASELINE empty.

**Week 4:** Long-paint fns split (3 panels) + GH Action wiring. Parallel work; each ~1-2 days.

**Week 5:** docs_bugs_have_gates backfill + open ADR-0043 for golden-image SSIM design (so Enio can sign off baselines async).

**Out of scope for Wave 11:** Path B async cook for BgRemoval (Wave 12+ — needs its own ADR), `panel-canonical-template` (waits on template decision), `no_tofu_glyphs` amplified (waits on real incident).

---

### Painter T1.8 carry-overs (audit T1.8 closure 2026-05-27)

**Source:** ADR-0046 + audit T1.8 L4-H13/L4-H19/L4-H7/L4-H12/L2-G12/L4-H16 + audit T1.8 L1-F1/L2-F5/L3-G4.

**1. Streaming `blake3::Hasher` para `PaintProject::recompute_checksum` + `verify_checksum`** (L1-F1/L2-F5/L3-G4/L4-H13/L5-I4)

Implementação atual usa clone-based (`self.clone()` + `postcard::to_allocvec`) pra preservar invariante anti-panic. Peak memory 2× serialized size; em sessão pesada (10k strokes ≈ 6 MB → 12 MB temporário). Em iPad 2018 (3 GB total) abrir canon 200 MB causa 400 MB peak.

**Plan:** custom `serde::Serializer` adapter que feed bytes pro `blake3::Hasher::update` direto, sem `Vec<u8>` intermediário. Skip `checksum` field via field-level filter. Zero peak memory overhead, mantém atomicidade anti-panic.

**Trigger to resume:** quando primeiro report de UX "abrir canvas grande engasga" no Painter ship. OU quando W16 cloud-sync precisar processar uploads server-side em hot path.

**2. `count_struct_fields` heurística multi-line** (L1-F12)

Arch-gate helper em `architecture_painter_contract_surface.rs` conta `pub <name>: <type>` por linha. Refactor rustfmt com width estreito que quebrar field longo em multi-linha (e.g., `pub points:\n  Vec<RawPointerSample>`) → undercount silencioso, caps passam spuriously.

**Plan:** trocar heurística por `syn` AST parser ou regex multi-line.

**Trigger to resume:** primeiro caso real onde gate falha em detectar cap violation.

**3. `BrushParamsHash` newtype cross-crate** (L4-H12)

Atual `pub type BrushParamsHash = [u8; 32]` em `ph2d-painter-brush` é alias = zero protection. Caller pode passar `texture_blake3` ou `source_blake3` em vez de `brush.params_blake3()` — compila silenciosamente, quebra content-addressing.

**Plan:** newtype `pub struct BrushParamsHash(pub [u8; 32])` em `ph2d-painter-brush` com `#[serde(transparent)]` (zero ABI cost postcard). Cross-crate refactor: `ph2d-painter-stroke::StrokeRecord.brush_params_hash` + `PaintProject::brush_snapshots`.

**Trigger to resume:** primeira sessão coordenada de polish cross-crate da família painter.

**4. Doctests em surface pública** (L4-H7)

Zero `# Examples` em fns públicas de `ph2d-painter-stroke`. CI roda `cargo test --doc` mas surface MCP/W11 vai precisar de exemplos pra onboarding LLM via `cargo doc --json`.

**Plan:** adicionar 4-6 doctests críticos em `PaintProject::new`/`recompute_checksum`, `StrokeRecord::try_push_sample`, `f32_to_q1616_saturating` vs `_checked`, `load` migration-aware flow, `StrokeHistory::for_budget`.

**5. `CanonVersion`/`SidecarVersion` newtypes** (L4-H19)

4 versions `u32` espalhados (`StrokeRecord`, `PaintProject`, `LayerSnapshot`, `PaintProjectCache`) sem distinção type-level entre HR-14 forward-compat vs sidecar regenerável. Caller pode silenciosamente atribuir um pro outro.

**Plan:** `pub struct CanonVersion(pub u32);` + `pub struct SidecarVersion(pub u32);` com `#[serde(transparent)]`. Substituir field types. Custo ~1h refactor + tests.

**6. `BrushSnapshotTable` newtype** (L2-F7/L5-I7)

`pub brush_snapshots: Vec<(BrushParamsHash, Brush)>` exige caller manter uniqueness. `load()` enforça via runtime check, mas idiom Rust seria `BTreeMap<BrushParamsHash, Brush>` newtype-wrapped pra impossibility-to-violate.

**Plan:** `pub struct BrushSnapshotTable(BTreeMap<BrushParamsHash, Brush>);` com `#[serde(serialize_with=…)]` emitting sorted `Vec<(K,V)>` (postcard determinism preservado).

**7. `OsPathBytes` wrapper pra `SnapshotStorage::OnDisk`** (L2-F6/L5-I6)

`PathBuf` em Windows pode conter UTF-16 non-roundtrippable em UTF-8 → save falha. Além: path attacker-controlled em sidecar exige `validate_path_for_load` (já implementado em T1.8). Long-term: wrapper `OsPathBytes(Vec<u8>)` armazena raw bytes + helper de conversão lossy.

**Trigger:** primeiro report Windows com path non-UTF-8 OU W16 cloud-sync precisar harden traversal defense além do que `validate_path_for_load` cobre.

**Sequencing pintura T1.8 carry-overs:** itens 1-4 são `W11 painter polish` (semana dedicada). Itens 5-7 podem aguardar W12 Reproject + W13 MCP (donde semantics ficam mais claras).

---

### T-durability carry-overs (auditoria completa 2026-05-27, ~13 LOW + 4 estruturais deferidos)

**Source:** ADR-0052 + auditoria completa 4-lente final (M holistic + N performance + O concurrency + P spec compliance). Onda completa fechou 1 CRITICAL + 14 HIGH + ~15 MEDIUM in-code; restante deferido aqui.

**1. Worker thread real em `AutoSave` + `commit_stroke` async** (N-6 / N-7 / O-3)

Implementation atual é state machine pura; caller (shell W11+) wire worker thread. Spec ADR-0052 §2.4 prometia "AutoSave em worker thread" — promessa quebrada pelo passive-crate design. Ship mobile real requer worker thread interno OR helper `commit_stroke_async` que enfileira em canal SPSC.

**Plan:** opt-in feature flag `autosave-worker` que ativa `std::thread::spawn` interno + canal SPSC. `commit_stroke_into(tx)` API pra enfileirar. Trade-off: threading em wasm32 requer fallback (canal síncrono).

**Trigger:** primeiro UX report de "Painter engasga ao terminar stroke" em iPad/Android. OU W11 shell ship.

**2. Real-streaming `blake3::Hasher` sem Vec intermediário** (N-3 / O-6 / N-8 / N-12)

`recompute_checksum` atual usa `postcard::to_allocvec(&*self)` (sem clone, mas ainda Vec full size — 200MB pra canon grande). `verify_checksum` ainda clona. Solução real: serializer custom field-by-field que feeda blake3::Hasher::update direto, pulando `checksum` field.

**Plan:** implementar `struct ChecksumSerializer<'a, H: blake3::Hasher>` impl `serde::Serializer` que delega tudo pro hasher + skip field "checksum" via `SerializeStruct::skip_field`. Eliminates Vec inteiro. Verify usa mesma adapter sobre `&self` (sem mutação).

**Trigger:** W11 painter polish OR primeiro OOM report em iPad com canvas grande.

**3. WAL scratch buffers reusáveis em `StrokeJournal`** (N-2 + N-9)

`append_entry` aloca 2 Vec<u8> por chamada (payload + buffer concat). `read_journal` aloca `vec![0u8; payload_size]` per entry no boot scan (~200k allocs em 50MB WAL). Scratch buffer permanente em struct elimina ambos.

**Plan:** adicionar `payload_scratch: Vec<u8>` + `frame_scratch: Vec<u8>` em `StrokeJournal`. Usar `.clear()` antes de cada uso. Reader: `payload_scratch_reusable: Vec<u8>` em loop de `read_journal`.

**Trigger:** profiling mostrar allocator pressure em hot path.

**4. `BeginUpdate` WAL entry pra cross-validate `samples_count_in_journal`** (K-3 + recovery K-3 fix)

Atual: `samples_count_in_journal` é write-only no Begin entry (gravado em t=0 quando samples ainda não chegaram). Cross-validation em recovery (K-3 spec'd) é impossível sem entry adicional. Solução: emit `BeginUpdate` entry em `commit_stroke` com count final + replay validation aceita stroke como valid SE count match.

**Plan:** adicionar `WalEntryType::BeginUpdate = 5` carregando `(StrokeId, final_samples_count: u32)`. Caller emit em commit. Recovery valida match — mismatch = `RecoveryState::Corrupted`.

**Trigger:** W11 polish OR primeiro report MCP-injected stroke count mismatch.

**5. ADR-0052 §2.4 spec correction — remover claim "AutoSave em worker thread"** (N-6 / M-15)

ADR original promete "AutoSave em worker thread; UI nunca bloqueada" mas implementation é state machine pura. Substituir por: "Caller (shell) é responsável por dispatch em worker thread. Crate fornece state machine + helpers (`AutoSave`, `WalCommand` enum); threading vive no shell W11+."

**Plan:** amend ADR §2.4 prose. Adicionar §2.4.1 "Worker thread pattern (caller-side)" com code example canônico.

**Trigger:** próxima sessão de ADR amendments OR pre-ship mobile real.

**6. `drain_for_suspend` helper canônico** (M-15)

ADR-0052 §2.5 specifica drain protocol em 5 steps mas crate só expõe state machine. Caller (W11 shell) reimplementa drain manualmente — risk: shell esquece phase 1 WAL flush.

**Plan:** `pub fn drain_for_suspend(handler: &mut SuspendHandler, journal: &mut StrokeJournal, autosave: &AutoSave, canon_path: &Path, now_ms_fn: impl Fn() -> u64) -> DrainOutcome` que executa os 5 steps conforme phases proporcionais.

**Trigger:** W11 shell wire mobile.

**7. `color_profile` snapshot em `PartialStroke`** (M-6)

`OklchColor` é working-space agnostic mas canvas `color_profile` pode mudar mid-session (W7 Color Mgmt). Recovery re-add stroke em runtime canvas com profile diferente do source-time → cor errada silenciosa.

**Plan:** adicionar `color_profile: ColorProfile` em `PartialStroke` (14/16 fields, cabe). Recovery rejeita strokes com profile mismatch OR re-converte explicitamente.

**Trigger:** W7 Color Management ship.

**8. Restantes LOW (J-15 tempfile, P-5 cache, P-10/P-11 derive prescriptions, P-16/P-18, M-11/M-14/M-16/M-17, N-2/N-9/N-10 scratch buffers, O-1/O-10/O-12/O-13 docs)** — agregar conforme W11 polish dedicado.

---

### Painter T1.9 carry-overs (auditoria 2-lente 2026-05-28: correctness Q + perf/concurrency R)

**Source:** T1.9 wire (`StrokeHistory` + `StrokeJournal` em `PainterTool`).
2 lentes paralelas (audit Q correctness + audit R perf/concurrency) =
1 CRITICAL + 8 HIGH + 6 MEDIUM + 7 LOW. CRITICAL+HIGH+MEDIUM (14 itens)
fechados in-code; 7 LOWs deferidos aqui.

**1. Worker thread real pra `commit_stroke` + `add_sample` em mobile** (R-7)

T1.9 wire chama `journal.add_sample()` + `journal.commit_stroke()` DIRETO
na UI thread. Mobile (eMMC ~5-10ms fsync) × pencil 240Hz × Hybrid{n:8}
flush = ~30 fsync/s = ~240ms/s perdidos só pra fsync.

**Plan:** wire SPSC channel `(WalCommand, Result)` no shell W11+. UI thread
push commands; worker thread executa `journal.*` + retorna result.

**Trigger:** ship mobile real OR primeiro UX report iPad "Painter jank stroke contínuo".

**2. `oklch_to_oklab` production `assert!` pode panicar `begin_stroke`** (Q-11)

`params.active_color: OklchColor` é `pub` field; bridge/W2 sidebar handler
que escrever `h` em degrees panica next `begin_stroke`.

**Plan:** trocar `assert!` por clamp+warn em `oklch_to_oklab`
(`debug_assert!` strict apenas dev); OR gate no `apply_ui_edit::SetColor`.

**Trigger:** W2 sidebar handlers wire.

**3. `CanvasId::UNASSIGNED` sentinel pra multi-canvas safety** (Q-12)

Default `CanvasId(0)` é silent quando caller esquece `set_canvas_id` em
multi-canvas → all strokes vazam pra CanvasId(0).

**Plan:** trocar default por `CanvasId(u64::MAX) == UNASSIGNED`;
`attach_journal` retorna `JournalError::CanvasIdUnassigned` se ainda for
UNASSIGNED.

**Trigger:** W11 multi-canvas wire.

**4. `PartialStroke` Copy assertion lock-in** (R-4)

`PartialStroke` é trivialmente clonável hoje (todos fields Copy).
Adicionar Vec/String/Box nos 3 headroom slots introduz alloc per stroke
silenciosamente.

**Plan:** `static_assertions::assert_impl_all!(PartialStroke: Copy)` OU
comment lock-in nos headroom slots.

**Trigger:** primeira proposta de adicionar field non-Copy em PartialStroke.

**5. `current_samples` cap inicial 256→2048 pra long-stroke realloc-free** (R-11)

Long stroke 1k+ samples paga 2× realloc (256→512→1024→2048).

**Plan:** trocar `Vec::with_capacity(256)` por `Vec::with_capacity(2048)`
em ambas as linhas.

**Trigger:** profiling mostrar realloc jitter mid-stroke.

**6. `attach_journal` Windows cross-process race window** (R-12)

Drop libera flock + open re-acquire tem μs-window onde outro processo pode
"vencer" o lock. Linux/macOS atomic per-PID; Windows é per-handle.

**Plan:** doc explícita "Windows race: caller deve retry com backoff em
`AlreadyLocked`" OU API `swap_journal_atomic(new_path)`.

**Trigger:** primeiro report Windows multi-instance conflict.

**7. `take_preview_arc` + `current_preview` foot-gun (both drain)** (R-13)

Ambos drenam `preview_dirty`; caller chamando ambos perde frame.

**Plan:** doc explícita "AMBOS DRENAM `preview_dirty`" OU refactor
`current_preview` read-only.

**Trigger:** W11 bridge debug overlay.

---

## 5. Open ADR drafts

When the relevant Wave 11 work starts, open these ADRs:

- **ADR-0043** — Golden-image SSIM strategy (which library, baseline storage, rebaseline workflow).
- **ADR-0044** — Async preview cook architecture (worker thread, double-buffer, params snapshot consistency).

These are PRE-DRAFTS — write them only when the work is about to start, not before.
