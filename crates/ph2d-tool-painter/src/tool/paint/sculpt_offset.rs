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
                for &(dx, dy, dq) in &disc {
                    let (px, py) = (qx + dx, qy + dy);
                    if px < 0 || px >= sw || py < 0 || py >= sh {
                        continue;
                    }
                    let pi = (py * sw + px) as usize;
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
                    let win = if dilate { v > best } else { v < best };
                    if win {
                        best = v;
                        bdx = dx;
                        bdy = dy;
                    }
                }
                hrow[rx] = best;
                srow[rx] = pack_src(bdx, bdy);
            }
        });
    (hbuf, sbuf)
}
