# HANDOFF — Stroke Multi-Shape (Enio 2026-07-04)

Multiple **simultaneously-editable stroke shapes** on one canvas, mirroring the Selection subsystem, with a
per-shape **Operation** (Overlay / Add / Remove). Landed as a tested interactive system; the boolean-region
*render* (Add/Remove combination) is the one scoped follow-up (see §Deferred).

## Architecture (the one to keep in your head)

- **`Vec<StrokeShape>` = the parked shapes** (`tool/paint/stroke_multi.rs`). Exactly ONE shape is the live
  editor in the existing `PaintState::{curve,ellipse,polygon,line}` slots; every OTHER shape is **parked**
  as plain geometry (`ShapeEditState`) + its `StrokeOp`. This keeps the ~100 `self.paint.curve` sites
  untouched — only a thin parked layer wraps them.
- **The canvas pixels are a DERIVED cache.** `restamp_shapes_preview(own_dabs)` re-stamps the active shape's
  dabs PLUS every parked shape's dabs onto ONE pristine baseline each recompose. The four `*_refill` paths
  funnel their final stamp through it. **Invariant: nothing is baked until the final Apply** — that is what
  makes overlapping vector edits clean (same idea as `recompose_selection_mask`).
- **No hard "active index".** A Down that would EDIT the active shape (precise per-editor hit — anchor /
  handle / gizmo / on-outline, incl. the rotate ring OUTSIDE the outline) stays with it. A Down on a PARKED
  shape's AABB re-activates it (`activate_parked_shape`, parking the current one first). A Down in EMPTY
  space with a complete active shape parks it and the following `*_down` starts a fresh shape.
- **Undo** clones the whole set: `ModelSnapshot.parked_shapes: Vec<ParkedShapeState>` (geometry + wire op),
  captured in `capture_shape_model`, restored (before the active editor) in `restore_shape_overlay`.

## Gesture model (Enio's spec)

- **Auto-accumulate:** a new drag in empty space begins another shape; clicking a shape re-edits it.
- **Curve pen:** a click near the curve inserts an anchor (unchanged); a click FAR (> `NEW_SHAPE_INSERT_BAND_PX`
  = 20 px, or `3×` grab tol) starts a NEW curve. Line point-placement is never interrupted.
- **Enter / Apply** bakes EVERY shape at once + drops them all (`commit_open_shape`, one undo entry).
- **Apply & Keep** bakes the pixels but keeps all shapes editable (parked persist naturally).
- **Cancel / Esc / leave method** reverts every shape's preview + drops the set.

## Operation (multi-shape)

- Panel **OPERATION card** (Overlay / Add / Remove) in the Stroke section — mirrors the Selection card
  (`paint_stroke.rs::operation_card`, ids `PAINTER_STROKE_OP*`, routed in `stencil.rs` →
  `set_stroke_op_mode`). Replaces "New" with **Overlay** (no boolean). Default = Overlay.
- Each shape stores its op; a new shape adopts the current mode (`begin_shape_session_base`).
- **Gizmo type-square glyph** `+` / `−` / `○` per shape (`painter_bridge_op_badges.rs`, drawn as vector, no
  text). Parked shapes also get a faint AABB frame so they read as editable. A quick **tap** on the centre
  square cycles the op (`op_tap` → `cycle_active_op`); a **drag** moves the shape (Enio: "clique muda o tipo,
  arraste move").

## Tests (headless, `tool/paint/tests.rs`)

`recompose_stamps_parked_shapes_with_no_active_editor` · `parked_shapes_round_trip_through_a_snapshot` ·
`empty_space_down_parks_the_active_shape_and_starts_a_new_one` · `clicking_a_parked_shape_reactivates_it` ·
`stroke_operation_mode_sets_the_new_shapes_op` · `centre_square_tap_cycles_the_op_but_a_drag_does_not`.
All 370 tool-painter tests green; the existing `curve_grab_tolerance_grabs_near_and_adds_far` was updated to
the new far-click-starts-a-shape semantics.

## DEFERRED — Phase 4: boolean-region render + combined offset (its own round)

Add/Remove currently **store** their op + show the glyph, but do NOT yet render the boolean COMBINATION of
overlapping shapes (Add = union outline, Remove = subtract), nor the "offset acts on the combined region"
behaviour. That is a mask/contour subsystem: rasterize the overlapping Add/Remove group → boolean-compose →
SDF-offset → trace contour → stamp dabs along it, vs. per-shape for Overlay/separated shapes. The Selection
subsystem already has the pieces (`rasterize_selection_shape`, `recompose_selection_mask`,
`trace_selection_contour`, the offset SDF) — bridge `ShapeEditState` → a fillable region and reuse them.
Deliberately its own **tested build + visual smoke** (as the whole feature was flagged to need). Overlay +
separated shapes already behave correctly (independent paint, per-shape offset baked at park).

Also open: op-cycle of the ACTIVE shape isn't captured in undo (the `active_op` field isn't snapshotted);
parked-shape offset is baked at park time (per-shape), so the live slider only offsets the active shape.
