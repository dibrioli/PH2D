//! **Inflate — the Blob**: the relief offset by a BALL, and the MATTER that rides it.
//!
//! Split from [`super::sculpt_blur`] for the workspace file-LOC cap, along the seam the code already
//! argued for: the Blob is **not a blur and never was**. Its siblings there are one expression over a
//! memoised `blur(pre, r)` — a stroke-constant, cached per tile. The ball's radius rides the LIVE `amount`,
//! so it is not a stroke-constant, it cannot be memoised, and it recomputes every render
//! (`SculptFamily::Memo` is the *knob* family; the engine's family stopped coinciding with it in the
//! 2026-07-14 correction). Its engine already lives next door in [`super::sculpt_offset`]; this is the
//! render that drives it.
//!
//! It is also the ONE verb that moves MATTER. The other seven reshape `h` inside paint that is already
//! there; this one grows the form, and a form that grows onto bare canvas without taking its paint along
//! does not grow at all — the light weighs by coverage. So the ball answers two questions, not one: how
//! high, and **where did this come from** (the argmax). See [`super::sculpt_offset::ball_taper`] for the
//! law that keeps those two answers from disagreeing about where the form ends.

use super::impasto_ceiling::H_MAX;
use super::region::grow_region;
use super::{Region, impasto_settle};
use crate::tool::PainterTool;
use std::sync::Arc;

/// The stand-in "height" of an UNTOUCHED texel in the Blob's envelope — low enough (dilating; mirrored
/// high when eroding) that it can never win against any real source in any window this tool can build.
///
/// Why untouched texels must not compete at all: rooting their parabolas at their own ground (`g = pre`)
/// turns the Blob into an **ambient terrain dilation** — on sloped thick paint, ground the brush never
/// touched lifts its downhill neighbours by `max(m·d − a·d²)` wherever the window reaches, and the
/// window's edge (a rectangle) becomes a visible tonal shelf: Enio's 4th smoke (2026-07-15), the residual
/// square that survived the runaway fix. An untouched texel can RECEIVE matter; it has none to give.
///
/// The margin that keeps the sentinel a sentinel: a real source's worst envelope value is
/// `−H_MAX − a·diag²`; with `a ≤ 1/32` (ρ ≥ 1 texel ⇒ |Depth| ≥ 1/16) and a 4096² canvas that is
/// ≈ `−1.05e6` — ten times shy of this. // CLAMP-OK
const UNTOUCHED_G: f32 = 1.0e7;

impl PainterTool {
    /// **Inflate — the Blob** (Enio 2026-07-14, third smoke: *"não é inflate … puxa as faces na direção das
    /// normais usando para intensidade o falloff … estude o Blob do Blender"*).
    ///
    /// A morphological offset by a ball whose **radius follows the falloff**: at the brush centre the ball is
    /// full (`|Depth|·DEPTH_UNIT_PX` px), tapering to zero at the edge. That is the whole correction. The
    /// constant-radius ball this replaced fattened the form but left a **flat top** (a max filter plateaus
    /// where the ball fits), and it was faded in by `amount` afterward — so the centre read as a flat Layer
    /// raise with an inflate rim, the *"mistura de inflate com layer"*. With the radius IN the falloff, the
    /// centre is the sphere's own curvature (a rounded peak), the rim fattens, and the edge tapers to
    /// nothing — a rounded blob, which is what a height field can offer of Blender's Blob.
    ///
    /// ## Why it is not memoised
    ///
    /// The ball radius is `|Depth|·amount(source)`, and `amount` grows across the stroke — so the offset is
    /// not a stroke-constant and the tile memo (which caches a function of the frozen `pre` alone) cannot
    /// hold it. It recomputes each render, over the batch's rect grown by the ball's reach. That is `O(ρ²)`
    /// per texel, but a source with zero coverage contributes nothing (the disc is mostly empty near the
    /// rim), so the real cost is well under the worst case.
    ///
    /// ## The form fattens BEYOND the brush, so the window is grown
    ///
    /// A big-radius source near the centre pushes the boundary out past where the brush touched — that IS the
    /// fattening — so this writes to `rect` grown by the ball's reach, not just the touched texels. And the
    /// **matter follows** (coverage / material / colour), the way it must for a form that grows onto bare
    /// canvas (the round-2 lesson): the paint dilates with the relief.
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
        // A full-strength ball has spent its whole |Depth| by ⌈ρ√2⌉ — the farthest ANY source may reach
        // (weaker sources reach less; the post-pass holds each to its own budget). The write region covers
        // that support plus the blur's spread; the compute region additionally sees every source (≤ reach)
        // and every blur tap (≤ smooth) that a write-region texel consults, so the crop inside `kr` is
        // exactly what a whole-canvas run would have written there.
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
        // The parabola's PEAK field over the region: `pre ± |Depth|·amount`, the falloff as the height
        // budget (`+` inflates, `−` deflates) — for TOUCHED texels. An untouched texel stands as the
        // sentinel, not as its own ground: see [`UNTOUCHED_G`] for the shelf that rooting it at `pre`
        // painted. The curvature `a` matches the sphere of radius `|Depth|·unit` at its apex, so the
        // separable parabolic dilation is a rounded, fattening dome — the exact spherical ball's
        // `O(area·ρ²)` collapsed to `O(area)`, the difference between 73 ms/move and shippable.
        let mut g = vec![0.0f32; cw * ch];
        for ry in 0..ch {
            let gy = (cr.y as usize + ry) * wu + cr.x as usize;
            for rx in 0..cw {
                let a = amount[gy + rx].clamp(0.0, 1.0);
                g[ry * cw + rx] = if a <= 0.0 {
                    if dilate { -UNTOUCHED_G } else { UNTOUCHED_G }
                } else if dilate {
                    pre[gy + rx] + depth.abs() * a
                } else {
                    pre[gy + rx] - depth.abs() * a
                };
            }
        }
        let a_curv = 1.0 / (2.0 * depth.abs() * unit * unit); // 1/(2·|Depth|·unit²)
        let (mut hbuf, mut sbuf) =
            super::sculpt_offset::blob_dilate(&g, cr.w, cr.h, a_curv, dilate);
        // The ball has a RADIUS, and it is EACH SOURCE'S OWN. A parabola of peak `p` has spent itself
        // after `d² = p/a` — that is where that source's ball ends (`2ρ²·amount`, the full ρ√2 only at
        // full strength). Past it the parabola is flank fiction: a real sphere's side plunges vertically
        // and lands, the parabola glides on and keeps lifting neighbours out of nothing — on THICK paint
        // that skirt reaches a hundred texels and gets clipped by the write region into a hard shelf
        // (Enio's 3rd smoke), and even inside a global ρ√2 cap it kept shelving sloped flanks (the 4th).
        // The composed argmax carries the winner and how far its matter travelled, so holding every texel
        // to ITS winner's budget is one comparison — CIRCULAR (it bounds `dx²+dy²`, never each axis), so
        // it cannot itself draw a square.
        //
        // Two more facts close the ball's edge:
        // * **The taper.** The parabola equals the sphere only near the apex — the outer 30% of the reach
        //   (past `d² = R²/2`, the sphere's equator) is where the two diverge, so the lift is faded to
        //   ZERO across it. Without it the support's edge is a cliff of exactly the local slope × R: a
        //   circular wall standing on every thick flank. With it, the field meets `pre` continuously and
        //   the support's boundary is not paintable at all. The plane-offset physics is untouched: a
        //   supporting sphere contacts a slope `G` at `d = ρG/S < ρ`, always inside the untapered zone.
        // * **The self-floor.** A texel's own ball always lifts it by its own falloff budget
        //   (`pre ± |Depth|·amount`), whatever happens to a far winner in the taper — so the falloff dome
        //   the artist aims never dents. When the floor wins, the matter is the texel's own (`src = 0`):
        //   height and matter answer "where did this come from" identically.
        //
        // Clamped and sentinel-won texels fall back to `pre` — bit-for-bit, which is what makes "outside
        // the ball nothing moves" a byte-identity gate rather than a tolerance — and it all happens
        // BEFORE the smooth blur, so the blur's shoulder grows from a clean field.
        let full_reach2 = (2 * rho * rho) as f32;
        for i in 0..(cw * ch) {
            let (rx, ry) = (i % cw, i / cw);
            let gi = (cr.y as usize + ry) * wu + cr.x as usize + rx;
            let p0 = pre[gi];
            let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[i]);
            // The winner's own budget, read at the winner (always inside `cr` — the envelope only ever
            // picks from the window it was handed).
            let sgi = ((cr.y as i64 + ry as i64 + dy) as usize) * wu
                + (cr.x as i64 + rx as i64 + dx) as usize;
            let a_s = amount[sgi].clamp(0.0, 1.0);
            let d2 = (dx * dx + dy * dy) as f32;
            let reach2_s = full_reach2 * a_s;
            let own = depth.abs() * amount[gi].clamp(0.0, 1.0);
            // A disqualified winner — untouched (a sentinel), or past its own ball — contributes nothing:
            // its travel collapses to zero. It does NOT erase the texel's own ball (`own` below): a crest
            // whose min-envelope winner was some far valley must still dig by its own falloff budget, or
            // erosion under the brush's centre silently stops happening the moment the terrain around it
            // is interesting. Only a texel with NEITHER — no reachable winner, no touch of its own —
            // falls back to `pre`, bit for bit, before any arithmetic can move a sign bit.
            // 1 at the sphere's equator, 0 at the reach — through the ONE door, because the matter's
            // advection asks the same question 120 lines below and a second copy of this formula is how
            // the two came to disagree in the first place ([`super::sculpt_offset::ball_taper`]).
            let t = super::sculpt_offset::ball_taper(d2, reach2_s);
            if t <= 0.0 && own <= 0.0 {
                hbuf[i] = p0;
                sbuf[i] = 0;
                continue;
            }
            if dilate {
                let lifted = p0 + t * (hbuf[i].max(p0) - p0);
                let own_v = p0 + own;
                if own_v >= lifted {
                    hbuf[i] = own_v;
                    sbuf[i] = 0; // the floor won: the matter here is its own
                } else {
                    hbuf[i] = lifted;
                }
            } else {
                let lowered = p0 + t * (hbuf[i].min(p0) - p0);
                let own_v = p0 - own;
                if own_v <= lowered {
                    hbuf[i] = own_v;
                    sbuf[i] = 0;
                } else {
                    hbuf[i] = lowered;
                    if lowered >= p0 {
                        sbuf[i] = 0; // erosion that dug nothing advects nothing
                    }
                }
            }
        }
        if smooth > 0 {
            // The knob smooths the ABSOLUTE surface where the ball acted — not the lift alone (adding a
            // blurred lift back onto a sharp `pre` would exhume, along the old silhouette, an edge the
            // raw ball had buried under its advance), and never the whole window (blurring `pre` out to
            // the region's corners is the rectangular shelf again, wearing the knob). A support mask,
            // blurred by the SAME kernel, feathers between the two exactly as far as the blur itself can
            // see: `m = 1` (exact — a window of ones sums to its own count) keeps the pure blur, `m = 0`
            // (exact) keeps `pre` to the bit, so the byte-identity of the support survives the knob.
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
            let target = Arc::make_mut(entry);
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
        // with it (round-2 lesson). `max` on the coverage: the paint extends, it does not thin.
        //
        // …and it arrives at the **taper's** strength, which is the whole of Enio's 2026-07-16 smoke. The
        // height fades across the ball's outer flank; the matter used to be copied at FULL strength to the
        // last qualifying texel and then stop dead, so the silhouette the light draws (it weighs by
        // coverage) was cut binary at `d² = reach2_s` — a bound read at the winner of a DISCRETE argmax,
        // which is a Voronoi pattern, binarised. The staircase in the photographs was that pattern. Now
        // both ask [`ball_taper`], so the height and the matter cannot disagree about where the form ends.
        //
        // **The destination each of these composes onto is the FROZEN plane, never the live buffer.** The
        // freehand path re-renders the whole stroke from `pre` on every pointer-move batch with no restore
        // in between (`sculpt_session`), and that is only sound because every write here is an ASSIGNMENT
        // sourced from frozen state — a render is a fresh answer to *what does this stroke, at its current
        // amount, do to the canvas it started from*, exactly as the height's `target[gi] = f(pre, amount)`
        // is. Composing `over` the live pixel instead would let a texel be composed once per batch, so the
        // fringe would darken with how SLOWLY the artist dragged — the sampling-dependence this line has
        // already paid for twice (the bite's telescoping share; the relief's capsule).
        if matter_ok
            && let (Some(cov_e), Some(mat_e)) =
                (self.covers.get_mut(&layer), self.mats.get_mut(&layer))
            && cov_e.len() == n
            && mat_e.len() == n
        {
            let cov = Arc::make_mut(cov_e);
            let mat = Arc::make_mut(mat_e);
            let rgba = Arc::make_mut(&mut self.canvas_rgba);
            for y in kr.y..kr.y + kr.h {
                for x in kr.x..kr.x + kr.w {
                    let ci = ((y - cr.y) as usize) * cw + (x - cr.x) as usize;
                    let (dx, dy) = super::sculpt_offset::unpack_src(sbuf[ci]);
                    if dx == 0 && dy == 0 {
                        continue; // the matter here is its own — nothing dilated onto it
                    }
                    let (sx, sy) = (i64::from(x) + dx, i64::from(y) + dy);
                    if sx < 0 || sy < 0 || sx >= wi || sy >= hi {
                        continue;
                    }
                    let si = (sy as usize) * wu + (sx as usize);
                    let gi = (y as usize) * wu + (x as usize);
                    // How much of the ball got here — the SAME number, from the same door, that faded the
                    // height at this texel. `d2` is the travel the argmax recorded; the budget is the
                    // winner's own (`2ρ²·amount`, read at the winner), exactly as the post-pass reads it.
                    let t = super::sculpt_offset::ball_taper(
                        (dx * dx + dy * dy) as f32,
                        full_reach2 * amount[si].clamp(0.0, 1.0),
                    );
                    // COVERAGE is a PRESENCE: it maxes, it does not thin — the paint extends. Faded by the
                    // taper, so the silhouette's rim ramps out instead of being cut.
                    let v = (f32::from(pre_cover[si]) * t) as u8;
                    if v <= pre_cover[gi] {
                        continue; // the source is no more painted than here — leave it
                    }
                    cov[gi] = v;
                    // MATERIAL and COLOUR are IDENTITIES, so they compose `over` at the opacity that
                    // actually arrived (`t`) — the same operator, for the same reason, as the deposit's own
                    // merge (`impasto_live::commit_stroke_height`): the paint on top is the paint you see,
                    // and it replaces what is under it in proportion to how opaquely it covers it. At
                    // `t = 1` over an opaque source both reduce to the verbatim copy that shipped, bit for
                    // bit — which is what keeps the approved interior exactly where Enio approved it, and
                    // is gated as byte-identity.
                    let a8 = (t * 255.0 + 0.5) as u32;
                    for c in 0..mat[gi].len() {
                        let (old, new) = (u32::from(pre_mats[gi][c]), u32::from(pre_mats[si][c]));
                        mat[gi][c] = ((old * (255 - a8) + new * a8 + 127) / 255) as u8;
                    }
                    // The pigment through the pigment's own door — straight-alpha source-over, in the space
                    // `canvas_rgba` is already stored and blended in (`stamp_color_dynamic`'s recomposite).
                    // The arriving paint's opacity is its OWN alpha scaled by the taper; over bare canvas
                    // that lands the source's colour at alpha·t (the rim feathers), and over the opaque
                    // ground of a normal canvas it lands as a lerp toward the paint (the rim blends in).
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
            }
            self.invalidate_composite();
        }

        if let Some(m) = moved {
            if let Some(g) = grow_region(m, 1, w, h) {
                self.mark_dirty(g);
            }
        } else if matter_ok {
            // The matter can change with no height crossing `RELIEF_EPS` (a flat blob fattening sideways),
            // and that write went straight to `canvas_rgba`. Dirty the write region so it uploads.
            self.mark_dirty(kr);
        }
    }
}
