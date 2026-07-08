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
}

impl PropKind {
    /// Every kind, in authoring order (matches the "+ Track" property list).
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
            PropKind::Opacity => None,
        }
    }
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
