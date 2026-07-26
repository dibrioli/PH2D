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
    /// **Morph** — the `t` of a `ph2d_ecs::VecMorph`: where along the path between its two source
    /// shapes the morphed shape sits (`0` = A, `1` = B). The Vector line's animatable channel, and
    /// the reason the Morph object exists at all: without a keyed `t` it is a slider, not
    /// animation.
    ///
    /// Outside [`PropKind::ALL`], like [`PropKind::TimeRemap`] and for the same reason: `ALL` is
    /// the sprite POSE that the auto-key samples ([`crate::PoseSample`] is exactly that array's
    /// shape), and `t` is not part of any sprite's pose. The artist keys it from the "+ Track"
    /// list. Appended — the discriminant is a frozen wire value.
    Morph = 7,
    /// **Position** — the object's place on the canvas as ONE channel, following an
    /// authored trajectory ([ADR-0141]): the After Effects model, where *Separate
    /// Dimensions precludes having Spatial Keyframes* and the two are therefore
    /// **modes**, not a mode plus a flag. Binding this kind IS choosing the path
    /// mode; [`PropKind::TranslationX`]/`Y` remain the separate-axes mode, and the
    /// conversion between them is an explicit, named operation.
    ///
    /// **The value of its track is DISTANCE ALONG THE PATH**, in world units — a
    /// plain scalar, which is what lets the graph editor, weighted tangents, the
    /// speed graph and roving keep working on it untouched. The trajectory itself
    /// lives on the binding ([`crate::TargetBinding::path`]), and key `i` of the
    /// track pairs with anchor `i` of that path.
    ///
    /// Outside [`PropKind::ALL`], like [`PropKind::TimeRemap`] and
    /// [`PropKind::Morph`]: `ALL` is the *separate-axes* sprite pose the auto-key
    /// samples ([`crate::PoseSample`] is exactly that array's shape), and a Position
    /// track is the alternative to two of its entries, never a seventh member of it.
    /// Appended — the discriminant is a frozen wire value.
    ///
    /// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
    Position = 8,
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
            7 => Some(PropKind::Morph),
            8 => Some(PropKind::Position),
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
            PropKind::Morph => "morph",
            PropKind::Position => "position",
        }
    }

    /// Resolve the tail of a **prop-link** identifier (`Name.<tail>`) to a kind —
    /// the ADR-0144 expression syntax. Artist-friendly aliases (`x`/`rot`/`sx`),
    /// case-insensitive. `TimeRemap` is deliberately absent (it is the timeline's
    /// meta-clock, not a scene value to read); a bare `time` is the playhead, so
    /// it is resolved by the pass's `Bindings`, not here.
    #[must_use]
    pub fn from_expr_name(name: &str) -> Option<PropKind> {
        match name.to_ascii_lowercase().as_str() {
            "x" | "translationx" | "tx" => Some(PropKind::TranslationX),
            "y" | "translationy" | "ty" => Some(PropKind::TranslationY),
            "rotation" | "rot" | "r" => Some(PropKind::Rotation),
            "scalex" | "sx" => Some(PropKind::ScaleX),
            "scaley" | "sy" => Some(PropKind::ScaleY),
            "opacity" | "alpha" | "a" => Some(PropKind::Opacity),
            "position" | "pos" | "p" => Some(PropKind::Position),
            "morph" | "m" => Some(PropKind::Morph),
            _ => None,
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
            // Position drives TWO Transform fields through a trajectory, so it is
            // not one `SpriteProp` — `crate::apply_path` is its resolver.
            PropKind::Opacity | PropKind::TimeRemap | PropKind::Morph | PropKind::Position => None,
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
    /// [`PropKind::Morph`] is **bounded** to `[0, 1]` for the same reason as `Opacity`, and it
    /// is not a coincidence: both are a fraction with two hard ends. A least-squares cubic through
    /// a morph that settles on B overshoots past `t = 1`, and the motor clamps there — so the
    /// graph editor would draw a curve that leaves the shape standing still, which reads as a bug
    /// in the easing rather than in the fit.
    ///
    /// The rest are unbounded scalars. [`PropKind::TimeRemap`] never records (it
    /// is not in [`PropKind::ALL`], the auto-key pose list), so its value here is
    /// only the safe default.
    #[must_use]
    pub const fn fit_channel(self) -> ph2d_anim::FitChannel {
        match self {
            PropKind::Rotation => ph2d_anim::FitChannel::ANGLE,
            PropKind::Opacity | PropKind::Morph => ph2d_anim::FitChannel::bounded(0.0, 1.0),
            PropKind::TranslationX
            | PropKind::TranslationY
            | PropKind::ScaleX
            | PropKind::ScaleY
            | PropKind::TimeRemap
            // Distance along a path. Bounded in principle by the path's length — but
            // that bound MOVES when an anchor does, and a fit that clamped to a
            // stale one would pin a recorded pose to the wrong end.
            | PropKind::Position => ph2d_anim::FitChannel::LINEAR,
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
            // The morph `t` is a POSITION along a path, not a proportion: neutral 0, and an
            // additive lane means "advance further along it". By ratio, two lanes at 0.3 and 0.5
            // would give 0.15 — less progress than either, which is not a thing anyone meant.
            PropKind::Morph => Algebra::Sum,
            // Distance travelled: neutral 0, and an additive lane means "go further
            // along it" — the Morph argument exactly, for the same kind of quantity.
            //
            // Blending DISTANCES is also what keeps a crossfade ON the trajectory:
            // halfway between "3 m along" and "7 m along" is "5 m along", still on
            // the curve, where blending the two POINTS would cut the corner off it.
            PropKind::Position => Algebra::Sum,
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
