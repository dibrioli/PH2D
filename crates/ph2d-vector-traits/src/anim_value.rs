//! `AnimValue` typed enum + `LinearInterp` trait.
//!
//! Six variants cover all animation use cases through W20:
//! - `Float(f32)`: scalar params (opacity, roughness amplitude, etc.)
//! - `Vec2(glam::Vec2)`: vertex positions, tangent handles
//! - `Vec3(glam::Vec3)`: 3D rotations (Vector → 3D bridge, future)
//! - `Color(ph2d_color::OklchColor)`: hue-stable color animation
//! - `Bool(bool)`: visibility toggles, mode flags
//! - `Enum(u32)`: variant index for `WindingRule`, `BooleanOp`, etc.
//!
//! Per [ADR-0056 §2.5](../../../../docs/architecture/decisions/0056-vector-network-data-model.md).

use glam::{Vec2, Vec3};
use ph2d_color::OklchColor;

/// Typed value returned by [`crate::AttributeEvaluator::sample`] /
/// [`crate::AnimationCurveSampler::at`].
///
/// Marked `#[non_exhaustive]` so future variant additions are
/// possible via ADR-0056 amendment (current cap = 6 variants;
/// `ph2d-vector-doc` arch-gate audits the count).
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum AnimValue {
    /// Scalar float (opacity, amplitude, t-param, etc.).
    Float(f32),
    /// 2D vector (positions, tangents, UV).
    Vec2(Vec2),
    /// 3D vector (rotation axes, 3D positions for the future Vector→3D bridge).
    Vec3(Vec3),
    /// Color in OKLCH polar form — hue-stable interpolation.
    Color(OklchColor),
    /// Boolean toggle.
    Bool(bool),
    /// Enum variant index (e.g. `WindingRule`, `BooleanOp` discriminants).
    Enum(u32),
}

/// Linear interpolation across the canonical `t ∈ [0.0, 1.0]` interval.
///
/// `t` is `f64` for precision over long sessions (per ADR-0056 §6 —
/// L1F1 Antigravity 3rd iteration).
///
/// Variant-mismatched lerp returns the second value (`b`) unchanged.
/// This is the canonical "incompatible types" fallback for state-machine
/// blends where a transition swaps the active value's type.
pub trait LinearInterp: Sized {
    /// Interpolate from `a` to `b` at parameter `t`.
    fn lerp(a: Self, b: Self, t: f64) -> Self;
}

impl LinearInterp for AnimValue {
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        let t32 = t as f32;
        match (a, b) {
            (AnimValue::Float(x), AnimValue::Float(y)) => AnimValue::Float(x + (y - x) * t32),
            (AnimValue::Vec2(x), AnimValue::Vec2(y)) => AnimValue::Vec2(x.lerp(y, t32)),
            (AnimValue::Vec3(x), AnimValue::Vec3(y)) => AnimValue::Vec3(x.lerp(y, t32)),
            (AnimValue::Color(x), AnimValue::Color(y)) => AnimValue::Color(lerp_oklch(x, y, t32)),
            (AnimValue::Bool(x), AnimValue::Bool(y)) => {
                if t < 0.5 {
                    AnimValue::Bool(x)
                } else {
                    AnimValue::Bool(y)
                }
            }
            (AnimValue::Enum(x), AnimValue::Enum(y)) => {
                if t < 0.5 {
                    AnimValue::Enum(x)
                } else {
                    AnimValue::Enum(y)
                }
            }
            (_, mismatched_b) => mismatched_b,
        }
    }
}

/// Hue-stable OKLCH lerp.
///
/// Lightness / chroma / alpha interpolate linearly. Hue takes the short arc
/// (handles 350° → 10° as 20° interpolation, not 340°).
fn lerp_oklch(a: OklchColor, b: OklchColor, t: f32) -> OklchColor {
    let l = a.l + (b.l - a.l) * t;
    let c = a.c + (b.c - a.c) * t;
    let alpha = a.a + (b.a - a.a) * t;

    let mut diff = b.h - a.h;
    if diff > 180.0 {
        diff -= 360.0;
    } else if diff < -180.0 {
        diff += 360.0;
    }
    let h = (a.h + diff * t).rem_euclid(360.0);

    OklchColor::new(l, c, h, alpha)
}
