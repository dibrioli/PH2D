//! **Inflate — the Blob**: the relief offset by a BALL, and the MATTER that rides it.
//!
//! Split from [`super::sculpt_blur`] for the workspace file-LOC cap, along the seam the code already
//! argued for: the Blob is **not a blur and never was**. Its siblings there are one expression over a
//! memoised `blur(pre, r)` — a stroke-constant, cached per tile. The ball's radius rides the LIVE `amount`,
//! so it is not a stroke-constant, it cannot be memoised, and it recomputes every render. Its engine lives
//! next door in [`super::sculpt_offset`] ([`super::sculpt_offset::blob_ball`]); this is the render that
//! drives it, writes the height, and advects the paint.
//!
//! It is the ONE verb that moves MATTER. The other seven reshape `h` inside paint that is already there;
//! this one grows the form, and a form that grows onto bare canvas without taking its paint along does not
//! grow at all — the light weighs by coverage. So it answers two questions, not one: how high (the ball's
//! dilation, [`super::sculpt_offset::blob_ball`]), and WHERE the footprint may go (a morphological CLOSING,
//! [`super::sculpt_close`] — fill the concave armpits, leave the convex flanks where the artist drew them,
//! Enio 2026-07-17). The coverage is read off the smooth grown height, so it cannot disagree with the relief
//! about where the form ends; the closing decides whether it grows there at all.

use super::impasto_ceiling::H_MAX;
use super::region::grow_region;
use super::{Region, impasto_settle};
use crate::tool::PainterTool;
use std::sync::Arc;

/// The coverage anti-alias width, **in loads of relief**. The grown form's coverage is read off the SMOOTH
/// grown-height field `hbuf` — full where the relief has body, feathering to zero over the last `COVER_AA`
/// loads as the relief itself runs out at the silhouette. Deriving it from the height VALUE (a max of
/// continuous balls, seamless even on a curved border) and not from the argmax POSITION is what keeps the
/// argmax's Voronoi cells out of the picture: `pre_cover · ball_fraction(d_win)` fades over the whole ball
/// (the ρ-wide translucent skirt Enio rejected, 2026-07-17) and jumps across the winner's seams (radial
/// spokes on a round blob). A solid silhouette with a clean edge, never a skirt, never a cookie-cutter.
const COVER_AA: f32 = 1.6;

impl PainterTool {
    /// **Inflate — the Blob** (Enio 2026-07-14, third smoke: *"não é inflate … puxa as faces na direção das
    /// normais usando para intensidade o falloff … estude o Blob do Blender"*).
    ///
    /// A morphological offset by a ball whose **radius follows the falloff**: at the brush centre the ball is
    /// full (`|Depth|·DEPTH_UNIT_PX` px), tapering to zero at the edge. That is the whole correction. A
    /// constant-radius ball fattened the form but left a **flat top** (a max filter plateaus where the ball
    /// fits); with the radius IN the falloff, the centre is the sphere's own curvature (a rounded peak), the
    /// rim fattens, and the edge lands at nothing — a rounded blob, which is what a height field can offer of
    /// Blender's Blob.
    ///
    /// ## Why it is the EXACT ball and not a separable parabola
    ///
    /// The parabola is separable and `O(N)`, but its support is UNBOUNDED, and that is the whole of the
    /// junction gash (see [`super::sculpt_offset`]). The exact bounded ball has `capture == reach` by
    /// construction — no dead zone, no gash, and no need for the sentinel / per-source budget / taper /
    /// self-floor that fenced the parabola. It is `O(area·ρ²)`, but parallel over rows (+ the disc-list
    /// reformulation in `blob_ball`) it lands the widest kill case under the 8 ms/move bar.
    ///
    /// ## The form fattens BEYOND the brush, so the window is grown
    ///
    /// A big-radius source near the centre pushes the boundary out past where the brush touched — that IS the
    /// fattening — so this writes to `rect` grown by the ball's reach, not just the touched texels. And the
    /// **matter follows** (coverage / material / colour), the way it must for a form that grows onto bare
    /// canvas: the paint dilates with the relief.
    pub(super) fn render_inflate(&mut self, rect: Region) {
        let (Some(layer), (w, h)) = (self.paint.sculpt.layer, self.source_size) else {
            return;
        };
        let n = (w as usize) * (h as usize);
        if self.paint.sculpt.pre.len() != n || self.paint.sculpt.amount.len() != n {
            return;
        }
        let depth = self.paint.sculpt.depth();
        let unit = super::impasto_light::DEPTH_UNIT_PX;
        let rho = (depth.abs() * unit).ceil() as i64;
        if rho == 0 {
            return; // Depth 0 is the identity — nothing to offset
        }
        let smooth = self.paint.sculpt.inflate_smooth_px();
        let dilate = depth > 0.0;
        // The window's reach, and why it is `ρ√2` and not the ball's own `ρ`. The ball WRITES only inside
        // `ρ` (`blob_ball` lifts nothing past a source's own `ρ·amount`; beyond it the field is `pre` AO
        // BIT, whatever the window). But the render is called PER BATCH over that batch's dabs, and a
        // texel's value keeps changing while a source within `ρ` of it still has a growing `amount` — i.e.
        // while a dab lands within `ρ + dab_radius`. So a texel must be RE-rendered at least that far out,
        // or the batching leaks into the picture (a slow mouse lays fewer, coarser batches and the rim
        // freezes at a partial `amount`): the sampling-independence this line has already paid for. The
        // window is re-coverage, not support; `ρ√2` gives the margin the equality `reach = ρ` lacks, and
        // costs only `pre`-identical texels. Two gates pin it (`a_faster_mouse…`, `a_knob_touched…`).
        let reach = ((2 * rho * rho) as f64).sqrt().ceil() as u32;
        let Some(kr) = grow_region(rect, reach + smooth, w, h) else {
            return;
        };
        let Some(cr) = grow_region(rect, 2 * (reach + smooth), w, h) else {
            return;
        };
        let pre = Arc::clone(&self.paint.sculpt.pre);
        let amount = Arc::clone(&self.paint.sculpt.amount);
        let (cw, ch) = (cr.w as usize, cr.h as usize);
        let (wu, wi, hi) = (w as usize, i64::from(w), i64::from(h));

        // The bounded ball over `cr` — height + the packed argmax the matter follows. Bounded by
        // construction: a source contributes only inside its own `ρ·amount` ball, so outside the touched
        // set dilated by the ball the field is `pre` AO BIT (no cap, no taper, no sentinel — the reasons
        // those existed left with the parabola).
        let (mut hbuf, sbuf) =
            super::sculpt_offset::blob_ball(&pre, &amount, w, h, cr, depth, unit, dilate);

        if smooth > 0 {
            // The knob smooths the ABSOLUTE surface where the ball acted — not the lift alone (adding a
            // blurred lift back onto a sharp `pre` would exhume, along the old silhouette, an edge the raw
            // ball had buried under its advance), and never the whole window (blurring `pre` out to the
            // region's corners is a rectangular shelf). A support mask, blurred by the SAME kernel, feathers
            // between the two exactly as far as the blur can see: `m = 1` (exact) keeps the pure blur,
            // `m = 0` (exact) keeps `pre` to the bit, so the byte-identity of the support survives the knob.
            let mut mask: Vec<f32> = Vec::with_capacity(cw * ch);
            for ry in 0..ch {
                let gy = (cr.y as usize + ry) * wu + cr.x as usize;
                for rx in 0..cw {
                    mask.push(if hbuf[ry * cw + rx] != pre[gy + rx] {
                        1.0
                    } else {
                        0.0
                    });
                }
            }
            let mut blurred = hbuf.clone();
            impasto_settle::box_blur(&mut blurred, cr.w, cr.h, smooth);
            impasto_settle::box_blur(&mut mask, cr.w, cr.h, smooth);
            for ry in 0..ch {
                let gy = (cr.y as usize + ry) * wu + cr.x as usize;
                for rx in 0..cw {
                    let i = ry * cw + rx;
                    let m = mask[i];
                    hbuf[i] = if m <= 0.0 {
                        pre[gy + rx]
                    } else if m >= 1.0 {
                        blurred[i]
                    } else {
                        pre[gy + rx] + m * (blurred[i] - pre[gy + rx])
                    };
                }
            }
        }

        // ── Write the height + advect the matter, over the write region `kr`. ───────────────────────────
        let pre_cover = Arc::clone(&self.paint.sculpt.pre_cover);
        let pre_mats = Arc::clone(&self.paint.sculpt.pre_mats);
        let pre_rgba = Arc::clone(&self.paint.sculpt.pre_rgba);
        // The footprint the MATTER may grow into is the CLOSING of the paint, not the ball's raw dilation:
        // fill the concave armpits, leave the convex flanks where the artist drew them (Enio, 2026-07-17). On
        // erosion (Depth < 0) there is no growth to gate, so this is only computed for the dilating pass.
        let cfill = if dilate && pre_cover.len() == n {
            super::sculpt_close::closing_fill(&pre_cover, wu, cr, depth.abs() * unit)
        } else {
            vec![1.0f32; cw * ch]
        };
        let matter_ok = pre_cover.len() == n
            && pre_mats.len() == n
            && pre_rgba.len() == n * 4
            && self.canvas_rgba.len() == n * 4;
        let mut moved: Option<Region> = None;
        {
            let Some(entry) = self.heights.get_mut(&layer) else {
                return;
            };
            if entry.len() != n {
                return;
            }
            let target = super::plane_fork::fork_heights(
                entry,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(kr),
            );
            for y in kr.y..kr.y + kr.h {
                for x in kr.x..kr.x + kr.w {
                    let ci = ((y - cr.y) as usize) * cw + (x - cr.x) as usize;
                    let gi = (y as usize) * wu + (x as usize);
                    let next = hbuf[ci].clamp(-H_MAX, H_MAX);
                    if (next - target[gi]).abs() > impasto_settle::RELIEF_EPS {
                        let rr = Region { x, y, w: 1, h: 1 };
                        moved = Some(moved.map_or(rr, |acc| super::union_region(acc, rr)));
                    }
                    target[gi] = next;
                }
            }
        }
        // The MATTER dilates with the form: where the ball carried paint from a source, that source's
        // coverage / material / colour extend to here — a form that grows onto bare canvas takes its paint
        // with it (round-2 lesson). It arrives at the ball's own strength (`ball_fraction`): the height
        // fades across the ball's flank, and the matter fades with it by the SAME number, so the silhouette
        // the light draws (it weighs by coverage) cannot disagree with the height about where the form ends.
        // That agreement is the whole of Enio's 2026-07-16 smoke — before it, the coverage was copied at
        // full strength to the last texel and cut binary, a Voronoi pattern binarised (the staircase).
        //
        // **The destination each of these composes onto is the FROZEN plane, never the live buffer.** The
        // freehand path re-renders the whole stroke from `pre` on every pointer-move batch with no restore
        // in between (`sculpt_session`), and that is only sound because every write here is an ASSIGNMENT
        // sourced from frozen state. Composing `over` the live pixel instead would let a texel be composed
        // once per batch, so the fringe would darken with how SLOWLY the artist dragged — the
        // sampling-dependence this line has already paid for twice (the bite's telescoping share; the
        // relief's capsule).
        let mut matter_moved: Option<Region> = None;
        if matter_ok
            && let (Some(cov_e), Some(mat_e)) =
                (self.covers.get_mut(&layer), self.mats.get_mut(&layer))
            && cov_e.len() == n
            && mat_e.len() == n
        {
            let cov = super::plane_fork::fork_covers(
                cov_e,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(kr),
            );
            let mat = super::plane_fork::fork_mats(
                mat_e,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(kr),
            );
            let rgba = super::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
            );
            for y in kr.y..kr.y + kr.h {
                for x in kr.x..kr.x + kr.w {
                    let ci = ((y - cr.y) as usize) * cw + (x - cr.x) as usize;
                    let gi = (y as usize) * wu + (x as usize);
                    // The write region is `kr` (correctness — every texel re-assigned), but the LIGHT only
                    // needs to recompute where the picture actually CHANGED. On a moving stroke that is a thin
                    // sliver (the leading edge that gained paint, the trailing edge whose stale composite was
                    // restored) — dirtying all of `kr` every move would recompute the CPU light over the whole
                    // footprint and is the difference between 3 ms and 14 ms. So compare before/after here.
                    let old_c = cov[gi];
                    let old_r: [u8; 4] = rgba[gi * 4..gi * 4 + 4].try_into().unwrap();
                    // **Every kr texel gets an UNCONDITIONAL final assignment** — the composite where a
                    // foreign source reaches it, the FROZEN plane where it does not. The height loop above is
                    // already unconditional (`target[gi] = next`); the matter MUST match. A conditional
                    // `continue` (the shape this had) leaves an earlier per-batch render's write in place, and
                    // the freehand path never restores between batches, so that stale value survives — while
                    // the winner and its arrival BOTH shift as `amount` grows (and the coverage-winner is not
                    // the height-winner, so the arrival is not even monotone). A slow mouse and a fast one
                    // would then leave different pictures: the batching leaking in, which the sampling gates
                    // forbid (`a_faster_mouse…`, `a_knob_touched…`). Restoring to the frozen plane is the undo
                    // the missing between-batch restore would have done.
                    let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[ci]);
                    let arrival = if dx != 0 || dy != 0 {
                        let (sx, sy) = (i64::from(x) + dx, i64::from(y) + dy);
                        if sx >= 0 && sy >= 0 && sx < wi && sy < hi {
                            let si = (sy as usize) * wu + (sx as usize);
                            // Coverage from the SMOOTH grown-height field, not the argmax POSITION. `hbuf` is a
                            // max of continuous balls, so on a curved border it is seamless; the argmax's
                            // Voronoi cells (which `ball_fraction(d_win)` exposed as radial spokes on a round
                            // blob, 2026-07-17) never reach the picture. Opaque where the relief has body,
                            // feathering to zero over the last `COVER_AA` loads as the ball runs out at the
                            // silhouette. The colour still comes from the argmax source `si` (the tallest,
                            // hence most solid, paint that reaches here) — but seams in the COLOUR are masked
                            // where the coverage is thin, and invisible in a uniform form.
                            // Opaque coverage from the smooth height (`hbuf`), gated by the CLOSING (`cfill`):
                            // full inside a filled armpit, zero on a convex flank (its edge stays put),
                            // feathering at the fill's own free boundary. The two AAs — the height's at the
                            // silhouette, the closing's at the armpit mouth — multiply.
                            let tc = (hbuf[ci] / COVER_AA).clamp(0.0, 1.0) * cfill[ci];
                            let v = (f32::from(pre_cover[si]) * tc) as u8;
                            // COVERAGE is a PRESENCE: the paint extends only where the ball brings MORE than
                            // is already frozen here; otherwise this texel keeps its own paint.
                            (v > pre_cover[gi]).then_some((si, tc, v))
                        } else {
                            None
                        }
                    } else {
                        None // the matter here is its own — nothing dilated onto it
                    };
                    match arrival {
                        Some((si, t, v)) => {
                            cov[gi] = v;
                            // MATERIAL and COLOUR are IDENTITIES, so they compose `over` at the opacity that
                            // actually arrived (`t`) — the same operator as the deposit's own merge: the paint
                            // on top is the paint you see. At `t = 1` over an opaque source both reduce to the
                            // verbatim copy, which keeps the approved interior exactly where Enio approved it.
                            let a8 = (t * 255.0 + 0.5) as u32;
                            for c in 0..mat[gi].len() {
                                let (old, new) =
                                    (u32::from(pre_mats[gi][c]), u32::from(pre_mats[si][c]));
                                mat[gi][c] = ((old * (255 - a8) + new * a8 + 127) / 255) as u8;
                            }
                            // The pigment through the pigment's own door — straight-alpha source-over, in the
                            // space `canvas_rgba` is already stored and blended in. The arriving paint's
                            // opacity is its OWN alpha scaled by the ball fraction; over bare canvas that lands
                            // the source's colour at alpha·t (the rim feathers), over an opaque ground a lerp
                            // toward the paint (the rim blends).
                            let d = |k: usize| f32::from(pre_rgba[gi * 4 + k]) / 255.0;
                            let s = |k: usize| f32::from(pre_rgba[si * 4 + k]) / 255.0;
                            let out = ph2d_painter_brush::blend_over(
                                ph2d_painter_brush::BrushBlend::Mix,
                                [d(0), d(1), d(2), d(3)],
                                [s(0), s(1), s(2)],
                                s(3) * t,
                            );
                            for c in 0..4 {
                                rgba[gi * 4 + c] = (out[c].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                            }
                        }
                        None => {
                            // The ball adds nothing here — assign the FROZEN plane, undoing any stale write.
                            cov[gi] = pre_cover[gi];
                            mat[gi] = pre_mats[gi];
                            rgba[gi * 4..gi * 4 + 4].copy_from_slice(&pre_rgba[gi * 4..gi * 4 + 4]);
                        }
                    }
                    if cov[gi] != old_c || rgba[gi * 4..gi * 4 + 4] != old_r {
                        let rr = Region { x, y, w: 1, h: 1 };
                        matter_moved =
                            Some(matter_moved.map_or(rr, |acc| super::union_region(acc, rr)));
                    }
                }
            }
            self.invalidate_composite();
        }

        // Dirty exactly where the picture changed — the height's `moved` sliver ∪ the matter's `matter_moved`.
        let dirty = match (moved, matter_moved) {
            (Some(a), Some(b)) => Some(super::union_region(a, b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        if let Some(g) = dirty.and_then(|m| grow_region(m, 1, w, h)) {
            self.mark_dirty(g);
        }
    }
}
