//! Real Size — pure scale-reset logic.
//!
//! `std`-only, no editor/ECS coupling: operates on a plain `[f32; 2]`
//! (`[scale_x, scale_y]`) so this crate stays a leaf island. The desktop
//! shell reads the selected sprite's `Transform.scale` into this array,
//! calls [`real_size_scale`], and writes the result back.
//!
//! ### Contract the Implementer fills
//!
//! Reset each axis to unit scale **preserving the flip sign** — the
//! legacy `toggle_realsize` behaviour:
//!
//! ```text
//!   scale_x = if scale_x < 0.0 { -1.0 } else { 1.0 }
//!   scale_y = if scale_y < 0.0 { -1.0 } else { 1.0 }
//! ```
//!
//! Edge cases to pin with tests:
//! - already 1:1 (`[1.0, 1.0]`) → unchanged (caller skips the undo entry).
//! - flipped (`[-2.5, 1.0]`) → `[-1.0, 1.0]` (sign kept, magnitude reset).
//! - zero / NaN scale → decide + document a defined result (legacy used
//!   `Math.sign(x) || 1`, i.e. a `0` or `NaN` sign falls back to `+1`).

/// Reset a sprite's `[scale_x, scale_y]` to 1:1, preserving the flip sign
/// of each axis.
///
/// Each axis collapses to `±1.0`: a negative magnitude keeps its flip as
/// `-1.0`, everything else (positive, `0.0`, `NaN`) resets to `+1.0`. The
/// `< 0.0` test gives the legacy `Math.sign(x) || 1` fallback for free,
/// since both `0.0 < 0.0` and `NaN < 0.0` are `false`.
pub fn real_size_scale(scale: [f32; 2]) -> [f32; 2] {
    [unit_with_sign(scale[0]), unit_with_sign(scale[1])]
}

/// `-1.0` for a negative axis, `+1.0` otherwise (positive / zero / NaN).
#[inline]
fn unit_with_sign(axis: f32) -> f32 {
    if axis < 0.0 { -1.0 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_unit_is_unchanged() {
        assert_eq!(real_size_scale([1.0, 1.0]), [1.0, 1.0]);
    }

    #[test]
    fn flip_keeps_sign_resets_magnitude() {
        assert_eq!(real_size_scale([-2.5, 1.0]), [-1.0, 1.0]);
        assert_eq!(real_size_scale([1.0, -2.5]), [1.0, -1.0]);
        assert_eq!(real_size_scale([-4.0, -0.25]), [-1.0, -1.0]);
    }

    #[test]
    fn positive_magnitude_resets_to_unit() {
        assert_eq!(real_size_scale([3.0, 0.5]), [1.0, 1.0]);
    }

    #[test]
    fn zero_falls_back_to_positive_unit() {
        // 0.0 has no flip sign → legacy `Math.sign(0) || 1` == +1.
        assert_eq!(real_size_scale([0.0, 0.0]), [1.0, 1.0]);
        // -0.0 < 0.0 is false in IEEE-754, so negative zero also → +1.
        assert_eq!(real_size_scale([-0.0, -0.0]), [1.0, 1.0]);
    }

    #[test]
    fn nan_falls_back_to_positive_unit() {
        assert_eq!(real_size_scale([f32::NAN, f32::NAN]), [1.0, 1.0]);
        assert_eq!(real_size_scale([f32::NAN, -2.0]), [1.0, -1.0]);
    }
}
