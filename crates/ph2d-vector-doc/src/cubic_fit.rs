//! Levien Béz fitting — **stub** for W1.T1.4.
//!
//! Per [ADR-0056 §2.4](../../../../docs/architecture/decisions/0056-vector-network-data-model.md)
//! (canonical Spiro/Hyperbezier → Bézier cúbico conversion) + plan
//! task T1.4 (Levien algorithm, max error < 0.5 px on 5 fixtures).
//!
//! Reference:
//! <https://raphlinus.github.io/curves/2021/03/11/bezier-fitting.html>
//!
//! The T1.4 task replaces [`fit_cubic_levien`] with the real
//! implementation; the stub returns the input poly-segment unchanged so
//! W1 downstream callers (and the smoke Day-7 triangle path) compile
//! and execute without a missing-symbol stall.

use glam::Vec2;

/// Convert a poly-segment (a sequence of points sampled along a curve)
/// into a single cubic Bézier best-fit per Levien (2021).
///
/// **Stub W1.T1.2**: returns the first/last as endpoints with zero
/// tangents (degenerates to a straight line). T1.4 lands the real
/// fitter with `< 0.5 px` max error on the 5 canonical fixtures.
///
/// Returns `(start, ctrl_a, ctrl_b, end)` — the four cubic control points.
#[must_use]
pub fn fit_cubic_levien(samples: &[Vec2]) -> (Vec2, Vec2, Vec2, Vec2) {
    let n = samples.len();
    if n < 2 {
        let p = samples.first().copied().unwrap_or(Vec2::ZERO);
        return (p, p, p, p);
    }
    let start = samples[0];
    let end = samples[n - 1];
    // Stub: linear interpolation along the chord.
    let ctrl_a = start + (end - start) / 3.0;
    let ctrl_b = start + (end - start) * (2.0 / 3.0);
    (start, ctrl_a, ctrl_b, end)
}
