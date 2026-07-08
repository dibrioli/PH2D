//! Serde shim for the frozen, non-serde [`AnimValue`] enum.
//!
//! `AnimValue` lives in `ph2d-vector-traits` (a frozen contract crate,
//! ADR-0056) and does **not** derive `Serialize`/`Deserialize` — and we cannot
//! add them there. So the persistable timeline types (`Key`, `Track`, `Clip`,
//! `CurveKey`, `AnimCurve`) route their value fields through this module via
//! `#[serde(with = "crate::anim_value_serde")]`, mapping to/from a local
//! serializable [`Repr`].

use glam::{Vec2, Vec3};
use ph2d_color::OklchColor;
use ph2d_vector_traits::AnimValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializable mirror of the six `AnimValue` variants. Kept in lockstep with
/// the frozen enum (`ph2d-vector-traits::anim_value`); a variant added there via
/// ADR-0056 cap-bump must be added here too.
#[derive(Serialize, Deserialize)]
enum Repr {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    /// OKLCH as `[l, c, h, a]`.
    Color([f32; 4]),
    Bool(bool),
    Enum(u32),
}

/// Serialize an [`AnimValue`] through [`Repr`].
pub fn serialize<S: Serializer>(value: &AnimValue, s: S) -> Result<S::Ok, S::Error> {
    let repr = match *value {
        AnimValue::Float(x) => Repr::Float(x),
        AnimValue::Vec2(p) => Repr::Vec2([p.x, p.y]),
        AnimValue::Vec3(p) => Repr::Vec3([p.x, p.y, p.z]),
        AnimValue::Color(c) => Repr::Color([c.l, c.c, c.h, c.a]),
        AnimValue::Bool(b) => Repr::Bool(b),
        AnimValue::Enum(e) => Repr::Enum(e),
        // `AnimValue` is `#[non_exhaustive]`: a future variant we don't yet know
        // falls back to `Float(0.0)`. Documented lossy path — revisit whenever
        // the frozen enum grows a variant (ADR-0056 cap-bump).
        _ => Repr::Float(0.0),
    };
    repr.serialize(s)
}

/// Deserialize an [`AnimValue`] through [`Repr`].
pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<AnimValue, D::Error> {
    Ok(match Repr::deserialize(d)? {
        Repr::Float(x) => AnimValue::Float(x),
        Repr::Vec2([x, y]) => AnimValue::Vec2(Vec2::new(x, y)),
        Repr::Vec3([x, y, z]) => AnimValue::Vec3(Vec3::new(x, y, z)),
        Repr::Color([l, c, h, a]) => AnimValue::Color(OklchColor::new(l, c, h, a)),
        Repr::Bool(b) => AnimValue::Bool(b),
        Repr::Enum(e) => AnimValue::Enum(e),
    })
}
