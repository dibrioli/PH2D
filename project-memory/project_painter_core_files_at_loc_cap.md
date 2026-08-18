---
name: project-painter-core-files-at-loc-cap
description: HISTÓRICO — a premissa (arquivos do Painter no teto de 600) DISSOLVEU-SE em 2026-08-18; o que sobrevive são os movimentos de split de baixo risco
metadata: 
  node_type: memory
  type: project
  originSessionId: b76ad54d-f3ea-4098-9378-ab19bd9a93eb
---

> ⚠️ **A PREMISSA DESTA MEMÓRIA DISSOLVEU-SE — não orce split por causa dela.** Medido em
> 2026-08-18: o cap do workspace é **700**, não 600 ([ADR-0105](../docs/architecture/decisions/0105-file-loc-cap-600-to-700.md)),
> e os quatro arquivos medem **315 · 621 · 650 · 627** — nenhum no teto, nenhum a estourar.
> O que sobrevive é a **técnica** (os movimentos de split abaixo, verificados) e a razão de
> fundo: *uma struct não se parte entre arquivos, então a adição de campo é irredutível — a
> folga tem de vir de outro lugar*. ⚠️ E há **quatro caps distintos** neste repo — workspace
> **700** · painel **600** arquivo / **200** função · shell **600** · widget **500** — confundi-los
> é erro recorrente. A fonte é o gate, nunca uma nota: [`architecture_workspace_file_loc_cap.rs`](../crates/ph2d-editor-core/tests/architecture_workspace_file_loc_cap.rs).
>
> *Registro do texto original, para quem quiser a história:*

Several **painter core files are pinned at EXACTLY 600 LOC** (the `architecture_workspace_file_loc_cap` ceiling, NOT allowlisted): `ph2d-tool-painter/src/tool/paint.rs`, `tool/paint/brush_settings.rs`, `ph2d-painter-brush/src/stroke.rs`, `ph2d-tool-painter/src/tool/trait_impls.rs`. So ANY feature that adds a `PaintState` field, a `BrushSettings` snapshot field, a stroke-emission hook, or a `set_source`/trait line **overflows the cap** and the build goes red on the LOC gate. Budget a split into the work — don't discover it at ship.

**Why:** the Painter is in heavy active dev; its god-files grew right up to the cap. A struct can't be split across files, so the field additions are irreducible — you must make ROOM elsewhere.

**How to apply — low-risk split moves (all preserve behaviour, verified 2026-06-29 symmetry feature):**
- Move `impl Default for <Struct>` to a **child module** (`mod state_default; use super::*;`). A child/descendant module retains construction access to the parent's module-PRIVATE fields, so no visibility churn. Frees ~60 LOC from `paint.rs`.
- Move a **pub-field struct + its helpers** to a sibling module that already consumes it (e.g. `BrushSettings` + `brush_falloff_weight_at` → `snapshot.rs`, where the builder lives). Pub fields ⇒ clean move; just fix the `pub use` re-export path.
- **Inline a 1-line chokepoint** at its call sites instead of a helper method when the file is at the cap (e.g. `crate::symmetry::push_symmetric(out, dab, &self.spec.symmetry)` at the 6 stroke emission sites — the doc lives in the chokepoint fn, not a wrapper).
- Per-frame work that would add a line to the capped `trait_impls.rs` (e.g. resolving geometry on `set_source`) → put it in `paint_tick` (in `paint.rs`, has room) instead, BEFORE its no-stroke early-return so it still runs when idle.

Do NOT reach for a FILE_OVERAGE_OK allowlist entry first — these are splittable. Allowlist is the `action_bus.rs` central-enum case, not this. See [[feedback-loc-cap-split-not-allowlist-and-fmt-reexpands]]. Also: `rustfmt` re-wraps long single-line calls to multi-line (the Anchored emission site needed a `let` local), so measure LOC AFTER `rustfmt <file>`, not before.
