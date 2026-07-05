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
mod jitter_settings;
/// Watercolor section setters + router (edge darkening / granulation / pigment); no fluid sim.
mod watercolor_settings;
/// The canvas pointer's operation mode (Paint / Smear / Blur / Clone / Mask); split from `paint.rs` (cap).
mod paint_mode;
/// Multi-layer Shape (z-ordered layers + per-layer-colour state); split from `paint.rs` (LOC cap).
mod shape_layers;
/// Imported-image slots (Grain + Shape) + Shape geometry + Grain Depth setters; split from `brush_settings`.
mod shape_settings;
mod shape_snapshot; // unified shape+paint undo: each create/edit/bake = one ModelSnapshot on the timeline
mod stamp_color_cache; // the cached multi-layer coloured stamp (bake the composite once, blit per dab)
mod stamp_color_dynamic;
pub(crate) use paint_mode::{PAINT_MODE_COUNT, PaintMode};
mod lifecycle; // transient-edit reset run at each document (re)bind — abandons pending Fill/stroke/etc.
/// Drawing symmetry (mirror / radial) — engine glue, canvas-centre resolution + on-canvas pick modes.
mod symmetry;
mod tool_link; // "Sync with other tools": per-mode brush-settings swap + the link toggle; LOC-cap split
pub(crate) use symmetry::SymmetryPick;
/// Seamless Tiling (wrap-around painting) — dab replication across sprite edges + the toggles.
mod tiling;
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
    /// Cached coloured Shape **preview** (premul RGBA), re-baked only on appearance change; [`stamp_color_cache`].
    shape_color_preview: stamp_color_cache::ShapeColorPreview,
    /// **Deform** (Liquify) settings + session state — sub-mode, brush knobs, Freeze, and the pre-deform
    /// buffer Reconstruct/Amount read from. Mode-exclusive; see [`warp`] (Deform Wave 1).
    deform: warp::DeformState,
}

impl PainterTool {
    /// `true` when the active layer can be painted and the working buffer is sized — a **Raster** layer
    /// OR a **Mask** (its coverage buffer is bound to `canvas_rgba` like a raster's, so painting writes
    /// Rec.601-luma coverage: black conceals, white reveals). Group/adjustment/texture aren't paintable.
    fn paint_target_ready(&self) -> bool {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return false;
        }
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Raster(_) | LayerKind::Mask(_)))
    }

    /// Begin a stroke at `ev` and stamp the first dab. Snapshots the model for undo
    /// **before** painting so the whole stroke restores to the pre-stroke pixels.
    fn paint_begin(&mut self, ev: CanvasPointer) {
        // Mask tool: ensure the TRANSIENT scratch mask exists for the current layer (tool-side, no layer
        // created) before the stroke paints into it.
        if matches!(self.paint.paint_mode, PaintMode::Mask) {
            self.ensure_mask_scratch();
        }
        // Inpaint heal brush: start a fresh defect mask for this stroke (the previous stroke's mark was
        // consumed + cleared on its pen-up).
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            let (w, h) = self.source_size;
            let n = (w as usize) * (h as usize);
            if self.paint.inpaint_mask.len() == n {
                self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
            } else {
                self.paint.inpaint_mask = vec![0u8; n];
            }
        }
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.paint.drag_preview = None;
        self.paint.line_anchor = Some(ev.pos);
        // Reset the Accumulate-OFF cap mask (re-grown by the first dab) + the per-layer-colour
        // accumulation (so the recomposite snapshots THIS stroke's pre-pixels) — both per stroke.
        self.paint.stroke_mask.clear();
        self.paint.per_layer_stroke.reset();
        // Smear chains its source from the previous dab; a fresh stroke has none yet.
        self.paint.last_smear_pos = None;
        // Clone: establish the source→dest offset for this stroke (aligned keeps it across strokes,
        // non-aligned re-anchors to the sampled source each stroke). No-op unless a source is sampled.
        self.clone_begin_offset(ev.pos);
        // Pin the symmetry centre to the current canvas centre for the auto-centre modes before the
        // stroke captures the spec (the engine mirrors/rotates about `brush.symmetry.center`).
        self.resolve_symmetry_geometry();
        // Clone ignores Symmetry (its panel section is hidden): mirrored dabs would clone from mirrored
        // source positions, which is nonsensical — strip it from the captured spec so a leftover-enabled
        // flag can't silently mirror. Other modes keep Symmetry.
        let mut spec = self.paint.brush;
        if matches!(self.paint.paint_mode, PaintMode::Clone) {
            spec.symmetry.enabled = false;
        }
        let mut stroke = Stroke::new(spec, self.paint.dynamics, self.paint.seed);
        // Seed the texture RNG from this stroke's seed, decorrelated from the jitter stream so the
        // two don't lock-step (HR-5: deterministic per stroke).
        self.paint.tex_rng = self.paint.seed ^ 0x7465_7874_7572_6573;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.begin(
            StrokePoint {
                pos: ev.pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.stamp_stroke_dabs(&dabs);
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// Extend the in-progress stroke to `ev`, stamping any dabs the spacing emits.
    /// Returns `false` if no stroke is active (a stray Move).
    fn paint_extend(&mut self, ev: CanvasPointer) -> bool {
        let Some(mut stroke) = self.paint.stroke.take() else {
            return false;
        };
        // Line + Alt: snap the cursor to a 45° increment around the press point (Blender
        // `constrain_line`). Tool-side so the engine's Line fill stays a plain anchor→cursor segment.
        let pos = match (self.paint.brush.stroke_method, self.paint.line_anchor) {
            (StrokeMethod::Line, Some(anchor)) if self.paint.line_constrain => {
                brush_settings::snap_to_45(anchor, ev.pos)
            }
            _ => ev.pos,
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.extend(
            StrokePoint {
                pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.stamp_stroke_dabs(&dabs);
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
        self.paint.moved_this_frame = true;
        true
    }

    /// Per-frame heartbeat while a stroke is held (`dt_s` = wall time since the last frame); clears the
    /// move flag and drives two time-based behaviours (both no-op when their method doesn't apply):
    /// - **Airbrush timer:** deposit dabs at the brush's Rate, moving OR parked (Blender fires it on a
    ///   timer, not on motion — builds up when held, sparse when swept). No-op for non-Airbrush.
    /// - **Stabilizer catch-up:** when parked, walk the lagged path to the cursor (`settle` is
    ///   Space-only) so a high-stabilizer stroke arrives without waiting for pointer-up.
    pub(crate) fn paint_tick(&mut self, dt_s: f32) {
        // Decay the transient in-gizmo Stencil preview (armed by panel param changes); runs every frame
        // even with no open stroke, so it fades out shortly after the user stops changing the params.
        if self.paint.stencil_preview_s > 0.0 {
            self.paint.stencil_preview_s = (self.paint.stencil_preview_s - dt_s).max(0.0);
        }
        // Keep the auto-centre symmetry pivot on the canvas centre every frame (also no-op when idle),
        // so the dashed overlay guide stays correct after a resize / fresh-sprite bind without paint.
        self.resolve_symmetry_geometry();
        let parked = !self.paint.moved_this_frame;
        self.paint.moved_this_frame = false;
        let Some(mut stroke) = self.paint.stroke.take() else {
            return;
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.tick(dt_s, &mut dabs);
        self.stamp_dabs(&dabs);
        if parked {
            stroke.settle(&mut dabs); // clears `dabs` first
            self.stamp_dabs(&dabs);
        }
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// Finish the stroke at `ev` (stamp the final segment, flush the freehand smoother's tail so
    /// the stroke reaches the release point, then close + record undo).
    fn paint_end(&mut self, ev: CanvasPointer) {
        self.paint_extend(ev);
        if let Some(mut stroke) = self.paint.stroke.take() {
            let mut dabs = std::mem::take(&mut self.paint.dabs);
            stroke.finish(&mut dabs);
            self.stamp_dabs(&dabs);
            self.paint.dabs = dabs;
            self.paint.stroke = Some(stroke);
        }
        // Drag Dot: the dab at the release point is the commit — keep it (drop the restore record).
        self.commit_drag_preview();
        // Inpaint heal brush: reconstruct the marked defect BEFORE close_stroke, so the structural-undo
        // entry captures pre-stroke → healed as a single Cmd+Z step.
        if matches!(self.paint.paint_mode, PaintMode::Inpaint) {
            self.heal_inpaint();
        }
        self.close_stroke();
    }

    /// Finalize the current stroke: drop the in-progress state and push one undo
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit. No-op when no stroke
    /// is open. Reuses the structural-undo stack (a full-canvas snapshot per stroke; tile delta later).
    fn close_stroke(&mut self) {
        self.paint.stroke = None;
        self.paint.line_anchor = None;
        self.paint.last_smear_pos = None;
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
    }

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
