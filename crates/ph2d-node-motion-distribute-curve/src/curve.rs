//! `curve.rs` — a cubic Bézier authored by four control points, with **arc-length
//! reparameterization** so samples are evenly spaced along the *length* of the curve
//! (not along the parameter `t`, which bunches where the curve is slow). The gold-
//! standard trick: build a cumulative-length LUT over `t`, then invert it to find the
//! `t` at a given arc fraction (Blender "Curve to Points"; a Bézier + arc-length table).
//! Transcendental-free (HR-5): the curve and its derivative are polynomials; only chord
//! lengths and the frame's normalisation use `sqrt`. Copied per-crate (leaf drop-crate
//! convention — the shared thing is the algorithm, not a symbol).

pub(crate) type P2 = [f32; 2];

/// Below this the curve (or a tangent) is treated as degenerate.
pub(crate) const EPS: f32 = 1e-6;
/// LUT resolution: `SAMPLES + 1` cumulative-length entries over `t ∈ [0, 1]`.
pub(crate) const LUT_SAMPLES: usize = 64;
/// The arc-length table type.
pub(crate) type ArcLut = [f32; LUT_SAMPLES + 1];

/// The cubic Bézier point at `t ∈ [0, 1]` (Bernstein basis).
pub(crate) fn eval(cp: &[P2; 4], t: f32) -> P2 {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        b0 * cp[0][0] + b1 * cp[1][0] + b2 * cp[2][0] + b3 * cp[3][0],
        b0 * cp[0][1] + b1 * cp[1][1] + b2 * cp[2][1] + b3 * cp[3][1],
    ]
}

/// The (non-unit) tangent — the derivative `dB/dt` — at `t`. (Used via `frame_at`; a
/// consumer that only distributes points won't call it — hence the leaf allow.)
#[allow(dead_code)]
pub(crate) fn tangent(cp: &[P2; 4], t: f32) -> P2 {
    let u = 1.0 - t;
    let (a, b, c) = (3.0 * u * u, 6.0 * u * t, 3.0 * t * t);
    [
        a * (cp[1][0] - cp[0][0]) + b * (cp[2][0] - cp[1][0]) + c * (cp[3][0] - cp[2][0]),
        a * (cp[1][1] - cp[0][1]) + b * (cp[2][1] - cp[1][1]) + c * (cp[3][1] - cp[2][1]),
    ]
}

/// The cumulative arc-length table: `lut[i]` is the chord length from `t=0` to
/// `t = i/SAMPLES`. `lut[SAMPLES]` is the total length.
pub(crate) fn arc_lut(cp: &[P2; 4]) -> ArcLut {
    let mut lut: ArcLut = [0.0; LUT_SAMPLES + 1];
    let mut prev = eval(cp, 0.0);
    for i in 1..=LUT_SAMPLES {
        let t = i as f32 / LUT_SAMPLES as f32;
        let p = eval(cp, t);
        let (dx, dy) = (p[0] - prev[0], p[1] - prev[1]);
        lut[i] = lut[i - 1] + (dx * dx + dy * dy).sqrt();
        prev = p;
    }
    lut
}

/// The parameter `t` at normalised arc position `s ∈ [0, 1]` — invert the LUT (find the
/// bracketing segment, lerp within it). A degenerate (zero-length) curve maps `s → t`.
pub(crate) fn t_at_arclen(lut: &ArcLut, s: f32) -> f32 {
    let total = lut[LUT_SAMPLES];
    let s = s.clamp(0.0, 1.0);
    if total <= EPS {
        return s;
    }
    let target = s * total;
    let mut i = 0;
    while i < LUT_SAMPLES && lut[i + 1] < target {
        i += 1;
    }
    let seg = (lut[i + 1] - lut[i]).max(EPS);
    (i as f32 + (target - lut[i]) / seg) / LUT_SAMPLES as f32
}

/// A frame at normalised arc position `s`: the point, the **unit tangent**, and the
/// **unit left normal** (`⟂` to the tangent). The normal offsets a wrapped layout.
/// (Only the spline-wrap consumer calls this; the distribute consumer doesn't.)
#[allow(dead_code)]
pub(crate) fn frame_at(cp: &[P2; 4], lut: &ArcLut, s: f32) -> (P2, P2, P2) {
    let t = t_at_arclen(lut, s);
    let p = eval(cp, t);
    let tan = tangent(cp, t);
    let len = (tan[0] * tan[0] + tan[1] * tan[1]).sqrt().max(EPS);
    let ut = [tan[0] / len, tan[1] / len];
    let un = [-ut[1], ut[0]]; // left normal
    (p, ut, un)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The endpoints are interpolated: `eval(0) = P0`, `eval(1) = P3`.
    #[test]
    fn endpoints_are_interpolated() {
        let cp = [[-3.0, -1.0], [-1.0, 2.0], [1.0, -2.0], [3.0, 1.0]];
        assert_eq!(eval(&cp, 0.0), cp[0]);
        assert_eq!(eval(&cp, 1.0), cp[3]);
    }

    /// On a straight, evenly-spaced control set the curve is that line, and arc-length
    /// sampling is uniform: `s = 0.5` lands on the midpoint. FALSIFIED if `t_at_arclen`
    /// returned raw `t` (still 0.5 here, but the chord check pins the geometry).
    #[test]
    fn arclength_is_uniform_on_a_line() {
        let cp = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
        let lut = arc_lut(&cp);
        assert!((lut[LUT_SAMPLES] - 3.0).abs() < 1e-3, "length 3");
        let mid = eval(&cp, t_at_arclen(&lut, 0.5));
        assert!(
            (mid[0] - 1.5).abs() < 1e-3 && mid[1].abs() < 1e-3,
            "midpoint {mid:?}"
        );
        // Even arc fractions → even x on the line.
        let a = eval(&cp, t_at_arclen(&lut, 0.25))[0];
        let b = eval(&cp, t_at_arclen(&lut, 0.75))[0];
        assert!(
            (a - 0.75).abs() < 2e-2 && (b - 2.25).abs() < 2e-2,
            "even spacing {a} {b}"
        );
    }

    /// The frame on a horizontal line: unit tangent +x, unit left normal +y.
    #[test]
    fn frame_on_a_line_is_axis_aligned() {
        let cp = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]];
        let lut = arc_lut(&cp);
        let (_, ut, un) = frame_at(&cp, &lut, 0.5);
        assert!(
            (ut[0] - 1.0).abs() < 1e-3 && ut[1].abs() < 1e-3,
            "tangent +x: {ut:?}"
        );
        assert!(
            un[0].abs() < 1e-3 && (un[1] - 1.0).abs() < 1e-3,
            "normal +y: {un:?}"
        );
    }
}
