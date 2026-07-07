//! Watercolor **Smudge** — the TRUE SMEAR: each dab physically DRAGS the frozen base's paint along the
//! stroke (Krita Color Smudge "Smearing" / Blender `paint_2d_lift_smear`), and the optical composite
//! renders the wash over the smeared base, so already-painted paint MOVES ("borrar", Enio 2026-07-06).
//! The wet-on-wet dissolve/lift lives per-pixel in the composite instead ([`super::watercolor_render`],
//! `wet_rewet`) — a colour-carry reservoir (Krita "Dulling" / Photoshop Mixer-Brush wells) was tried and
//! retired: cadence-bound, self-feeding, and perceptually weak (Enio 2026-07-06).
//! Split from `watercolor_render.rs` for the workspace LOC cap.

use super::*;

impl PainterTool {
    /// Drag the base's paint along the dab chain: each dab lifts the base under the PREVIOUS dab
    /// centre and stamps it at its own, weight `wet_smudge × coverage` through the brush falloff.
    /// Blank paper smears nothing (canonical: a smudge brush without paint under it barely paints).
    ///
    /// The base is per-stroke and baked by the pen-up composite, so undo restores it whole. Chained by
    /// [`PaintState::wet_smear_pos`](super::PaintState) across batches; each smear's write rect is folded
    /// into the dirty tracking (belt-and-braces — it stays inside the dab bbox the accumulate tracked).
    /// Cumulative stroke methods only: the re-stamp previews (Drag Dot / Anchored / Line) would re-smear
    /// the base every frame, accumulating mutations the coverage clear can't undo.
    pub(super) fn smear_wet_base(&mut self, dabs: &[Dab]) {
        let smudge = self.paint.brush.wet_smudge.clamp(0.0, 1.0);
        let (w, h) = self.source_size;
        let Some(base_arc) = self.paint.watercolor_base.as_mut() else {
            return;
        };
        if smudge <= 0.0 || w == 0 || h == 0 || base_arc.len() != (w as usize * h as usize * 4) {
            return;
        }
        let spec = self.paint.brush;
        let buf = Arc::make_mut(base_arc);
        let mut from = self.paint.wet_smear_pos;
        let mut touched: Option<Region> = None;
        for d in dabs {
            if let Some(prev) = from
                && let Some(r) = ph2d_painter_brush::smear_dab(
                    buf,
                    w,
                    h,
                    prev,
                    d.center,
                    &BrushSpec {
                        radius_px: d.radius_px,
                        ..spec
                    },
                    smudge * d.coverage,
                    [false, false],
                )
            {
                let rect = Region {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: r.h,
                };
                touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
            }
            from = Some(d.center);
        }
        self.paint.wet_smear_pos = from;
        if let Some(rect) = touched {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, rect),
                None => rect,
            });
            self.paint.wet_cum_dirty = Some(match self.paint.wet_cum_dirty {
                Some(c) => union_region(c, rect),
                None => rect,
            });
        }
    }
}
