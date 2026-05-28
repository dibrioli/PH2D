//! Q16.16 fixed-point coordinate helpers — opt-in determinism per
//! [ADR-0056 §2.7](../../../../docs/architecture/decisions/0056-vector-network-data-model.md).
//!
//! When [`crate::VectorNetwork::deterministic`] is `true`, callers must
//! quantize vertex / tangent coordinates through these helpers before
//! comparing across machines. f32 is bit-identical *enough* for the
//! happy path, but FMA contraction (Apple Silicon vs x86_64 with AVX2
//! vs WebAssembly SIMD) introduces 1-ULP drift in chained multiply-adds.
//! Quantizing to a fixed-point grid (1 unit = 1 / 65 536 px) erases that
//! drift.
//!
//! Round-trip semantics: `unquantize(quantize(x))` is exactly the
//! grid-snapped value of `x`, NOT `x` itself. Callers expecting
//! full-precision round-trip should not use these helpers.

/// One Q16.16 grid step in `f32` units (`1 / 65 536`).
pub const Q16_16_STEP: f32 = 1.0 / 65_536.0;

/// Quantize an `f32` to the Q16.16 fixed-point grid, returning an `i32`.
///
/// Round-to-nearest. Inputs outside `±32 768.0` saturate.
#[inline]
#[must_use]
pub fn quantize(x: f32) -> i32 {
    let scaled = (x * 65_536.0).round();
    if scaled >= i32::MAX as f32 {
        i32::MAX
    } else if scaled <= i32::MIN as f32 {
        i32::MIN
    } else {
        scaled as i32
    }
}

/// Convert a quantized `i32` back to `f32`.
#[inline]
#[must_use]
pub fn unquantize(q: i32) -> f32 {
    q as f32 * Q16_16_STEP
}

/// Snap an `f32` to the nearest Q16.16 grid point. Convenience for
/// callers that need a quantized `f32` (rather than an `i32`).
#[inline]
#[must_use]
pub fn snap(x: f32) -> f32 {
    unquantize(quantize(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_unquantize_zero_round_trips() {
        assert_eq!(unquantize(quantize(0.0)), 0.0);
    }

    #[test]
    fn quantize_one_pixel_is_65536() {
        assert_eq!(quantize(1.0), 65_536);
        assert_eq!(unquantize(65_536), 1.0);
    }

    #[test]
    fn snap_to_grid_is_idempotent() {
        // Arbitrary non-grid-aligned f32; we just need a value that
        // snap rounds to a Q16.16 grid point — using std::f32::consts::PI
        // here would tempt clippy to flag PI-as-approximation lint
        // (irrelevant to the test).
        let x = 12.345_678_f32;
        let s = snap(x);
        assert_eq!(snap(s), s);
    }

    #[test]
    fn saturates_at_extremes() {
        assert_eq!(quantize(f32::MAX), i32::MAX);
        assert_eq!(quantize(f32::MIN), i32::MIN);
    }
}
