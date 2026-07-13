//! **Impasto** — the paint's MATERIAL, on the canvas.
//!
//! Split out of `impasto.rs` (the workspace LOC cap) and cohesive on its own: everything that puts the
//! four material bytes (`ph2d_painter_brush::material`) onto the layer, and keeps them live on the
//! stroke the artist is looking at.
//!
//! The material is the *third* output of the same dab list — the relief being the second — and it is
//! deposited by the very same commit. That is the rule the whole section is built on: a pass with
//! geometry of its own is how "Tiling does not work under Impasto" gets born.

use super::impasto_settle::for_each_in;
use crate::tool::PainterTool;

impl PainterTool {
    /// Whether a knob edit reaches the paint **already on the canvas** — the "Adjust Last Stroke"
    /// checkbox (Enio, 2026-07-13).
    ///
    /// ON (the default, and how the section has always behaved): the artist lays a stroke and dials it
    /// in *while looking at it* — every knob in the Body and Material cards re-derives it, because the
    /// stroke stored its INGREDIENTS rather than its result. OFF: the sliders speak only to the strokes
    /// still to come, because the paint on the canvas is FINISHED — which is what an artist wants the
    /// moment they are happy with it and start setting up the next one.
    ///
    /// The two re-derivers (`refresh_live_relief` for the geometry, `rebake_live_material` for the
    /// material) both ask HERE, rather than each testing the flag: a toggle honoured by one of two
    /// choke points is a toggle that half-works, and that is a bug report waiting to be written.
    ///
    /// Unticking does NOT drop the stroke's ingredients — they stay, so ticking it back on makes the
    /// next edit reach the stroke again, in full. A checkbox that quietly discards work is not a
    /// checkbox (`adjust_last_stroke_does_not_destroy_the_strokes_ingredients`).
    pub(super) fn impasto_live_edit(&self) -> bool {
        self.paint.impasto_live_edit
    }

    /// Re-bake the LAST stroke's MATERIAL from the brush — the four material knobs, live on the stroke
    /// the artist is looking at, exactly like Depth and Body.
    ///
    /// It re-merges from the stored BASE rather than merging again on top of what is there: `over` does
    /// not compose (merging twice at 50% leaves 75% of the new material, not 50%), so a re-bake that
    /// skipped the base would creep toward the brush every time the artist nudged a slider.
    pub(super) fn rebake_live_material(&mut self) {
        if !self.impasto_live_edit() {
            return; // finished paint stays finished — the knob speaks to the NEXT stroke
        }
        self.invalidate_composite();
        let (Some(layer), Some(rect)) = (
            self.paint.relief.live_relief_layer,
            self.paint.relief.live_relief_rect,
        ) else {
            return; // nothing live: the knob speaks to the NEXT stroke, which needs no re-bake
        };
        if self.layers.active() != Some(layer) {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let base = std::mem::take(&mut self.paint.relief.live_mat_base);
        let film = std::mem::take(&mut self.paint.relief.live_film);
        let want = (rect.w as usize) * (rect.h as usize);
        if base.len() != want || film.len() != want || n == 0 {
            self.paint.relief.live_mat_base = base;
            self.paint.relief.live_film = film;
            return; // a stale patch is dropped, never indexed into (the shape guard, as everywhere here)
        }
        let mat = self.paint.brush.material().to_bytes();
        let neutral = ph2d_painter_brush::material::Material::NEUTRAL.to_bytes();
        {
            let dst = std::sync::Arc::make_mut(self.mats.entry(layer).or_default());
            if dst.len() != n {
                dst.resize(n, neutral);
            }
            let mut k = 0usize;
            for_each_in(rect, w, |i| {
                let a = u32::from(film[k]);
                let b = base[k];
                k += 1;
                if a == 0 {
                    dst[i] = b;
                    return;
                }
                for c in 0..4 {
                    let old = u32::from(b[c]);
                    let new = u32::from(mat[c]);
                    dst[i][c] = ((old * (255 - a) + new * a + 127) / 255) as u8;
                }
            });
        }
        self.paint.relief.live_mat_base = base;
        self.paint.relief.live_film = film;
    }
}
