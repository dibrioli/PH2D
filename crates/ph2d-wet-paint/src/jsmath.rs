//! JavaScript numeric semantics, spelled out.
//!
//! The reference engine is JS: every number is an f64, bitwise operators work
//! on 32-bit lanes, and the rounding conventions differ from Rust's in exactly
//! the places a port silently diverges. Every helper here exists because the
//! straight Rust spelling computes something ELSE:
//!
//! - `Math.round` rounds half toward +infinity (`Math.round(-2.5) == -2`);
//!   Rust's `f64::round` rounds half away from zero (`(-2.5f64).round() == -3`).
//! - `| 0` is ToInt32: truncate toward zero, wrap mod 2^32, reinterpret signed.
//! - `x & mask` first runs ToInt32 on both sides, so a negative lattice
//!   coordinate wraps two's-complement (`-1 & 511 == 511`) — which is what
//!   makes the tile samplers seamless.
//! - `Math.imul` is a wrapping 32-bit multiply.
//! - A `Float32Array` store rounds to nearest, ties to even — identical to
//!   Rust `as f32`, so plain casts are correct for array writes.
//! - A `Uint8ClampedArray` store (the renderer's output) clamps to [0, 255]
//!   and rounds ties to EVEN — see [`clamp_u8`].

/// `Math.round`: round half toward +infinity.
#[inline]
pub fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// `value | 0` (ToInt32) for values already within i32 range — every call
/// site in the engine passes coordinates or small counters, never anything
/// near 2^31, so a plain truncating cast is the whole semantic.
#[inline]
pub fn to_i32(v: f64) -> i32 {
    v as i32
}

/// `x & mask` where `x` may be negative (JS ToInt32 + two's-complement AND).
/// `mask` must be a power of two minus one.
#[inline]
pub fn wrap_mask(x: i32, mask: i32) -> usize {
    (x & mask) as usize
}

/// `Math.imul` on the u32 bit patterns (identical bits to JS's signed result).
#[inline]
pub fn imul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

/// A `Uint8ClampedArray` store: clamp to [0, 255], round ties to even.
#[inline]
pub fn clamp_u8(v: f64) -> u8 {
    let c = v.clamp(0.0, 255.0);
    c.round_ties_even() as u8
}

/// `Math.min(1, Math.max(0, v))` — the engine's clamp01.
#[inline]
pub fn clamp01(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_round_halves_go_toward_positive_infinity() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(-2.6), -3.0);
        assert_eq!(js_round(0.4), 0.0);
    }

    #[test]
    fn wrap_mask_wraps_negatives_like_js() {
        assert_eq!(wrap_mask(-1, 511), 511);
        assert_eq!(wrap_mask(-3, 127), 125);
        assert_eq!(wrap_mask(514, 511), 2);
    }

    #[test]
    fn clamp_u8_is_uint8clamped() {
        assert_eq!(clamp_u8(-4.0), 0);
        assert_eq!(clamp_u8(300.0), 255);
        // Ties to even, per the Uint8ClampedArray conversion.
        assert_eq!(clamp_u8(0.5), 0);
        assert_eq!(clamp_u8(1.5), 2);
    }
}
