---
name: reference-number-input-register-range
description: "Bounded NumberInput boxes must register (min,max,step) via set_number_range so the drag-scrub is range-proportional + clamped (else a ±1 box races past 100, the stepper jumps)"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 93ee2f69-a04d-4352-82ac-69624f6a510d
---

Every **bounded** `NumberInput` box must register its range via `WidgetStore::set_number_range(id, min, max, step)` (call it where you paint the box, e.g. each frame). Without it:
- the **drag-scrub** uses the legacy `rate × step` (`DRAG_RATE_X`=50 / `DRAG_RATE_Y`=5 step-units/px), so a small-range box (Offset ±1) **races past 100** on a few pixels and never clamps;
- the **stepper** +/- arrows (and continuous hold) increment by a **buffer-inferred** step (1.0 for an integer-looking buffer → 1→2→… "goes to 10" on a hold).

With `number_range` set (foundational, `ph2d-editor-core`):
- the drag maps the cursor displacement PROPORTIONALLY to `[min,max]` — full range over `DRAG_RANGE_PX_H`=250 px horizontal (fast) / `DRAG_RANGE_PX_V`=2500 px vertical (precise), Shift ×0.001 — and **clamps** to `[min,max]` (pointer_move.rs);
- the stepper Down + hold tick use the registered `step` and clamp (number_input.rs / tick.rs).

Reference impl: `ph2d-panel-painter-layers::number_field::chip` registers the range for every Grain/Shape param box — works in BOTH the Brush panel and the Layers texture-layer editor (same fixed ids). Unbounded boxes (e.g. Inspector pixel Position) register NO range and keep the legacy step-based rate. **New number boxes anywhere in the app SHOULD register their range.** Enio 2026-06-25. See also [[feedback_panel_2d_drag_needs_dispatch]] for 2D-free drags.
