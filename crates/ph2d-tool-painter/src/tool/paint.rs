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
use ph2d_painter_brush::{stamp_dab, BrushSpec, Dab, Dynamics, Stroke, StrokePoint};

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

    /// Begin a stroke at `ev` and stamp the first dab.
    fn paint_begin(&mut self, ev: CanvasPointer) {
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

    /// Finish the stroke at `ev` (stamp the final segment, then drop the stroke).
    fn paint_end(&mut self, ev: CanvasPointer) {
        if self.paint_extend(ev) {
            self.paint.stroke = None;
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
        let brush = self.paint.brush;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let spec = BrushSpec { radius_px: d.radius_px, ..brush };
            if let Some(r) = stamp_dab(buf, w, h, d.center, &spec, d.coverage) {
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

impl CanvasPaintTool for PainterTool {
    fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase == PointerPhase::Hover {
            return false; // hover is cursor/preview only
        }
        if !self.paint_target_ready() {
            // Active layer isn't paintable (mask/group/adjustment) or no canvas:
            // drop any half-open stroke so a later valid Down starts clean.
            self.paint.stroke = None;
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
}
