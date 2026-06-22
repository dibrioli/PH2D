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
    AIRBRUSH_RATE_MAX_S, AIRBRUSH_RATE_MIN_S, BrushSpec, Dab, Dynamics, Falloff, FalloffPoint,
    MAX_FALLOFF_POINTS, MAX_INPUT_SAMPLES, Stroke, StrokeMethod, StrokePoint, eval_falloff_curve,
    stamp_dab,
};

/// Brush + Stroke-section parameter snapshot & setters (submodule so it shares `PaintState`'s
/// private brush access; split out to keep both files under the workspace LOC cap).
mod brush_settings;

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

/// A compact snapshot of the active brush for the layers panel's Brush section.
/// Published each frame by the shell bridge (mirror of the `LayerStack`
/// snapshot) — the panel reads it to position the size/colour sliders and the
/// blend chip; it never owns brush state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSettings {
    /// Radius in image pixels (UI label "Size").
    pub size_px: f32,
    /// [`Self::size_px`] mapped onto the size slider's `0..1` track (squared, so
    /// small brushes get more of the track).
    pub size_norm: f32,
    /// Overall opacity, `0..1` (UI "Strength").
    pub strength: f32,
    /// Distance-falloff preset wire discriminant ([`Falloff::to_u8`]) — Blender's
    /// "Falloff Curve Preset". Defines the dab profile (replaces a Hardness slider).
    /// [`Falloff::Custom`] (`9`) reads [`Self::falloff_points`].
    pub falloff: u8,
    /// The `Custom` falloff curve's control points (id + `[distance, strength]` +
    /// handle), the first [`Self::falloff_len`] valid, ascending by distance. The
    /// panel plots these + places draggable handles (keyed by the stable id) when
    /// `falloff == Custom`.
    pub falloff_points: [FalloffPoint; MAX_FALLOFF_POINTS],
    /// Count of valid entries in [`Self::falloff_points`] (`2..=MAX_FALLOFF_POINTS`).
    pub falloff_len: u8,
    /// Straight-RGB paint colour in `[0, 1]`.
    pub color: [f32; 3],
    /// Blend-mode wire discriminant ([`BrushBlend::to_u8`]).
    pub blend: u8,
    /// Eraser mode — paints with Erase Alpha regardless of [`Self::blend`].
    pub eraser: bool,

    // ── Stroke section (raw values; the panel maps to slider tracks via the BRUSH_*_MAX consts) ──
    /// Stroke-method wire discriminant ([`StrokeMethod::to_u8`]).
    pub stroke_method: u8,
    /// Spacing as a fraction of diameter (`0.10` = 10%); the slider track is this value.
    pub spacing: f32,
    /// "Adjust Strength for Spacing" on/off.
    pub space_attenuation: bool,
    /// Relative jitter (`0..1`, fraction of diameter) — the Jitter slider under the Brush unit.
    pub jitter: f32,
    /// Absolute jitter in pixels — the Jitter slider under the View unit.
    pub jitter_absolute_px: f32,
    /// Jitter-unit wire discriminant ([`JitterUnit::to_u8`]; `0` = Brush, `1` = View).
    pub jitter_unit: u8,
    /// Dash on-fraction (`0..1`).
    pub dash_ratio: f32,
    /// Dash period in dab-slots.
    pub dash_samples: u32,
    /// Input-samples averaging window (`>= 1`).
    pub input_samples: u32,
    /// Stroke stabilizer intensity, `0..1` (the "how regular" knob).
    pub stabilizer: f32,
    /// Airbrush emission period in seconds (the "Rate" slider; only meaningful for the Airbrush
    /// method). The panel maps it onto the slider track via `BRUSH_AIRBRUSH_RATE_{MIN,MAX}_S`.
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" toggle — Anchored only (the stamp spans anchor→cursor instead of growing
    /// from the anchor).
    pub edge_to_edge: bool,
}

/// Strength of the brush's active falloff at normalized distance `t` (`0` =
/// centre, `1` = rim), for the panel's live curve preview. Reads the editable
/// [`BrushSettings::falloff_points`] when the `Custom` preset is selected, else
/// the matching [`Falloff`] formula — so the graph the panel draws matches the
/// dab the engine stamps.
#[must_use]
pub fn brush_falloff_weight_at(s: &BrushSettings, t: f32) -> f32 {
    if s.falloff == Falloff::Custom.to_u8() {
        eval_falloff_curve(&s.falloff_points[..s.falloff_len as usize], t)
    } else {
        Falloff::from_u8(s.falloff).weight(t)
    }
}

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
            stroke_undo: None,
            eraser: false,
            moved_this_frame: false,
            drag_preview: None,
            line_anchor: None,
            line_constrain: false,
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
                snap_to_45(anchor, ev.pos)
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
    /// - **Stabilizer catch-up:** when the pointer is parked, walk the lagged (smoothed) path up to
    ///   the cursor so a high-stabilizer stroke arrives without waiting for pointer-up. Only while
    ///   parked, so during-movement smoothing keeps full strength (`settle` is Space-only).
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

    /// Stamp a batch of dabs into `canvas_rgba`, accumulate the dirty rect, and flag
    /// the preview dirty. Each dab carries its own pressure-scaled radius + coverage;
    /// the static brush appearance (falloff / hardness / blend / colour) comes from
    /// `self.paint.brush`.
    fn stamp_dabs(&mut self, dabs: &[Dab]) {
        if dabs.is_empty() {
            return;
        }
        let (w, h) = self.source_size;
        let mut brush = self.paint.brush;
        // Eraser mode overrides the blend with Erase Alpha (removes coverage from
        // the layer's alpha); the drawing blend in `brush.blend` is untouched.
        if self.paint.eraser {
            brush.blend = ph2d_painter_brush::BrushBlend::EraseAlpha;
        }
        // Alpha lock ("Lock"/Preserve Transparency): the dab paints only into the
        // active layer's existing alpha. (Clip + mask are composite-time effects the
        // compositor already honours; only alpha-lock constrains the dab itself.)
        let alpha_locked = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..brush
            };
            if let Some(r) = stamp_dab(buf, w, h, d.center, &spec, d.coverage, alpha_locked) {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
        }
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

    /// Stamp an interactive preview batch (Drag Dot / Anchored = 1 dab, Line = N): erase the
    /// previous footprint (restore its saved pixels), then save the pristine pixels under the new
    /// dabs' UNION bbox and stamp them there — so the moving / resizing / growing preview leaves no
    /// trail. [`Self::commit_drag_preview`] keeps the last batch on pen-up.
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

/// Snap `cursor` to the nearest 45° ray from `anchor` (Blender Line Alt-constrain), projecting the
/// cursor onto that ray. Transcendental-free (only abs/signum/compare/mul — `tan(22.5°)≈0.4142`,
/// `tan(67.5°)≈2.4142` are the sector boundaries, `√½` the diagonal unit), so it stays
/// bit-deterministic across platforms (HR-5) instead of going through `atan2`/`sin`/`cos`.
fn snap_to_45(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    const TAN_22_5: f32 = 0.414_213_56; // tan(22.5°)
    const TAN_67_5: f32 = 2.414_213_5; // tan(67.5°)
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2; // √½
    let dx = cursor[0] - anchor[0];
    let dy = cursor[1] - anchor[1];
    let (adx, ady) = (dx.abs(), dy.abs());
    // The snapped unit direction (one of the 8 rays).
    let (ux, uy) = if ady <= adx * TAN_22_5 {
        (dx.signum(), 0.0) // horizontal
    } else if ady >= adx * TAN_67_5 {
        (0.0, dy.signum()) // vertical
    } else {
        (dx.signum() * DIAG, dy.signum() * DIAG) // diagonal
    };
    // Project the cursor onto the ray (dot product = signed distance along the unit direction).
    let proj = dx * ux + dy * uy;
    [anchor[0] + ux * proj, anchor[1] + uy * proj]
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
            // finalize any half-open stroke (records its undo) before bailing.
            self.close_stroke();
            return false;
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
