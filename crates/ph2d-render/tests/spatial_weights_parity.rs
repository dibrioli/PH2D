//! Single-source-of-truth lock for the spatial-blur kernel weights.
//!
//! `ph2d-render`'s `LayerCompositor` keeps its OWN `gaussian_weights` /
//! `motion_weights` ON PURPOSE: it is a foundational render crate that speaks raw
//! scalar/`u8` params and must NOT gain a *production* dependency on the painter
//! tool's domain crate `ph2d-painter-brush` (the documented decoupling — see this
//! crate's `Cargo.toml` dev-dep note: "production `LayerCompositor` speaks raw
//! `u8` blend codes (no enum coupling) … so this is acyclic").
//!
//! The painter impl owns the CANONICAL kernel math in
//! `ph2d_painter_brush::adjustments` (`apply_gaussian` / `apply_sharpen` / …).
//! Rather than couple the libs by re-exporting it (which would break the
//! decoupling), this DEV-test pins the two copies bit-identical: any drift in
//! either crate fails CI, so the duplication can never silently diverge — the
//! benefit of a single source without the coupling cost.
//!
//! `assert_eq!` on `Vec<f32>` is an exact (bit) comparison; the two functions are
//! line-for-line identical code, so equality is exact (no tolerance) and a NaN
//! could only arise from a real bug, which the test would correctly surface.

use ph2d_painter_effects::adjustments::{
    gaussian_weights as brush_gaussian, motion_weights as brush_motion,
};
use ph2d_render::{gaussian_weights as render_gaussian, motion_weights as render_motion};

#[test]
fn gaussian_weights_match_canonical_painter_brush() {
    // Span: zero, sub-pixel, integer, fractional, large, and the MAX_BLUR_HALF cap.
    for &r in &[
        0.0f32, 0.25, 0.5, 1.0, 2.7, 4.0, 8.0, 15.3, 64.0, 255.0, 300.0,
    ] {
        let (rw, rh) = render_gaussian(r);
        let (bw, bh) = brush_gaussian(r);
        assert_eq!(rh, bh, "gaussian half mismatch at radius {r}");
        assert_eq!(rw, bw, "gaussian weights mismatch at radius {r}");
    }
}

#[test]
fn motion_weights_match_canonical_painter_brush() {
    for &d in &[0.0f32, 0.5, 1.0, 3.0, 9.0, 20.5, 128.0, 511.0, 1000.0] {
        let (rw, rh) = render_motion(d);
        let (bw, bh) = brush_motion(d);
        assert_eq!(rh, bh, "motion half mismatch at distance {d}");
        assert_eq!(rw, bw, "motion weights mismatch at distance {d}");
    }
}
