//! Curves — per-channel display-space tone curves (monotone cubic-Hermite spline,
//! LUT-baked). Split out of the former monolithic `compute.rs` (pure move).

use super::*;

/// Fritsch–Carlson monotone tangent at control point `i` of a tone curve — the
/// Hermite slope used by the segments on either side. Clamped against the
/// adjacent secants so the spline stays MONOTONE (never overshoots past the
/// control points — a tone curve must not wiggle out of the points' range).
fn monotone_tangent(points: &[[f32; 2]], i: usize) -> f32 {
    let n = points.len();
    let secant = |a: usize, b: usize| {
        let dx = points[b][0] - points[a][0];
        if dx.abs() <= 1e-9 {
            0.0
        } else {
            (points[b][1] - points[a][1]) / dx
        }
    };
    // Raw tangent: endpoints use the one-sided secant, interior the average.
    let mut m = if i == 0 {
        secant(0, 1)
    } else if i == n - 1 {
        secant(n - 2, n - 1)
    } else {
        0.5 * (secant(i - 1, i) + secant(i, i + 1))
    };
    // Monotonicity clamp (Fritsch–Carlson sufficient condition |m| ≤ 3|d| per
    // adjacent secant; a flat or sign-flipping secant forces a flat tangent).
    let neighbors = [
        (i > 0).then(|| secant(i - 1, i)),
        (i + 1 < n).then(|| secant(i, i + 1)),
    ];
    for d in neighbors.into_iter().flatten() {
        if d == 0.0 {
            m = 0.0;
        } else {
            let r = m / d;
            if r < 0.0 {
                m = 0.0;
            } else if r > 3.0 {
                m = 3.0 * d;
            }
        }
    }
    m
}

/// Evaluate a tone curve `points` (normalized `[x, y]` in `0..=1`, sorted by x)
/// at display-space input `x`. Empty → the identity (`y = x`); one point → a
/// constant; two or more → a monotone cubic-Hermite spline ([`monotone_tangent`]).
/// Outside the point span the endpoints extend flat (Photoshop's clamp).
pub(crate) fn eval_curve(points: &[[f32; 2]], x: f32) -> f32 {
    match points.len() {
        0 => return x.clamp(0.0, 1.0),
        1 => return points[0][1].clamp(0.0, 1.0),
        _ => {}
    }
    let x = x.clamp(0.0, 1.0);
    let n = points.len();
    if x <= points[0][0] {
        return points[0][1].clamp(0.0, 1.0);
    }
    if x >= points[n - 1][0] {
        return points[n - 1][1].clamp(0.0, 1.0);
    }
    let mut i = 0;
    while i + 1 < n && points[i + 1][0] < x {
        i += 1;
    }
    let (x0, y0) = (points[i][0], points[i][1]);
    let (x1, y1) = (points[i + 1][0], points[i + 1][1]);
    let h = x1 - x0;
    if h <= 1e-9 {
        return y1.clamp(0.0, 1.0); // coincident control points — take the later
    }
    let m0 = monotone_tangent(points, i);
    let m1 = monotone_tangent(points, i + 1);
    let t = (x - x0) / h;
    let (t2, t3) = (t * t, t * t * t);
    // Cubic Hermite basis.
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    (h00 * y0 + h10 * h * m0 + h01 * y1 + h11 * h * m1).clamp(0.0, 1.0)
}

/// Sample a tone curve `points` (normalized `[x, y]`, ascending x) at `x`,
/// returning the output `y`. Public entry to the monotone spline eval
/// ([`eval_curve`]) for the tool — e.g. placing a newly-inserted control point ON
/// the current curve so adding it leaves the output unchanged.
#[must_use]
pub fn curve_value_at(points: &[[f32; 2]], x: f32) -> f32 {
    eval_curve(points, x)
}

/// Per-channel display-space transfer LUTs for a [`CurvesParams`] — `[R, G, B]`,
/// each a 256-entry display→display table baking the master (RGB) curve composed
/// over the per-channel curve (`out = master(channel(in))`, Photoshop order).
/// **The GPU-mandate deliverable's math**: the compositor's `adj_luts` binding
/// uploads exactly these tables and the WGSL `ADJ_CURVES` case samples them, so
/// CPU [`apply_curves`] and the GPU read the SAME function.
#[must_use]
pub fn curves_display_luts(p: &CurvesParams) -> [[f32; DISPLAY_LUT_N]; 3] {
    let chans = [&p.points_r, &p.points_g, &p.points_b];
    core::array::from_fn(|c| {
        build_display_lut(|s| eval_curve(&p.points_rgb.points, eval_curve(&chans[c].points, s)))
    })
}

/// Curves — per-channel tone curve in DISPLAY space. Builds the per-channel LUTs
/// ([`curves_display_luts`], the same tables the GPU samples) once, then maps each
/// pixel via an sRGB round-trip. `acc` is straight LINEAR f32 RGBA (alpha
/// preserved). All-empty curves (the neutral default) early-return an exact
/// identity (skipping the round-trip — the hot-path win, mirror of [`apply_hsb`]).
pub(crate) fn apply_curves(p: &CurvesParams, acc: &mut [[f32; 4]]) {
    if p.points_rgb.points.is_empty()
        && p.points_r.points.is_empty()
        && p.points_g.points.is_empty()
        && p.points_b.points.is_empty()
    {
        return;
    }
    let luts = curves_display_luts(p);
    for px in acc.iter_mut() {
        for (ch, v) in px.iter_mut().take(3).enumerate() {
            let s = linear_to_srgb_f32(*v);
            *v = srgb_to_linear_f32(sample_display_lut(&luts[ch], s));
        }
    }
}
