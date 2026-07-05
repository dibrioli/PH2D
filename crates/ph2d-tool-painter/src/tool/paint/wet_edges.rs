//! Watercolor **edge darkening** (#1 — the wet "fringe"): the pen-up pass that pools pigment at the
//! stroke's receding boundary. No fluid sim — a blur-difference of the stroke's coverage, the
//! image-space form of Curtis 1997 §4.3.3 (`p ← p − η·(1 − blur(M))·M`) and the exact mechanism of
//! `docs/Painter/wet_edges_paint.html` (`edge = cov·(1 − blur(cov))·gain`).
//!
//! The stroke's coverage is accumulated per dab (a soft disc, **max**-blended — the union footprint,
//! independent of colour, like `stampCoverage`) during stamping; on pen-up it is blurred and the
//! deposited colour is darkened in the thin band just inside the rim. All ops are sums/clamps →
//! deterministic (HR-5): no transcendental but `sqrt`, which is IEEE-exact and platform-stable.

use super::*;

/// The cap on how far the rim may darken. The darken is a straight RGB multiply, which keeps the
/// pigment's hue + saturation and only lowers its value (more pigment ⇒ a darker version of the SAME
/// colour, never black); this cap keeps even a strong Edge gain from reading as a black outline.
const RIM_MAX_DARKEN: f32 = 0.45; // LITERAL-PX-OK: watercolor rim-strength behaviour constant

impl PainterTool {
    /// Whether the edge-darkening pass runs (Watercolor section on + a non-zero Edge gain).
    pub(super) fn wet_edges_active(&self) -> bool {
        self.paint.brush.watercolor && self.paint.brush.edge_gain > 0.0
    }

    /// Zero the per-stroke coverage (retain capacity). The drag-preview restore rebuilds coverage from
    /// the re-stamped batch each frame, mirroring the pixel restore — so Drag Dot / Anchored / Line
    /// leave no coverage trail either.
    pub(super) fn clear_wet_coverage(&mut self) {
        self.paint.stroke_coverage.iter_mut().for_each(|c| *c = 0);
    }

    /// Splat each dab's soft disc into the per-stroke coverage (max-blend = the wet "one pass" union).
    /// Coverage is geometric but scaled by the dab's own opacity, so a faint wash pools a fainter rim.
    pub(super) fn accumulate_wet_coverage(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_coverage.len() != fw * fh {
            self.paint.stroke_coverage = vec![0u8; fw * fh];
        }
        let cov = &mut self.paint.stroke_coverage;
        for d in dabs {
            let r = d.radius_px;
            let peak = d.coverage.clamp(0.0, 1.0);
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
                    // Soft disc: full to ~0.62, then feather to 0 at the rim (wet_edges `stampCoverage`).
                    let t = if dn <= 0.62 {
                        1.0
                    } else {
                        1.0 - (dn - 0.62) / (1.0 - 0.62)
                    };
                    let v = (peak * t * 255.0) as u8;
                    let idx = base + x;
                    if v > cov[idx] {
                        cov[idx] = v;
                    }
                }
            }
        }
    }

    /// Restore the pixels the **live** edge overlay darkened last frame (so the canvas returns to the
    /// clean-painted stroke before the next dab lands / the final bake). No-op when no overlay is saved.
    /// This is the wet-edge twin of the drag-preview restore — it lets the fringe update every frame
    /// (live, not a jump on pen-up) without the darkening compounding.
    pub(super) fn restore_wet_edge_overlay(&mut self) {
        let Some((region, pixels)) = self.paint.wet_edge_saved.take() else {
            return;
        };
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if self.canvas_rgba.len() != fw * fh * 4 {
            return;
        }
        let (rw, rh) = (region.w as usize, region.h as usize);
        if pixels.len() != rw * rh * 4 {
            return;
        }
        let (rx, ry) = (region.x as usize, region.y as usize);
        let buf = std::sync::Arc::make_mut(&mut self.canvas_rgba);
        for row in 0..rh {
            let dst = ((ry + row) * fw + rx) * 4;
            let src = row * rw * 4;
            buf[dst..dst + rw * 4].copy_from_slice(&pixels[src..src + rw * 4]);
        }
        self.mark_dirty(region);
    }

    /// Edge-darkening pass: blur the stroke coverage, pool pigment at the receding boundary
    /// (`edge = cov·(1 − blur(cov))·gain`), and darken the deposited colour there.
    ///
    /// `commit == false` is the **live** pass run every frame while the pointer is down: it first saves
    /// the pre-darken pixels of the affected region (into `wet_edge_saved`, restored next frame by
    /// [`Self::restore_wet_edge_overlay`]) so the fringe grows with the stroke instead of appearing as a
    /// jump on release. `commit == true` is the pen-up bake — permanent, inside the undo transaction
    /// (before `close_stroke`, mirroring [`Self::heal_inpaint`]); the caller restores the live overlay
    /// first so this bakes over the clean stroke.
    pub(super) fn apply_wet_edges(&mut self, commit: bool) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 || self.paint.stroke_coverage.len() != fw * fh {
            return;
        }
        if self.canvas_rgba.len() != fw * fh * 4 {
            return;
        }
        let gain = self.paint.brush.edge_gain;
        let radius = self.paint.brush.edge_spread.round().clamp(1.0, 24.0) as usize;

        // Bbox of the covered pixels, padded by the blur radius so the rim's feather is seen.
        let (mut minx, mut miny, mut maxx, mut maxy) = (fw, fh, 0usize, 0usize);
        let mut any = false;
        for y in 0..fh {
            let base = y * fw;
            for x in 0..fw {
                if self.paint.stroke_coverage[base + x] > 0 {
                    any = true;
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                    miny = miny.min(y);
                    maxy = maxy.max(y);
                }
            }
        }
        if !any {
            return;
        }
        let x0 = minx.saturating_sub(radius);
        let y0 = miny.saturating_sub(radius);
        let x1 = (maxx + radius + 1).min(fw);
        let y1 = (maxy + radius + 1).min(fh);
        let (bw, bh) = (x1 - x0, y1 - y0);
        let region = Region {
            x: x0 as u32,
            y: y0 as u32,
            w: bw as u32,
            h: bh as u32,
        };

        // Live pass: snapshot the clean pixels of this region BEFORE darkening, so the next frame (or
        // the pen-up bake) can peel the overlay back off. (The pen-up bake is permanent → no snapshot.)
        if !commit {
            let mut saved = vec![0u8; bw * bh * 4];
            let cur = &**self.canvas_rgba;
            for row in 0..bh {
                let src = ((y0 + row) * fw + x0) * 4;
                let dst = row * bw * 4;
                saved[dst..dst + bw * 4].copy_from_slice(&cur[src..src + bw * 4]);
            }
            self.paint.wet_edge_saved = Some((region, saved));
        } else {
            self.paint.wet_edge_saved = None;
        }

        // Coverage over the padded bbox in `[0, 1]` (the padding reads the true 0 outside the stroke,
        // so the blur feathers correctly at the real rim), then its blur.
        let mut src = vec![0.0f32; bw * bh];
        for by in 0..bh {
            let sbase = (y0 + by) * fw + x0;
            let dbase = by * bw;
            for bx in 0..bw {
                src[dbase + bx] = f32::from(self.paint.stroke_coverage[sbase + bx]) / 255.0;
            }
        }
        let blur = box_blur(&src, bw, bh, radius);

        // Darken the deposited colour in the rim band. Multiply the sRGB channels equally so the hue +
        // saturation are preserved and only the value drops (more pigment = a darker version of the SAME
        // colour, never black); F4's pigment path makes this optically exact.
        let buf = std::sync::Arc::make_mut(&mut self.canvas_rgba);
        for by in 0..bh {
            let dbase = by * bw;
            let pbase = (y0 + by) * fw + x0;
            for bx in 0..bw {
                let cf = src[dbase + bx];
                if cf <= 0.0 {
                    continue;
                }
                let edge = (cf * (1.0 - blur[dbase + bx]) * gain).clamp(0.0, 1.0);
                let darken = edge * RIM_MAX_DARKEN;
                if darken <= 0.0 {
                    continue;
                }
                let factor = 1.0 - darken;
                let po = (pbase + bx) * 4;
                for c in 0..3 {
                    buf[po + c] = (f32::from(buf[po + c]) * factor).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        self.mark_dirty(region);
    }
}

/// Separable box blur of a scalar field, O(n) via per-row/column prefix sums (window count clamped at
/// the borders, so no darkening bias). Deterministic (only sums + one divide). `radius == 0` is a copy.
fn box_blur(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    // Horizontal pass.
    let mut tmp = vec![0.0f32; w * h];
    let mut pref = vec![0.0f32; w + 1];
    for y in 0..h {
        let base = y * w;
        for x in 0..w {
            pref[x + 1] = pref[x] + src[base + x];
        }
        for x in 0..w {
            let lo = x.saturating_sub(radius);
            let hi = (x + radius).min(w - 1);
            tmp[base + x] = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    }
    // Vertical pass.
    let mut out = vec![0.0f32; w * h];
    let mut prefc = vec![0.0f32; h + 1];
    for x in 0..w {
        for y in 0..h {
            prefc[y + 1] = prefc[y] + tmp[y * w + x];
        }
        for y in 0..h {
            let lo = y.saturating_sub(radius);
            let hi = (y + radius).min(h - 1);
            out[y * w + x] = (prefc[hi + 1] - prefc[lo]) / (hi - lo + 1) as f32;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::box_blur;

    /// Blur of a constant field is the same constant (window-count clamp handles the borders).
    #[test]
    fn box_blur_preserves_constant() {
        let src = vec![0.5f32; 8 * 6];
        let out = box_blur(&src, 8, 6, 2);
        for v in out {
            assert!((v - 0.5).abs() < 1e-6, "constant field must blur to itself");
        }
    }

    /// A single spike spreads mass to neighbours (peak drops, a neighbour rises) — proves it blurs.
    #[test]
    fn box_blur_spreads_a_spike() {
        let (w, h) = (9, 9);
        let mut src = vec![0.0f32; w * h];
        src[4 * w + 4] = 1.0;
        let out = box_blur(&src, w, h, 1);
        assert!(out[4 * w + 4] < 1.0, "peak dropped");
        assert!(out[4 * w + 5] > 0.0, "mass reached the neighbour");
    }
}
