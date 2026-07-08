---
name: feedback_panel_arch_gates_scope_and_clamp_const
description: no_magic_numeric + arch_safe_clamp_only scan EVERY ph2d-panel-*/src; hoisting clamp bounds to consts triggers the clamp gate
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6a447b61-cfd2-42ca-849b-8e88b33dc598
---

Two editor-core arch-gate tests scan **every `ph2d-panel-*/src/**`**, not just
`ph2d-editor-core/src/{widget,screens}` (Wave 10 / Etapa 5.1–5.3 extended the
scan roots): `no_magic_numeric` (bare f32/f64 UI literals) and
`arch_safe_clamp_only` (`.clamp(min,max)` calls). Both run via
`cargo test -p ph2d-editor-core`. A new panel crate inherits them silently — and
`cargo check` (the inner loop) NEVER runs them, so a panel can accumulate a red
gate for whole phases before anyone runs the full editor-core suite.

Concrete bite (Motion Nodes M1 Phase 1b-2, 2026-07-07): `ph2d-panel-motion-graph`
had **both gates latently red since Phase 1a/1b-1** — the graph-canvas geometry
(CARD_W, socket radii, wire widths, zoom clamps) tripped both, but nobody ran
`cargo test -p ph2d-editor-core` during 1a/1b dev. See [[feedback_full_gate_periodically]].

**Fixes that satisfy the gates:**
- `no_magic_numeric`: use a design token (Spacing/Radius/TypeToken/…) for chrome,
  OR a trailing `// LITERAL-PX-OK: <reason>` for genuine canvas/math values. A
  node-graph canvas has its own coordinate geometry (like the sprite/vector
  canvases) → LITERAL-PX-OK is legitimate. `ph2d-panel-painter-layers` uses ~181
  of them (mostly on named `const` lines so fmt keeps the comment on-line).
- `arch_safe_clamp_only`: `safe_clamp(v,min,max)` from `ph2d_editor_core::math`,
  OR a trailing `// CLAMP-OK: <reason>` when both bounds are literal non-NaN.

**Non-obvious interaction (the trap):** the two gates pull OPPOSITE ways on clamp
bounds. `no_magic_numeric` wants inline literals hoisted to named consts; but
`arch_safe_clamp_only`'s detector treats **identifier** bounds as potentially
dynamic, so `.clamp(ZOOM_MIN, ZOOM_MAX)` is flagged while `.clamp(0.2, 2.5)` is
not. Hoisting the bounds to consts (to please the magic gate) NEWLY trips the
clamp gate → you must also add `// CLAMP-OK` on the clamp line.

**Why:** the inner loop (`cargo check -p`) hides both gates; they only fire in
the batched editor-core test run. **How to apply:** when touching any
`ph2d-panel-*` file, run `cargo test -p ph2d-editor-core` at phase close
(catches magic/clamp/LOC), and when you hoist clamp bounds to consts, expect to
add `// CLAMP-OK` too. Related: [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]],
[[project_integration_prefork_lines_ship_drift]].
