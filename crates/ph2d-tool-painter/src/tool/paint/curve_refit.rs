//! **Closed-curve REFIT** — the research-grade Simplify (Enio 2026-07-05: "curvas simplificadas perfeitas,
//! com números de pontos bem reduzidos"). The industry-standard pipeline every serious vector tool uses
//! (Inkscape's node-editor simplify, paper.js `path.simplify()`, Illustrator's Simplify — all built on
//! Schneider, "An Algorithm for Automatically Fitting Digitized Curves", Graphics Gems 1990):
//!
//! 1. **Flatten** the curve to a dense spine (a closed ring of sample points).
//! 2. **Detect corners** (cusps) — windowed turn angle over the ring; corners are geometry the fit must
//!    never smooth away, so they become mandatory split points.
//! 3. **Piecewise least-squares cubic fit** (the brush crate's `fit_curve` = Schneider: least-squares +
//!    Newton reparameterisation + adaptive splitting) on each OPEN run between corners. Open runs avoid
//!    Bug #4 entirely (`fit_curve` degenerates on a start==end loop; every run here has distinct ends).
//! 4. **Handle kinds from the fit** — an interior fitted join is G1 (the split tangent is shared, so the
//!    two arms are collinear with independent least-squares lengths) → **Aligned**; a corner keeps its two
//!    independently-fitted arms → **Free**. `Symmetric` (equal arms) and `Vector` (arms at ⅓ toward the
//!    neighbours) would both *distort* a least-squares fit — the arm lengths carry the shape — which is why
//!    Illustrator smooth points are aligned-not-symmetric and its corners are free (the earlier
//!    Symmetric/Vector attempt read "quase certo, mas não bom").
//! 5. A ring with fewer than 3 cusps gets ARTIFICIAL seams at thirds of the ring (a closed assembly needs
//!    ≥ 3 anchors, and one cubic can swallow a half-ring); those seam anchors are re-smoothed to collinear
//!    (**Aligned**) afterwards — a circle refits to the minimal 3-anchor ring of 120° arcs.
//!
//! Transcendental-free (dot/cross + `sqrt`, HR-5). Free fns, called as `curve_refit::*`.

use super::curve_geom::{dist2, flatten_spine};
use super::curve_handle::HandleKind;
use ph2d_painter_brush::fit_curve;

/// Turn threshold for a CORNER on the dense ring: `cos(angle between the ±window chords) ≤ 0.35` ⇒ ~70°+.
/// High enough that a tight-but-smooth bend (fitted arc) stays smooth; a real cusp (rectangle corner, the
/// merged peanut waist) always exceeds it.
const CORNER_COS: f32 = 0.35;
/// Arc length (px) of the chord window each side of a sample when measuring its turn — spans the ~1px
/// staircase noise of a traced contour without blurring a genuine cusp.
const CORNER_WINDOW_PX: f32 = 3.0;
/// Minimum arc separation (px) between two accepted corners (non-max suppression radius). MUST exceed the
/// full corner-response span (`2 × CORNER_WINDOW_PX`): every sample whose window straddles a vertex reads a
/// big turn, so one vertex answers over ~6px of arc — a 6px radius accepted BOTH ends (a doubled corner).
const CORNER_SUPPRESS_PX: f32 = 8.0;
/// Arc (px) trimmed from each side of a REAL corner before fitting: the mask-trace smoothing rounds ~2px of
/// cusp tip, and an offset AMPLIFIES that rounding by |d| (a 2px-radius tip offset 20px out reads as a 22px
/// round — Enio 2026-07-05 "arredonda as quinas no offset"). Dropping the rounded tip and re-anchoring the
/// runs on the true EDGE-LINE INTERSECTION restores a razor cusp, so the offset miters sharp.
const CORNER_TRIM_PX: f32 = 3.0;
/// The escalating-fit base error (px): the FIRST Simplify press fits within this of the current shape.
const REFIT_BASE_ERR_PX: f32 = 0.5;
/// Escalation cap (px): a press stops raising the fit tolerance here even if the target count wasn't hit.
const REFIT_MAX_ERR_PX: f32 = 32.0;

/// A refitted closed curve: anchors + fitted `[in, out]` handles + per-anchor kinds (`Aligned` smooth joins,
/// `Free` corners — the only two kinds a least-squares fit produces).
pub(super) struct RefitOut {
    pub points: Vec<[f32; 2]>,
    pub handles: Vec<[[f32; 2]; 2]>,
    pub kinds: Vec<HandleKind>,
}

/// **Progressive Simplify** (Enio: each press keeps shedding points): escalate the fit tolerance from
/// [`REFIT_BASE_ERR_PX`] until the anchor count drops to `keep_fraction` of the current count (or the
/// escalation caps out) — so a press always reduces when a reduction exists, and the result is always the
/// best least-squares fit at the accepted tolerance. `None` when the curve is too short or nothing reduces
/// (e.g. a rectangle already at its 4 corner anchors).
pub(super) fn refit_progressive(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    keep_fraction: f32,
    min_points: usize,
) -> Option<RefitOut> {
    let n = points.len();
    if n < 4 {
        return None;
    }
    let target = ((n as f32 * keep_fraction).floor() as usize).max(min_points);
    let mut err = REFIT_BASE_ERR_PX;
    let mut best: Option<RefitOut> = None;
    while err <= REFIT_MAX_ERR_PX {
        if let Some(r) = refit_closed_curve(points, handles, err) {
            let m = r.points.len();
            best = Some(r);
            if m <= target {
                break;
            }
        }
        err *= 1.7;
    }
    best.filter(|r| r.points.len() < n) // only accept a real reduction
}

/// Refit an editable CLOSED curve within `max_err` px: flatten to the dense spine, then
/// [`refit_closed_spine`].
pub(super) fn refit_closed_curve(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    max_err: f32,
) -> Option<RefitOut> {
    let mut spine = Vec::new();
    flatten_spine(points, handles, true, &mut spine);
    if spine.len() >= 2 && dist2(spine[0], spine[spine.len() - 1]) < 1e-6 {
        spine.pop(); // drop the closing seam duplicate — the ring is implicit
    }
    refit_closed_spine(&spine, max_err)
}

/// Refit a dense CLOSED ring of sample points (a flattened curve or a traced contour) within `max_err` px:
/// corner-split + piecewise Schneider fit + kind assembly. The single quality funnel behind Simplify AND
/// Merge. `None` when the ring is degenerate (< 8 samples).
pub(super) fn refit_closed_spine(spine: &[[f32; 2]], max_err: f32) -> Option<RefitOut> {
    let n = spine.len();
    if n < 8 {
        return None;
    }
    let mut corners = corner_spine_indices(spine);
    // A closed assembly needs ≥ 3 anchors, and a single cubic can swallow a whole half-ring within
    // tolerance — so guarantee ≥ 3 split points by adding ARTIFICIAL seams at thirds of the ring (they are
    // NOT real corners: recorded in `seam_smooth` and re-smoothed to Aligned after assembly). A smooth loop
    // (circle) thus refits to the classic minimal 3-anchor ring of 120° arcs.
    let mut seam_smooth: Vec<usize> = Vec::new();
    if corners.is_empty() {
        corners.push(0); // no cusps at all — index 0 is the first artificial seam
        seam_smooth.push(0);
    }
    if corners.len() < 3 {
        let base = corners[0];
        for add in [n / 3, (2 * n) / 3] {
            let s = (base + add) % n;
            if !corners.contains(&s) {
                corners.push(s);
                seam_smooth.push(s);
            }
        }
        if corners.len() < 3 {
            return None; // degenerate ring (all seams collided)
        }
        corners.sort_unstable();
    }
    // Reconstruct each REAL corner's true vertex (edge-line intersection past the rounded tip); a synthetic
    // seam keeps its sample verbatim.
    let real: Vec<bool> = corners.iter().map(|c| !seam_smooth.contains(c)).collect();
    let verts: Vec<[f32; 2]> = corners
        .iter()
        .enumerate()
        .map(|(idx, &c)| {
            if real[idx] {
                corner_vertex(spine, c)
            } else {
                spine[c]
            }
        })
        .collect();
    // Fit each open run between consecutive corners (wrapping) with the least-squares Schneider fit. At a
    // REAL corner end, trim the rounded tip ([`CORNER_TRIM_PX`]) and re-anchor the run on the true vertex —
    // the fit's end tangent is then estimated over a ≥3px baseline of clean edge, not the smeared tip.
    let k = corners.len();
    let mut fits = Vec::with_capacity(k);
    for ci in 0..k {
        let a = corners[ci];
        let b = corners[(ci + 1) % k];
        let bi = (ci + 1) % k;
        let mut run: Vec<[f32; 2]> = ring_run(spine, a, b);
        if real[ci] {
            let cut = trim_count(&run, false, CORNER_TRIM_PX).min(run.len().saturating_sub(4));
            run.drain(..cut);
            run[0] = verts[ci];
        }
        if real[bi] {
            let cut = trim_count(&run, true, CORNER_TRIM_PX).min(run.len().saturating_sub(4));
            run.truncate(run.len() - cut);
            let last = run.len() - 1;
            run[last] = verts[bi];
        }
        if run.len() < 2 {
            return None; // two corners collapsed onto each other — degenerate ring
        }
        let fit = fit_curve(&run, max_err);
        if fit.anchors.len() < 2 {
            return None;
        }
        fits.push(fit);
    }
    // Assemble: each run contributes its anchors except the last (owned by the next run as its corner).
    let mut points = Vec::new();
    let mut handles = Vec::new();
    let mut kinds = Vec::new();
    for ci in 0..k {
        let fit = &fits[ci];
        let prev = &fits[(ci + k - 1) % k];
        let m = fit.anchors.len();
        for ai in 0..m - 1 {
            if ai == 0 {
                // The corner anchor: IN arm from the previous run's fitted end, OUT arm from this run's start.
                let in_h = prev.handles[prev.anchors.len() - 1][0];
                points.push(fit.anchors[0]);
                handles.push([in_h, fit.handles[0][1]]);
                kinds.push(HandleKind::Free);
            } else {
                // Interior fitted join: Schneider shares the split tangent → collinear arms → Aligned.
                points.push(fit.anchors[ai]);
                handles.push(fit.handles[ai]);
                kinds.push(HandleKind::Aligned);
            }
        }
    }
    if points.len() < 3 {
        return None;
    }
    // Re-smooth the artificial seam anchors (they are NOT real corners): make the two arms collinear along
    // the averaged tangent, preserving each arm's fitted length → a G1 join, kind Aligned.
    if !seam_smooth.is_empty() {
        for (i, &p) in points.iter().enumerate() {
            if kinds[i] == HandleKind::Free && seam_smooth.iter().any(|&s| spine[s] == p) {
                let [in_h, out_h] = handles[i];
                let t_in = sub(p, in_h);
                let t_out = sub(out_h, p);
                let dir = norm([t_in[0] + t_out[0], t_in[1] + t_out[1]]);
                if dir != [0.0, 0.0] {
                    let len_in = dist2(p, in_h).sqrt();
                    let len_out = dist2(out_h, p).sqrt();
                    handles[i] = [
                        [p[0] - dir[0] * len_in, p[1] - dir[1] * len_in],
                        [p[0] + dir[0] * len_out, p[1] + dir[1] * len_out],
                    ];
                }
                kinds[i] = HandleKind::Aligned;
            }
        }
    }
    Some(RefitOut {
        points,
        handles,
        kinds,
    })
}

/// The ring samples from index `a` to index `b` INCLUSIVE, walking forward with wrap-around.
fn ring_run(spine: &[[f32; 2]], a: usize, b: usize) -> Vec<[f32; 2]> {
    let n = spine.len();
    let len = if b > a { b - a } else { n - a + b };
    (0..=len).map(|i| spine[(a + i) % n]).collect()
}

/// How many samples to drop from the run's start (or end, `from_back`) to shed `arc` px of length.
fn trim_count(run: &[[f32; 2]], from_back: bool, arc: f32) -> usize {
    let n = run.len();
    let mut acc = 0.0f32;
    let mut cut = 0usize;
    while cut + 1 < n && acc < arc {
        let (a, b) = if from_back {
            (run[n - 1 - cut], run[n - 2 - cut])
        } else {
            (run[cut], run[cut + 1])
        };
        acc += dist2(a, b).sqrt();
        cut += 1;
    }
    cut
}

/// The TRUE vertex of the cusp at ring sample `c`: intersect the two EDGE lines fitted just past the rounded
/// tip (each line goes through the sample [`CORNER_TRIM_PX`] of arc away from the cusp, along the direction
/// measured over a further `2×` baseline — clean edge, no tip smear). Falls back to the cusp sample itself
/// when the edges are near-parallel or the intersection lands implausibly far (a noisy/blunt cusp).
fn corner_vertex(spine: &[[f32; 2]], c: usize) -> [f32; 2] {
    let n = spine.len();
    // Walk `arc` px from `c` (wrapping) and return that sample.
    let probe = |forward: bool, arc: f32| -> [f32; 2] {
        let mut acc = 0.0f32;
        let mut j = c;
        while acc < arc {
            let next = if forward {
                (j + 1) % n
            } else {
                (j + n - 1) % n
            };
            acc += dist2(spine[j], spine[next]).sqrt();
            j = next;
            if j == c {
                break;
            }
        }
        spine[j]
    };
    let pa = probe(false, CORNER_TRIM_PX);
    let pa2 = probe(false, CORNER_TRIM_PX * 3.0);
    let pb = probe(true, CORNER_TRIM_PX);
    let pb2 = probe(true, CORNER_TRIM_PX * 3.0);
    let d_in = norm(sub(pa, pa2)); // incoming edge direction (toward the cusp)
    let d_out = norm(sub(pb2, pb)); // outgoing edge direction (away from the cusp)
    let cross = d_in[0] * d_out[1] - d_in[1] * d_out[0];
    if cross.abs() < 0.05 {
        return spine[c]; // near-parallel edges — no reliable intersection
    }
    let dp = sub(pb, pa);
    let t = (dp[0] * d_out[1] - dp[1] * d_out[0]) / cross;
    let v = [pa[0] + d_in[0] * t, pa[1] + d_in[1] * t];
    if dist2(v, spine[c]).sqrt() > CORNER_TRIM_PX * 6.0 {
        return spine[c]; // implausibly far — keep the traced cusp
    }
    v
}

/// Detect CORNER (cusp) sample indices on the closed ring: at each sample measure the turn between the
/// backward and forward chords spanning [`CORNER_WINDOW_PX`] of arc, flag turns ≥ ~70°, then greedily keep
/// the sharpest with [`CORNER_SUPPRESS_PX`] arc separation (non-max suppression). Sorted ascending.
pub(super) fn corner_spine_indices(spine: &[[f32; 2]]) -> Vec<usize> {
    let n = spine.len();
    // Prefix arc lengths (wrapping): arc[i] = length from sample 0 to sample i; total = arc[n].
    let mut arc = Vec::with_capacity(n + 1);
    arc.push(0.0f32);
    for i in 0..n {
        arc.push(arc[i] + dist2(spine[i], spine[(i + 1) % n]).sqrt());
    }
    let total = arc[n];
    if total < CORNER_WINDOW_PX * 4.0 {
        return Vec::new();
    }
    // Walk from `i` by `±CORNER_WINDOW_PX` of arc to a chord endpoint (wrapping).
    let walk = |i: usize, forward: bool| -> [f32; 2] {
        let mut acc = 0.0f32;
        let mut j = i;
        while acc < CORNER_WINDOW_PX {
            let next = if forward {
                (j + 1) % n
            } else {
                (j + n - 1) % n
            };
            acc += dist2(spine[j], spine[next]).sqrt();
            j = next;
            if j == i {
                break; // tiny ring — walked all the way around
            }
        }
        spine[j]
    };
    let mut turns: Vec<(usize, f32)> = Vec::new(); // (index, 1 - cos_turn) above threshold
    for (i, &p) in spine.iter().enumerate() {
        let b = walk(i, false);
        let f = walk(i, true);
        let v1 = norm(sub(p, b));
        let v2 = norm(sub(f, p));
        if v1 == [0.0, 0.0] || v2 == [0.0, 0.0] {
            continue;
        }
        let cos = v1[0] * v2[0] + v1[1] * v2[1];
        if cos <= CORNER_COS {
            turns.push((i, 1.0 - cos));
        }
    }
    // Sharpest-first greedy accept with arc-separation suppression (min distance around the ring).
    turns.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut accepted: Vec<usize> = Vec::new();
    for (i, _) in turns {
        let ok = accepted.iter().all(|&j| {
            let d = (arc[i.max(j)] - arc[i.min(j)]).abs();
            d.min(total - d) >= CORNER_SUPPRESS_PX
        });
        if ok {
            accepted.push(i);
        }
    }
    accepted.sort_unstable();
    accepted
}

/// `a − b`.
fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/// Unit vector of `a`, or `[0,0]` when ~zero-length.
fn norm(a: [f32; 2]) -> [f32; 2] {
    let m = (a[0] * a[0] + a[1] * a[1]).sqrt();
    if m > 1e-6 {
        [a[0] / m, a[1] / m]
    } else {
        [0.0, 0.0]
    }
}
