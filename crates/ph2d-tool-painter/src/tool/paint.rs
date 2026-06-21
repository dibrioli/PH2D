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
    BrushBlend, BrushSpec, Dab, Dynamics, Falloff, FalloffPoint, HandleType, JitterUnit,
    MAX_FALLOFF_POINTS, MAX_INPUT_SAMPLES, Stroke, StrokeMethod, StrokePoint, eval_falloff_curve,
    stamp_dab,
};

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
/// Max stabilizer dead-zone radius the slider reaches, in pixels.
pub const BRUSH_SMOOTH_RADIUS_MAX_PX: f32 = 200.0;
/// Max value for the Input Samples / Dash Length count sliders (mirrors the engine's
/// input-sample window cap).
pub const BRUSH_COUNT_SLIDER_MAX: u32 = MAX_INPUT_SAMPLES as u32;

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
    /// "Stabilize Stroke" (smooth-stroke) on/off.
    pub smooth_stroke: bool,
    /// Stabilizer dead-zone radius in pixels.
    pub smooth_radius_px: f32,
    /// Stabilizer lag (`0..1`).
    pub smooth_factor: f32,
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

/// Map a radius in pixels onto the size slider's `0..1` track (inverse of
/// [`size_norm_to_px`]). Squared track → finer control at small sizes.
fn size_px_to_norm(px: f32) -> f32 {
    let span = BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX;
    ((px - BRUSH_SIZE_MIN_PX) / span).clamp(0.0, 1.0).sqrt()
}

/// Map the size slider's `0..1` track onto a radius in pixels.
fn size_norm_to_px(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    BRUSH_SIZE_MIN_PX + t * t * (BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX)
}

/// Brush settings + in-progress stroke state held by the [`PainterTool`].
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
        self.stamp_dabs(&dabs);
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
    }

    /// Extend the in-progress stroke to `ev`, stamping any dabs the spacing emits.
    /// Returns `false` if no stroke is active (a stray Move).
    fn paint_extend(&mut self, ev: CanvasPointer) -> bool {
        let Some(mut stroke) = self.paint.stroke.take() else {
            return false;
        };
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.extend(
            StrokePoint {
                pos: ev.pos,
                pressure: ev.pressure,
            },
            &mut dabs,
        );
        self.stamp_dabs(&dabs);
        self.paint.dabs = dabs;
        self.paint.stroke = Some(stroke);
        true
    }

    /// Finish the stroke at `ev` (stamp the final segment, then close + record undo).
    fn paint_end(&mut self, ev: CanvasPointer) {
        self.paint_extend(ev);
        self.close_stroke();
    }

    /// Finalize the current stroke: drop the in-progress state and push one undo
    /// entry (pre-stroke → current) so the whole stroke undoes/redoes as a unit.
    /// No-op when no stroke is open. Reuses the structural-undo stack (a full-canvas
    /// snapshot per stroke; a tile-based delta is a later optimization).
    fn close_stroke(&mut self) {
        self.paint.stroke = None;
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
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
            self.dirty_rect = Some(self.dirty_rect.map_or(rect, |acc| union_region(acc, rect)));
            self.preview_dirty = true;
            let active = self.layers.active();
            self.bump_layer_pixels(active);
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
impl PainterTool {
    /// Snapshot the active brush for the panel's Brush section.
    #[must_use]
    pub fn brush_settings(&self) -> BrushSettings {
        let b = &self.paint.brush;
        // Snapshot the Custom curve's control points into the Copy array (the
        // panel reads these to plot + place handles when the Custom preset is on).
        let mut falloff_points = [FalloffPoint::default(); MAX_FALLOFF_POINTS];
        let pts = b.custom_falloff.points();
        falloff_points[..pts.len()].copy_from_slice(pts);
        BrushSettings {
            size_px: b.radius_px,
            size_norm: size_px_to_norm(b.radius_px),
            strength: b.strength,
            falloff: b.falloff.to_u8(),
            falloff_points,
            falloff_len: b.custom_falloff.len() as u8,
            color: b.color,
            blend: b.blend.to_u8(),
            eraser: self.paint.eraser,
            stroke_method: b.stroke_method.to_u8(),
            spacing: b.spacing,
            space_attenuation: b.space_attenuation,
            jitter: b.jitter,
            jitter_absolute_px: b.jitter_absolute_px,
            jitter_unit: b.jitter_unit.to_u8(),
            dash_ratio: b.dash_ratio,
            dash_samples: b.dash_samples,
            input_samples: b.input_samples,
            smooth_stroke: b.smooth_stroke,
            smooth_radius_px: b.smooth_radius_px,
            smooth_factor: b.smooth_factor,
        }
    }

    /// Set the brush distance-falloff preset from a wire discriminant
    /// (Blender's "Falloff Curve Preset"; out-of-range → Smooth). `9` = the
    /// editable `Custom` curve ([`Self::set_brush_falloff_point`]).
    pub fn set_brush_falloff(&mut self, preset: u8) {
        self.paint.brush.falloff = Falloff::from_u8(preset);
    }

    /// Move `Custom` falloff control point `id` to `(distance, strength)` in
    /// `[0, 1]²`. The point may pass its neighbours (the curve re-sorts and
    /// adapts); the stable `id` keeps the dragged handle grabbed. Pure brush
    /// state — no undo/preview (a brush param change only affects future dabs).
    pub fn set_brush_falloff_point(&mut self, id: u8, distance: f32, strength: f32) {
        self.paint
            .brush
            .custom_falloff
            .set_point(id, distance, strength);
    }

    /// Insert a `Custom` falloff control point at the widest gap (its strength
    /// sampled on the current curve). Returns the new stable id, or `None` at the
    /// point cap. Drives the panel's "+" button.
    pub fn add_brush_falloff_point(&mut self) -> Option<u8> {
        self.paint.brush.custom_falloff.add_point()
    }

    /// Insert a `Custom` falloff control point at `(distance, strength)` — where
    /// the artist clicked on the curve canvas. Returns the new stable id, or
    /// `None` at the point cap.
    pub fn add_brush_falloff_point_at(&mut self, distance: f32, strength: f32) -> Option<u8> {
        self.paint
            .brush
            .custom_falloff
            .add_point_at(distance, strength)
    }

    /// Set the handle type of `Custom` falloff control point `id` (`0` = Auto,
    /// `1` = Vector). Drives the right-click handle menu.
    pub fn set_brush_falloff_point_handle(&mut self, id: u8, handle: u8) {
        self.paint
            .brush
            .custom_falloff
            .set_handle(id, HandleType::from_u8(handle));
    }

    /// Remove `Custom` falloff control point `id` (no-op when only the two
    /// endpoints remain). Drives the panel's "−" button + the Delete key.
    pub fn remove_brush_falloff_point(&mut self, id: u8) {
        self.paint.brush.custom_falloff.remove_point(id);
    }

    /// Set the brush strength (`0..1`, overall opacity).
    pub fn set_brush_strength(&mut self, t: f32) {
        self.paint.brush.strength = t.clamp(0.0, 1.0);
    }

    /// Toggle eraser mode (overrides the blend with Erase Alpha while on).
    pub fn toggle_brush_eraser(&mut self) {
        self.paint.eraser = !self.paint.eraser;
    }

    /// Set the brush radius in pixels, clamped to the interactive size range.
    pub fn set_brush_size_px(&mut self, px: f32) {
        self.paint.brush.radius_px = px.clamp(BRUSH_SIZE_MIN_PX, BRUSH_SIZE_MAX_PX);
    }

    /// Set the brush radius from the size slider's `0..1` track.
    pub fn set_brush_size_norm(&mut self, t: f32) {
        self.set_brush_size_px(size_norm_to_px(t));
    }

    /// Nudge the brush radius by one step — `[` (`dir < 0`) / `]` (`dir >= 0`).
    /// Multiplicative for a constant *perceptual* step, with a ±1 px floor so the
    /// smallest brushes still change. Returns the new radius in pixels.
    pub fn nudge_brush_size(&mut self, dir: i32) -> f32 {
        const STEP: f32 = 1.15;
        let cur = self.paint.brush.radius_px;
        let next = if dir >= 0 {
            (cur * STEP).max(cur + 1.0)
        } else {
            (cur / STEP).min(cur - 1.0)
        };
        self.set_brush_size_px(next);
        self.paint.brush.radius_px
    }

    /// Set one straight-RGB colour channel (`0..3`) of the brush, clamped `0..1`.
    pub fn set_brush_color_channel(&mut self, ch: usize, v: f32) {
        if ch < 3 {
            self.paint.brush.color[ch] = v.clamp(0.0, 1.0);
        }
    }

    /// Set the brush blend mode from a wire discriminant (out-of-range → Mix).
    pub fn set_brush_blend(&mut self, mode: u8) {
        self.paint.brush.blend = BrushBlend::from_u8(mode);
    }

    // ── Stroke section setters (the single clamp source; the panel forwards raw UI values) ──

    /// Set the stroke method from a wire discriminant (out-of-range → Space).
    pub fn set_brush_stroke_method(&mut self, m: u8) {
        self.paint.brush.stroke_method = StrokeMethod::from_u8(m);
    }

    /// Set spacing as a fraction of diameter (slider track), clamped to the interactive range.
    pub fn set_brush_spacing(&mut self, frac: f32) {
        self.paint.brush.spacing = frac.clamp(0.01, BRUSH_SPACING_MAX);
    }

    /// Toggle "Adjust Strength for Spacing".
    pub fn toggle_brush_space_attenuation(&mut self) {
        self.paint.brush.space_attenuation = !self.paint.brush.space_attenuation;
    }

    /// Set the Jitter slider (`0..1` track), routed by the current unit: `Brush` → relative jitter
    /// (`0..1`), `View` → absolute pixels (`track × BRUSH_JITTER_ABS_MAX_PX`).
    pub fn set_brush_jitter_norm(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match self.paint.brush.jitter_unit {
            JitterUnit::View => self.paint.brush.jitter_absolute_px = t * BRUSH_JITTER_ABS_MAX_PX,
            JitterUnit::Brush => self.paint.brush.jitter = t,
        }
    }

    /// Set the jitter unit from a wire discriminant (out-of-range → Brush).
    pub fn set_brush_jitter_unit(&mut self, u: u8) {
        self.paint.brush.jitter_unit = JitterUnit::from_u8(u);
    }

    /// Set the dash on-fraction (`0..1`).
    pub fn set_brush_dash_ratio(&mut self, t: f32) {
        self.paint.brush.dash_ratio = t.clamp(0.0, 1.0);
    }

    /// Set the dash period from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX` slots.
    pub fn set_brush_dash_length_norm(&mut self, t: f32) {
        self.paint.brush.dash_samples = count_from_norm(t);
    }

    /// Set the input-samples window from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX`.
    pub fn set_brush_input_samples_norm(&mut self, t: f32) {
        self.paint.brush.input_samples = count_from_norm(t);
    }

    /// Toggle "Stabilize Stroke" (smooth-stroke).
    pub fn toggle_brush_smooth_stroke(&mut self) {
        self.paint.brush.smooth_stroke = !self.paint.brush.smooth_stroke;
    }

    /// Set the stabilizer dead-zone radius from the slider's `0..1` track → `0..MAX` px.
    pub fn set_brush_smooth_radius_norm(&mut self, t: f32) {
        self.paint.brush.smooth_radius_px = t.clamp(0.0, 1.0) * BRUSH_SMOOTH_RADIUS_MAX_PX;
    }

    /// Set the stabilizer lag (`0..1`).
    pub fn set_brush_smooth_factor(&mut self, t: f32) {
        self.paint.brush.smooth_factor = t.clamp(0.0, 1.0);
    }
}

/// Map a slider's `0..1` track onto a count in `1..=BRUSH_COUNT_SLIDER_MAX` (Input Samples /
/// Dash Length). Inverse of `count_to_norm` in the panel.
fn count_from_norm(t: f32) -> u32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    1 + (t.clamp(0.0, 1.0) * span).round() as u32
}

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
#[path = "paint_tests.rs"]
mod tests;
