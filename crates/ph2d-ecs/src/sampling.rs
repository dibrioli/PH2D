//! Texture sampling components — Sprite Inspector v2 W3 (spec
//! [`09_sampling_e_material.md`](../../../docs/Sprite_projeto/09_sampling_e_material.md)
//! §9.1–9.2). Both are **hierarchical**: an entity's value overrides
//! the inherited one; `Inherit` defers to the nearest ancestor that
//! sets it, falling back to the project default.
//!
//! Material & Blend (spec §9.4–9.7) are a *W4* deliverable and not
//! defined here.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// Per-node texture filter (Godot per-node filter, spec §9.1).
/// Hierarchical: `Inherit` reads the nearest ancestor override, then
/// the project default.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    /// Defer to the ancestor / project default (component default).
    #[default]
    Inherit,
    /// No filtering — ideal pixel-art.
    Nearest,
    /// Bilinear — ideal vector UI / smooth.
    Linear,
    /// Mipmapped, nearest within mip.
    NearestMipmap,
    /// Mipmapped, linear within mip (trilinear).
    LinearMipmap,
    /// Anisotropic + nearest.
    NearestAniso,
    /// Anisotropic + linear.
    LinearAniso,
}

/// Per-node texture filter override.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureFilter(pub FilterMode);

/// Per-node texture wrap mode (spec §9.2). Hierarchical like
/// [`FilterMode`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    /// Defer to the ancestor / project default.
    #[default]
    Inherit,
    /// Clamp to `[0, 1]`; outside pixels clamp the border.
    Disabled,
    /// Repeat tile (wrap).
    Enabled,
    /// Mirror-repeat (alternate).
    Mirror,
}

/// Per-node texture repeat override.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureRepeat(pub RepeatMode);

impl FilterMode {
    /// Resolve `self` against an inherited value: a concrete mode wins;
    /// `Inherit` defers to `inherited`. Used by the extract's
    /// ancestor walk.
    pub fn resolve(self, inherited: FilterMode) -> FilterMode {
        match self {
            FilterMode::Inherit => inherited,
            concrete => concrete,
        }
    }
}

impl RepeatMode {
    pub fn resolve(self, inherited: RepeatMode) -> RepeatMode {
        match self {
            RepeatMode::Inherit => inherited,
            concrete => concrete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_defaults_to_inherit() {
        assert_eq!(TextureFilter::default().0, FilterMode::Inherit);
        assert_eq!(TextureRepeat::default().0, RepeatMode::Inherit);
    }

    #[test]
    fn resolve_prefers_concrete_over_inherited() {
        assert_eq!(
            FilterMode::Inherit.resolve(FilterMode::Nearest),
            FilterMode::Nearest
        );
        assert_eq!(
            FilterMode::Linear.resolve(FilterMode::Nearest),
            FilterMode::Linear
        );
        assert_eq!(
            RepeatMode::Inherit.resolve(RepeatMode::Enabled),
            RepeatMode::Enabled
        );
        assert_eq!(
            RepeatMode::Disabled.resolve(RepeatMode::Enabled),
            RepeatMode::Disabled
        );
    }

    #[test]
    fn modes_serde_round_trip() {
        for m in [
            FilterMode::Inherit,
            FilterMode::Nearest,
            FilterMode::Linear,
            FilterMode::NearestMipmap,
            FilterMode::LinearMipmap,
            FilterMode::NearestAniso,
            FilterMode::LinearAniso,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<FilterMode>(&b).unwrap(), m);
        }
        for m in [
            RepeatMode::Inherit,
            RepeatMode::Disabled,
            RepeatMode::Enabled,
            RepeatMode::Mirror,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<RepeatMode>(&b).unwrap(), m);
        }
    }
}
