---
name: project-painter-core-files-at-loc-cap
description: Painter core state files (paint.rs / brush_settings.rs / stroke.rs / trait_impls.rs) sit at EXACTLY 600 LOC — any feature touching PaintState/BrushSettings/stroke emission overflows the cap; budget a split
metadata: 
  node_type: memory
  type: project
  originSessionId: b76ad54d-f3ea-4098-9378-ab19bd9a93eb
---

Several **painter core files are pinned at EXACTLY 600 LOC** (the `architecture_workspace_file_loc_cap` ceiling, NOT allowlisted): `ph2d-tool-painter/src/tool/paint.rs`, `tool/paint/brush_settings.rs`, `ph2d-painter-brush/src/stroke.rs`, `ph2d-tool-painter/src/tool/trait_impls.rs`. So ANY feature that adds a `PaintState` field, a `BrushSettings` snapshot field, a stroke-emission hook, or a `set_source`/trait line **overflows the cap** and the build goes red on the LOC gate. Budget a split into the work — don't discover it at ship.

**Why:** the Painter is in heavy active dev; its god-files grew right up to the cap. A struct can't be split across files, so the field additions are irreducible — you must make ROOM elsewhere.

**How to apply — low-risk split moves (all preserve behaviour, verified 2026-06-29 symmetry feature):**
- Move `impl Default for <Struct>` to a **child module** (`mod state_default; use super::*;`). A child/descendant module retains construction access to the parent's module-PRIVATE fields, so no visibility churn. Frees ~60 LOC from `paint.rs`.
- Move a **pub-field struct + its helpers** to a sibling module that already consumes it (e.g. `BrushSettings` + `brush_falloff_weight_at` → `snapshot.rs`, where the builder lives). Pub fields ⇒ clean move; just fix the `pub use` re-export path.
- **Inline a 1-line chokepoint** at its call sites instead of a helper method when the file is at the cap (e.g. `crate::symmetry::push_symmetric(out, dab, &self.spec.symmetry)` at the 6 stroke emission sites — the doc lives in the chokepoint fn, not a wrapper).
- Per-frame work that would add a line to the capped `trait_impls.rs` (e.g. resolving geometry on `set_source`) → put it in `paint_tick` (in `paint.rs`, has room) instead, BEFORE its no-stroke early-return so it still runs when idle.

Do NOT reach for a FILE_OVERAGE_OK allowlist entry first — these are splittable. Allowlist is the `action_bus.rs` central-enum case, not this. See [[feedback-loc-cap-split-not-allowlist-and-fmt-reexpands]]. Also: `rustfmt` re-wraps long single-line calls to multi-line (the Anchored emission site needed a `let` local), so measure LOC AFTER `rustfmt <file>`, not before.
