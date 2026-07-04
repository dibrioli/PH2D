---
name: reference-panel-2d-drag-needs-dispatch
description: "Bespoke 2D-drag widgets (curve editors, XY pads) can't be built panel-side alone — they need an editor-core InteractiveState+dispatch arm; the 1D Slider is the only per-Move primitive panels get"
metadata: 
  node_type: memory
  type: reference
  originSessionId: bf528dd3-c72f-4062-8004-4ba44709ee12
---

A panel crate (`ph2d-panel-*`) CANNOT implement a live 2D-drag custom widget on
its own. `WidgetStore` exposes no live-pointer accessor (only `cmd_held`/
`shift_held`); panels receive only `WidgetEvent`/`PanelEvent` carrying a `NodeId`,
not pointer (x,y). The ONLY interactive primitive that emits a value per pointer-
Move is `InteractiveState::Slider` (1-D; `dispatch/number_input.rs::update_drag_value`
computes value from `active_rect` + pointer, both H and V orientations).

**Consequences for bespoke adjustment UIs (Painter W4+, 12+ kinds coming):**
- A 1-D handle (Levels black/gamma/white, a curve point at FIXED x) = register an
  `InteractiveState::Slider` (use `Vertical` for a Y-only handle) over a hit strip;
  the drag flows through the existing `SetValue → handle_panel_event` path. No
  foundational change. This is how W4 Curves' fixed-x editor + Levels ship.
- A free 2-D drag (arbitrary X+Y curve point, XY pad) NEEDS a new
  `InteractiveState` variant + a `dispatch/pointer.rs` arm in `ph2d-editor-core`
  (FOUNDATIONAL = Coord). Copy the precedent: BlenderColorPicker's SV wheel —
  `InteractiveState::BlenderHit{parent,kind}` + `dispatch/blender.rs::apply_blender_hit`
  + `wheel_pick(rect,px,py)` (`widget/blender_color_picker/`).
- The panel toolkit (`editor-core/paint.rs`) exposes only rect fill/stroke
  (`fill_rounded_rect`/`stroke_rounded_rect`/`stroke_rect`), NO polyline/circle —
  plot curves as dense dots, or add a `stroke_polyline` helper (foundational).

**Why:** saves re-discovering this (I fanned out the whole interaction/dispatch
tree to confirm it). Frozen `PanelEvent` (4 variants) also can't carry (idx,x,y) —
encode in `SelectOption(id,"...")` or add a variant via ADR. See
[[feedback-panel-populate-register]], [[project-diretriz-v68-2026-05-22]].
