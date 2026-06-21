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
use ph2d_painter_brush::{stamp_dab, BrushBlend, BrushSpec, Dab, Dynamics, Stroke, StrokePoint};

/// Smallest brush radius the size UI maps to, in image pixels. The size slider's
/// `0..1` track and the `[` / `]` keyboard nudge both clamp here.
pub const BRUSH_SIZE_MIN_PX: f32 = 1.0;
/// Largest brush radius the size UI maps to, in image pixels. (The engine's own
/// allocation cap is higher; this is the interactive range, not a hard limit.)
pub const BRUSH_SIZE_MAX_PX: f32 = 512.0;

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
    /// Edge softness plateau, `0..1` (UI "Hardness"; `1` = hard disk).
    pub hardness: f32,
    /// Per-dab build-up, `0..1` (UI "Flow").
    pub flow: f32,
    /// Overall opacity, `0..1` (UI "Strength").
    pub strength: f32,
    /// Straight-RGB paint colour in `[0, 1]`.
    pub color: [f32; 3],
    /// Blend-mode wire discriminant ([`BrushBlend::to_u8`]).
    pub blend: u8,
    /// Eraser mode — paints with Erase Alpha regardless of [`Self::blend`].
    pub eraser: bool,
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
            brush: BrushSpec { radius_px: 10.0, ..BrushSpec::default() },
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
        stroke.begin(StrokePoint { pos: ev.pos, pressure: ev.pressure }, &mut dabs);
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
        stroke.extend(StrokePoint { pos: ev.pos, pressure: ev.pressure }, &mut dabs);
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
            let spec = BrushSpec { radius_px: d.radius_px, ..brush };
            if let Some(r) = stamp_dab(buf, w, h, d.center, &spec, d.coverage, alpha_locked) {
                let rect = Region { x: r.x, y: r.y, w: r.w, h: r.h };
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
    Region { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// Brush-settings accessors — the Brush section of the layers panel and the
/// `[` / `]` keyboard nudge drive these; the brush is plain state (changing it
/// touches no pixels, so there is no undo entry or preview invalidation here).
impl PainterTool {
    /// Snapshot the active brush for the panel's Brush section.
    #[must_use]
    pub fn brush_settings(&self) -> BrushSettings {
        let b = &self.paint.brush;
        BrushSettings {
            size_px: b.radius_px,
            size_norm: size_px_to_norm(b.radius_px),
            hardness: b.hardness,
            flow: b.flow,
            strength: b.strength,
            color: b.color,
            blend: b.blend.to_u8(),
            eraser: self.paint.eraser,
        }
    }

    /// Set the brush hardness (`0..1`, edge softness).
    pub fn set_brush_hardness(&mut self, t: f32) {
        self.paint.brush.hardness = t.clamp(0.0, 1.0);
    }

    /// Set the brush flow (`0..1`, per-dab build-up).
    pub fn set_brush_flow(&mut self, t: f32) {
        self.paint.brush.flow = t.clamp(0.0, 1.0);
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
mod tests {
    use super::*;
    use ph2d_editor_core::tool::RasterEditTool;
    use ph2d_painter_brush::Falloff;

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer { pos, pressure: 1.0, tilt: [0.0, 0.0], phase }
    }

    /// A `PainterTool` sourced with a white opaque `size`×`size` canvas (one
    /// active raster layer) and a small hard black brush for crisp assertions.
    fn white_canvas(size: u32, radius: f32) -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.paint.brush = BrushSpec {
            radius_px: radius,
            hardness: 1.0, // hard disk → deterministic centre
            falloff: Falloff::Constant,
            color: [0.0, 0.0, 0.0],
            ..Default::default()
        };
        t
    }

    fn px(t: &PainterTool, size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [t.canvas_rgba[i], t.canvas_rgba[i + 1], t.canvas_rgba[i + 2], t.canvas_rgba[i + 3]]
    }

    #[test]
    fn down_paints_into_active_raster_and_marks_dirty() {
        let mut t = white_canvas(64, 6.0);
        assert!(t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down)));
        assert_eq!(px(&t, 64, 32, 32), [0, 0, 0, 255], "centre painted black");
        assert!(t.preview_dirty, "preview flagged dirty");
        assert!(t.dirty_rect.is_some(), "dirty rect accumulated");
        // A far corner is untouched.
        assert_eq!(px(&t, 64, 0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn hover_never_paints() {
        let mut t = white_canvas(32, 4.0);
        let _ = t.take_preview_dirty(); // clear the dirty flag `set_source` raised
        assert!(!t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Hover)));
        assert_eq!(px(&t, 32, 16, 16), [255, 255, 255, 255], "hover left canvas untouched");
        assert!(!t.preview_dirty, "hover did not re-dirty the preview");
    }

    #[test]
    fn stroke_down_move_up_paints_a_line() {
        let mut t = white_canvas(64, 3.0);
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
        // Spacing emits many dabs along the horizontal segment → the midpoint is
        // painted, while a point well off the line stays white.
        assert_eq!(px(&t, 64, 32, 32), [0, 0, 0, 255], "midpoint of the stroke painted");
        assert_eq!(px(&t, 64, 32, 10), [255, 255, 255, 255], "off-line pixel untouched");
        // Stroke ended → no stroke in progress.
        assert!(t.paint.stroke.is_none());
    }

    #[test]
    fn move_without_down_is_ignored() {
        let mut t = white_canvas(32, 4.0);
        assert!(!t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Move)), "stray move");
        assert_eq!(px(&t, 32, 16, 16), [255, 255, 255, 255]);
    }

    #[test]
    fn alpha_lock_blocks_paint_on_transparency() {
        // Canvas: left half opaque white, right half transparent.
        let size = 16u32;
        let mut src = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size / 2 {
                let i = ((y * size + x) * 4) as usize;
                src[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let mut t = PainterTool::default();
        t.set_source(src, size, size);
        t.paint.brush = BrushSpec {
            radius_px: 3.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.0, 0.0, 0.0],
            ..Default::default()
        };
        // Enable alpha lock on the active layer.
        let active = t.layers.active().expect("active layer");
        t.layers.get_mut(active).expect("layer").alpha_locked = true;

        // Paint on the transparent side → blocked (no alpha created).
        t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([12.0, 8.0], PointerPhase::Up));
        assert_eq!(px(&t, size, 12, 8)[3], 0, "alpha-lock blocked paint on transparency");

        // Paint on the opaque side → recoloured, alpha preserved.
        t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Up));
        assert_eq!(px(&t, size, 3, 8), [0, 0, 0, 255], "recoloured the opaque side");
    }

    #[test]
    fn brush_size_norm_round_trips_through_settings() {
        let mut t = PainterTool::default();
        t.set_brush_size_norm(0.5);
        let s = t.brush_settings();
        // Squared track: 0.5 → 1 + 0.25·(512−1) px, and the snapshot maps back.
        assert!((s.size_px - 128.75).abs() < 0.01, "size_px = {}", s.size_px);
        assert!((s.size_norm - 0.5).abs() < 1e-4, "size_norm = {}", s.size_norm);
        // Clamps at the ends.
        t.set_brush_size_norm(2.0);
        assert!((t.brush_settings().size_px - BRUSH_SIZE_MAX_PX).abs() < 0.01);
        t.set_brush_size_norm(-1.0);
        assert!((t.brush_settings().size_px - BRUSH_SIZE_MIN_PX).abs() < 0.01);
    }

    #[test]
    fn nudge_grows_and_shrinks_and_clamps() {
        let mut t = PainterTool::default();
        let start = t.brush_settings().size_px;
        let up = t.nudge_brush_size(1);
        assert!(up > start, "`]` grows ({start} → {up})");
        let down = t.nudge_brush_size(-1);
        assert!(down < up, "`[` shrinks ({up} → {down})");
        // Bracket-down never goes below the floor.
        for _ in 0..200 {
            t.nudge_brush_size(-1);
        }
        assert!((t.brush_settings().size_px - BRUSH_SIZE_MIN_PX).abs() < 0.01);
    }

    #[test]
    fn brush_color_channels_set_and_clamp() {
        let mut t = PainterTool::default();
        t.set_brush_color_channel(0, 0.5);
        t.set_brush_color_channel(1, 2.0); // over → 1
        t.set_brush_color_channel(2, -1.0); // under → 0
        t.set_brush_color_channel(9, 0.7); // out-of-range channel → ignored
        assert_eq!(t.brush_settings().color, [0.5, 1.0, 0.0]);
    }

    #[test]
    fn panel_events_drive_brush_size_colour_blend() {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::{PanelEvent, Tool};

        let mut t = PainterTool::default();
        // Size slider drag (0..1 track).
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_SIZE_SLIDER, 0.5));
        assert!((t.brush_settings().size_px - 128.75).abs() < 0.01);
        // Colour from the shared Blender picker read-back ("r,g,b", 8-bit native).
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_COLOR_THUMB,
            "255,64,0".to_string(),
        ));
        let c = t.brush_settings().color;
        assert!((c[0] - 1.0).abs() < 1e-6 && (c[1] - 64.0 / 255.0).abs() < 1e-6 && c[2] == 0.0);
        // Blend dropdown pick (wire u8 → Multiply == 3).
        t.handle_panel_event(PanelEvent::SelectOption(
            core_ids::PAINTER_BRUSH_BLEND,
            "3".to_string(),
        ));
        assert_eq!(t.brush_settings().blend, 3);
        // The chosen brush colour (255,64,0) + Multiply blend actually drive the
        // next stroke: a hard dab over white → white·colour = the colour itself at
        // full coverage.
        let size = 16u32;
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        t.set_brush_size_px(4.0);
        t.paint.brush.hardness = 1.0; // hard disk → deterministic full coverage
        t.paint.brush.falloff = Falloff::Constant;
        t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([8.0, 8.0], PointerPhase::Up));
        assert_eq!(
            px(&t, size, 8, 8),
            [255, 64, 0, 255],
            "Multiply brush colour over white painted the colour"
        );
    }

    #[test]
    fn panel_events_drive_hardness_flow_strength_and_eraser() {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::{PanelEvent, Tool};

        let mut t = PainterTool::default();
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_HARDNESS_SLIDER, 0.5));
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_FLOW_SLIDER, 0.25));
        t.handle_panel_event(PanelEvent::SetValue(core_ids::PAINTER_BRUSH_STRENGTH_SLIDER, 0.75));
        let s = t.brush_settings();
        assert!((s.hardness - 0.5).abs() < 1e-6, "hardness {}", s.hardness);
        assert!((s.flow - 0.25).abs() < 1e-6, "flow {}", s.flow);
        assert!((s.strength - 0.75).abs() < 1e-6, "strength {}", s.strength);
        assert!(!s.eraser);
        // Eraser toggle via the panel button.
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ERASER));
        assert!(t.brush_settings().eraser, "eraser toggled on");
        t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_ERASER));
        assert!(!t.brush_settings().eraser, "eraser toggled off");
    }

    #[test]
    fn eraser_removes_alpha_from_opaque_pixels() {
        // Opaque white canvas, hard brush; eraser on → a dab clears alpha.
        let mut t = white_canvas(32, 6.0);
        t.toggle_brush_eraser();
        t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));
        assert_eq!(px(&t, 32, 16, 16)[3], 0, "eraser cleared alpha at the centre");
        // A far corner is untouched (still opaque).
        assert_eq!(px(&t, 32, 0, 0)[3], 255);
    }

    #[test]
    fn dock_defaults_to_layers_then_toggles() {
        let mut t = PainterTool::default();
        assert!(t.dock_shows_layers(), "dock opens on the Layers/Effects view");
        t.toggle_dock();
        assert!(!t.dock_shows_layers(), "header toggle flips to the Brush view");
        t.toggle_dock();
        assert!(t.dock_shows_layers(), "toggling back returns to Layers");
    }

    #[test]
    fn stroke_is_one_undo_step_and_redoable() {
        let mut t = white_canvas(64, 6.0);
        let pristine = Vec::clone(&t.canvas_rgba); // white, pre-stroke
        assert!(!t.can_undo(), "fresh source has nothing to undo");

        // One stroke (down → up).
        t.on_canvas_pointer(cp([32.0, 32.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Move));
        t.on_canvas_pointer(cp([40.0, 40.0], PointerPhase::Up));
        assert_ne!(*t.canvas_rgba, pristine, "stroke changed pixels");
        assert!(t.can_undo(), "stroke pushed exactly one undo step");

        // Undo restores the pre-stroke pixels byte-for-byte.
        assert!(t.undo_last());
        assert_eq!(*t.canvas_rgba, pristine, "undo restored the canvas");
        assert!(!t.can_undo(), "one stroke == one undo step");

        // Redo repaints.
        assert!(t.redo_last());
        assert_ne!(*t.canvas_rgba, pristine, "redo repainted the stroke");
        assert_eq!(px(&t, 64, 32, 32), [0, 0, 0, 255], "stroke start back to black");
    }
}
