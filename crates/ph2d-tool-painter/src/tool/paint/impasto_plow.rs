//! **Impasto — the palette knife.** The Smear route dragging the relief along with the colour.
//!
//! Sibling of [`super::impasto`] (the deposit's plumbing) and [`super::impasto_settle`] (the material's
//! behaviour), split for the workspace file-LOC cap. `impl PainterTool`, over the same private
//! `PaintState` fields, via `super::*`.

use super::impasto_settle::owned;
use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::height::plow_dab_height;
use ph2d_painter_brush::{BrushSpec, Dab};

impl PainterTool {
    /// **Plow** — the Smear route drags the relief along with the colour (the palette knife).
    ///
    /// Called from the smear route with the SAME dab list and the SAME `from`/`to` chain the colour
    /// uses, so pigment and body move as one thing. The knife *displaces*; it never deposits — so it
    /// needs no Depth, and there is nothing to commit: it edits the layer's relief in place, exactly as
    /// the colour smear edits the layer's pixels in place.
    ///
    /// No-op unless the layer HAS relief. That is what makes this free for everyone else: a painter who
    /// never touched Impasto smears through here and pays a map lookup.
    pub(super) fn plow_dabs(&mut self, dabs: &[Dab], brush: &BrushSpec, strength: f32) {
        if dabs.is_empty() || !brush.impasto_plow_active() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(active) = self.layers.active() else {
            return;
        };
        // Nothing sculpted here ⇒ nothing to plow. (And a stale, differently-sized field is dropped
        // rather than indexed into — the shape guard the 2026-07-12 sweep taught us to write.)
        let Some(mut field) = self
            .heights
            .remove(&active)
            .map(owned)
            .filter(|f| f.len() == n)
        else {
            return;
        };
        let mut cover = self
            .covers
            .remove(&active)
            .map(owned)
            .filter(|c| c.len() == n)
            .unwrap_or_else(|| vec![0u8; n]);

        // The smear's own displacement chain — the same `last_smear_pos` the colour route advances, so
        // the two cannot drift apart. (Reading it here rather than keeping a second chain is the whole
        // reason a plowed ridge stays under its own paint.)
        let mut from = self.paint.last_smear_pos;
        let mut touched: Option<Region> = None;
        for d in dabs {
            if let Some(prev) = from {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..*brush
                };
                let amount = strength * d.coverage;
                if let Some(r) = plow_dab_height(
                    &mut field,
                    &mut cover,
                    w,
                    h,
                    &spec,
                    prev,
                    d.center,
                    d.radius_px,
                    amount,
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
            from = Some(d.center);
        }
        // The plow rewrites the committed relief, so a live stroke's ground (which is the relief BEFORE
        // that stroke) is no longer what it says it is — drop it rather than let a later Depth drag
        // resurrect the un-plowed ridge.
        self.drop_live_relief();
        self.heights.insert(active, std::sync::Arc::new(field));
        self.covers.insert(active, std::sync::Arc::new(cover));
        self.sync_relief_flags();
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
