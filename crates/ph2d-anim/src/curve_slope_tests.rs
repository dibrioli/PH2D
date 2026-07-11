//! Tests for [`super`]'s slope machinery (`Interp::slope` + the CSS
//! endpoint cascade) — extracted to a sibling module (`#[path]`) under
//! the workspace file cap. Pure relocation.
use super::*;
use crate::easing::{EasingFamily, EasingMode};

/// Central difference of `remap` — the ground truth the analytic slope must
/// reproduce (`remap` is what the runtime plays).
fn fd(interp: Interp, u: f64) -> f64 {
    let h = 1e-5;
    (interp.remap(u + h) - interp.remap(u - h)) / (2.0 * h)
}

#[test]
fn the_slope_is_the_derivative_of_the_remap_that_plays() {
    let cases = [
        Interp::Linear,
        Interp::Hold,
        Interp::bezier(0.42, 0.0, 0.58, 1.0), // CSS ease-in-out
        Interp::bezier(0.2, 1.4, 0.8, -0.3),  // overshoot both ends
        Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
    ];
    for interp in cases {
        for i in 1..100 {
            let u = f64::from(i) / 100.0;
            let (got, want) = (interp.slope(u), fd(interp, u));
            assert!(
                (got - want).abs() < 1e-3 + want.abs() * 1e-2,
                "{interp:?} at u = {u}: slope {got} vs finite-diff {want}"
            );
        }
    }
}

#[test]
fn bezier_endpoint_slopes_match_the_css_reference() {
    // CSS `ease` (0.25, 0.1, 0.25, 1.0): start = y1/x1 = 0.4; end has
    // P2 short of P3, so end = (y2-1)/(x2-1) = 0/(−0.75) = 0. Tolerance:
    // the chain rule reaches these through `3·p` products, one ULP off
    // the exact ratios.
    let ease = Interp::bezier(0.25, 0.1, 0.25, 1.0);
    assert!((ease.slope(0.0) - 0.4).abs() < 1e-12);
    assert!(ease.slope(1.0).abs() < 1e-12);
    // Hold is flat everywhere; Linear is the identity.
    assert_eq!(Interp::Hold.slope(0.0), 0.0);
    assert_eq!(Interp::Linear.slope(0.5), 1.0);
}

#[test]
fn a_coincident_control_point_falls_through_the_reference_cascade() {
    // P1 = (0, 0): the start tangent is the line toward P2 — the
    // `InitGradients` second branch (y2/x2).
    let i = Interp::bezier(0.0, 0.0, 0.5, 0.25);
    assert_eq!(i.slope(0.0), 0.5);
    // P2 = (1, 1): the end tangent falls through to P1.
    let i = Interp::bezier(0.5, 0.5, 1.0, 1.0);
    assert_eq!(i.slope(1.0), 1.0);
    // Both coincident on one side: the tangent degenerates to the chord.
    let i = Interp::bezier(0.0, 0.0, 1.0, 1.0);
    assert_eq!(i.slope(0.0), 1.0);
}

#[test]
fn a_vertical_tangent_reads_infinite_not_a_lie() {
    // P1 = (0, 1): the curve leaves the origin straight up — the value
    // moves instantly. The slope is the true limit (+∞), which display
    // code skips as non-finite; reporting 0 here would hide real motion.
    let i = Interp::bezier(0.0, 1.0, 0.58, 1.0);
    assert!(i.slope(0.0).is_infinite() && i.slope(0.0) > 0.0);
}
