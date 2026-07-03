# ADR-0103 — Selection system (Procreate parity), snapshot-integrated undo

**Status:** Accepted (Enio, 2026-07-02) · **Supersedes:** none · **Related:** ADR-0040 (tool
contract), ADR-0099/0100 (painter host), ADR-0102 (inpaint mode precedent).

## Context

The Painter needs a Procreate-grade **Selection** tool: a canvas-wide selection mask that gates every
other operation (paint / fill / smear / adjustment), with modes **Automatic / Freehand / Rectangle /
Ellipse**, boolean operators **Add / Remove / Invert**, **Feather**, and actions **Copy & Paste / Color
Fill / Save & Load / Clear / Select layer contents**. The visual language is Procreate's: **marching
ants** while editing, **diagonal hatching over the deselected area** once committed.

Two hard constraints from Enio:
1. The panel of parameters lives **in the same left dock as the Brush properties**, styled with
   sections + cards (Widget Gallery components), modes in a **toggle/segmented group**, and it must
   **reflow on narrow panels** (tablets/iPads). In Selection mode the panel shows **only** selection
   params — nothing shared with other tools.
2. Selection must be **fully integrated into the painter's single interleaved undo/redo queue** — the
   same chronological sequence that already interleaves brush strokes, fills, mask edits, layer ops,
   shape commits and adjustments.

## Decision

**Selection is a `PaintMode` inside `ph2d-tool-painter`, not a new crate or a frozen-contract change.**

1. **State — document-wide coverage mask.** A `selection_mask: Arc<Vec<u8>>` (canvas-sized, 0..255
   coverage for feather) plus `selection_active: bool`, living in `PaintState`. It is **document-wide**
   (Procreate semantics), not per-layer.

2. **Undo — snapshot-based, one queue (no new machinery).** The painter undo is snapshot-based: every
   op is a `before/after` pair of `ModelSnapshot` pushed via `commit_structural_edit`. Selection joins
   by adding `selection_mask` + `selection_active` **fields to `ModelSnapshot`**, captured in
   `snapshot_model` and restored in `restore_model` — the **exact three-touchpoint precedent of
   `mask_scratch`** (`tool/paint/mask.rs`). Each committing selection edit (new / Add / Remove / Invert
   / Feather-apply / Clear / Copy) is one `snapshot → mutate → commit_structural_edit`, so it enters the
   single queue interleaved with all other ops. `undo()/redo()` are unchanged (type-agnostic swaps).

3. **Paint gate — reuse the mask precedent.** When a selection is active, each dab multiplies its
   coverage by `selection_mask`, mirroring `mask.rs::restore_protected_region`. Every existing paint
   mode respects the selection for free.

4. **Engines — reuse.** Automatic = the existing **Fill flood** (`fill.rs`, threshold-drag). Rectangle/
   Ellipse/Freehand rasterize a region and `combine(op, region)`. Feather = a separable
   (transcendental-free, HR-5) blur on the mask alpha.

5. **Panel — mode-exclusive section inside `ph2d-panel-painter-layers`.** A `is_selection` flag on the
   brush snapshot drives an **early-return exclusive branch** in `paint_brush_body` (the `is_inpaint`
   precedent), painting a Selection-only section built from `paint_collapsible_section` + two
   `SegmentedAdaptive` groups (modes + boolean ops) + `paint_slider_chip_row` (Feather/Threshold) +
   an Actions card. `SegmentedAdaptive` provides the required narrow-width reflow.

6. **Persistence — in-memory first.** Save & Load selections are **in-memory named slots** (no
   `SCHEMA_VERSION` bump). Persisting selections in the `.ph2d` file is an explicit follow-up with its
   own schema change.

## Acceptance criteria (frozen — DoD per DIRETIVA §5)

- A committed selection **restricts painting**: pixels outside the mask are provably unchanged.
- A selection edit **appears in the single undo queue interleaved** with a brush stroke; `undo`/`redo`
  round-trip the mask **and** pixels in lock-step (headless test).
- All four modes + three operators produce the expected mask (behavioral seam tests driving real
  pointer/panel events, not compile-green).
- The panel in Selection mode shows **only** selection controls (a test proves no shared control leaks,
  mirror `inpaint_mode_hides_every_unused_brush_section`).
- Panel controls reflow (no clip) at a narrow width.
- Visual: marching ants + hatching match the mask (Enio smoke).

## Kill-criteria (before the 3rd topology rebuild — two-strikes)

- If marching-ants + flood at 4K exceed **16 ms/frame** after the 2nd attempt, stop and prove the model
  before a 3rd rewrite; fall back to hatching-only + low-res ants.

## Amendment 1 (Enio, 2026-07-02) — Selection Edit Mode reuses the Shape system

The selection is **two-phase**. Until the panel's **"Edit Selection"** button is pressed the behaviour is
Procreate-identical (marquee / lasso / auto → mask, no handles). Pressing it enters an **edit mode** where
the selection boundary becomes an **editable Shape**, reusing the Painter's Stroke-Method shape editors:

1. **Handles / gizmos:** Freehand / Rectangle / Ellipse reuse `ShapeEditState` + the `curve`/`ellipse`/
   `polygon`/`line` on-canvas editors and their handles/tangents/`TransformGizmo`. Boundary→shape mapping:
   Ellipse→`EllipseState`, Rectangle→closed polyline (`LineState`, 4 corners), Freehand→`CurveState`. Each
   edit re-rasterizes the **filled closed region** into `selection_mask` (reuse `curve_geom` flatten + the
   even-odd scanline fill).
2. **Stroke Offset** (`curve_offset`/`line_offset` + the Offset slider accumulator) applies to a selection
   as **grow/shrink** (expand/contract) of the boundary.
3. **Freehand brush stabilization** smooths the lasso path (through the brush stabilizer) before it becomes
   a Curve.
4. **Undo:** boundary edits join the ONE timeline via `shape_snapshot` (`begin/commit_shape_txn`).

Obstacle noted: the shape editors currently STROKE a path; selection needs the FILLED closed region — the
fill path (flatten → even-odd scanline) is the bridge.

## Alternatives rejected

- **Separate `ph2d-panel-selection` crate** — fragments the tool↔panel snapshot channel, duplicates the
  dock/scroll/resize chrome, contradicts "same panel as the Brush". Rejected.
- **Command/inverse-op undo for selection** — the painter undo is snapshot-based; a parallel command
  stack could not interleave (the exact reason `curve_undo` was collapsed into the one queue). Rejected.
- **Per-layer selection** — Procreate selection is document-wide; per-layer diverges from parity.
