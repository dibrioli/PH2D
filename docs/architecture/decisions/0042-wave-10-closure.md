# ADR-0042 — Wave 10 closure: gates ortogonais, typed color, fan-out drop-crate consolidated

**Status:** Accepted (2026-05-24)
**Decisor(es):** Enio + Claude (Coord-A, autonomous-loop session).
**Refer:** [ADR-0030..0039 — node substrate](0030-node-system-as-substrate.md)..[ADR-0039 — nodegraph contract freeze](0039-nodegraph-contract-freeze-w2t4.md), [ADR-0040 — tool-as-isolated-crate](0040-tool-as-isolated-feature-crate.md), [ADR-0041 — RasterEditTool amendment](0041-rasteredit-rename-and-deactivate.md), [docs/plans/2026-05-wave-10-perfection.md](../../plans/2026-05-wave-10-perfection.md).
**Tags:** wave-10, closure, gates, ph2d-color, padrão-ouro

---

## 1. Context

The Wave 10 perfection plan (2026-05-21..2026-05-24, 4-day autonomous-loop sprint) targeted three structural problems the user named on day zero:

1. **Multi-agent parallel work was still slow** — "someone always waiting on someone."
2. **The app center (editor-core + render + shells/desktop) was large + interdependent** — trivial bugs became "true hell."
3. **The UI was the worst spot** — agents weren't reliably consuming the canonical source-of-truth (`docs/design/tokens.json` → `ph2d-tokens`).

Plan v4 (after adversarial multi-agent review) shipped 7 etapas (`Etapa 0..7`) targeting:

| Etapa | Goal |
|---|---|
| 0 | Multi-agent infra (slot-env, git-stage-guard, COORD_OVERRIDE, parallel CI) |
| 1.A | Amendment ADR-0041: rename + `deactivate` hook |
| 1.B | `ph2d-tool-runtime` (5 helpers); BgRemoval first impl |
| 2 | CEQ + Upscale impl on RasterEditTool; Padding/EqSizes documented exceptions |
| 3 | 3 arch-gates + `drive_multi_preview_cache` helper |
| 4 | 3 new codegens (panel-sync, chrome-sync, widget-sync) |
| 5 | 5 UI gates extended to `panel-*`; 4 gates ortogonais; `ph2d-color` crate; CEQ split |
| 6 | Self-contained subset: LOC trend detector + memory GC + audit refinements |
| 7 | merge-on-green eligibility script + DIRETRIZ amendments + this closure ADR |

The plan also targeted (per "padrão ouro" mandate): golden-image SSIM (§6.1), panel-canonical-template AST (§6.2), no_tofu_glyphs amplified (§5.3.5), docs_bugs_have_gates (§5.3.7). These are explicitly deferred — see §3 below.

---

## 2. Decision

Close Wave 10 at commit `<closure-sha>` (this ADR) with the following frozen state:

### 2.1 Contracts frozen (no further amendments in Wave 10)

- **Tool** ≤ 10 methods. The `deactivate` slot was added in ADR-0041 (`as_image_edit_mut` → `as_raster_edit_mut`, same slot count).
- **RasterEditTool** ≤ 5 methods (`set_source` / `current_preview` / `take_pending_commit` / `run_full` / `deactivate`). The next amendment requires a separate ADR + adversarial review.
- **PanelEvent** ≤ 4 variants.
- **ph2d-tool-runtime LOC** ≤ 650. The final cap. Helpers added in Wave 11+ require splitting the crate or arguing for the bump.

### 2.2 Gates active post-Wave-10 (architecture invariants)

**UI gates** (extended in Etapa 5.1 to `crates/ph2d-panel-*/src/**`):
- `no_literal_color` — no raw OKLCH literals; everything via `ColorToken::resolve`.
- `no_magic_numeric` — no raw float literals; via `Spacing`/`Radius`/`StrokeToken`/`TypeToken`/`Density` token or `// LITERAL-PX-OK: <reason>` marker.
- `no_tofu_glyphs` — no out-of-font arrow/cmd glyphs in UI strings.
- `hr12_widgets_a11y` — every widget file wires `ph2d_a11y` (panels delegate via canonical primitives or appear in `PANEL_A11Y_DELEGATE_OK`).
- `hr15_no_hardcoded_ui_strings` — frozen baseline; new code uses `t!(...)` when Fluent runtime ships.

**Gates ortogonais** (Etapa 5.3 + 6 refinements):
- `arch_no_absolute_drag_pattern` — bans `event.x - start_x`; use delta-per-frame.
- `arch_no_char_count_widths` — bans `chars().count() * GLYPH_W` (proportional fonts).
- `arch_safe_clamp_only` — forces `crate::math::safe_clamp` (NaN/swap-tolerant) for dynamic clamp bounds.
- `arch_mode_has_reconcile` — `pub fn set_*_mode` setters must reconcile via canonical keyword or named `RECONCILES_VIA` entry.
- `arch_color_space_typed` — bans bare `Vec<u8>`/`&[u8]` color params; use `ph2d_color::{SrgbRgba,LinearRgba,Premultiplied,OklchColor}`.

**LOC + structural gates:**
- `architecture_panel_loc_cap` — 600 LOC/file + 200 LOC/fn for `panel-*/src/**` (with documented overage list).
- `architecture_widget_loc_cap` — 500 LOC for widget primitives (HR-18 inherited).
- `ph2d-loc-trend check` — fails if a critical file grew > 10% in 30 days without ADR.

**Codegen + staleness gates:**
- `tool-sync`, `panel-sync`, `chrome-sync`, `widget-sync`, `node-sync` — 5 codegens; each has a semantic staleness gate that catches "forgot to run sync after dropping a crate."

### 2.3 Crates added

- **`ph2d-color`** (Etapa 5.4) — 4 modules (linear, srgb, premultiplied, oklch), 17 unit tests, zero external deps. Explicit conversions only (`to_linear`, `to_srgb`, `premultiply`, `unmultiply`). Migration of the 10 BASELINE files (existing `rgba: &[u8]` surfaces) is the Etapa 5.4 follow-up sweep (1-2 weeks), tracked in `arch_color_space_typed` BASELINE.

- **`ph2d-editor-core::math`** (Etapa 5.3) — small module hosting `safe_clamp` (NaN-aware, swap-tolerant). 3 unit tests.

- **`tools/ph2d-loc-trend`** + **`tools/ph2d-memory-gc`** (Etapa 6.3 + 6.4) — zero-dep std-only CLI tools. `loc-trend` records LOC over time in `metrics/loc-trend.json`; `memory-gc` validates that paths in the agent's persistent memory file still resolve.

### 2.4 Merge-on-green eligibility

`scripts/auto-merge-eligibility.sh <base-ref> [<head-ref>]` exits 0 only when **all three** hold:

1. Diff is entirely within `crates/ph2d-{node,tool,panel}-<slug>/` OR `docs/Testes/`.
2. No foundational/contract-frozen path touched (workspace `Cargo.toml`/`Cargo.lock`, `scripts/`, `.github/`, ADRs, DIRETRIZ, `CLAUDE.md`, `SKILL_*`, `shells/`, the core/runtime/registry/render/gpu/color/tokens crates, `tools/`).
3. At most ONE drop-crate touched (multi-crate goes through Coord review).

Fail-safe: ANY ambiguity → exit 1 (coord-review). This is the policy half of Etapa 7.1; the GH Action wiring is a Wave 11 follow-up (the script is the contract).

### 2.5 Deferred to Wave 11 (with reason)

| Item | Why deferred | Trigger to resume |
|---|---|---|
| §6.1 Golden-image SSIM | Needs Enio to visually approve ~17 baseline PNGs (widgets + 9 panels). Vello-headless infra is mechanical; approval is the semantic gate. | Dedicated session with Enio in the loop. |
| §6.2 panel-canonical-template AST | Needs Coord-A to commit to which section of `pre_populate.rs` is the canonical template. | Same — joint Enio + Coord session. |
| §5.3.5 `no_tofu_glyphs` amplified (U+2000-FFFF minus Inter coverage) | Requires loading Inter's actual glyph coverage table. Significant infra; basic version (arrows + cmd block) extended in §5.1 covers ≥80% of historical incidents. | When a tofu bug ships outside the arrows/cmd block. |
| §5.3.7 `docs_bugs_have_gates` | Backfill 90 entries (UI_Bugs + Image Tools Bugs) with `**Gate:**` annotations. Paired with §6.2 doc sweep. | Same session as §6.1/§6.2. |
| `arch_color_space_typed` BASELINE sweep | 10 sites with `rgba: &[u8]`; migration to `ph2d-color` typed wrappers is the 1-2 week sweep called out in the plan. | Wave 11 Etapa A — tracked. |
| 3 long-paint fns (bgremoval/grid-snap) | Each is 200-410 LOC. Split into section helpers needs per-panel smoke. | Wave 11 Etapa B — tracked in `architecture_panel_loc_cap::FN_OVERAGE_OK`. |
| GH Action wiring of `auto-merge-eligibility.sh` | Out-of-scope per §7.2 ("policy first, automation next"). | When Coord-A is unblocked from autonomous-loop work. |

---

## 3. Consequences

### Positive

- **Drop-crate fan-out is the canonical path** for ~95% of feature additions (DIRETRIZ §3.8 unified). Tool / node / panel / widget / chrome additions all follow `drop file → run sync → commit`.
- **Bug classes blocked at the gate, not at code review:** absolute drag, char-count widths, naive clamp, mode-without-reconcile, sRGB-vs-linear confusion. The shape of every recurring bug from the prior wave (UI_Bugs §1-§9, Image Tools Bugs §1-§3) has at least one architectural gate.
- **CEQ split (824 → 318+555+137 LOC) proves the panel split recipe** — replicable for the 3 long-paint fns (bgremoval/grid-snap) in Wave 11.
- **`ph2d-color` exists** even before consumers — the contract is the deliverable. Future tools/render code write against typed wrappers from day one.
- **Memory GC tool catches stale agent memory refs**, closing the "agent acts on a stale path" failure mode.
- **LOC trend detector** establishes the longitudinal record that lets us detect god-files BEFORE they reach the 600 LOC cap.

### Negative

- **`ph2d-color` is not yet consumed** — the 10 BASELINE files still pass bytes around. Until the migration sweep runs, the gate is a "no new bare-byte surfaces" lock; existing surfaces unchanged.
- **3 long-paint fns remain (bgremoval/grid-snap/grid-snap)** — frozen at current LOC via `FN_OVERAGE_OK`, but the actual splitting (cf. CEQ in Etapa 5.2) is Wave 11 work.
- **Golden-image SSIM is the biggest gap** — without it, visual regressions still rely on the Enio smoke. The Wave 10 plan budgeted 2-3 weeks for it; deferred because baseline approval is the gating step and the autonomous-loop session couldn't get Enio in the loop.
- **`hr15` baseline has 4 entries** (showcase + 2 placeholder strings) — represents real i18n debt until the Fluent runtime ships.

### Neutral

- DIRETRIZ.md was NOT amended in Etapa 7 — the plan §7.4/§7.5 specified a tooling-driven §1.4 rewrite. The closure decision is to keep the current DIRETRIZ as-is until §7.4's `tools/ph2d-triagem` ships. The CLAUDE.md + memory pointers already document Etapa 5/6 outcomes.

---

## 4. Acceptance criteria

This ADR was applied successfully if:

- [x] `cargo check --workspace` clean.
- [x] `cargo fmt --all -- --check` clean.
- [x] `cargo clippy -p ph2d-color -p ph2d-editor-core -p ph2d-loc-trend -p ph2d-memory-gc --all-targets -- -D warnings` clean.
- [x] All 5 UI gates + 5 ortogonais + LOC cap + ph2d-color tests pass (60+ sub-tests).
- [x] `scripts/auto-merge-eligibility.sh main HEAD` correctly identifies the Etapa 5 + Etapa 6 commits as `coord-review required` (because they touched `Cargo.lock` + `crates/ph2d-editor-core/`).
- [x] `docs/Testes/README.md` has §E5 + §E6 sections with smoke checklists G10-G16 for Enio's final visual audit.
- [x] `docs/architecture/decisions/0042-wave-10-closure.md` (this file) committed.
- [ ] Deferred items 5.1/5.2/5.3.5/5.3.7/color-migration/long-paint-split tracked in `docs/plans/2026-05-wave-10-perfection.md` as "Wave 11 carry-over."

---

## 5. Wave 10 stats

| Metric | Value |
|---|---|
| Commits | 9 (`d9379ee`, `a03d830`, `74b6d27`, `cbb9cb3`, `666a85a`, `6776da7`, `0460415`, Etapa 6 partial commit, this closure commit) |
| New crates | 2 (`ph2d-color`, planned ph2d-canonical deferred) |
| New tools | 5 (`ph2d-panel-sync`, `ph2d-chrome-sync`, `ph2d-widget-sync`, `ph2d-loc-trend`, `ph2d-memory-gc`) |
| New gates | 11 (5 UI extended + 5 ortogonais + 1 LOC cap) |
| New modules | 2 (`ph2d-editor-core::math`, `ph2d-color::*`) |
| LOC sweeps | 69 `no_magic_numeric` violations migrated, CEQ paint 824 → 318+555+137, 6 panel-* files restructured |
| Adversarial audits | 4 (Etapa 1.B, 2, 3, 4, 5) — each surfaced 1-3 critical fixes pre-commit |
| Sessions | 1 autonomous-loop (this) + N background agents (CEQ split agent for §5.2, audit agents per etapa) |

---

## 6. Wave 11 carry-over (recommendations)

Listed in priority order — Wave 11 implementer should pick the highest-leverage ones first.

1. **Golden-image SSIM (§6.1)** — biggest reduction in Enio smoke time (~70% per plan estimate).
2. **`ph2d-color` migration sweep** — unlocks `arch_color_space_typed` to deny all 10 BASELINE sites.
3. **3 long-paint fns split** (`bgremoval/paint::paint`, `grid-snap/paint::paint_body`, `grid-snap/populate::populate`) — uses the CEQ split recipe.
4. **panel-canonical-template AST (§6.2)** — prevents store/widget storage divergence.
5. **GH Action wiring** of `auto-merge-eligibility.sh` (§7.1 automation half).
6. **`docs_bugs_have_gates` backfill** — closes the doc-vs-gate drift.
7. **`no_tofu_glyphs` amplified** — only when a real bug ships outside the arrows/cmd block.
