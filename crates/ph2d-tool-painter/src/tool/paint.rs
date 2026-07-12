//! Canvas painting — drives the clean-room Blender brush engine (`ph2d-painter-brush`) from pointer
//! samples, writing dabs into the active raster layer's `canvas_rgba`.
//!
//! The brush *behaviour* (falloff, spacing, pressure, blend) is the engine's; this module is only the
//! glue between the editor's [`CanvasPaintTool`] contract (ADR-0040 Amendment 3) and the painter's
//! layer buffers + preview/dirty plumbing.

use super::*;

use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::{
    AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec, Dab, Dynamics, MAX_INPUT_SAMPLES, Stroke,
    StrokeMethod, StrokePoint,
};

/// The imported brush-texture image type (`BrushTextureImage`), split from `brush_settings` (LOC cap).
mod brush_image;
/// Brush + Stroke-section parameter snapshot & setters (shares `PaintState`'s private brush access).
mod brush_settings;
mod brush_texture_settings; // Grain-texture / Stencil / Dab setters; split from `brush_settings` (LOC cap)
/// The Curve stroke method's on-canvas point editor (submodule, as `brush_settings`).
mod curve;
mod curve_commit; // Apply / Apply & Keep commit verbs for the Curve editor; split from `curve`
mod curve_geom; // flatten (incl. closed seam) + point hit-test/nearest/insert; split from `curve`
mod curve_gizmo; // whole-curve transform gizmo (move/scale/rotate the entire curve); split from `curve`
mod curve_handle; // per-anchor handle kinds (Free/Aligned/Vector/Auto) + derived geometry; split from `curve`
mod curve_join; // corner joins for the offset: smooth-merge / convex-miter / concave-split; split from `curve_offset`
mod curve_model; // shared editing core (points/handles/kinds/selected + ops) — stroke + selection both own one
mod curve_offset; // perpendicular offset (parallel curve) + CAD-grade reconstruction; split from `curve_geom`
mod curve_refit; // Simplify/Merge quality funnel: corner-split + piecewise Schneider least-squares refit
mod curve_tangent; // Bézier tangent-handle hit-test, aligned mirror, overlay snapshot; split from `curve`
mod curve_trim; // self-intersection trim of the offset spine (open + closed); split from `curve_offset`
mod stroke_boolean; // multi-shape Add/Remove boolean composite (rasterise → union/subtract → trace contours)
mod stroke_multi; // multi-shape: parked (inactive-but-editable) stroke shapes + their Operation; pixels are a derived recompose
pub use stroke_multi::StrokeOpBadge;
/// Per-dab randomize setters (Jitter Scale / Rotate / Randomize Color); split from `brush_settings`.
mod impasto; // Impasto: the height channel (paint thickness) — the dab pipeline's SECOND output
mod impasto_light; // Impasto: the light pass — normal from the height field + Lambert/Blinn-Phong
mod impasto_settings; // Impasto: section setters + the panel-event route (mirror of watercolor_settings)
mod jitter_settings;
/// The canvas pointer's operation mode (Paint / Smear / Blur / Clone / Mask); split from `paint.rs` (cap).
mod paint_mode;
/// Multi-layer Shape (z-ordered layers + per-layer-colour state); split from `paint.rs` (LOC cap).
mod shape_layers;
/// Imported-image slots (Grain + Shape) + Shape geometry + Grain Depth setters; split from `brush_settings`.
mod shape_settings;
mod shape_snapshot; // unified shape+paint undo: each create/edit/bake = one ModelSnapshot on the timeline
mod stamp_color_cache; // the cached multi-layer coloured stamp (bake the composite once, blit per dab)
mod stamp_color_dynamic;
/// Stroke begin/extend/tick/end lifecycle; split from `paint.rs` (LOC cap).
mod stroke_lifecycle;
/// Watercolor stroke buffers: per-stroke coverage + deposited-colour accumulation (+ dirty tracking).
mod watercolor_accum;
/// Watercolor real GROUND (backdrop under the active layer + document paper colour) + water soak.
mod watercolor_backdrop;
/// Watercolor field math (noise, blur, samplers, per-stroke styles); split from `watercolor_render`.
mod watercolor_field;
/// Watercolor optical LUTs (`s2l`/`ln`/`exp`) + pigment-body helpers; split for the LOC cap (HR-5).
mod watercolor_lut;
/// Watercolor Wet Mix mixer-brush state (Charge/Dilution/Pull) — per-dab colour pickup + carry.
mod watercolor_mixer;
/// Watercolor canvas-anchored value noise + [`NoiseTile`] sprite-wrap (seamless tiling, doc 13 #2).
mod watercolor_noise;
/// Watercolor edge darkening (#1): per-stroke coverage + the pen-up blur-difference "fringe" pass.
mod watercolor_render;
/// Watercolor per-pixel rewet terms (lift/dissolve/pool/backrun); split from `watercolor_render`.
mod watercolor_rewet_px;
/// Watercolor section setters + router (edge darkening / granulation / pigment); no fluid sim.
mod watercolor_settings;
/// Watercolor Wet Mix reservoir (Smudge/Pickup): lift the pre-stroke paint, mix into the dab colour.
mod watercolor_smudge;
pub(crate) use paint_mode::{PAINT_MODE_COUNT, PaintMode};
mod lifecycle; // transient-edit reset run at each document (re)bind — abandons pending Fill/stroke/etc.
/// Drawing symmetry (mirror / radial) — engine glue, canvas-centre resolution + on-canvas pick modes.
mod symmetry;
mod tool_link; // "Sync with other tools": per-mode brush-settings swap + the link toggle; LOC-cap split
pub(crate) use symmetry::SymmetryPick;
/// Seamless Tiling (wrap-around painting) — dab replication across sprite edges + the toggles.
mod tiling;
mod wet_editable;
pub use curve::CurveOverlay;
pub use curve_gizmo::TransformGizmo;
pub use curve_tangent::TangentHandles;
/// The Ellipse stroke method's on-canvas ellipse editor (same submodule rationale as `curve`).
mod ellipse;
pub use ellipse::EllipseOverlay;
/// The Line stroke method's on-canvas polyline editor (plain corner points, no Bézier handles).
mod line;
pub use line::LineOverlay;
/// Per-corner Fillet / Chamfer geometry + gizmos for the Line editor (split from `line` for the LOC cap).
mod line_corner;
pub use line_corner::LineCornerGizmo;
/// Live dx/dy + corner-angle dimensions for the active Line segment (split from `line` for the LOC cap).
mod line_dim;
pub use line_dim::LineDimensions;
/// Line editor commit / cancel / finish paths (split from `line` for the LOC cap).
mod line_commit;
/// Perpendicular Offset (parallel-polyline) geometry for the Line editor (split from `line` for the cap).
mod line_offset;
/// Drag-time snapping for the Line editor (Shift 15° + auto point-to-point align); split from `line` (cap).
mod line_snap;
/// The Polygon stroke method's on-canvas regular-N-gon editor (same submodule rationale).
mod polygon;
pub use polygon::PolygonOverlay;
/// The Stencil texture mapping's on-canvas handle editor (move/resize the image-space rect).
mod stencil;
/// Stroke-method control (set / non-shape memory / restore) — the Brush-panel + rail Shapes seam.
mod stroke_ctl;
pub use stencil::{StencilOverlay, StencilPreview};
mod blur_route;
/// The `impl CanvasPaintTool` pointer entry (`on_canvas_pointer`); split from `paint.rs` (LOC cap).
mod canvas_pointer;
mod composite;
pub(crate) use composite::{CompositeLayer, CompositeOp};
mod clone;
mod eyedropper;
mod fill; // Fill (Bucket) — Procreate ColorDrop flood fill + live threshold adjust; split for LOC cap
mod inpaint; // content-aware heal brush (mark defect + reconstruct on pen-up); split for LOC cap
/// The **Mask** tool's extras — sub-brush (Paint/Erase/Blur/Smear), whole-canvas ops, overlay tint. [LOC split].
mod mask;
mod ramp;
mod ramp_lut; // ramp LUT baking (colour owner + colour/tone LUTs); split from `stamp_cache` (LOC cap)
/// Pixel-region save/restore helpers for the drag preview (`dab_bbox`/`save_region`/`restore_region`).
mod region;
/// The **Selection** tool (ADR-0103) — the document-wide selection mask, undo integration + paint gate. [LOC split].
mod selection;
/// Selection **actions** (Wave 5): Select layer contents / Color Fill / Copy-Paste / Save-Load slots. [LOC split].
mod selection_actions;
mod selection_curve_gizmo; // converted-curve point editor: Convert/Simplify → editable anchors + handles
/// Selection **Edit** mode (ADR-0103 Am.2): Convert-to-Curve + Simplify (list ops). [LOC split].
mod selection_edit;
/// Selection **isolated gizmos** (ADR-0103 Am.2 v2): per-shape gizmos decoupled from the stroke editors.
mod selection_gizmo;
/// **Deform** (Liquify) — the single inverse-warp kernel + per-mode displacement fields + Reconstruct/Amount.
mod warp;
pub use selection_gizmo::SelectionGizmoView;
pub use warp::DeformGizmoView;
/// Selection creation input: mode/op/threshold setters + on-canvas pointer gestures (marquee/lasso/flood). [LOC split].
mod selection_input;
/// Selection **Offset** (ADR-0103 Am.3): signed-distance grow/shrink + concentric alternating protected /
/// paint bands driven by Apply & Keep. [LOC split].
mod selection_offset;
mod selection_offset_geom; // sharp-corner offset: trace(+holes) → refit → CAD offset per level (no SDF rounding)
/// Selection on-canvas overlay (marching ants + hatching) + panel event routing. [LOC split].
mod selection_overlay;
/// Selection rasterization: shape → coverage buffers, boolean combine, Feather (box-blur). [LOC split].
mod selection_raster;
/// Selection **shape list** model (ADR-0103 Am.2): the `Vec<SelectionShape>` source of truth + compositing. [LOC split].
pub(super) mod selection_shapes; // SelectionEntry is re-exported at `crate::tool` for the undo snapshot
/// Selection **Edit** mode contour tracing (mask → editable boundary polyline); split for the LOC cap.
mod selection_trace;
mod shape_ramp;
mod snapshot;
/// The Blender-style cached brush stamp (render falloff×texture once, scale-blit per dab).
mod stamp_cache;
mod stamp_preview; // interactive drag-preview stamping (restore+re-stamp, dirty-rect); split for the LOC cap
/// The stamp route dispatcher (Shape + Grain → which of the 4 stamp paths); split for the LOC cap.
mod stamp_route;
/// `PaintState::default` body — split out for the workspace file-LOC cap (struct stays in `paint.rs`).
mod state_default;

mod brush_ranges; // Stroke-slider UI range consts (BRUSH_*_MAX / airbrush rate) + shape grab tol; split for the LOC cap
pub use brush_ranges::*;

// The panel-facing snapshot [`BrushSettings`] + the falloff preview helper `brush_falloff_weight_at`
// live in the `brush_settings` submodule (their single clamp source); re-exported for the `paint::` path.
pub use brush_settings::{
    BrushSettings, DEFORM_TEMPERAMENT_NONE, DEFORM_TEMPERAMENT_RESHAPE,
    DEFORM_TEMPERAMENT_TRANSFORM, PANEL_RAMP_STOPS,
};
pub use shape_layers::MAX_SHAPE_LAYERS;
pub use snapshot::brush_falloff_weight_at;

/// A Drag Dot's restore record: pristine pixels under the dab footprint (RGBA8 over `rect`), saved before stamping so the next move erases it (no trail).
struct DragPreview {
    rect: Region,
    pixels: Vec<u8>,
}

pub(crate) struct PaintState {
    /// The active brush.
    brush: BrushSpec,
    /// Pen pressure → size/coverage mapping.
    dynamics: Dynamics,
    /// The stroke in progress between pointer-down and pointer-up (`None` when idle).
    stroke: Option<Stroke>,
    /// Reused dab buffer so a hot pointer stream allocates nothing per sample.
    dabs: Vec<Dab>,
    /// Per-stroke jitter seed; bumped each stroke so jitter is reproducible yet varies.
    seed: u64,
    /// Splitmix64 for the texture's per-dab Random rotation/offset — reset per stroke (seed-decorrelated), advanced once per textured dab (HR-5).
    tex_rng: u64,
    /// Model snapshot at pointer-down (before the first dab) — committed to undo at pointer-up so the whole stroke undoes as one unit.
    stroke_undo: Option<crate::undo::ModelSnapshot>,
    /// Eraser mode: overrides the brush blend with Erase Alpha at stamp time (`brush.blend` preserved for when it's off).
    eraser: bool,
    /// Which operation the pointer performs (Brush=Paint / Smear); driven by the left-rail tool selection.
    paint_mode: PaintMode,
    /// Per-mode saved brush settings — the "independent tools" model (the default). Each [`PaintMode`]
    /// keeps its OWN [`BrushSpec`], swapped into `brush` on a mode change so editing one tool's panel
    /// never bleeds into another. Indexed by [`PaintMode::slot`]. Ignored while `link_shared_settings` is
    /// on (all modes then share the live `brush`). The active mode's slot is stale while active (edits go
    /// to `brush`); it's written back on the next mode switch. See [`tool_link`].
    brush_by_mode: [BrushSpec; PAINT_MODE_COUNT],
    /// "Sync with other tools": when `true`, every paint tool SHARES the live `brush` (a mode change no
    /// longer swaps slots), so a change in one panel shows in all. Default `false` = each tool independent.
    link_shared_settings: bool,
    /// **Line** stroke method: show the live dx/dy distances + corner angles while drawing (the CAD
    /// dimension overlay). Default `true`. A per-Line display pref, toggled by the "Dimensions" checkbox.
    line_show_dimensions: bool,
    /// **Mask** sub-brush (Mask mode): `0` Paint (conceal/black) · `1` Erase (reveal/white) · `2` Blur · `3` Smear. [`mask`].
    mask_brush: u8,
    /// **Mask** overlay tint index (`0` gray + 4 fluorescent) — tints the composite where a mask conceals. [`mask`].
    mask_overlay_color: u8,
    /// **Eyedropper** armed: the next canvas Down samples the composited pixel into the brush colour, then disarms. [`eyedropper`].
    eyedropper_armed: bool,
    /// **Mask** brush transient scratch (tool-side mask for `mask_scratch_target`, white=reveal; NOT a stack layer). [`mask`].
    mask_scratch_rgba: Arc<Vec<u8>>,
    mask_scratch_target: Option<crate::layers::LayerId>,
    /// **Selection** mask (ADR-0103): a document-wide single-channel coverage buffer (`w*h` bytes,
    /// `0` = outside / `255` = inside; Feather softens the edge). Gates every paint op to the selected
    /// region ([`selection`]) and is undo-integrated via the `ModelSnapshot` exactly like `mask_scratch`.
    selection_mask: Arc<Vec<u8>>,
    /// `true` while a selection is live (has coverage). `false` = no selection, painting unrestricted.
    selection_active: bool,
    /// **Selection** sub-mode: `0` Automatic · `1` Freehand · `2` Rectangle · `3` Ellipse. Driven by the
    /// Selection panel (Wave 3); consumed by the on-canvas router ([`selection`]).
    selection_mode: u8,
    /// **Selection** boolean operator for the next gesture: `0` New (replace) · `1` Add · `2` Remove.
    selection_bool_op: u8,
    /// **Selection** Automatic threshold (`0..1`) — colour tolerance for the flood-select mode.
    selection_threshold: f32,
    /// The in-progress selection gesture (marquee rubber-band / lasso path / Automatic seed); `None` when
    /// idle. The overlay (Wave 4) draws from this; the mask is rasterized on pen-up. [`selection`].
    selection_drag: Option<selection_input::SelectionDrag>,
    /// The selection mask at the START of the current gesture — Add/Remove combine against this base. [`selection`].
    selection_base: Arc<Vec<u8>>,
    /// The CRISP selection mask (pre-Feather), the accumulator the Feather slider re-derives from — so
    /// dragging Feather never compounds a blur-of-a-blur (mirrors the Shape Offset accumulator). [`selection`].
    selection_crisp: Arc<Vec<u8>>,
    /// **Free-selection stabilizer** (`0..1`) — the lazy-mouse smoothing applied to the Freehand lasso path
    /// (its own knob, independent of the brush stabilizer). [`selection_input`].
    selection_stabilizer: f32,
    /// **Feather** amount (`0..1` → edge-softening radius); the effective `selection_mask` is a blur of
    /// `selection_crisp` at this radius. [`selection`].
    selection_feather: f32,
    /// **Show Selection Gizmos** mode: when `true`, EVERY editable selection shape shows its own isolated
    /// gizmo at once (ellipse / polygon / freehand), each manipulable — WITHOUT touching the stroke shape
    /// editors (ADR-0103 Am.2 v2). A transient UI mode. [`selection_gizmo`].
    selection_edit_mode: bool,
    /// **Selection overlay opacity** (`0..1`) — how strongly the deselected-area hatching reads. A view
    /// preference (not undoable); scales the hatch alpha in [`selection`]. Default `0.2` (Enio 2026-07-02).
    selection_overlay_opacity: f32,
    /// **Selection shape list** (ADR-0103 Am.2) — the parametric source of truth (Ellipse / Polygon /
    /// Freehand / Raster + a boolean op each). The `selection_mask` is a DERIVED cache: rasterize + composite
    /// this list. A gizmo drag mutates one entry's params in place and recomposites. [`selection_shapes`].
    selection_shapes: Vec<selection_shapes::SelectionEntry>,
    /// **Per-shape rasterization cache** (perf): `(shape, coverage)` parallel to `selection_shapes` at the
    /// last recompose. On the next recompose a shape whose geometry is UNCHANGED reuses its cached coverage
    /// (an `Arc` clone) instead of re-rasterizing — so a gizmo drag over N boolean shapes only re-rasterizes
    /// the ONE that moved (O(A) vs O(N·A) per frame). Self-validating by value, so no manual invalidation.
    selection_raster_cache: Vec<(selection_shapes::SelectionShape, Arc<Vec<u8>>)>,
    /// The isolated gizmo grab currently dragged (shape idx + handle + pristine geometry for drift-free
    /// whole-shape transforms); `None` when idle. [`selection_gizmo`].
    selection_grab: Option<selection_gizmo::SelectionGrab>,
    /// In-memory **Copy** buffer of selected pixels (source bbox + coverage-premultiplied RGBA), consumed by
    /// **Paste**. `None` until a Copy. [`selection_actions`].
    selection_clipboard: Option<selection_actions::SelectionClip>,
    /// **Selection Offset** state (ADR-0103 Am.3) — grow/shrink + concentric protected/paint rings. See
    /// [`selection_offset`]: `_norm` = slider (`0.5` = no offset); `_active` = ring mode (post-Apply & Keep);
    /// `_rings` = frozen cumulative band offsets (px, PAINT iff even index); `_source` = the pre-offset crisp
    /// the offset reads from; `_sdf` = its lazily-cached signed distance field (`<0` inside).
    selection_offset_norm: f32,
    selection_offset_active: bool,
    selection_offset_rings: Vec<f32>,
    selection_offset_source: Arc<Vec<u8>>,
    /// Corner-true contours of the offset source (outer + holes, refit + grow-calibrated) — the sharp
    /// offset's derived cache; rebuilt lazily off `selection_offset_source`. [`selection_offset_geom`].
    selection_offset_curves: Vec<selection_offset_geom::OffsetContour>,
    /// Per-level effective masks (ring boundaries + the live level) — derived cache keyed by exact level.
    selection_offset_level_cache: Vec<(f32, Arc<Vec<u8>>)>,
    /// **Ring stack**: `true` once the offset rings were materialised into the editable Freehand curves in
    /// `selection_shapes` (Edit Gizmos on an offset selection, Enio 2026-07-04). While set, the mask is a
    /// BAND-PARITY composite of those nested curves (paint iff enclosed by `≡ n (mod 2)` of them, `n>0`),
    /// so editing any ring curve reshapes its intercalated band. Cleared by Clear / a new selection gesture.
    selection_ring_stack: bool,
    /// **Composite Brush**: run Brush + Smear + Blur together (a Brush-tool upgrade, panel checkbox). See [`composite`].
    composite_enabled: bool,
    /// The composite layer stack in display order (index 0 = layer 1 = top; run bottom→top per dab). [`composite`].
    composite: [CompositeLayer; 3],
    /// **Clone** sampled source anchor (image px), set by the "Set Source" pick mode; `None` until sampled. [`clone`].
    clone_source: Option<[f32; 2]>,
    /// **Clone** established source→dest offset (px) = `clone_source − stroke_start`; `None` until a stroke begins. [`clone`].
    clone_offset: Option<[f32; 2]>,
    /// **Clone** Aligned mode: keep the offset fixed across strokes (on); re-anchor to the source each stroke (off).
    clone_aligned: bool,
    /// **Clone** "Set Source" pick mode armed — the next canvas Down sets [`Self::clone_source`] instead of painting.
    clone_sample_armed: bool,
    /// Previous dab centre during a **Smear** stroke (the source each dab lifts from); `None` at stroke start. [`stamp_route`].
    last_smear_pos: Option<[f32; 2]>,
    /// **Tiling** `[x, y]`: seamless wrap-around painting — a dab near an edge also stamps the wrapped part on the opposite edge. Off by default.
    tiling: [bool; 2],
    /// **Repeat Image**: the shell draws the sprite in the 8 neighbour directions (3×3); with Tiling on, those tiles are paintable (the shell wraps the pointer back).
    repeat_image: bool,
    /// **Symmetry** on-canvas pick mode armed (draw mirror line / pick radial centre); `None` = paint normally. [`symmetry`].
    symmetry_pick: Option<symmetry::SymmetryPick>,
    /// First endpoint captured while drawing the custom symmetry line; `None` when not mid-draw.
    symmetry_line_start: Option<[f32; 2]>,
    /// Whether the symmetry centre auto-tracks the canvas centre (X/Y mirror + radial default); a user-drawn line / picked centre clears it. See [`PainterTool::resolve_symmetry_geometry`].
    symmetry_auto_center: bool,
    /// Cleared per frame; set by `paint_extend` on a move. A parked frame lets `paint_tick` settle the stabilizer.
    moved_this_frame: bool,
    /// Restore record for the in-progress Drag Dot's single moving dab; `None` for every other method.
    drag_preview: Option<DragPreview>,
    /// The press point of the in-progress stroke — the pivot the Line's Alt-constrain snaps around (45°).
    line_anchor: Option<[f32; 2]>,
    /// Alt held this event — constrains the Line to 45° increments (Blender `constrain_line`); set by the shell each pointer event.
    line_constrain: bool,
    /// Shift held this event — a Stencil corner scale becomes UNIFORM (aspect-locked, like the Sprite gizmo); set by the shell each pointer event.
    scale_uniform: bool,
    /// Shift held this event — the polyline **Line** editor snaps each new segment to 15° increments from the previous point; set by the shell each pointer event. See [`line`].
    line_snap: bool,
    /// The editor GRID-snapped image position for the current pointer, forwarded by the shell each event
    /// (world→grid via `GridSnapState::snap_world`, mapped back to image px) — `None` when grid snap is off.
    /// Drawing-tool point placement/drag uses it as the base position; gizmo / corner-handle drags ignore
    /// it (so grid snap can't corrupt a parameter drag). See [`line_snap`].
    grid_snap_pos: Option<[f32; 2]>,
    /// Last NON-shape method — the rail's Brush button restores it. See [`PainterTool::restore_non_shape_stroke_method`].
    last_non_shape_method: StrokeMethod,
    /// In-progress Curve session (the on-canvas point editor); `None` when idle. [`curve`].
    curve: Option<curve::CurveEditor>,
    /// In-progress Ellipse session (the on-canvas ellipse editor); `None` when idle. [`circle`].
    ellipse: Option<ellipse::EllipseEditor>,
    /// In-progress Line session (the on-canvas polyline editor); `None` when idle. [`line`].
    line: Option<line::LineEditor>,
    /// In-progress Polygon session (the on-canvas regular-N-gon editor); `None` when idle. [`polygon`].
    polygon: Option<polygon::PolygonEditor>,
    /// **Multi-shape** PARKED stroke shapes (source of truth; pixels are a derived recompose) — every editable shape but the live editor. Empty = single-shape. [`stroke_multi`] (Enio 2026-07-04).
    parked_shapes: Vec<stroke_multi::StrokeShape>,
    /// Operation of the ACTIVE shape (its `+`/`−`/`o` gizmo glyph); a new shape adopts [`stroke_op_mode`]. [`stroke_multi`].
    active_op: stroke_multi::StrokeOp,
    /// Current panel Operation mode a NEW shape is created with (stroke analogue of `selection_bool_op`; "New"→Overlay).
    stroke_op_mode: stroke_multi::StrokeOp,
    /// Pending op-cycle **tap** (Down pos on the active shape's centre square): Up without a drag cycles the op; a drag past the slop clears it + moves the shape. [`stroke_multi`].
    op_tap: Option<[f32; 2]>,
    /// Seamless-Tiling **edit-in-tile** offset (Enio 2026-07-11): a shape's overlay is drawn in the wrapped
    /// neighbour tiles, so a grab there must edit the ORIGINAL. Fixed at the grab Down = the tile offset (px)
    /// that lands the pointer on the active shape's bbox (`route_shape_pointer_multi`); subtracted from every
    /// pointer of the gesture so the drag is CONTINUOUS (no seam jump — unlike a per-sample wrap) and works
    /// for geometry drawn beyond the sprite. `[0, 0]` = no wrap (off-tiling / drawing / empty-space click).
    shape_edit_wrap: [f32; 2],
    /// Pending SELECTION op-cycle tap — Down on a shape's centre-move square arms `Some((shape, pos))`; Up without a drag past the slop cycles THAT shape's Add↔Remove op; a drag clears it + moves the shape. Mirrors [`op_tap`] but selection toggles only Add/Remove. [`selection_gizmo`].
    selection_op_tap: Option<(usize, [f32; 2])>,
    /// Control-handle grab radius (image px) for the shape editors — shell forwards a footprint-scaled value.
    shape_grab_tol_px: f32,
    /// **Offset** slider track (`0..1`, `0.5` = none) — perpendicular path offset for the shape editors.
    shape_offset_norm: f32,
    /// **Accumulated** offset (px) from prior Apply & Keep; EFFECTIVE = base + slider (a single offset of the pristine base).
    shape_offset_base_px: f32,
    /// **Trim** (Offset card): cut the offset spine's self-intersections — drawing-only (see [`curve_offset`]).
    offset_trim: bool,
    /// In-progress Stencil overlay drag (move/resize/rotate the texture rect); `None` when idle.
    stencil_grab: Option<stencil::StencilGrab>,
    /// Seconds left on the transient in-gizmo Stencil texture preview (decayed each `paint_tick`).
    stencil_preview_s: f32,
    /// Imported brush-**Grain** luminance (heavy → not in the `Copy` spec); borrowed as an `ImageMask`.
    texture_image: Option<brush_settings::BrushTextureImage>,
    /// Watercolor **Paper** slot luminance (a tagged layer used as the substrate; `paper.kind == Image`).
    /// Heavy, so out of the `Copy` spec; borrowed as an `ImageMask` by the render-path ([`watercolor_render`]).
    /// (The **Granulation** map is the Grain slot, so it reuses [`Self::texture_image`].)
    paper_image: Option<brush_settings::BrushTextureImage>,
    /// Bumped whenever [`Self::paper_image`] changes, so the shell re-publishes it for the Paper preview.
    paper_image_version: u64,
    /// Set when the user picks the Image kind; the shell polls it to open a file picker.
    texture_image_pending: bool,
    /// Bumped whenever [`texture_image`] changes, so the stamp cache re-renders the Image mask.
    texture_image_version: u64,
    /// Imported brush-**Shape** luminance (silhouette tip; borrowed as `ImageMask`). `None` ⇒ silhouette = falloff.
    shape_image: Option<brush_settings::BrushTextureImage>,
    /// Set when the user picks the Image source in the Shape dropdown; the shell polls it to open a picker.
    shape_image_pending: bool,
    /// Bumped whenever [`shape_image`] changes, so the stamp cache re-renders the Shape mask.
    shape_image_version: u64,
    /// Multi-layer Shape (z-ordered luminance layers) + per-layer-colour mode/colours; OFF ⇒ flattened into [`shape_image`]; see [`crate::tool::paint::shape_layers`].
    shape_layers: shape_layers::ShapeLayers,
    /// Cached brush stamp (falloff × View texture) + its key; re-rendered on appearance/mask-size change, scale-blitted per dab. See [`crate::tool::paint::stamp_cache`].
    stamp_cache: Option<(ph2d_painter_brush::StampMask, stamp_cache::StampKey)>,
    /// Cached per-layer coloured stamps (bottom→top) + key, blitted in cross-stroke z-order; `stamp_color_cache`.
    color_stamp_cache: Option<(
        Vec<ph2d_painter_brush::ColorStampMask>,
        stamp_color_cache::ColorStampKey,
    )>,
    /// Cached Grain+Ramp coloured stamp + key (the cacheable grain-ramp colour path); `stamp_color_cache`.
    ramp_color_stamp_cache: Option<(
        ph2d_painter_brush::ColorStampMask,
        stamp_color_cache::RampColorStampKey,
    )>,
    /// Lazily-filled canvas-space texture cache for Tiled / Stencil mappings (computed once per canvas pixel per stroke). See [`crate::tool::paint::stamp_cache`].
    canvas_tex_cache: Option<stamp_cache::CanvasTexCache>,
    /// The brush Grain + Shape **Color Ramps** + Shape **tone** LUT (engine model + baking: [`ramp_lut`]).
    texture_ramp: ph2d_color::ColorRamp,
    texture_ramp_enabled: bool,
    texture_ramp_bw: bool,
    texture_ramp_lut: Vec<[f32; 4]>,
    texture_ramp_dirty: bool,
    /// Bumped when `ensure_ramp_lut` re-bakes the owner LUT — the colour-ramp **stamp** cache keys on it.
    ramp_lut_version: u64,
    texture_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode,
    shape_color_ramp: ph2d_color::ColorRamp,
    shape_color_ramp_enabled: bool,
    shape_color_ramp_bw: bool,
    shape_color_ramp_alpha_mode: ph2d_painter_brush::RampAlphaMode,
    shape_ramp_lut: Vec<f32>,
    shape_ramp_dirty: bool,
    shape_ramp_version: u64,
    ramp_lut_owner: ramp_lut::RampLutOwner,
    /// **Accumulate OFF** per-stroke coverage mask (1 byte/px), cleared on down; caps a stroke at Strength.
    stroke_mask: Vec<u8>,
    /// **Impasto** per-stroke height envelope (f32, `w*h`) — the relief THIS stroke has laid down so
    /// far, combined by magnitude ([`ph2d_painter_brush::height::envelope`]) so passing the brush back
    /// over its own line leaves one thickness, not a staircase. Merged into the active layer's
    /// committed height at `close_stroke` (separate strokes DO add). Empty ⇒ no impasto this stroke
    /// (zero cost); sized lazily by the first dab, cleared on down and on each shape-editor re-stamp.
    /// See [`super::impasto`].
    stroke_height: Vec<f32>,
    /// **Impasto** per-stroke PAINT-coverage envelope (1 byte/px) — the sibling of [`Self::stroke_height`].
    /// Merged into the layer's `covers` at stroke end. See `PainterTool::covers`.
    stroke_cover: Vec<u8>,
    /// **Impasto**: where each Symmetry copy of the stroke was when the LAST pointer batch ended, so the
    /// first dab of the next batch can sweep back to it. Without it the relief would bead at every
    /// pointer event — a beading chosen by the artist's mouse polling rate, not by their hand. One slot
    /// per copy; cleared on pen-down. Mirrors `last_smear_pos` (which is the same idea, for the smear
    /// chain), but per-copy, because Symmetry paints several strokes at once.
    last_height_center: Vec<Option<[f32; 2]>>,
    /// **Impasto live-edit** — the LAST stroke's relief, kept UNSETTLED and at the depth it was laid
    /// with, so **Depth** and **Smoothing** can be re-derived after the fact instead of only affecting
    /// the next stroke. (Enio 2026-07-12: "devem atualizar em tempo real após o traço ser feito como as
    /// outras propriedades fazem." The same live-editability the watercolor wash has while the paper is
    /// still wet.) Empty ⇒ nothing to re-derive.
    live_relief: Vec<f32>,
    /// The active layer's committed relief BEFORE that stroke — the ground the re-derived stroke is
    /// added back onto. Empty ⇒ the layer had none (the common case: a first stroke).
    live_relief_base: Vec<f32>,
    /// Which layer [`Self::live_relief`] belongs to, and the Depth it was deposited with (the divisor
    /// that turns it back into a unit envelope). `None` ⇒ nothing live.
    live_relief_layer: Option<crate::tool::RtLayerId>,
    live_relief_depth: f32,
    /// **Watercolor render-path** per-stroke coverage (1 byte/px, `w*h`): the union footprint of the
    /// stroke's dabs (max-blended discs = wet_edges `stampCoverage`), the silhouette the optical composite
    /// reconstructs the wash from ([`super::watercolor_render`]). Empty unless the Watercolor section is
    /// active; sized lazily by the first dab, cleared on down.
    stroke_coverage: Vec<u8>,
    /// **Watercolor render-path** per-stroke deposited colour (RGBA, `w*h*4` = wet_edges `colC`): each
    /// dab's colour splatted source-over (recent dab wins), so the composite pigment can vary along the
    /// stroke (RYB pickup when Pigment is on). Empty / cleared with [`Self::stroke_coverage`].
    stroke_color: Vec<u8>,
    /// **Watercolor render-path** frozen base — the pre-stroke `canvas_rgba` (shared `Arc`, so holding it
    /// is O(1); the first composite `make_mut` forks the live buffer, leaving this pristine). The optical
    /// composite reads the "paper + prior paint" from here every frame instead of over-painting in place,
    /// so the wash never accumulates per-dab structure. `Some` only for the duration of a watercolor stroke.
    watercolor_base: Option<Arc<Vec<u8>>>,
    /// **Watercolor render-path** frozen GROUND — the real backdrop under the active layer: the
    /// composite of the layers BELOW it, over the document [`Self::paper_color`] where nothing is
    /// painted (RGBA8, opaque by construction). The optics read the Beer–Lambert base, the rewet
    /// presence reference and the lift target from HERE, never from a global paper constant — a
    /// virtual cream baked into the wash was the "puxa pro bege" bug (Enio 2026-07-06). Frozen with
    /// [`Self::watercolor_base`]; `None` outside a watercolor stroke.
    wet_backdrop: Option<Arc<Vec<u8>>>,
    /// Document **paper colour** (straight sRGB `0..1`) — the ground the watercolor optics see where
    /// the backdrop is fully transparent. Default WHITE (a plain canvas); panel swatch
    /// `PAINTER_WATERCOLOR_PAPER_COLOR_THUMB` edits it via the shared picker (Rebelle: canvas colour
    /// is a user-pickable document property). Tool-global (not persisted in the document yet).
    paper_color: [f32; 3],
    /// **Impasto — Show** (canvas-level, like the paper colour / drying time): whether the relief is
    /// LIT. Off ⇒ the light pass does not run and the composite is byte-identical to a build with no
    /// Impasto at all. Default on — someone who sculpts wants to see it, and with no relief anywhere
    /// the pass costs a single `is_empty()`.
    impasto_show: bool,
    /// **Light Angle** in whole degrees — the azimuth the light comes from. Default 135° (upper-left:
    /// the convention every paint program and every human eye reads as "raised", not "engraved").
    impasto_light_angle_deg: u16,
    /// **Light Elevation** in whole degrees above the canvas plane. Low = long dramatic shadows across
    /// the tooth; high = flat, even light. Clamped away from 0 (a grazing light divides by ~0).
    impasto_light_elev_deg: u16,
    /// **Amount** (`0..1`) — how strongly the relief bends the surface normal (height-to-slope). `0` =
    /// the relief is there but invisible; `1` = thick, sculptural paint.
    impasto_light_amount: f32,
    /// **Shine** (`0..1`) — strength of the specular highlight riding the crests. `0` = matte paint
    /// (watercolour/gouache), high = wet oil.
    impasto_shine: f32,
    /// **Watercolor render-path** per-stroke water DWELL (1 byte/px, `w*h`): how long the held brush
    /// soaked each pixel (grown by [`PainterTool::grow_wet_soak`] on the tick heartbeat). The rewet
    /// reads it as a `0..1` field: more soak = the dissolve reaches FARTHER (blur-scale lerp) and the
    /// lift digs DEEPER — "quanto mais a água fica, mais dissolve", without physics. Sized lazily
    /// with the coverage; persists through the WET SESSION (cleared on a fresh one).
    wet_soak: Vec<u8>,
    /// Current soak disc = the last dab's `(centre, radius)` — where the tick heartbeat pours dwell
    /// while the pointer is parked. `None` = stroke start.
    wet_soak_pos: Option<([f32; 2], f32)>,
    /// Whether THIS stroke poured any soak yet — gates the composite's 2×-blur (far) fields, so a
    /// stroke with no dwell pays exactly the plain 4-blur rewet cost.
    wet_soak_active: bool,
    /// Manual Shape stamp (Automatic OFF): the per-stroke **tip-density** buffer (`w*h`, `0..255`;
    /// sized lazily with the coverage). A TEXTURED tip must not HOLE the wash — in real watercolor
    /// the water fills the tip's outer silhouette while the texture modulates the PIGMENT deposited —
    /// so the coverage splat stores the saturated wetness ENVELOPE and this buffer carries the tip's
    /// texture (max-blend, "one pass"), which the composite multiplies into the interior fill term
    /// (`cw·fill·dens`): typical watercolor body + rim at the OUTER boundary, tip texture as pigment
    /// variation within. Empty / untouched ⇒ density 1 (byte-identical). Doc 13 #1 round 3.
    stroke_density: Vec<u8>,
    /// Wet Mix (MIX-1): the per-stroke **pigment-reserve** map (`w*h`, `0..255`; sized lazily, only
    /// while the mixer is on). Charge depletion must fade the PIGMENT, never the WATER: scaling the
    /// coverage instead leaves the blurred `inner` short of 1.0 across the whole interior, so the
    /// edge term (`cw·(1−inner)·gain`) floods the centre and the wash reads as a flat opaque slab
    /// (Enio smoke 2026-07-08 — "matou a borda em qualquer valor < 0.93"). This buffer carries each
    /// dab's fresh+carry reserve (max-blend: re-inking a faded trail restores it) and the composite
    /// multiplies it into the whole BRUSH density term (fill + edge) AFTER the rim is derived from
    /// the intact coverage — head keeps the full watercolor anatomy, the tail fades rim and body
    /// together toward plain water. Empty ⇒ factor 1 (byte-identical default).
    stroke_deplete: Vec<u8>,
    /// EDGE-1 (doc 12): canvas-wide MOISTURE map (`w*h`) surviving pen-up — dries on the
    /// heartbeat (~8.5 s, DiVerdi/Adobe; Curtis wet-area mask); the bake pours the HARDENED
    /// coverage (max-blend). While wet, watercolor strokes CONTINUE one **wet session**
    /// ([`PainterTool::wet_session_continues`]): the buffers accumulate the UNION, re-rendered
    /// over the session base — one wash, one rim. Empty = dry (tick drops it + the session).
    canvas_wet: Vec<u8>,
    /// Live bounding rect of the wet area (the decay/pour window) — `None` = dry, zero idle cost.
    canvas_wet_rect: Option<(usize, usize, usize, usize)>,
    /// Fractional drying carry between whole-byte decay steps (heartbeat dt accumulator).
    canvas_wet_carry: f32,
    /// EDGE-1 (doc 13 #11): paper drying RATE in wetness-bytes/second — CANVAS-level (not per-brush,
    /// so it never varies by paint mode). The Wetness card's Drying-Time slider drives it
    /// (`set_dry_time_s`, seconds → `255/seconds`); default `CANVAS_WET_DRY_DEFAULT` (~10 s).
    dry_rate_per_s: f32,
    /// #12a (doc 14): the on-canvas wetness PREVIEW strength — the max veil alpha the shell paints over
    /// the wet region (`0` = no preview). CANVAS-level display setting (not per-brush); the Wetness card's
    /// slider drives it (`set_wet_preview_intensity`), the shell reads [`Self::wet_preview_intensity`].
    wet_preview_intensity: f32,
    /// #3 (doc 14, Enio 2026-07-11): the **Wet the layer** forced rewet. `wet_canvas_now` (the Wet button)
    /// re-opens a wet session over the current canvas and sets this so strokes made now LIFT/blend the
    /// EXISTING paint even with the brush's own Rewet at `0` (Rebelle "Wet the layer"). `0` = no forcing
    /// (byte-identical). Cleared by [`Self::dry_session_now`] (the session's teardown / drying deadline).
    wet_session_wetness: f32,
    /// #3 (doc 13): a SHAPE-editor watercolor preview is live (Curve/Line/Circle/Polygon/Free Hand with
    /// Watercolor on). Shapes have no `paint_begin`, so the wash ground (backdrop / substrate — expensive,
    /// static within the session) is frozen ONCE lazily on the first `stamp_drag_preview_watercolor` and
    /// this flag guards the rebuild; torn down (false) at the shape commit / cancel. `false` = no shape
    /// wash session ⇒ the freehand path is untouched.
    wet_shape_active: bool,
    /// EDGE-1 per-stroke style (doc 13 topo): session param table + per-pixel owner map — an
    /// older wash keeps ITS look on the union re-bake ([`watercolor_field::WetSessionStyles`]).
    wet_styles: watercolor_field::WetSessionStyles,
    /// EDGE-2 backrun: the CARRIED-water pool (`w*h`, session-scoped) — Dilution pours it per dab
    /// regardless of pigment; the composite lifts/blooms against it (serrated ring = backrun
    /// edge). Separate from the session dwell soak (`wet_soak`). Empty = inert.
    stroke_water: Vec<u8>,
    /// EDGE-1 wet session: the optical base frozen at the SESSION start (first stroke of the wet
    /// window) — every bake of the session re-composites the UNION buffers over this, never over
    /// its own previous bake (which would double-count). Per-stroke `watercolor_base` (refrozen
    /// each pen-down, so it INCLUDES the union baked so far) keeps serving the mixer pickup and
    /// the rewet fields.
    wet_session_base: Option<Arc<Vec<u8>>>,
    /// EDGE-1 wet session guard: the exact `canvas_rgba` Arc OUR last session bake produced. Any
    /// foreign mutation (undo, layer switch, fill, resize, other tools) swaps the canvas Arc, so a
    /// failed `Arc::ptr_eq` at pen-down ends the session — no per-site invalidation hooks needed.
    wet_session_canvas: Option<Arc<Vec<u8>>>,
    /// **Live-editable wash** (Enio 2026-07-11): the LAST committed wash stays re-renderable while the
    /// paper is still wet (until the next stroke or Dry), so changing a Grain/Paper texture param
    /// (Size/Angle/Offset/kind/…) re-renders the whole wash — central AND every Tiling copy — instead of
    /// only affecting the NEXT stroke. This is the pre-wash BASE + the frozen GROUND our last bake
    /// composited over; `apply_watercolor` reconstructs the wash from them over [`Self::wet_editable_region`]
    /// with the CURRENT brush texture. `None` ⇒ no editable wash (byte-identical: nothing re-renders).
    wet_editable_base: Option<Arc<Vec<u8>>>,
    /// The frozen GROUND ([`Self::wet_backdrop`]) of the editable wash — kept past `close_stroke` (which
    /// drops the live one) so the re-render's Beer–Lambert base matches the committed look.
    wet_editable_backdrop: Option<Arc<Vec<u8>>>,
    /// The committed wash's footprint (already full-axis on a tiled axis, from `dab_batch_region`), so the
    /// live re-render touches exactly the wash + its Tiling copies, not the whole canvas.
    wet_editable_region: Option<Region>,
    /// The **substrate signature** the editable wash was last rendered with — the paint tick re-renders
    /// when the live brush differs (a param moved), then refreshes this. `None` ⇒ inert.
    ///
    /// It used to be just `(Grain, Paper)` `TextureSettings`, which left the rest of the substrate OUT of
    /// the detector (sweep 2026-07-12): **Paper Depth** and **Granulation** are read by `apply_watercolor`
    /// but live on `BrushSpec`, not inside `TextureSettings`, and swapping the Paper/Grain IMAGE while
    /// keeping `kind: Image` changes no setting at all — only the pixel version. So dragging Paper *Size*
    /// re-rendered the wet pool and dragging Paper *Depth*, right next to it, did nothing: the same gesture,
    /// two different behaviours, side by side.
    wet_editable_tex: Option<wet_editable::WetEditableSig>,
    /// Manual Shape stamp (Automatic OFF): the tip image's luminance NORMALISER (`1 / max_lum`,
    /// `1.0` when no image / all-black). The watercolor coverage is WETNESS GEOMETRY (a max-blend
    /// union that must SATURATE in the wash core — `cw → 1` gives the body, `inner → 1` confines the
    /// edge term to the rim), not the plain brush's tonal per-dab alpha (which accumulates by
    /// source-over). A raw grey tip therefore starved the optics: pale centre + no rim (Enio
    /// 2026-07-07). Scaling samples by this keeps the tip's RELATIVE texture but guarantees its core
    /// reaches full coverage. Computed once per stroke at pen-down (`freeze_watercolor_ground`).
    wet_shape_norm: f32,
    /// **Watercolor substrate cache** (perf, byte-identical): the paper-tooth height `paper_h` at each
    /// canvas pixel (`f32`, `w*h`; `NaN` = not yet computed). The paper is CANVAS-ANCHORED — the same
    /// canvas pixel yields the same `paper_h` for the whole stroke — but the optical composite recomputes
    /// it (~28 integer-hashes for a procedural paper) every frame, so a big brush over many frames re-did
    /// the same work. This memoises it: filled on first touch, reused by every later frame AND the pen-up
    /// bake. **Reset to all-`NaN` at pen-down** ([`PainterTool::freeze_watercolor_ground`]) so a stroke
    /// never reads a previous stroke's settings — the paper cannot change mid-stroke, so there is no
    /// in-stroke invalidation to get wrong. Empty outside a watercolor stroke. Pure memoisation of a
    /// deterministic function keyed by the exact canvas index ⇒ the composite is byte-identical.
    wet_substrate: Vec<f32>,
    /// **Watercolor mixer** (Wet Mix — `wet_charge`/`wet_pull`/`wet_dilution`, `docs/Painter/07` §4)
    /// per-stroke state: the picked-up colour reservoir (unpremultiplied rgb + a presence-weighted
    /// confidence `w`) and its `recentness` (the Pull-gated resample clock). The brush deposits
    /// `lerp(brush, reservoir, (1−charge)·w)` — it picks up the frozen surface it crosses and (with
    /// Pull) drags it downstream. Reset on pen-down; inert unless `wet_charge < 1` (default → skipped,
    /// byte-identical). See [`super::watercolor_mixer`].
    wet_mix: watercolor_mixer::WetMix,
    /// The previous dab centre of the Smudge TRUE-SMEAR chain (`None` = stroke start / no smear yet).
    /// With `wet_smudge > 0` each dab DRAGS the frozen base's paint from here to its own centre
    /// (`smear_dab` on the forked [`Self::watercolor_base`]) before the wash composites over it — the
    /// physical "borrar" that moves already-painted paint (Enio 2026-07-06), not just a colour tint.
    wet_smear_pos: Option<[f32; 2]>,
    /// **Watercolor render-path** per-frame dirty rect — the union footprint of the dabs accumulated
    /// since the last optical composite (wet_edges `fMin..fMax`/`resetFrame`). The live
    /// [`Self::apply_watercolor`] recomposites ONLY this (padded by the influence radius), so the
    /// per-frame cost tracks the new dabs, not the whole stroke. Consumed (reset) by each composite.
    wet_frame_dirty: Option<Region>,
    /// **Watercolor render-path** cumulative dirty rect — the union footprint of EVERY dab this stroke
    /// (wet_edges `cMin..cMax`), tracked incrementally so the pen-up bake never scans the canvas for
    /// its bbox. [`Self::clear_wet_coverage`] folds it into the frame dirty (the cleared shape must be
    /// recomposited — the moving-preview union) before dropping it.
    wet_cum_dirty: Option<Region>,
    /// **Watercolor render-path** THIS-STROKE dirty rect — reset every `paint_begin` (even inside a wet
    /// session, unlike [`Self::wet_cum_dirty`] which accumulates the whole session's union). Only the
    /// current stroke's OWN footprint re-wets the moisture map at the bake ([`Self::pour_canvas_wet`]),
    /// so a second stroke never resets the drying clock of the earlier washes (doc 14 #4, Enio 2026-07-11).
    wet_stroke_dirty: Option<Region>,
    /// **Inpaint** defect mask (1 byte/px, `>= 128` ⇒ heal). Accumulated as the user brushes in Inpaint
    /// mode; on pen-up [`super::inpaint`] reconstructs the marked region and clears it. Sized `w*h`.
    inpaint_mask: Vec<u8>,

    // ── Fill (Bucket) — Procreate ColorDrop state ([`super::fill`]). ──
    /// ColorDrop threshold (`0..1`) → per-channel colour tolerance; adjusted live by the post-drop drag.
    fill_threshold: f32,
    /// Image-space seed of the current drop (`None` when idle).
    fill_seed: Option<[f32; 2]>,
    /// Pre-fill layer pixels, so every threshold change re-fills from the ORIGINAL region (not the
    /// already-filled result).
    fill_snapshot: Vec<u8>,
    /// The previous refill's filled bbox — so a SHRINKING fill dirties the vacated pixels too (the
    /// union of the old + new rects), not just the smaller new region (else the overflow ghosts).
    fill_last_rect: Option<Region>,
    /// The mode id to RESTORE after a momentary **ColorDrop** (C&F drag) finalizes (`None` = deliberate Fill).
    /// Set by the shell's C&F drag, consumed by `fill_commit` / `fill_cancel` ([`fill`], Enio 2026-07-03).
    fill_return_mode: Option<String>,
    /// **Inpaint** Patch Size (`0..1` track → patch radius `2..=6`); the reconstruction's patch footprint.
    inpaint_patch_norm: f32,
    /// **Inpaint** Quality (`0..1` track → EM iterations `3..=12`); more iterations = better fit, slower.
    inpaint_quality_norm: f32,
    /// **Inpaint** Search (`0..1` track → context-margin multiplier `0.5..3.0`); how much surrounding
    /// context PatchMatch samples from around the hole.
    inpaint_search_norm: f32,
    /// Per-stroke per-layer-colour accumulation (recomposite); see [`stamp_color_cache`].
    per_layer_stroke: stamp_color_cache::PerLayerStroke,
    /// For the dab list currently being stamped: which ORIGINAL dab each entry was replicated from
    /// ([`tiling::tiled_dabs_grouped`]). Empty ⇒ no Tiling ⇒ every entry is its own dab. The routes feed
    /// it to [`tiling::DabRng`] so a dab's wrapped copies SHARE its random frame — they are the same dab
    /// seen from both sides of the seam, and a per-copy draw made the tile stop matching itself.
    dab_groups: Vec<u32>,
    /// Cached coloured Shape **preview** (premul RGBA), re-baked only on appearance change; [`stamp_color_cache`].
    shape_color_preview: stamp_color_cache::ShapeColorPreview,
    /// **Deform** (Liquify) settings + session state — sub-mode, brush knobs, Freeze, and the pre-deform
    /// buffer Reconstruct/Amount read from. Mode-exclusive; see [`warp`] (Deform Wave 1).
    deform: warp::DeformState,
}

impl PainterTool {
    /// Set whether the Line method constrains to 45° increments this event (Blender Alt-drag). The
    /// shell forwards the live Alt state before each [`Self::on_canvas_pointer`], since the frozen
    /// `CanvasPointer` carries no modifiers. No effect on the other methods.
    pub fn set_line_constrain(&mut self, on: bool) {
        self.paint.line_constrain = on;
    }

    /// Set the shape editors' control-handle grab radius in image px (the shell forwards a
    /// screen-constant value scaled by the sprite footprint, so the hit targets stay the same size at
    /// any zoom). Shared by Curve and Ellipse.
    pub fn set_shape_grab_tol_px(&mut self, px: f32) {
        self.paint.shape_grab_tol_px = px.max(1.0);
    }

    // The open-shape aggregators (commit / cancel / discard / commit-keep) live in `curve_commit`.
    // The drag-preview stamping (mark_dirty / stamp_drag_preview / commit / stamp_stroke_dabs) lives in
    // the sibling `stamp_preview` module (workspace file-LOC cap).
}

/// Smallest region covering both `a` and `b`.
fn union_region(a: Region, b: Region) -> Region {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

// The `impl CanvasPaintTool` pointer entry point (`on_canvas_pointer`) lives in the sibling
// `canvas_pointer` module (workspace file-LOC cap); it drives the private stroke-lifecycle methods above.

#[cfg(test)]
mod tests;
