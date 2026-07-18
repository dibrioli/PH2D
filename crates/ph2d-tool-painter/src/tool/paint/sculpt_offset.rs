//! **Inflate — the Blob.** A morphological offset of the relief by a **ball**: at each source the surface is
//! raised by a hemisphere whose radius follows the falloff, and the envelope (the `max` over sources) is the
//! grown form. Plus the `(dx, dy)` source packing its matter advection carries.
//!
//! ## Why the TRUE ball, and not a separable parabola
//!
//! The Blob shipped for a year as a separable parabolic dilation (Felzenszwalb `O(N)`) — fast, but the
//! parabola has **unbounded support**: a source of height `H` wins a texel's envelope out to `√(H/a)` while
//! it can only *serve* out to `ρ√2`, and on paint thicker than the Depth (all real impasto) the gap between
//! the two is a dead zone. At a JUNCTION — the tallest point on the canvas — that dead zone is widest, and
//! the boundary of its Voronoi cell is a hard line across otherwise-uniform paint: **the white gash** of
//! Enio's cross (2026-07-16). Four containment layers (a sentinel, a per-source budget, a squared taper, a
//! self-floor) existed only to fence that unbounded support, and each was a thing the artist could see.
//!
//! A **bounded** ball has `capture == reach` by construction: it cannot claim what it cannot serve, so the
//! four fences have nothing to do and are gone. The exact ball is `O(area·ρ²)` — measured **44 ms/move** on
//! a big brush, far past the kill — but it is embarrassingly parallel (disjoint output rows, no reduction,
//! no RNG, byte-identical to the serial version: the same property ADR-0109 used to admit `rayon` for the
//! watercolor composite), and parallelised it is **3 ms/move** on the workstation. The proof that it removes
//! the gash on Enio's own cross is [`super::sculpt_tests::inflate_junction_probes`].

use rayon::prelude::*;

/// Pack a source offset `(dx, dy)` — where the matter at a texel came FROM — into one `u32`.
///
/// `(0, 0)` packs to `0`, which is what a texel whose own ball won gets, and which the render reads as
/// *nothing moved here*. The flat interior of every stroke is all zeros, so the advection costs nothing
/// where nothing happens.
#[inline]
pub(super) fn pack_src(dx: i64, dy: i64) -> u32 {
    (((dx as i16) as u16 as u32) << 16) | ((dy as i16) as u16 as u32)
}

/// The inverse of [`pack_src`].
#[inline]
pub(super) fn unpack_src(v: u32) -> (i64, i64) {
    (
        i64::from(((v >> 16) as u16) as i16),
        i64::from((v as u16) as i16),
    )
}

/// **The Blob's engine — an exact bounded-ball dilation, parallelised over output rows.**
///
/// For every texel `q` in the compute region `cr`, the grown height is
/// `max` over sources `p` within `p`'s own ball of `pre[p] + |Depth|·amount[p]·√(1 − d²/ρ_p²)`,
/// where the per-source radius is `ρ_p = ρ·amount[p]` texels (`ρ = |Depth|·unit`) — so a strong touch grows
/// a full ball and an untouched texel (`amount = 0`) grows none, which is the sentinel, for free. `dilate`
/// false mirrors it to a `min` (erosion, for negative Depth). Returns the height and the packed **argmax**
/// ([`pack_src`]) — where each texel's matter came from — so the paint can follow the relief.
///
/// The two axes are not the same unit: `d` is texels, the lift is loads, so the ball is an ellipsoid in
/// (texel, load) space — horizontal radius `ρ` texels, vertical radius `|Depth|` loads. The `√(1 − d²/ρ²)`
/// profile is dimensionless; the `|Depth|·amount` prefactor carries the load
/// ([[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]).
///
/// `pre`/`amount` are the whole-canvas frozen planes; `cr` is the sub-rectangle written. Reads are clamped
/// to the canvas, and the caller insets its WRITE region inside `cr` by the reach, so no written texel ever
/// consults a source outside `cr`.
#[allow(clippy::too_many_arguments)]
pub(super) fn blob_ball(
    pre: &[f32],
    amount: &[f32],
    w: u32,
    h: u32,
    cr: super::Region,
    depth: f32,
    unit: f32,
    dilate: bool,
) -> (Vec<f32>, Vec<u32>) {
    let rho = depth.abs() * unit;
    let r = rho.ceil() as i64;
    let mag = depth.abs();
    let inv_rho2 = 1.0 / (rho * rho);
    let (sw, sh) = (i64::from(w), i64::from(h));
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    // Precompute the DISC once: every offset inside the full ball (`d² ≤ ρ²`), carrying `dq = d²/ρ²`. This is
    // the hot loop's whole geometry — computed here, not per (texel, source). The reformulation folds the
    // per-source math to one subtract and one sqrt: a source at `dq` with falloff `a_p` lifts a texel by
    //   `|Depth|·a_p·√(1 − d²/(ρ·a_p)²) = |Depth|·√(a_p² − d²/ρ²) = mag·√(a_p² − dq)`  (in-ball ⟺ `a_p² > dq`),
    // with no per-pair divide and no int→float. It is what turned 18 ms/move into shippable. (The MATTER no
    // longer rides this fraction — it reads the smooth grown height + the closed footprint, `sculpt_close` —
    // so only the height's own lift uses it now.)
    let mut disc: Vec<(i64, i64, f32)> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            let dq = (dx * dx + dy * dy) as f32 * inv_rho2;
            if dq <= 1.0 {
                disc.push((dx, dy, dq));
            }
        }
    }
    // **Sorted by `dq` ascending — which is what turns the reach bound below into a `break`.** Walking the
    // disc from the centre outwards means the moment one offset is too far to contribute, every offset
    // after it is too. Unsorted, the same knowledge only buys a `continue`, and a `continue` still pays for
    // the load, the clamp and the multiply that a `break` skips entirely.
    //
    // **STABLE**, and that is load-bearing rather than tidy. The winner recorded in `sbuf` is the FIRST
    // offset to reach the maximum (the test is a strict `>`), so on an exact tie — which is the ordinary
    // case on a flat plateau of uniform `amount`, where every source at the same distance lifts by the
    // same float — the iteration order IS the answer. A stable sort keeps the original raster order
    // within each `dq`, so the matter still comes from the source it came from before.
    disc.sort_by(|a, b| a.2.total_cmp(&b.2));
    // `A(q)`: the largest `amount` anywhere in the texel's disc, as a SQUARE box-max (a superset of the
    // disc, so the bound is conservative and the result is unchanged to the bit).
    let abuf = box_max(amount, w, h, cr, r);
    // The same disc, as FLAT offsets into the canvas (`dy·w + dx`). For a texel far enough from every
    // edge that its whole disc lands inside, the source index is `qi + off` — no per-tap coordinate
    // arithmetic and no bounds test at all. Interior texels are almost the entire region, and those four
    // comparisons plus a multiply were being paid 30 million times a move to answer a question whose
    // answer is known for the whole row.
    let flat: Vec<(isize, f32)> = disc
        .iter()
        .map(|&(dx, dy, dq)| ((dy * sw + dx) as isize, dq))
        .collect();
    let mut hbuf = vec![0.0f32; cw * ch];
    let mut sbuf = vec![0u32; cw * ch];
    // Disjoint output rows: the parallelism ADR-0109's property allows. No reduction, no RNG.
    hbuf.par_chunks_mut(cw)
        .zip(sbuf.par_chunks_mut(cw))
        .enumerate()
        .for_each(|(ry, (hrow, srow))| {
            let qy = i64::from(cr.y) + ry as i64;
            for rx in 0..cw {
                let qx = i64::from(cr.x) + rx as i64;
                // The own floor is just the `d = 0` term of the same max: if `q` is touched it enters the
                // race as its own source; if not, `best` stays `pre[q]` and `src = 0` (its matter is its
                // own). No special case — the representation deleted it.
                let qi = (qy * sw + qx) as usize;
                let (mut best, mut bdx, mut bdy) = (pre[qi], 0i64, 0i64);
                // **The reach bound.** A source contributes only where `a_p² > dq` (that IS the in-ball
                // test below), and no `a_p` in this disc exceeds `A(q)`. So once the walk reaches
                // `dq >= A(q)²` every remaining offset is provably dead — including, for an untouched
                // neighbourhood (`A(q) = 0`), the whole disc on the first comparison.
                //
                // Measured on the kill fixture: 35% of texels have `A(q) = 0`, and only 32% of the 73M
                // taps could ever contribute. This bound is EXACT, not a heuristic — it deletes work the
                // old loop did and threw away, so the output is byte-identical
                // (`the_reach_bound_is_exact_the_ball_is_byte_identical_without_it`).
                let a2 = {
                    let a = abuf[ry * cw + rx];
                    a * a
                };
                // Interior ⟺ the whole disc lands on the canvas, so the clamp can never fire. Decided
                // ONCE per texel instead of four comparisons per tap; the two arms are otherwise the same
                // arithmetic in the same order, so they cannot disagree (pinned byte-for-byte by
                // `the_reach_bound_is_exact_…`, whose fixture straddles the border on purpose).
                let interior = qx >= r && qy >= r && qx < sw - r && qy < sh - r;
                if interior {
                    for (i, &(off, dq)) in flat.iter().enumerate() {
                        if dq >= a2 {
                            break;
                        }
                        let pi = (qi as isize + off) as usize;
                        let a_p = amount[pi].clamp(0.0, 1.0);
                        let arg = a_p * a_p - dq;
                        if arg <= 0.0 {
                            continue; // outside this source's own `ρ·a_p` ball — or the source has none
                        }
                        let lift = mag * arg.sqrt();
                        let v = if dilate {
                            pre[pi] + lift
                        } else {
                            pre[pi] - lift
                        };
                        if if dilate { v > best } else { v < best } {
                            best = v;
                            (bdx, bdy) = (disc[i].0, disc[i].1);
                        }
                    }
                } else {
                    for &(dx, dy, dq) in &disc {
                        if dq >= a2 {
                            break;
                        }
                        let (px, py) = (qx + dx, qy + dy);
                        if px < 0 || px >= sw || py < 0 || py >= sh {
                            continue;
                        }
                        let pi = (py * sw + px) as usize;
                        let a_p = amount[pi].clamp(0.0, 1.0);
                        let arg = a_p * a_p - dq;
                        if arg <= 0.0 {
                            continue;
                        }
                        let lift = mag * arg.sqrt();
                        let v = if dilate {
                            pre[pi] + lift
                        } else {
                            pre[pi] - lift
                        };
                        if if dilate { v > best } else { v < best } {
                            best = v;
                            bdx = dx;
                            bdy = dy;
                        }
                    }
                }
                hrow[rx] = best;
                srow[rx] = pack_src(bdx, bdy);
            }
        });
    (hbuf, sbuf)
}

/// **`A(q)` — the largest `amount` in the `(2r+1)²` box around every texel of `cr`**, clamped to `0..1`.
///
/// The bound [`blob_ball`]'s inner loop breaks on. A SQUARE box rather than the disc, deliberately: the
/// square contains the disc, so `A_box >= A_disc` and the break can only ever fire LATER than strictly
/// necessary — never earlier. A bound that is merely conservative keeps the result byte-identical, and the
/// square is the shape that separates.
///
/// Separable and `O(area)` via the chunked prefix/suffix trick (van Herk / Gil-Werman): within each block
/// of `k = 2r+1`, `pre` is the running max from the block's start and `suf` the running max to its end, and
/// any window of width `k` spans exactly two adjacent blocks — so its max is two comparisons, whatever `r`
/// is. The naive sliding window is `O(area·r)` and at the widest ball (`r = 16`) would spend a third of
/// what the bound saves.
///
/// **Both passes are ROW-WISE, and that is the whole of its speed.** The obvious vertical pass walks a
/// column at a time, which strides the cache by a row per step; measured, that plus a per-row allocation
/// cost **0.85 ms of the ball's 4.3** — a fifth of the very budget this function exists to protect. Written
/// as three sequential sweeps over whole rows, with the scratch allocated once, it disappears into the
/// noise. A helper that "just" reduces one line at a time is the readable shape and the wrong one.
pub(super) fn box_max(amount: &[f32], w: u32, h: u32, cr: super::Region, r: i64) -> Vec<f32> {
    let (sw, sh) = (i64::from(w), i64::from(h));
    let (cw, ch) = (cr.w as usize, cr.h as usize);
    let k = (2 * r + 1) as usize;
    let ru = r as usize;
    // The vertical pass reads horizontal results `r` rows beyond `cr` on each side, so the horizontal pass
    // covers that band. Rows and columns are clamped to the canvas, exactly as the ball's own reads are.
    let rows = ch + 2 * ru;
    let span = cw + 2 * ru;
    let mut horiz = vec![0.0f32; rows * cw];
    // Horizontal, parallel over rows: gather the clamped span, then reduce it in place.
    horiz
        .par_chunks_mut(cw)
        .enumerate()
        .for_each(|(row, out_row)| {
            let gy = (i64::from(cr.y) + row as i64 - r).clamp(0, sh - 1);
            let base = (gy * sw) as usize;
            let mut pre = vec![0.0f32; span];
            let mut suf = vec![0.0f32; span];
            for i in 0..span {
                let gx = (i64::from(cr.x) + i as i64 - r).clamp(0, sw - 1);
                let v = amount[base + gx as usize].clamp(0.0, 1.0);
                pre[i] = if i % k == 0 { v } else { pre[i - 1].max(v) };
            }
            for i in (0..span).rev() {
                let gx = (i64::from(cr.x) + i as i64 - r).clamp(0, sw - 1);
                let v = amount[base + gx as usize].clamp(0.0, 1.0);
                suf[i] = if i % k == k - 1 || i == span - 1 {
                    v
                } else {
                    suf[i + 1].max(v)
                };
            }
            for (x, o) in out_row.iter_mut().enumerate() {
                *o = suf[x].max(pre[x + k - 1]);
            }
        });
    // Vertical, the same reduction down the rows — but swept ROW-WISE so every access is sequential.
    let mut pre_v = vec![0.0f32; rows * cw];
    let mut suf_v = vec![0.0f32; rows * cw];
    for row in 0..rows {
        let (lo, hi) = (row * cw, (row + 1) * cw);
        if row % k == 0 {
            pre_v[lo..hi].copy_from_slice(&horiz[lo..hi]);
        } else {
            let (above, here) = pre_v.split_at_mut(lo);
            let prev = &above[lo - cw..];
            for (x, o) in here[..cw].iter_mut().enumerate() {
                *o = prev[x].max(horiz[lo + x]);
            }
        }
    }
    for row in (0..rows).rev() {
        let (lo, hi) = (row * cw, (row + 1) * cw);
        if row % k == k - 1 || row == rows - 1 {
            suf_v[lo..hi].copy_from_slice(&horiz[lo..hi]);
        } else {
            let (here, below) = suf_v.split_at_mut(hi);
            for (x, o) in here[lo..].iter_mut().enumerate() {
                *o = below[x].max(horiz[lo + x]);
            }
        }
    }
    let mut out = vec![0.0f32; cw * ch];
    for y in 0..ch {
        let (o, s) = (y * cw, y * cw);
        let p = (y + k - 1) * cw;
        for x in 0..cw {
            out[o + x] = suf_v[s + x].max(pre_v[p + x]);
        }
    }
    out
}
