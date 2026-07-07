//! Watercolor **stroke buffers** — the per-stroke coverage mask + deposited-colour accumulation the
//! optical composite ([`super::watercolor_render`]) reconstructs the wash from, plus their incremental
//! dirty-rect tracking (wet_edges `stampCoverage`/`stampColor`/`expand`). Split from
//! `watercolor_render.rs` for the workspace LOC cap.

use super::*;

/// The soft-disc weight at normalised radius `dn ∈ [0, 1]`: `1.0` at the centre, `0.92` at `0.62`,
/// `0` at the rim (the two-segment linear gradient of `stampCoverage` / `stampColor`). Shared with
/// [`super::watercolor_mixer`], whose disc pickup uses the same weighting as the deposit.
pub(super) fn feather(dn: f32) -> f32 {
    if dn <= 0.62 {
        1.0 - dn / 0.62 * 0.08 // 1.00 → 0.92 across the plateau
    } else {
        0.92 * (1.0 - (dn - 0.62) / 0.38) // 0.92 → 0 across the rim
    }
}

impl PainterTool {
    /// Zero the per-stroke coverage (retain capacity). Shape-preview methods (Drag Dot / Anchored / Line)
    /// rebuild coverage each frame from the current batch, so the moving preview leaves no trail.
    ///
    /// The cleared shape's footprint is folded into the frame dirty rect before the cumulative one is
    /// dropped: the moving preview's OLD position must be recomposited (base restored where coverage is
    /// now zero), which a dirty rect of only the NEW dabs would miss (the moving-overlay union).
    pub(super) fn clear_wet_coverage(&mut self) {
        if let Some(c) = self.paint.wet_cum_dirty.take() {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, c),
                None => c,
            });
        }
        self.paint.stroke_coverage.iter_mut().for_each(|c| *c = 0);
    }

    /// Zero the per-stroke deposited-colour buffer (retain capacity); twin of [`Self::clear_wet_coverage`].
    pub(super) fn clear_wet_color(&mut self) {
        self.paint.stroke_color.iter_mut().for_each(|c| *c = 0);
    }

    /// Splat each dab's soft disc into the per-stroke coverage (max-blend = the wet "one pass" union,
    /// independent of colour, like wet_edges `stampCoverage`). Coverage is geometric but scaled by the
    /// dab's own opacity, so a faint wash pools a fainter rim.
    pub(super) fn accumulate_wet_coverage(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_coverage.len() != fw * fh {
            self.paint.stroke_coverage = vec![0u8; fw * fh];
        }
        // Track the batch footprint incrementally (wet_edges `expand`): the per-frame dirty rect bounds
        // the live recomposite; the cumulative one gives the pen-up bake its bbox without a canvas scan.
        if let Some(r) = self.dab_batch_region(dabs) {
            self.paint.wet_frame_dirty = Some(match self.paint.wet_frame_dirty {
                Some(f) => union_region(f, r),
                None => r,
            });
            self.paint.wet_cum_dirty = Some(match self.paint.wet_cum_dirty {
                Some(c) => union_region(c, r),
                None => r,
            });
        }
        // The soak disc follows the newest dab — where the tick heartbeat pours water dwell.
        if let Some(d) = dabs.last() {
            self.paint.wet_soak_pos = Some((d.center, d.radius_px.max(0.0)));
        }
        // Dilution (Wet Mix, `docs/Painter/07` §4): more water lays down LESS coverage → a thinner,
        // paler wash. `flow = 1 − dilution`; default `0` → `flow = 1` (byte-identical).
        let flow = (1.0 - self.paint.brush.wet_dilution).clamp(0.0, 1.0);
        let cov = &mut self.paint.stroke_coverage;
        for d in dabs {
            let r = d.radius_px;
            let peak = (d.coverage * flow).clamp(0.0, 1.0);
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let v = (peak * feather(dn) * 255.0) as u8;
                    let idx = base + x;
                    if v > cov[idx] {
                        cov[idx] = v;
                    }
                }
            }
        }
    }

    /// Splat each dab's colour into the per-stroke colour buffer, **source-over** (recent dab wins on
    /// overlap, carrying its colour — wet_edges `stampColor`), straight-alpha RGBA. Same soft-disc
    /// feather as the coverage, so the deposited colour and the silhouette agree.
    pub(super) fn accumulate_wet_color(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_color.len() != fw * fh * 4 {
            self.paint.stroke_color = vec![0u8; fw * fh * 4];
        }
        // Wet Mix (`docs/Painter/07` §4): the deposited colour is the mixer's per-dab blend of the
        // brush colour with the surface it picked up (Charge/Pull). Off (default `wet_charge = 1`) ⇒
        // each dab's own colour (byte-identical). Computed BEFORE borrowing `stroke_color`.
        let mixed = self.wet_mix_dab_colors(dabs);
        let buf = &mut self.paint.stroke_color;
        for (d, (dcol, prio)) in dabs.iter().zip(&mixed) {
            let r = d.radius_px;
            let peak = d.coverage.clamp(0.0, 1.0);
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            // Deposit PRIORITY (Wet Mix): a high-pickup dab writes at full alpha; a low-pickup one
            // (leaving a pool over bare ground) barely writes, so it can't overwrite the stronger
            // picked-up colour — the pool's exit edge stays as coloured as its entry (Enio 2026-07-07).
            // `prio == 1` when the mixer is off ⇒ byte-identical source-over.
            let prio = prio.clamp(0.0, 1.0);
            let col = [
                (dcol[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (dcol[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ];
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let a = peak * feather(dn) * prio; // source alpha, scaled by deposit priority
                    if a <= 0.0 {
                        continue;
                    }
                    let idx = (base + x) * 4;
                    let da = f32::from(buf[idx + 3]) / 255.0;
                    let na = a + da * (1.0 - a); // straight-alpha over
                    if na <= 0.0 {
                        continue;
                    }
                    for c in 0..3 {
                        let dc = f32::from(buf[idx + c]) / 255.0;
                        let sc = f32::from(col[c]) / 255.0;
                        let out = (sc * a + dc * da * (1.0 - a)) / na;
                        buf[idx + c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    }
                    buf[idx + 3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}
