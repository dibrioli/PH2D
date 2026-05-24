# ADR-0041 — Amendment to ADR-0040: `ImageEditTool` → `RasterEditTool` + `deactivate` method

**Status:** Accepted (2026-05-23)
**Decisor(es):** Enio + Claude (Coord-A).
**Estende:** [ADR-0040 §7](0040-tool-as-isolated-feature-crate.md) — the tool contract freeze. This is the **only sanctioned cap bump** in Wave 10; further amendments deferred per `docs/archive/plans-completed/2026-05-wave-10-perfection.md` §X.
**Tags:** wave-10, contract-amendment, frozen-contract

---

## 1. Context

ADR-0040 (TG-E, 2026-05-22) froze the tool contract at:
- `Tool` ≤ 10 methods
- `ImageEditTool` ≤ 4 methods (`set_source` / `preview` / `take_pending_commit` / `run_full`)
- `PanelEvent` ≤ 4 variants

The freeze achieved its goal: every `ph2d-tool-*` satellite crate now builds against a fixed contract. **But** the freeze also exposed a documented gap (DIRETRIZ §3.8.3.1): "ImageEditTool está definido e congelado, mas nenhum tool de produção implementa hoje (downcast ainda é o padrão)". The shell does 33 `as_any_mut().downcast_mut::<ConcreteTool>()` calls today, because:

1. There is no lifecycle hook for "tool deactivated" inside `ImageEditTool`. The shell must call `tool.on_deactivate()` (from `Tool`) and then *separately* know to clear concrete state — which it does via downcast. A trait-level `deactivate` makes the contract self-contained.

2. The name `ImageEditTool` is correct for the bgremoval / padding / color-equalization / upscale / equalize-sizes family (5 stateful raster tools), but the next generation of tools the project plans for (vector edits, physics edits, gameplay node-emit) will need parallel sub-traits with the same shape. Forcing them into `ImageEditTool` creates impedance mismatch; introducing a parallel `VectorEditTool` later beside `ImageEditTool` makes the naming asymmetric. Renaming **now** — while no production crate implements the trait — costs ~17 hits in 6 files (mostly doc-comments); renaming later, after Etapa 2 puts 5 impls on it, costs 5× more.

3. The `preview(&self)` signature returns a `&[u8]` borrow of cached state. The current 5 bridges in the shell pair this with a separate `take_params_dirty()` inherent method on each concrete tool. Folding the "drain dirty" into the preview accessor (now `current_preview(&mut self)`) is the small refactor that lets the generic `ph2d-tool-runtime` (Etapa 1.B) replace the 5 bridges with one loop. Without it, the runtime needs per-tool downcast just to know when to repaint — exactly the anti-pattern the freeze was supposed to retire.

This is the **only contract amendment** in Wave 10. All other contract changes (deferred items D.1–D.6 in the perfection plan) reopen only on concrete evidence of need.

---

## 2. Decision

### 2.1 Rename `ImageEditTool` → `RasterEditTool`

Rationale: future-proof for parallel sub-traits in other domains (vector, physics, node-emit) without forcing an asymmetric naming where the raster family alone has a generic name. The string-needle in `architecture_tool_contract_surface.rs::image_edit_tool_contract_is_capped` becomes `raster_edit_tool_contract_is_capped`.

Method rename on `Tool`: `as_image_edit_mut` → `as_raster_edit_mut`. (`Tool` cap stays at 10 — same slot, new name.)

### 2.2 Rename `preview(&self)` → `current_preview(&mut self)`

Semantic change: the new method **drains the tool's dirty flag** before returning the current preview frame, returning `Some` only when the preview is new since the last call. This makes the generic `ph2d-tool-runtime` shell driver (Etapa 1.B) self-contained — it knows when to repaint without a downcast to a tool-specific `take_params_dirty()` inherent.

Implementors who don't track dirty state can return `Some(...)` every call (constant-poll fallback) at zero correctness cost; the runtime overhead is one cache write per frame, well under HR-4 budget.

### 2.3 Add `deactivate(&mut self)` on `RasterEditTool`

Lifecycle hook for "tool was deactivated". Separate from `Tool::on_deactivate` because `Tool::on_deactivate` is invoked on **any** active-tool switch (including swapping between two RasterEditTools), whereas `RasterEditTool::deactivate` is invoked specifically when the active tool transitions away from raster editing entirely — the moment the runtime must clear preview overlays, drop `pending_apply` flags, and release source buffers. The generic runtime calls it; concrete tools no longer rely on the shell remembering to call their own `clear_preview()` inherent methods via downcast.

**Cap bump:** `RasterEditTool` 4 → 5. This is the single line in `architecture_tool_contract_surface.rs` that grows.

### 2.4 NOT bumped in this amendment

The full v4 plan (`docs/archive/plans-completed/2026-05-wave-10-perfection.md` §II) initially proposed bumping `Tool` 10 → 11 (adding `as_raster_edit_mut`). Audit verification showed the method already exists at slot 9 today as `as_image_edit_mut` — renaming does not change the count. Cap stays at 10.

The plan also reserved space for `RasterFrame` / `ImageView<'_>` / `ImageBuf` typed wrappers (preventing `Vec<u8>` shape drift). Those wait for the `ph2d-color` crate in Etapa 5 (typed color spaces); introducing them now without `ph2d-color` would be a half-typed contract. The interface stays `Vec<u8>` / `&[u8]` for now; cap is unchanged on that axis.

---

## 3. Consequences

### Accepted

- The shell can replace 5 per-tool bridges with one generic loop driven by `RasterEditTool::current_preview` + `take_pending_commit` + `deactivate`. The 33 downcasts shrink to ≤2 (allowlist for eyedropper / protect_brush, ADR-0040 §3 documented exception).
- Future tool families (vector, physics, node-emit) get parallel sub-traits without conflicting with `RasterEditTool`.
- The contract stays minimal (5 methods on `RasterEditTool` — same as nodegraph's `NodeOp ≤ 2`).

### Costs

- ~17 rename hits across 6 files (5 code, 1 cycle test, 3 tool-crate doc-comments). All mechanical, single commit.
- DIRETRIZ §3.8.3.1 needs rewriting (the "ImageEditTool definido mas zero tools usam" caveat) once Etapa 2 ships its 5 impls. Update is part of Etapa 5 (DIRETRIZ amendments).
- `preview(&self) → current_preview(&mut self)` is the only behavioral change. No production code implements the old method yet, so there is no migration outside the doc-comment changes.

### Frozen again

After this amendment, the contract is frozen at:
- `Tool` ≤ 10 methods (unchanged)
- `RasterEditTool` ≤ 5 methods (was `ImageEditTool` ≤ 4)
- `PanelEvent` ≤ 4 variants (unchanged)

The arch-gate `architecture_tool_contract_surface.rs` caps are bumped to match. Further changes are rare Coord-A events with their own ADR.

---

## 4. Alternatives considered

- **Add `deactivate` without renaming**: rejected — renaming costs less now (no impls yet) than later (5 impls after Etapa 2). Coupling the two minimal changes into one amendment avoids two political ceremonies.
- **Introduce `RasterFrame` typed wrapper now**: rejected — without `ph2d-color` (Etapa 5) the wrapper would be a one-off type that needs replacing again later. Deferring keeps the contract honest about its actual typed surface.
- **Skip the `preview` → `current_preview` rename**: rejected — the rename is what enables the generic runtime to retire the 5 bridges. Without it, the runtime needs per-tool downcast to know when to repaint, which retires zero downcasts in practice.

---

## 5. Verification

After this amendment lands:

```sh
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface
# Must pass with new caps (Tool=10, RasterEditTool=5, PanelEvent=4).

cargo test -p ph2d-editor-core
# Internal Raster test in tool.rs::tests must pass with new names.

cargo test --workspace
# All tool-* crates must still compile (doc-comment-only changes).
```

The `Raster` test struct in `crates/ph2d-editor-core/src/tool.rs::tests` is the canonical reference impl. It demonstrates `impl RasterEditTool for Raster` with all 5 methods (including `deactivate`).

---

## 6. Tracking

- This ADR amends [ADR-0040 §7](0040-tool-as-isolated-feature-crate.md).
- Part of [Wave 10 perfection plan](../../archive/plans-completed/2026-05-wave-10-perfection.md) Etapa 1.A.
- Tag baseline: `pre-perfection-2026-05-24`. Rollback via `git reset --hard pre-perfection-2026-05-24`.
- Next step: Etapa 1.B (create `ph2d-tool-runtime` crate + migrate `BgRemovalTool` as the first `RasterEditTool` impl, with shell-delete codegen via extended `ph2d-tool-sync`).
