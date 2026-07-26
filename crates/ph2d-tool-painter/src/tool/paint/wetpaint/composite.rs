//! The wet COMPOSITE half (child of [`super`] — workspace file-LOC cap):
//! pigment (with the doc-22 visual terms) OVER the frozen base, the
//! Selection/protection/alpha-lock gates, and the show-wet veil.

use super::*;
// The row fan-out and the engine's per-row door live HERE and nowhere else, so the imports do too: the
// parent was three lines over the workspace LOC cap carrying names only this file uses.
use ph2d_wet_paint::render::{RenderLayer, render_pigment_row_visual};
use rayon::prelude::*;

/// Show-wet veil (doc 22 §2.7 — the model's damp overlay, adapted from an
/// opaque sheet to a LAYER): a cool translucent slate composited over the
/// result wherever the sheet is damp, so wetness reads over paint AND over
/// nothing; the meniscus glint keeps the model's rim law. Display-only —
/// the end-session door recomposites clean, so none of this ever bakes.
const WET_VEIL_ALPHA: f64 = 0.18;
const WET_VEIL_RGB: [f64; 3] = [56.0, 80.0, 104.0];
/// The model's light direction (top-left) for the meniscus glint.
const WET_LIGHT_X: f64 = -std::f64::consts::FRAC_1_SQRT_2;
const WET_LIGHT_Y: f64 = -std::f64::consts::FRAC_1_SQRT_2;

impl PainterTool {
    /// Composite the engine's dirty rect: pigment (straight alpha) OVER the
    /// frozen base, written into `canvas_rgba`, preview marked.
    ///
    /// Selection / protection / alpha-lock (W2.5) are enforced HERE, at the
    /// single place the module writes the canvas: the fluid flows wherever
    /// the SIM takes it, and the gated texels keep-lerp back to the frozen
    /// base (`splat_keep` — the watercolor composite's exact law, a hard
    /// guarantee independent of where the water reached). Alpha-lock pins
    /// the layer's α to the frozen base (colour deposits into existing
    /// paint; the silhouette never moves). All gates off = byte-identical.
    pub(super) fn wetpaint_composite(&mut self) {
        self.wetpaint_composite_veiled(self.paint.wetpaint.show_wet);
    }

    /// [`Self::wetpaint_composite`] with the show-wet veil explicit — the
    /// end-session door forces `false` so the veil never bakes.
    pub(super) fn wetpaint_composite_veiled(&mut self, veil: bool) {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        let (gate_sel, gate_prot, gate_alock) = self.wet_splat_gates();
        let visual = ph2d_wet_paint::render::PigmentVisual {
            paper: self.paint.wetpaint.paper_visual,
            km_glaze: self.paint.wetpaint.km_glaze,
        };
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // Engine dirty (1-based cells, inclusive) → clamped cell rect.
        let (cx0, cy0, cx1, cy1) = match sess.engine.take_dirty() {
            Dirty::Clean => return,
            Dirty::Full => (1, 1, w, h),
            Dirty::Rect { x0, y0, x1, y1 } => (
                (x0.max(1) as usize).min(w),
                (y0.max(1) as usize).min(h),
                (x1.max(1) as usize).min(w),
                (y1.max(1) as usize).min(h),
            ),
        };
        if cx1 < cx0 || cy1 < cy0 || sess.base.len() != w * h * 4 {
            return;
        }
        let layers: Vec<RenderLayer<'_>> = sess
            .engine
            .layers
            .iter()
            .map(|l| RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        // The visual terms (doc 22): the paper printed into the pigment and
        // the K–M glaze stacking ride the SAME single render body; both off
        // is the historical pigment render, byte for byte (engine-gated).
        //
        // ROW-PARALLEL ([ADR-0109](../../../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md),
        // amendment 2): the same shape of work the ADR already sanctioned for
        // the watercolor optical composite — a pure per-pixel map whose rows
        // are independent, in the crate the ADR already opened. The ENGINE
        // stays dependency-free and spawns nothing; it exposes one row
        // (`render_pigment_row_visual`) and the fan-out lives here.
        //
        // Byte-identical by construction: every row computes exactly what the
        // serial loop computed, and rows never read each other's output.
        let params = sess.engine.sim.gather_params(&sess.engine.tuning);
        let stride = w * 4;
        sess.pigment[(cy0 - 1) * stride..cy1 * stride]
            .par_chunks_mut(stride)
            .enumerate()
            .for_each(|(k, row)| {
                render_pigment_row_visual(Some(&params), &layers, visual, cy0 + k, cx0, cx1, row);
            });
        drop(layers);
        // Straight-alpha OVER the frozen base, cell (cx,cy) → pixel (cx-1,cy-1).
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let gate_on = gsel.is_some() || gprot.is_some();
        let alock_on = gate_alock.is_some();
        let canvas = crate::tool::paint::plane_fork::fork_par(&mut self.canvas_rgba);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let o = ((cy - 1) * w + (cx - 1)) * 4;
                let pa = sess.pigment[o + 3] as f32 / 255.0;
                if pa <= 0.0 {
                    // Base verbatim — the gates keep-lerp TOWARD the base, so
                    // they are exact no-ops on this branch.
                    canvas[o..o + 4].copy_from_slice(&sess.base[o..o + 4]);
                    continue;
                }
                let ba = sess.base[o + 3] as f32 / 255.0;
                let oa = pa + ba * (1.0 - pa);
                for ch in 0..3 {
                    let pc = sess.pigment[o + ch] as f32;
                    let bc = sess.base[o + ch] as f32;
                    canvas[o + ch] = ((pc * pa + bc * ba * (1.0 - pa)) / oa).round() as u8;
                }
                canvas[o + 3] = (oa * 255.0).round() as u8;
                if gate_on {
                    let keep = super::watercolor_accum::splat_keep(gsel, gprot, None, o / 4);
                    if keep < 1.0 {
                        for c in 0..4 {
                            let painted = f32::from(canvas[o + c]);
                            let orig = f32::from(sess.base[o + c]);
                            canvas[o + c] = (painted * keep + orig * (1.0 - keep))
                                .round()
                                .clamp(0.0, 255.0)
                                as u8;
                        }
                    }
                }
                if alock_on {
                    // Alpha-lock: the α reference IS `sess.base` (the gate's
                    // buffer, served by `wet_splat_gates`' wet-session arm).
                    canvas[o + 3] = sess.base[o + 3];
                }
            }
        }
        if veil {
            // Show-wet: the damp overlay, display-only (never baked). Reads
            // the ACTIVE grid's wetness byte + film exactly like the model;
            // the veil is composited OVER the finished pixel so it shows on
            // painted and unpainted texels alike (an inspection overlay may
            // override the alpha-lock's look — the bake never carries it).
            let g = sess.engine.active_grid();
            let sg = g.s;
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    let i = cx + cy * sg;
                    let film = f64::from(g.film[i]);
                    let mut wet_sig = f64::from(g.wet[i]) / 255.0;
                    if film > 0.02 {
                        let t = (film / 2.5).min(1.0);
                        let sv = t * t * (3.0 - 2.0 * t);
                        if sv > wet_sig {
                            wet_sig = sv;
                        }
                    }
                    if wet_sig <= 0.01 {
                        continue;
                    }
                    let o = ((cy - 1) * w + (cx - 1)) * 4;
                    let a_v = WET_VEIL_ALPHA * wet_sig;
                    let ca = f64::from(canvas[o + 3]) / 255.0;
                    let oa = a_v + ca * (1.0 - a_v);
                    // Meniscus glint — the model's rim shine, additive.
                    let gx = (f64::from(g.film[i + 1]) - f64::from(g.film[i - 1])) * 0.5;
                    let gy = (f64::from(g.film[i + sg]) - f64::from(g.film[i - sg])) * 0.5;
                    let mag = (gx * gx + gy * gy).sqrt();
                    let shine = if mag > 0.01 {
                        let rim = (mag * 0.7).min(1.0);
                        18.3 * rim
                            * rim
                            * (0.5 + 0.5 * ((gx / mag) * WET_LIGHT_X + (gy / mag) * WET_LIGHT_Y))
                    } else {
                        0.0
                    };
                    for ch in 0..3 {
                        let cc = f64::from(canvas[o + ch]);
                        let mixed =
                            (WET_VEIL_RGB[ch] * a_v + cc * ca * (1.0 - a_v)) / oa.max(1e-6) + shine;
                        canvas[o + ch] = mixed.round().clamp(0.0, 255.0) as u8;
                    }
                    canvas[o + 3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        // Re-arm the guard with the allocation our own make_mut may have
        // re-seated (weak — [`WetSession::canvas`]).
        sess.canvas = Arc::downgrade(&self.canvas_rgba);
        let region = Region {
            x: (cx0 - 1) as u32,
            y: (cy0 - 1) as u32,
            w: (cx1 - cx0 + 1) as u32,
            h: (cy1 - cy0 + 1) as u32,
        };
        self.mark_dirty(region);
    }
}
