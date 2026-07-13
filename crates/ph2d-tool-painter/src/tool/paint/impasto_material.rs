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
    /// Re-bake the LAST stroke's MATERIAL from the brush — the four material knobs, live on the stroke
    /// the artist is looking at, exactly like Depth and Body.
    ///
    /// It re-merges from the stored BASE rather than merging again on top of what is there: `over` does
    /// not compose (merging twice at 50% leaves 75% of the new material, not 50%), so a re-bake that
    /// skipped the base would creep toward the brush every time the artist nudged a slider.
    pub(super) fn rebake_live_material(&mut self) {
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
