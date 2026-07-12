//! [`PropKind`] — the **general** enumeration of animatable properties the
//! timeline document can bind, plus its opaque-target mapping.
//!
//! `ph2d-anim` keeps [`AnimTarget`] meaningless (HR-8). `PropKind` is the
//! document-level *authority* on what a target means, and each per-system
//! resolver (sprite first, vector/painter/node later) interprets the subset it
//! knows. The sprite resolver lives in [`crate::sprite`] as [`SpriteProp`];
//! `PropKind::TranslationX ..= ScaleY` share their [`AnimTarget`] ids with it so
//! a track authored either way names the same target.

use ph2d_anim::AnimTarget;
use serde::{Deserialize, Serialize};

use crate::sprite::SpriteProp;

/// A property a [`crate::TargetBinding`] can drive, across every animatable
/// system. The `u64` discriminant is the opaque [`AnimTarget`] id (HR-8), so it
/// is a **frozen wire value** — only append new variants, never renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u64)]
pub enum PropKind {
    /// Sprite local translation X (meters).
    TranslationX = 0,
    /// Sprite local translation Y (meters).
    TranslationY = 1,
    /// Sprite rotation (radians, CCW from +X).
    Rotation = 2,
    /// Sprite local scale X.
    ScaleX = 3,
    /// Sprite local scale Y.
    ScaleY = 4,
    /// Sprite opacity — the alpha channel of `Sprite.tint` (`[0, 1]`).
    Opacity = 5,
    /// **Time remap** (W5, AE model): the timeline's own meta-property — a
    /// keyed curve mapping playhead time → the SOURCE time this entity's other
    /// tracks sample at (seconds → seconds). Slope < 1 is slow motion, > 1
    /// speeds up, flat freezes, negative slope reverses. Never writes a scene
    /// property (`as_sprite_transform` is `None`; the apply consumes it as the
    /// entity's clock) and never auto-keys (it is not in [`PropKind::ALL`],
    /// the pose list). Appended — the discriminant is a frozen wire value.
    TimeRemap = 6,
}

impl PropKind {
    /// The six SCENE properties, in authoring order — the pose the auto-key
    /// pass samples ([`crate::PoseSample`] is exactly this array's shape).
    /// [`PropKind::TimeRemap`] is deliberately absent: it is the timeline's
    /// own clock, not a scene value (the "+ Track" list adds it separately).
    pub const ALL: [PropKind; 6] = [
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::Rotation,
        PropKind::ScaleX,
        PropKind::ScaleY,
        PropKind::Opacity,
    ];

    /// The opaque [`AnimTarget`] a track uses to drive this property.
    #[must_use]
    pub const fn target(self) -> AnimTarget {
        AnimTarget::new(self as u64)
    }

    /// Recover a kind from its opaque target id, if it names a known property.
    #[must_use]
    pub const fn from_target(target: AnimTarget) -> Option<PropKind> {
        match target.get() {
            0 => Some(PropKind::TranslationX),
            1 => Some(PropKind::TranslationY),
            2 => Some(PropKind::Rotation),
            3 => Some(PropKind::ScaleX),
            4 => Some(PropKind::ScaleY),
            5 => Some(PropKind::Opacity),
            6 => Some(PropKind::TimeRemap),
            _ => None,
        }
    }

    /// The i18n key suffix for this property's label (`panel.timeline.prop.*`).
    /// Presentation strings are resolved by the panel (HR-15); this is the
    /// stable key, never a display string.
    #[must_use]
    pub const fn i18n_suffix(self) -> &'static str {
        match self {
            PropKind::TranslationX => "translation_x",
            PropKind::TranslationY => "translation_y",
            PropKind::Rotation => "rotation",
            PropKind::ScaleX => "scale_x",
            PropKind::ScaleY => "scale_y",
            PropKind::Opacity => "opacity",
            PropKind::TimeRemap => "time",
        }
    }

    /// The sprite-transform resolver's view of this kind, if it is one of the
    /// five `Transform` properties. `Opacity` returns `None` — it resolves to
    /// `Sprite.tint[3]`, not a `Transform` field (see [`crate::apply`]).
    #[must_use]
    pub const fn as_sprite_transform(self) -> Option<SpriteProp> {
        match self {
            PropKind::TranslationX => Some(SpriteProp::TranslationX),
            PropKind::TranslationY => Some(SpriteProp::TranslationY),
            PropKind::Rotation => Some(SpriteProp::Rotation),
            PropKind::ScaleX => Some(SpriteProp::ScaleX),
            PropKind::ScaleY => Some(SpriteProp::ScaleY),
            PropKind::Opacity | PropKind::TimeRemap => None,
        }
    }

    /// What a record-cleanup fit must know about this channel beyond its numbers
    /// ([`ph2d_anim::FitChannel`]) — the property's SEMANTICS, which live here
    /// with the property and not in the fit (which stays a pure numeric routine).
    ///
    /// [`PropKind::Rotation`] is **angular**: the rotate gizmo writes it through
    /// `atan2`, so a recorded spin arrives as a ±2π sawtooth and must be unwrapped
    /// or a two-turn spin reconstructs as a net rotation of zero.
    /// [`PropKind::Opacity`] is **bounded** to `[0, 1]` — the alpha of
    /// `Sprite.tint`; a least-squares cubic through a fade that settles on 1.0
    /// otherwise overshoots past it, which the graph editor draws.
    ///
    /// The rest are unbounded scalars. [`PropKind::TimeRemap`] never records (it
    /// is not in [`PropKind::ALL`], the auto-key pose list), so its value here is
    /// only the safe default.
    #[must_use]
    pub const fn fit_channel(self) -> ph2d_anim::FitChannel {
        match self {
            PropKind::Rotation => ph2d_anim::FitChannel::ANGLE,
            PropKind::Opacity => ph2d_anim::FitChannel::bounded(0.0, 1.0),
            PropKind::TranslationX
            | PropKind::TranslationY
            | PropKind::ScaleX
            | PropKind::ScaleY
            | PropKind::TimeRemap => ph2d_anim::FitChannel::LINEAR,
        }
    }

    /// How an **additive** clip lane combines with what is under it (ADR-0115).
    ///
    /// This is the distinction that Blender got wrong first and had to invent
    /// `COMBINE` to fix ([T47035]): "additive" cannot mean "add the number".
    /// Adding two scale clips of 1.0 gives **2.0** — double size, where the
    /// honest answer is *no change*. A channel whose neutral value is 1 and whose
    /// meaning is proportional composes by **ratio**, not by sum.
    ///
    /// [T47035]: https://developer.blender.org/T47035
    #[must_use]
    pub const fn algebra(self) -> Algebra {
        match self {
            // Position and angle: neutral 0, additive means "displace by".
            PropKind::TranslationX | PropKind::TranslationY | PropKind::Rotation => Algebra::Sum,
            // Scale and alpha: neutral 1, additive means "scale by".
            PropKind::ScaleX | PropKind::ScaleY | PropKind::Opacity => Algebra::Ratio,
            // Never stacked: it IS the clock (ADR-0115 R6). The value is a safe
            // default, not a semantic claim.
            PropKind::TimeRemap => Algebra::Sum,
        }
    }
}

/// How a channel composes with another value of itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algebra {
    /// Neutral 0. Additive contribution is a **difference** (`v - base`), applied
    /// by addition.
    Sum,
    /// Neutral 1. Additive contribution is a **ratio** (`v / base`), applied by
    /// multiplication. Scale, and alpha.
    Ratio,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_roundtrips_for_every_kind() {
        for k in PropKind::ALL {
            assert_eq!(PropKind::from_target(k.target()), Some(k));
        }
        assert_eq!(PropKind::from_target(AnimTarget::new(999)), None);
    }

    #[test]
    fn sprite_transform_ids_match_sprite_prop() {
        // The four+one transform kinds share their opaque id with SpriteProp,
        // so a track authored via either names the same target.
        for k in PropKind::ALL {
            if let Some(sp) = k.as_sprite_transform() {
                assert_eq!(sp.target(), k.target());
            }
        }
        assert!(PropKind::Opacity.as_sprite_transform().is_none());
    }
}
