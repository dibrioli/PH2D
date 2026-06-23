//! Canvas painting — drives the clean-room Blender brush engine
//! (`ph2d-painter-brush`) from pointer samples, writing dabs into the active
//! raster layer's `canvas_rgba`.
//!
//! The brush *behaviour* (falloff, spacing, pressure, blend) is the engine's;
//! this module is only the glue between the editor's [`CanvasPaintTool`] contract
//! (ADR-0040 Amendment 3) and the painter's layer buffers + preview/dirty plumbing.
//! Mask / clipping / alpha-lock honouring and per-stroke undo are later-phase
//! refinements (see `docs/Painter/02_plano_de_implementacao.md` Fase 3).

use super::*;

use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use ph2d_painter_brush::{
    AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec, Dab, Dynamics, MAX_INPUT_SAMPLES, Stroke,
    StrokeMethod, StrokePoint, stamp_dab_textured,
};

/// Brush + Stroke-section parameter snapshot & setters (submodule so it shares `PaintState`'s
/// private brush access; split out to keep both files under the workspace LOC cap).
mod brush_settings;
/// The Curve stroke method's on-canvas point editor (submodule, as `brush_settings`).
mod curve;
pub use curve::CurveOverlay;
/// The Circle stroke method's on-canvas ellipse editor (same submodule rationale as `curve`).
mod circle;
pub use circle::CircleOverlay;
/// The Polygon stroke method's on-canvas regular-N-gon editor (same submodule rationale).
mod polygon;
pub use polygon::PolygonOverlay;
/// The Stencil texture mapping's on-canvas handle editor (move/resize the image-space rect).
mod stencil;
pub use stencil::StencilOverlay;
mod ramp;
/// The Blender-style cached brush stamp (render falloff×texture once, scale-blit per dab).
mod stamp_cache;

/// Default control-handle grab radius in image px (the Curve and Circle editors share one tolerance),
/// until the shell forwards a screen-scaled value via [`PainterTool::set_shape_grab_tol_px`].
const DEFAULT_SHAPE_GRAB_TOL_PX: f32 = 8.0;

/// Smallest brush radius the size UI maps to, in image pixels. The size slider's
/// `0..1` track and the `[` / `]` keyboard nudge both clamp here.
pub const BRUSH_SIZE_MIN_PX: f32 = 1.0;
/// Largest brush radius the size UI maps to, in image pixels. (The engine's own
/// allocation cap is higher; this is the interactive range, not a hard limit.)
pub const BRUSH_SIZE_MAX_PX: f32 = 512.0;

// Interactive UI ranges for the Stroke section's non-`0..1` sliders. The slider's
// `0..1` track maps onto `0..MAX` (or `1..MAX` for counts); these are the single
// source of truth shared by the tool setters and the panel's value↔track maps.
/// Max spacing the slider reaches, as a fraction of diameter (`1.0` = 100% = one full
/// diameter between dab centres). The engine accepts more; this is the interactive top.
pub const BRUSH_SPACING_MAX: f32 = 1.0;
/// Max absolute jitter the slider reaches, in pixels (View unit).
pub const BRUSH_JITTER_ABS_MAX_PX: f32 = 64.0;
/// Max value for the Input Samples / Dash Length count sliders (mirrors the engine's
/// input-sample window cap).
pub const BRUSH_COUNT_SLIDER_MAX: u32 = MAX_INPUT_SAMPLES as u32;
/// Airbrush **Rate** slider floor / ceiling, in seconds — the panel maps its `0..1` track linearly
/// onto `[MIN, MAX]` and the tool clamps to it. Re-exported from the engine's Blender soft range
/// (default `0.1`) so the panel value↔track map shares the single source.
pub const BRUSH_AIRBRUSH_RATE_MIN_S: f32 = AIRBRUSH_RATE_MIN_S;
/// See [`BRUSH_AIRBRUSH_RATE_MIN_S`].
pub const BRUSH_AIRBRUSH_RATE_MAX_S: f32 = AIRBRUSH_RATE_MAX_S;

// The panel-facing snapshot [`BrushSettings`] + the falloff preview helper
// `brush_falloff_weight_at` live in the `brush_settings` submodule (alongside the setters that
// are their single clamp source), so this file stays under the workspace LOC cap. Re-exported
// here to keep their `paint::` public path.
pub use brush_settings::{BrushSettings, PANEL_RAMP_STOPS, brush_falloff_weight_at};

/// Brush settings + in-progress stroke state held by the [`PainterTool`].
/// A single Drag Dot's restore record: the pristine canvas pixels under the dab's footprint
/// (RGBA8, row-major over `rect`) saved *before* it was stamped, so the next move can erase it.
/// The dot then follows the cursor leaving no trail, and only the dab at the release point survives.
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
    /// Running splitmix64 state for the brush texture's per-dab Random rotation / offset. Reset
    /// from the stroke seed at pointer-down (decorrelated from the jitter stream) and advanced once
    /// per textured dab in [`PainterTool::stamp_dabs`], so a stroke's texture randomness is
    /// reproducible (HR-5) yet differs across dabs and strokes.
    tex_rng: u64,
    /// Model snapshot captured at pointer-down (before the first dab) — committed
    /// to the undo stack at pointer-up so the whole stroke undoes as one unit.
    stroke_undo: Option<crate::undo::ModelSnapshot>,
    /// Eraser mode: overrides the brush blend with Erase Alpha at stamp time
    /// (the drawing blend in `brush.blend` is preserved for when it's off).
    eraser: bool,
    /// Set by [`PainterTool::paint_extend`] each pointer move and cleared by the per-frame tick.
    /// While a stroke is held and this stays `false` for a frame (the pointer is parked), the tick
    /// settles the stabilizer toward the cursor — so a high-stabilizer stroke catches up on a pause,
    /// not only on pointer-up. Gating on it keeps the during-movement smoothing at full strength.
    moved_this_frame: bool,
    /// Restore record for the in-progress Drag Dot's single moving dab; `None` for every other
    /// method (and once the dot is committed on pointer-up).
    drag_preview: Option<DragPreview>,
    /// The press point of the in-progress stroke — the pivot the Line method's Alt-constrain snaps
    /// the cursor around (45° increments). `None` when no stroke is open.
    line_anchor: Option<[f32; 2]>,
    /// Alt held this event — constrains the Line to 45° increments (Blender `constrain_line`). Set
    /// by the shell each pointer event (the frozen `CanvasPointer` can't carry modifiers).
    line_constrain: bool,
    /// In-progress Curve session (the on-canvas point editor); `None` when no curve is being
    /// authored. See [`crate::tool::paint::curve`].
    curve: Option<curve::CurveEditor>,
    /// In-progress Circle session (the on-canvas ellipse editor); `None` when none is being authored.
    /// See [`crate::tool::paint::circle`].
    circle: Option<circle::CircleEditor>,
    /// In-progress Polygon session (the on-canvas regular-N-gon editor); `None` when none is open.
    /// See [`crate::tool::paint::polygon`].
    polygon: Option<polygon::PolygonEditor>,
    /// Control-handle grab radius in image px for the shape editors (Curve + Circle) — the shell
    /// forwards a screen-constant value scaled by the sprite footprint before each pointer event.
    shape_grab_tol_px: f32,
    /// In-progress Stencil overlay drag (move/resize the texture rect); `None` when not dragging a
    /// handle. See [`crate::tool::paint::stencil`].
    stencil_grab: Option<stencil::StencilGrab>,
    /// Imported brush-texture luminance (heavy → not in the `Copy` spec); borrowed as an `ImageMask`.
    texture_image: Option<brush_settings::BrushTextureImage>,
    /// Set when the user picks the Image kind; the shell polls it to open a file picker.
    texture_image_pending: bool,
    /// Bumped whenever [`texture_image`] changes, so the stamp cache re-renders the Image mask.
    texture_image_version: u64,
    /// Cached brush stamp (falloff × View texture) + the key it was rendered for. Re-rendered on
    /// appearance / mask-size change; scale-blitted per dab. See [`crate::tool::paint::stamp_cache`].
    stamp_cache: Option<(ph2d_painter_brush::StampMask, stamp_cache::StampKey)>,
    /// Lazily-filled canvas-space texture cache for the Tiled / Stencil mappings (canvas-fixed); the
    /// texture is computed once per canvas pixel per stroke. See [`crate::tool::paint::stamp_cache`].
    canvas_tex_cache: Option<stamp_cache::CanvasTexCache>,
    /// The brush texture's **Color Ramp** (reusable `ph2d_color::ColorRamp`): when `enabled`, the
    /// texture's scalar indexes it for the per-texel paint colour. Baked to `lut` (linear → sRGB
    /// straight, 256 entries) when `dirty`; passed per dab to `stamp_dab_ramped`.
    texture_ramp: ph2d_color::ColorRamp,
    texture_ramp_enabled: bool,
    texture_ramp_lut: Vec<[f32; 4]>,
    texture_ramp_dirty: bool,
}

impl Default for PaintState {
    fn default() -> Self {
        Self {
            // A moderate black brush so strokes read clearly on both small and
            // large canvases. The brush-settings UI drives size/colour later
            // (`docs/Painter/` Fase 4); the engine's own default is 25 px radius.
            brush: BrushSpec {
                radius_px: 10.0,
                ..BrushSpec::default()
            },
            dynamics: Dynamics::default(),
            stroke: None,
            dabs: Vec::new(),
            seed: 0,
            tex_rng: 0,
            stroke_undo: None,
            eraser: false,
            moved_this_frame: false,
            drag_preview: None,
            line_anchor: None,
            line_constrain: false,
            curve: None,
            circle: None,
            polygon: None,
            shape_grab_tol_px: DEFAULT_SHAPE_GRAB_TOL_PX,
            stencil_grab: None,
            texture_image: None,
            texture_image_pending: false,
            texture_image_version: 0,
            stamp_cache: None,
            canvas_tex_cache: None,
            texture_ramp: ph2d_color::ColorRamp::default(),
            texture_ramp_enabled: false,
            texture_ramp_lut: Vec::new(),
            texture_ramp_dirty: true,
        }
    }
}

impl PainterTool {
    /// `true` when the active layer is a raster layer that can be painted and the
    /// working buffer is sized. Mask / group / adjustment layers are not paintable.
    fn paint_target_ready(&self) -> bool {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.is_empty() {
            return false;
        }
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Raster(_)))
    }

    /// Begin a stroke at `ev` and stamp the first dab. Snapshots the model for undo
    /// **before** painting so the whole stroke restores to the pre-stroke pixels.
    fn paint_begin(&mut self, ev: CanvasPointer) {
        let before = self.snapshot_model();
        self.paint.stroke_undo = Some(before);
        self.paint.drag_preview = None;
        self.paint.line_anchor = Some(ev.pos);
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, self.paint.seed);
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

    /// Per-frame heartbeat while a stroke is held (`dt_s` = wall time since the last frame). Drives
    /// the two time-based behaviours; both no-op when their method / condition doesn't apply, and the
    /// per-frame move flag is always cleared:
    /// - **Airbrush timer:** deposit dabs at the brush's Rate, moving OR parked (Blender fires the
    ///   airbrush on a timer, not on motion — so the spray builds up when held still and stays sparse
    ///   when swept fast). The engine's `tick` is a no-op for every non-Airbrush method.
    /// - **Stabilizer catch-up:** when parked, walk the lagged path up to the cursor so a
    ///   high-stabilizer stroke arrives without waiting for pointer-up (`settle` is Space-only).
    pub(crate) fn paint_tick(&mut self, dt_s: f32) {
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
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit.
    /// No-op when no stroke is open. Reuses the structural-undo stack (a full-canvas
    /// snapshot per stroke; a tile-based delta is a later optimization).
    fn close_stroke(&mut self) {
        self.paint.stroke = None;
        self.paint.line_anchor = None;
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

    /// Stamp a batch of dabs into `canvas_rgba` (with the brush texture, if any) + accumulate the
    /// dirty rect. Each dab carries its pressure-scaled radius + coverage; the static appearance
    /// (falloff / blend / colour) comes from `self.paint.brush`. Large dabs parallelise internally.
    fn stamp_dabs(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let mut brush = self.paint.brush;
        // Eraser overrides the blend with Erase Alpha (the drawing blend in `brush.blend` is kept).
        if self.paint.eraser {
            brush.blend = ph2d_painter_brush::BrushBlend::EraseAlpha;
        }
        // Alpha lock ("Lock" / Preserve Transparency): the dab paints only into the active layer's
        // existing alpha (clip + mask are composite-time; only alpha-lock constrains the dab itself).
        let alpha_locked = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked);
        // Color Ramp: the texture's scalar drives the per-texel paint COLOUR (via the baked LUT), so
        // it bypasses the scalar-only caches for the per-pixel colour path (the LUT keeps it cheap).
        if self.paint.texture_ramp_enabled && brush.texture.is_active() {
            self.stamp_dabs_ramped(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // A static **View** texture is scale-invariant: render the stamp once into a cached mask +
        // scale-blit per dab (Blender's brush-image cache), so a large Anchored re-stamp pays a cheap
        // blit, not a per-pixel recompute. No-texture / canvas-relative / per-dab mappings stay
        // per-pixel (nothing to cache, or not scale-invariant).
        if brush.texture.is_active() && brush.texture.is_cacheable() {
            self.stamp_dabs_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Tiled / Stencil are canvas-fixed but dab-independent — cache each canvas pixel's texture
        // once per stroke + look it up per dab (Anchored re-stamps the same region, so inner = reuse).
        if brush.texture.is_canvas_cacheable() {
            self.stamp_dabs_canvas_cached(dabs, &brush, alpha_locked, w, h);
            return;
        }
        // Per-pixel path. Resolve the per-dab texture frame only when a texture is assigned (else the
        // engine's texture-free fast path); the texture RNG is copied out (canvas borrow) + restored.
        let textured = brush.texture.is_active();
        let image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let mut tex_rng = self.paint.tex_rng;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for (i, d) in dabs.iter().enumerate() {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..brush
            };
            let basis = textured.then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &spec.texture,
                    brush_settings::dab_tangent(dabs, i),
                    &mut tex_rng,
                    [w as f32, h as f32],
                )
            });
            if let Some(r) = stamp_dab_textured(
                buf,
                w,
                h,
                d.center,
                &spec,
                d.coverage,
                alpha_locked,
                basis.as_ref(),
                image.as_ref(),
            ) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
        self.paint.tex_rng = tex_rng;
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Flag `rect` dirty for the next GPU preview upload + bump the active layer's pixel epoch.
    fn mark_dirty(&mut self, rect: Region) {
        self.dirty_rect = Some(self.dirty_rect.map_or(rect, |acc| union_region(acc, rect)));
        self.preview_dirty = true;
        let active = self.layers.active();
        self.bump_layer_pixels(active);
    }

    /// Conservative pixel bbox a dab of `radius` at `center` can touch, clamped to the canvas.
    fn dab_bbox(&self, center: [f32; 2], radius: f32) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let r = radius.ceil() as i64 + 1;
        let cx = center[0].round() as i64;
        let cy = center[1].round() as i64;
        let x0 = (cx - r).clamp(0, w as i64);
        let y0 = (cy - r).clamp(0, h as i64);
        let x1 = (cx + r).clamp(0, w as i64);
        let y1 = (cy + r).clamp(0, h as i64);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }

    /// Copy the RGBA8 pixels of `rect` out of `canvas_rgba` (row-major over the region).
    fn save_region(&self, rect: &Region) -> Vec<u8> {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let mut out = Vec::with_capacity(rw * rect.h as usize);
        for row in 0..rect.h {
            let start = (rect.y + row) as usize * stride + rect.x as usize * 4;
            out.extend_from_slice(&self.canvas_rgba[start..start + rw]);
        }
        out
    }

    /// Write `pixels` (from [`Self::save_region`]) back into `rect` and flag it dirty.
    fn restore_region(&mut self, rect: &Region, pixels: &[u8]) {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for row in 0..rect.h {
            let dst = (rect.y + row) as usize * stride + rect.x as usize * 4;
            let src = row as usize * rw;
            buf[dst..dst + rw].copy_from_slice(&pixels[src..src + rw]);
        }
        self.mark_dirty(*rect);
    }

    /// Stamp an interactive preview batch (Drag Dot / Anchored = 1 dab, Line = N): restore the
    /// previous footprint's saved pixels, then save the pristine pixels under the new dabs' UNION
    /// bbox and stamp there — so the moving preview leaves no trail. Pen-up: `commit_drag_preview`.
    fn stamp_drag_preview(&mut self, dabs: &[Dab]) {
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        let bbox = dabs.iter().fold(None, |acc, d| {
            match (acc, self.dab_bbox(d.center, d.radius_px)) {
                (Some(a), Some(r)) => Some(union_region(a, r)),
                (a, r) => a.or(r),
            }
        });
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
    /// preview methods: route their batch through the restore+re-stamp path (restore the prior
    /// footprint + re-stamp) so the moving/resizing/growing preview leaves no trail and
    /// `commit_drag_preview` keeps the last on pen-up. Every other method uses the cumulative stamp.
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
            StrokeMethod::Curve => return self.curve_pointer(ev),
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
