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
        if smudge <= 0.0 || w == 0 || h == 0 {
            return;
        }
        // Selection + protection gates: the smear mutates the FROZEN BASE, which the composite shows
        // verbatim wherever the wash doesn't cover — so an ungated smear leaked dragged paint outside
        // the selection / into the protected region. Mirror of the canvas stamp gate
        // (`stamp_dabs`): snapshot the smear footprint of the BASE, smear, then lerp the gated texels
        // back by `keep` (`splat_keep` — the same weights `restore_deselected_region` /
        // `restore_protected_region` apply to the canvas). Ungated (the default) ⇒ zero-cost path.
        let (sel, prot, alock) = self.wet_splat_gates();
        let gate_region = (sel.is_some() || prot.is_some() || alock.is_some())
            .then(|| Self::smear_footprint(dabs, self.paint.wet_smear_pos, self.source_size))
            .flatten();
        // EDGE-1 wet session: the smear must mutate the base THE COMPOSITE READS — the session
        // base when a wet session is live (falling back to the per-stroke base). On the session's
        // first stroke both fields share one Arc, so `make_mut` forks them: re-share afterwards
        // (below) to keep the mixer pickup reading the smeared paint, the pre-session behaviour.
        let shared = match (&self.paint.wet_session_base, &self.paint.watercolor_base) {
            (Some(s), Some(w)) => Arc::ptr_eq(s, w),
            _ => false,
        };
        let Some(base_arc) = self
            .paint
            .wet_session_base
            .as_mut()
            .or(self.paint.watercolor_base.as_mut())
        else {
            return;
        };
        if base_arc.len() != (w as usize * h as usize * 4) {
            return;
        }
        let spec = self.paint.brush;
        let buf = Arc::make_mut(base_arc);
        // Pre-smear snapshot of the gated footprint (base bytes, row-major within the region).
        let before: Option<Vec<u8>> = gate_region.map(|r| {
            let mut out = Vec::with_capacity((r.w * r.h * 4) as usize);
            for ry in 0..r.h {
                let s = (((r.y + ry) * w + r.x) * 4) as usize;
                out.extend_from_slice(&buf[s..s + (r.w * 4) as usize]);
            }
            out
        });
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
        // Restore the gated texels of the base (keep-lerp — the exact restore semantics of the
        // canvas-side gates), so the smear never lands outside the selection / on protected texels.
        if let (Some(r), Some(orig)) = (gate_region, before.as_ref()) {
            for ry in 0..r.h {
                for rx in 0..r.w {
                    let gidx = ((r.y + ry) * w + (r.x + rx)) as usize;
                    let keep = super::watercolor_accum::splat_keep(
                        sel.as_deref().map(Vec::as_slice),
                        prot.as_deref().map(Vec::as_slice),
                        alock.as_deref().map(Vec::as_slice),
                        gidx,
                    );
                    if keep >= 1.0 {
                        continue;
                    }
                    let b = gidx * 4;
                    let s = ((ry * r.w + rx) * 4) as usize;
                    for c in 0..4 {
                        let smeared = f32::from(buf[b + c]);
                        let o = f32::from(orig[s + c]);
                        buf[b + c] = (smeared * keep + o * (1.0 - keep))
                            .round()
                            .clamp(0.0, 255.0) as u8;
                    }
                }
            }
        }
        if shared {
            self.paint.watercolor_base = self.paint.wet_session_base.clone();
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
            self.paint.wet_stroke_dirty = Some(match self.paint.wet_stroke_dirty {
                Some(c) => union_region(c, rect),
                None => rect,
            });
        }
    }

    /// The base-buffer footprint a smear batch can touch: the union of each dab's disc (± radius + 1 px)
    /// plus the chain's previous position (the lift source), clamped to the canvas. `None` when empty /
    /// fully off-canvas. Mirror of [`Self::dab_batch_region`] but keyed on the WET smear chain
    /// (`wet_smear_pos`), not the plain-Smear one.
    fn smear_footprint(dabs: &[Dab], prev: Option<[f32; 2]>, (w, h): (u32, u32)) -> Option<Region> {
        if w == 0 || h == 0 || dabs.is_empty() {
            return None;
        }
        let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut max_r = 0.0_f32;
        for d in dabs {
            let r = d.radius_px + 1.0;
            max_r = max_r.max(r);
            minx = minx.min(d.center[0] - r);
            maxx = maxx.max(d.center[0] + r);
            miny = miny.min(d.center[1] - r);
            maxy = maxy.max(d.center[1] + r);
        }
        if let Some(p) = prev {
            minx = minx.min(p[0] - max_r);
            maxx = maxx.max(p[0] + max_r);
            miny = miny.min(p[1] - max_r);
            maxy = maxy.max(p[1] + max_r);
        }
        let x0 = minx.floor().max(0.0) as u32;
        let y0 = miny.floor().max(0.0) as u32;
        let x1 = (maxx.ceil() as i64).clamp(0, i64::from(w)) as u32;
        let y1 = (maxy.ceil() as i64).clamp(0, i64::from(h)) as u32;
        (x1 > x0 && y1 > y0).then_some(Region {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        })
    }
}
