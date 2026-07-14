//! **Inflate** — the morphological offset of the relief, and why the obvious formula is not it.
//!
//! ## The obvious formula, and how it shipped
//!
//! "Raise the surface along its normal by `d`" reads, in a height field, as `h + d · n_z` — a height field
//! can only move in `z`, so take the `z` component of the normal and scale by it. That is what this tool
//! shipped with, and it is wrong twice.
//!
//! **It is upside down.** Offset the graph `z = h(x, y)` along its unit normal by `d` and re-project the
//! result back into a height field, and the algebra comes out (to first order, exactly on a plane):
//!
//! ```text
//! (x, y, h) + d·n = (x − d·hₓ/S,  y − d·h_y/S,  h + d/S)          S = √(1 + |∇h|²)
//! ⇒ H(X, Y) = h(X, Y) + d·(hₓ² + h_y²)/S + d/S = h + d·S²/S = h + d·S
//! ```
//!
//! So the true offset raises by **`d·S = d / n_z`** — the *secant*, not the cosine. Steep places move
//! **more**, not less, and that is not a curiosity: it is precisely how a wall shifts sideways, which is
//! how a form gets *fatter*. The shipped formula did the opposite — it moved the steep places **less**,
//! which rounds a crest off. That is a smoothing. Inflate was a worse [`Smooth`](super::sculpt::SculptMode).
//!
//! And the gate that guarded it was called `inflate_rounds_the_crest_instead_of_translating_it`: the name
//! asserted the wrong intention, so the gate faithfully pinned the bug. A green gate proves the code does
//! what you *said*; nothing in it tells you what you said was wrong.
//!
//! ## Fixing the sign would not have fixed the tool
//!
//! This is the part that matters, and the measurement is what says so. Over the relief the **deposit
//! actually lays**, the median painted texel has `n_z = 1.000` (a stroke's interior is dead flat; the
//! settle blurs what little slope the rim has). So `d/n_z = d·n_z = d`, and **`Inflate` is `Layer`, to the
//! bit**, over every texel the artist is looking at. Enio's smoke found this in one stroke.
//!
//! The reason is structural rather than arithmetic: `h + d·S` is one explicit **Euler step** of the offset
//! PDE `∂h/∂t = √(1 + |∇h|²)`, and a single step of a hyperbolic PDE **cannot move material sideways**.
//! Sideways is the entire visible content of the word *inflate*.
//!
//! ## What it is instead
//!
//! The exact solution of that PDE at time `d` is the **morphological offset**: grayscale dilation (for
//! `d > 0`) or erosion (`d < 0`) of the relief by a **ball** of radius `d`. That operation is nonlocal by
//! construction — it is where the lateral growth comes from — and it is what an artist means:
//!
//! * a form gets **fatter**, not merely taller (its rim is pushed outward, up to a ball-radius);
//! * concave **creases fill in** (the max climbs over them);
//! * a negative Depth **erodes** — thin ridges are eaten away and the form shrinks.
//!
//! ## `Inflate` and `Layer` agree on a flat, and that is geometry, not a bug
//!
//! Offsetting a *plane* along its normal is a *translation*: dilation adds exactly `d` on flat ground,
//! whatever the algorithm. So over the flat interior of a stroke Inflate does raise by Depth, exactly as
//! Layer does — and Blender's Inflate is likewise Draw on a flat plane. The two tools differ in what they
//! do to the **shape**, and [`super::sculpt_tests`] gates that identity so that nobody "fixes" it back
//! into an inversion.

use super::Region;
use super::impasto_light::DEPTH_UNIT_PX;

/// How many independent accumulators the row reductions carry.
///
/// A `max` reduction into ONE variable is a **serial dependency**: every iteration waits on the previous
/// one's compare, so the loop runs at one tap per few cycles no matter how wide the machine is, and the
/// vectoriser will not touch it (reassociating floats is not something it may do on its own). Eight lanes,
/// combined at the end, break the chain — the ball's rows then reduce at memory speed instead of at latency
/// speed.
///
/// Measured, at the WIDEST Depth (a 16-px ball, ~800 taps per texel) on a 2048² canvas: the tuple-bag first
/// cut **15.9 ms/move**; held as contiguous rows, **8.7**; with the chain broken, **5.7**. The kill criterion
/// is 8, so the last of those three is not a polish — it is the verb existing. // CLAMP-OK
const LANES: usize = 8;

/// `max(s[i] + c[i])` over a row of the ball — the dilation's inner loop.
///
/// Split out (rather than inlined twice) so the erosion below is visibly the SAME loop with two operators
/// flipped. A `min` written out by hand next to a `max` is where a sign eventually goes missing.
#[inline]
fn row_max(s: &[f32], c: &[f32]) -> f32 {
    let mut acc = [f32::NEG_INFINITY; LANES];
    let mut si = s.chunks_exact(LANES);
    let mut ci = c.chunks_exact(LANES);
    for (sc, cc) in si.by_ref().zip(ci.by_ref()) {
        for k in 0..LANES {
            acc[k] = acc[k].max(sc[k] + cc[k]);
        }
    }
    let (rs, rc) = (si.remainder(), ci.remainder());
    for k in 0..rs.len() {
        acc[0] = acc[0].max(rs[k] + rc[k]);
    }
    acc.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

/// `min(s[i] − c[i])` over a row of the ball — the erosion's inner loop. The mirror of [`row_max`].
#[inline]
fn row_min(s: &[f32], c: &[f32]) -> f32 {
    let mut acc = [f32::INFINITY; LANES];
    let mut si = s.chunks_exact(LANES);
    let mut ci = c.chunks_exact(LANES);
    for (sc, cc) in si.by_ref().zip(ci.by_ref()) {
        for k in 0..LANES {
            acc[k] = acc[k].min(sc[k] - cc[k]);
        }
    }
    let (rs, rc) = (si.remainder(), ci.remainder());
    for k in 0..rs.len() {
        acc[0] = acc[0].min(rs[k] - rc[k]);
    }
    acc.iter().copied().fold(f32::INFINITY, f32::min)
}

/// Offset the height-field window `src` by a ball of **signed** radius `r_px`, writing the `tile` of it
/// (window-local coords) into `out` (`tile.w × tile.h`, row-major).
///
/// `r_px > 0` dilates (max-plus), `r_px < 0` erodes (min-plus), `0` is the identity.
///
/// ## The ball is a ball in the space the ARTIST sees
///
/// `x` and `y` are pixels; `h` is paint-loads; and one load stands [`DEPTH_UNIT_PX`] pixels tall — that
/// is the constant the **light** converts through, so it is the one the geometry is *seen* in. The
/// structuring function is therefore `√(ρ² − dx² − dy²)` **pixels**, converted back into loads on the way
/// out. Skip that conversion and the "ball" is an ellipsoid sixteen times too tall — which is the same
/// class of bug (a length in the wrong axis's unit) the Chisel shipped with, and the reason this comment
/// exists rather than the constant being inlined.
///
/// ## Why a tile's offset is bit-for-bit the canvas's offset
///
/// The same argument as the blur memo ([`super::sculpt_blur`]), and it has to be, because the caller is
/// the same tile loop. A texel's output reads `src` only inside its ball, so a tile computed through a
/// read window grown by `⌈ρ⌉` on every side and clipped to the canvas sees, for each of its texels,
/// exactly the taps a whole-canvas offset would have seen: either the window is untruncated on that side
/// (so the ball never reaches the window edge) or it is truncated *because that side is the canvas edge*
/// (so a tap missing from the window is a tap missing from the canvas). Taps out of the window are
/// skipped, and off-canvas taps are skipped by a whole-canvas run for the same reason.
pub(super) fn ball_offset_into(
    src: &[f32],
    sw: u32,
    sh: u32,
    r_px: f32,
    tile: Region,
    out: &mut [f32],
) {
    let rho = r_px.abs();
    let dilate = r_px > 0.0;
    let r2 = rho * rho;

    // ── The ball, sampled once per tile — as ROWS, not as a bag of offsets ──────────────────────────
    //
    // This is `O(ρ²)` taps per texel (1089 of them at the widest Depth), which is by far the heaviest
    // arithmetic in the Sculpt, so the LAYOUT is the algorithm. The first cut held the disc as a
    // `Vec<(dx, dy, cap)>` and bounds-checked every tap: 26 KB of tuples streamed per texel, an L1 miss on
    // most of them, and a branch the predictor cannot help with. It measured **15.9 ms/move** — twice the
    // kill criterion, and the perf gate caught it.
    //
    // Held as rows, the inner loop walks one contiguous slice of `src` against one contiguous slice of caps
    // and reduces with a `max`. No branch, no gather, and the compiler can vectorise it. Same taps, same
    // order, same answer — a different shape of the same sum.
    //
    // `sqrt` is IEEE-exact, not a transcendental (HR-5 is about `sin`/`cos`/`exp`/`pow`, whose last bit
    // moves between platforms), and it runs `O(ρ²)` times per TILE rather than per texel.
    let reach = rho.ceil() as i64;
    let mut rows: Vec<(i64, i64, usize)> = Vec::new(); // (dy, dx_half, offset into `caps`)
    let mut caps: Vec<f32> = Vec::new();
    for dy in -reach..=reach {
        let rem = r2 - (dy * dy) as f32;
        if rem < 0.0 {
            continue;
        }
        let half = rem.sqrt().floor() as i64;
        let off = caps.len();
        for dx in -half..=half {
            let d2 = (dx * dx + dy * dy) as f32;
            // `half` is floor(√rem), so `d2 ≤ r2` holds for every dx in the row — but floating point is not
            // obliged to agree with integer arithmetic about the boundary, and a negative under the root
            // would be a NaN that poisons every `max` it touches. Clamp, do not trust.
            caps.push((r2 - d2).max(0.0).sqrt() / DEPTH_UNIT_PX);
        }
        rows.push((dy, half, off));
    }
    // Row `dy = 0` always exists (`rem = r2 ≥ 0`) and always contains `dx = 0`, which is always inside the
    // window — so the running extremum below is never left at an infinity, and there is no empty-disc branch.

    let (sw_i, sh_i) = (i64::from(sw), i64::from(sh));
    let sw_us = sw as usize;
    for ty in 0..tile.h {
        let py = i64::from(tile.y + ty);
        for tx in 0..tile.w {
            let px = i64::from(tile.x + tx);
            let mut best = if dilate {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
            for &(dy, half, off) in &rows {
                let qy = py + dy;
                if qy < 0 || qy >= sh_i {
                    continue; // off the canvas: the paint simply does not extend there
                }
                // Clip the row's dx span to the window ONCE, then walk it with no test inside.
                let lo = (px - half).max(0);
                let hi = (px + half).min(sw_i - 1);
                if lo > hi {
                    continue;
                }
                let k = off + (lo - (px - half)) as usize; // where in the cap row `lo` lands
                let n = (hi - lo + 1) as usize;
                let base = (qy as usize) * sw_us + (lo as usize);
                let s = &src[base..base + n];
                let c = &caps[k..k + n];
                best = if dilate {
                    best.max(row_max(s, c))
                } else {
                    best.min(row_min(s, c))
                };
            }
            out[(ty as usize) * (tile.w as usize) + (tx as usize)] = best;
        }
    }
}
