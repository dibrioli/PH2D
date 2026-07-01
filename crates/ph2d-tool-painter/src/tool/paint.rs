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
/// The Curve stroke method's on-canvas point editor (submodule, as `brush_settings`).
mod curve;
mod curve_commit; // Apply / Apply & Keep commit verbs for the Curve editor; split from `curve`
mod curve_geom; // flatten (incl. closed seam) + point hit-test/nearest/insert; split from `curve`
mod curve_gizmo; // whole-curve transform gizmo (move/scale/rotate the entire curve); split from `curve`
mod curve_handle; // per-anchor handle kinds (Free/Aligned/Vector/Auto) + derived geometry; split from `curve`
mod curve_join; // corner joins for the offset: smooth-merge / convex-miter / concave-split; split from `curve_offset`
mod curve_offset; // perpendicular offset (parallel curve) + CAD-grade reconstruction; split from `curve_geom`
mod curve_tangent; // Bézier tangent-handle hit-test, aligned mirror, overlay snapshot; split from `curve`
mod curve_trim; // self-intersection trim of the offset spine (open + closed); split from `curve_offset`
/// Per-dab randomize setters (Jitter Scale / Rotate / Randomize Color); split from `brush_settings`.
mod jitter_settings;
/// Multi-layer Shape (z-ordered layers + per-layer-colour state); split from `paint.rs` (LOC cap).
mod shape_layers;
/// Imported-image slots (Grain + Shape) + Shape geometry + Grain Depth setters; split from `brush_settings`.
mod shape_settings;
mod shape_snapshot; // unified shape+paint undo: each create/edit/bake = one ModelSnapshot on the timeline
mod stamp_color_cache; // the cached multi-layer coloured stamp (bake the composite once, blit per dab)
mod stamp_color_dynamic;
/// Drawing symmetry (mirror / radial) — engine glue, canvas-centre resolution + on-canvas pick modes.
mod symmetry;
pub(crate) use symmetry::SymmetryPick;
/// Seamless Tiling (wrap-around painting) — dab replication across sprite edges + the toggles.
mod tiling;
pub use curve::CurveOverlay;
pub use curve_gizmo::TransformGizmo;
pub use curve_tangent::TangentHandles;
/// The Circle stroke method's on-canvas ellipse editor (same submodule rationale as `curve`).
mod circle;
pub use circle::CircleOverlay;
/// The Polygon stroke method's on-canvas regular-N-gon editor (same submodule rationale).
mod polygon;
pub use polygon::PolygonOverlay;
/// The Stencil texture mapping's on-canvas handle editor (move/resize the image-space rect).
mod stencil;
pub use stencil::{StencilOverlay, StencilPreview};
/// The **Blur** (soften) route — a stationary neighbourhood blur per dab; sibling of `stamp_dabs_smear`.
mod blur_route;
/// The **Composite Brush** — Brush + Smear + Blur as a reorderable 3-layer stack run per dab.
mod composite;
pub(crate) use composite::{CompositeLayer, CompositeOp};
/// The **Clone** (clone-stamp) route — copy canvas pixels from a sampled source at a fixed offset.
mod clone;
mod ramp;
mod ramp_lut; // ramp LUT baking (colour owner + colour/tone LUTs); split from `stamp_cache` (LOC cap)
/// Pixel-region save/restore helpers for the drag preview (`dab_bbox`/`save_region`/`restore_region`).
mod region;
mod shape_ramp;
mod snapshot;
/// The Blender-style cached brush stamp (render falloff×texture once, scale-blit per dab).
mod stamp_cache;
/// The stamp route dispatcher (Shape + Grain → which of the 4 stamp paths); split for the LOC cap.
mod stamp_route;
/// `PaintState::default` body — split out for the workspace file-LOC cap (struct stays in `paint.rs`).
mod state_default;

/// Default control-handle grab radius in image px (the Curve and Circle editors share one tolerance),
/// until the shell forwards a screen-scaled value via [`PainterTool::set_shape_grab_tol_px`].
const DEFAULT_SHAPE_GRAB_TOL_PX: f32 = 8.0;

/// Smallest brush radius the size UI maps to (image px); the size slider + `[` / `]` nudge clamp here.
pub const BRUSH_SIZE_MIN_PX: f32 = 1.0;
/// Largest brush radius the size UI maps to (image px); the interactive range, not the engine's hard cap.
pub const BRUSH_SIZE_MAX_PX: f32 = 512.0;

// Interactive UI ranges for the Stroke section's non-`0..1` sliders: the `0..1` track maps onto `0..MAX`.
/// Max spacing the slider reaches, as a fraction of diameter (`1.0` = one full diameter between dabs).
pub const BRUSH_SPACING_MAX: f32 = 1.0;
/// Max absolute jitter the slider reaches, in pixels (View unit).
pub const BRUSH_JITTER_ABS_MAX_PX: f32 = 64.0;
/// Max value for the Input Samples / Dash Length count sliders (the engine's input-sample window cap).
pub const BRUSH_COUNT_SLIDER_MAX: u32 = MAX_INPUT_SAMPLES as u32;
/// Airbrush **Rate** slider floor / ceiling (seconds) — the panel maps its `0..1` track linearly onto
/// `[MIN, MAX]`. Re-exported from the engine's Blender soft range so the value↔track map shares one source.
pub const BRUSH_AIRBRUSH_RATE_MIN_S: f32 = AIRBRUSH_RATE_MIN_S;
/// See [`BRUSH_AIRBRUSH_RATE_MIN_S`].
pub const BRUSH_AIRBRUSH_RATE_MAX_S: f32 = AIRBRUSH_RATE_MAX_S;

// The panel-facing snapshot [`BrushSettings`] + the falloff preview helper `brush_falloff_weight_at`
// live in the `brush_settings` submodule (their single clamp source); re-exported for the `paint::` path.
pub use brush_settings::{BrushSettings, PANEL_RAMP_STOPS};
pub use shape_layers::MAX_SHAPE_LAYERS;
pub use snapshot::brush_falloff_weight_at;

/// A Drag Dot's restore record: pristine pixels under the dab footprint (RGBA8 over `rect`), saved before stamping so the next move erases it (no trail).
struct DragPreview {
    rect: Region,
    pixels: Vec<u8>,
}

/// Which operation the canvas pointer performs — selected from the left rail's Painter tools and
/// routed in via `PanelEvent::SelectOption(PAINTER_PAINT_MODE, …)`. `Paint` is the normal dab-stamp
/// path (brush colour, Shape, Grain, ramps); `Smear` drags the canvas content along the stroke
/// ([`ph2d_painter_brush::smear_dab`], the Blender/Krita "Smearing" algorithm); `Blur` softens the
/// canvas under each dab ([`ph2d_painter_brush::blur_dab`], the Blender Soften algorithm); `Clone`
/// copies canvas pixels from a sampled source at a fixed offset ([`ph2d_painter_brush::clone_dab`], the
/// clone stamp); `Mask` paints a GRAYSCALE coverage value (the brush colour desaturated to Rec.601
/// luma) — for a layer mask (white reveals / black conceals), reusing the full brush pipeline. Eraser
/// stays a separate blend override layered on top of `Paint`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum PaintMode {
    #[default]
    Paint,
    Smear,
    Blur,
    Clone,
    Mask,
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
    /// Previous dab centre during a **Smear** stroke — the source position each dab lifts from; `None`
    /// at stroke start (the first dab has nothing to smear from). Chained across pointer batches. See [`stamp_route`].
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
    /// In-progress Curve session (the on-canvas point editor); `None` when idle. [`curve`].
    curve: Option<curve::CurveEditor>,
    /// In-progress Circle session (the on-canvas ellipse editor); `None` when idle. [`circle`].
    circle: Option<circle::CircleEditor>,
    /// In-progress Polygon session (the on-canvas regular-N-gon editor); `None` when idle. [`polygon`].
    polygon: Option<polygon::PolygonEditor>,
    /// Control-handle grab radius (image px) for the shape editors — shell forwards a footprint-scaled value.
    shape_grab_tol_px: f32,
    /// **Offset** slider track (`0..1`, `0.5` = none) — perpendicular path offset for the shape editors.
    shape_offset_norm: f32,
    /// **Accumulated** offset (px) from prior Apply & Keep presses; the EFFECTIVE offset is
    /// `shape_offset_base_px + slider` — always a single offset of the pristine base, so it never compounds.
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
    /// Imported brush-**Shape** luminance (the silhouette tip; heavy → not in the `Copy` spec), borrowed as
    /// an `ImageMask`. `None` ⇒ silhouette = falloff. Set by the shell ("Use as Brush Shape"); reset clears it.
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
    /// Per-stroke per-layer-colour accumulation (recomposite); see [`stamp_color_cache`].
    per_layer_stroke: stamp_color_cache::PerLayerStroke,
    /// Cached coloured Shape **preview** (premul RGBA), re-baked only on appearance change; [`stamp_color_cache`].
    shape_color_preview: stamp_color_cache::ShapeColorPreview,
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
        // Mask tool: retarget the stroke onto a mask (switch to / auto-create the active layer's mask)
        // BEFORE the stroke's undo snapshot, so the mask create/switch is its own undo step.
        if matches!(self.paint.paint_mode, PaintMode::Mask) {
            self.ensure_mask_edit_target();
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
    /// any zoom). Shared by Curve and Circle.
    pub fn set_shape_grab_tol_px(&mut self, px: f32) {
        self.paint.shape_grab_tol_px = px.max(1.0);
    }

    /// Commit whichever on-canvas shape editor (Curve / Circle / Polygon) is open — the verb behind
    /// Enter and the first undo. Returns `true` when one was committed. At most one is ever open.
    pub fn commit_open_shape(&mut self) -> bool {
        self.curve_commit() || self.circle_commit() || self.polygon_commit()
    }

    /// Cancel whichever shape editor is open (revert its preview) — the verb behind Esc and leaving
    /// the shape's method. Returns `true` when one was cancelled.
    pub fn cancel_open_shape(&mut self) -> bool {
        self.curve_cancel() || self.circle_cancel() || self.polygon_cancel()
    }

    /// Drop any open shape editor without touching pixels — for teardown where the canvas is replaced
    /// or cleared (fresh source / deactivate / non-paintable layer).
    pub(crate) fn discard_open_shape(&mut self) {
        self.curve_discard();
        self.circle_discard();
        self.polygon_discard();
    }

    /// Flag `rect` dirty for the next GPU preview upload + bump the active layer's pixel epoch.
    fn mark_dirty(&mut self, rect: Region) {
        self.dirty_rect = Some(self.dirty_rect.map_or(rect, |acc| union_region(acc, rect)));
        self.preview_dirty = true;
        self.edited_since_bind = true; // unbaked work — the shell auto-persists on leave/deactivate
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// Stamp an interactive preview batch (Drag Dot / Anchored = 1 dab, Line = N): restore the
    /// previous footprint's saved pixels, then save the pristine pixels under the new dabs' UNION
    /// bbox and stamp there — so the moving preview leaves no trail. Pen-up: `commit_drag_preview`.
    ///
    /// The stamp goes through the full [`Self::stamp_dabs`] dispatcher (NOT the bare brush route), so a
    /// **Composite Brush** runs all three layers here too. `stamp_dabs` tiles internally (so it takes
    /// the UNtiled dabs); the save-region bbox is measured over the tiled set so it still covers the
    /// wrapped copies (else the wrapped paint falls outside the restore region — a trail).
    fn stamp_drag_preview(&mut self, dabs: &[Dab]) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        // Coverage bbox over the wrapped Tiling copies (the stamp re-tiles them itself).
        let coverage_storage;
        let coverage: &[Dab] = if self.paint.tiling[0] || self.paint.tiling[1] {
            coverage_storage = tiling::tiled_dabs(dabs, self.source_size, self.paint.tiling);
            &coverage_storage
        } else {
            dabs
        };
        let bbox = coverage.iter().fold(None, |acc, d| {
            match (acc, self.dab_bbox(d.center, d.radius_px)) {
                (Some(a), Some(r)) => Some(union_region(a, r)),
                (a, r) => a.or(r),
            }
        });
        // Each preview frame re-stamps the WHOLE current batch onto the restored (pristine) canvas, so
        // a Composite Brush's Smear layer must chain fresh within THIS batch — clear the cross-batch
        // source (a Line's dabs then smear from the anchor; a single Drag-Dot dab simply has no source).
        self.paint.last_smear_pos = None;
        match bbox {
            Some(rect) => {
                let pixels = self.save_region(&rect);
                self.stamp_dabs(dabs);
                self.paint.drag_preview = Some(DragPreview { rect, pixels });
            }
            None => self.stamp_dabs(dabs),
        }
    }

    /// Commit the interactive preview: drop the restore record so the last batch stays painted.
    /// Safe to call for any method (a no-op unless a preview is live).
    fn commit_drag_preview(&mut self) {
        self.paint.drag_preview = None;
    }

    /// Stamp the dabs a `begin`/`extend` produced. Drag Dot, Anchored AND Line are interactive
    /// preview methods: route their batch through the restore+re-stamp path so the moving/resizing/
    /// growing preview leaves no trail and `commit_drag_preview` keeps the last on pen-up. Every other
    /// method uses the cumulative stamp.
    fn stamp_stroke_dabs(&mut self, dabs: &[Dab]) {
        if matches!(
            self.paint.brush.stroke_method,
            StrokeMethod::DragDot | StrokeMethod::Anchored | StrokeMethod::Line
        ) {
            self.stamp_drag_preview(dabs);
        } else {
            self.stamp_dabs(dabs);
        }
    }
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

/// Brush-settings accessors — the Brush section of the layers panel and the
/// `[` / `]` keyboard nudge drive these; the brush is plain state (changing it
/// touches no pixels, so there is no undo entry or preview invalidation here).
impl CanvasPaintTool for PainterTool {
    fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase == PointerPhase::Hover {
            return false; // hover is cursor/preview only
        }
        // While a Symmetry pick mode is armed, the canvas sets the mirror line / radial centre instead
        // of painting (works on any layer, so it precedes the paintable-target gate).
        if self.symmetry_pick_active() {
            return self.symmetry_pick_pointer(ev);
        }
        // Clone "Set Source" pick mode: the next canvas Down samples the source anchor (consumes the
        // click, no paint), like the Symmetry picks. Works on any layer (it records a coordinate).
        if self.clone_sample_armed() {
            return self.clone_sample_pointer(ev);
        }
        if !self.paint_target_ready() {
            // Active layer isn't paintable (mask/group/adjustment) or no canvas:
            // finalize any half-open stroke (records its undo) before bailing. Drop any open
            // shape session too (its restore would read a stale buffer once the layer changed).
            self.discard_open_shape();
            self.close_stroke();
            return false;
        }
        // Curve and Circle are persistent on-canvas shape editors (draw → edit → commit), not a
        // single press→release stroke — route every canvas event through them instead of the generic
        // path.
        match self.paint.brush.stroke_method {
            // Free Hand shares the Curve editor (its draw phase captures a freehand path, then it's an
            // ordinary editable curve), so it routes through `curve_pointer` too.
            StrokeMethod::Curve | StrokeMethod::FreeHand => return self.curve_pointer(ev),
            StrokeMethod::Circle => return self.circle_pointer(ev),
            StrokeMethod::Polygon => return self.polygon_pointer(ev),
            _ => {}
        }
        // Stencil texture: grabbing an overlay handle (corner = resize, centre = move) edits the
        // rect and consumes the event; a Down away from every handle (or any move without a grab)
        // falls through to normal painting — the handles disambiguate, so no modifier is needed.
        if self.stencil_edit_active()
            && (ev.phase == PointerPhase::Down || self.paint.stencil_grab.is_some())
            && self.stencil_pointer(ev)
        {
            return true;
        }
        match ev.phase {
            PointerPhase::Down => {
                self.paint_begin(ev);
                true
            }
            PointerPhase::Move => self.paint_extend(ev),
            PointerPhase::Up => {
                self.paint_end(ev);
                true
            }
            PointerPhase::Hover => false,
        }
    }
}

#[cfg(test)]
mod tests;
