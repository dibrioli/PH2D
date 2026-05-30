//! Texture sampling components — Sprite Inspector v2 W3 (spec
//! [`09_sampling_e_material.md`](../../../docs/Sprite_projeto/09_sampling_e_material.md)
//! §9.1–9.2). Both are **hierarchical**: an entity's value overrides
//! the inherited one; `Inherit` defers to the nearest ancestor that
//! sets it, falling back to the project default.
//!
//! Material & Blend (spec §9.4–9.7) are a *W4* deliverable and not
//! defined here.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
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

    /// Enum discriminant as a `u8` tag (Inspector §9 segmented / snapshot
    /// / the renderer's packed sampling key). `0 Inherit … 6 LinearAniso`.
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::tag`]; out-of-range → `Inherit`.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => FilterMode::Nearest,
            2 => FilterMode::Linear,
            3 => FilterMode::NearestMipmap,
            4 => FilterMode::LinearMipmap,
            5 => FilterMode::NearestAniso,
            6 => FilterMode::LinearAniso,
            _ => FilterMode::Inherit,
        }
    }
}

impl RepeatMode {
    /// Enum discriminant as a `u8` tag. `0 Inherit · 1 Disabled · 2
    /// Enabled · 3 Mirror`.
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// Inverse of [`Self::tag`]; out-of-range → `Inherit`.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => RepeatMode::Disabled,
            2 => RepeatMode::Enabled,
            3 => RepeatMode::Mirror,
            _ => RepeatMode::Inherit,
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

/// Resolve the effective [`FilterMode`] for `entity` by walking the
/// `ChildOf` chain (Godot per-node hierarchy, spec §9.1): the nearest
/// ancestor-or-self with a concrete (non-`Inherit`) [`TextureFilter`]
/// wins; if every node up the chain is `Inherit` / absent, fall back to
/// `project_default`. Allocation-free; the chain is shallow.
pub fn resolve_texture_filter(
    world: &World,
    entity: Entity,
    project_default: FilterMode,
) -> FilterMode {
    let mut node = Some(entity);
    while let Some(n) = node {
        let here = world
            .get::<TextureFilter>(n)
            .map_or(FilterMode::Inherit, |f| f.0);
        if here != FilterMode::Inherit {
            return here;
        }
        node = world.get::<ChildOf>(n).map(|c| c.parent());
    }
    project_default
}

/// Resolve the effective [`RepeatMode`] for `entity` (mirror of
/// [`resolve_texture_filter`], spec §9.2).
pub fn resolve_texture_repeat(
    world: &World,
    entity: Entity,
    project_default: RepeatMode,
) -> RepeatMode {
    let mut node = Some(entity);
    while let Some(n) = node {
        let here = world
            .get::<TextureRepeat>(n)
            .map_or(RepeatMode::Inherit, |r| r.0);
        if here != RepeatMode::Inherit {
            return here;
        }
        node = world.get::<ChildOf>(n).map(|c| c.parent());
    }
    project_default
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
    fn resolve_walks_childof_to_nearest_concrete() {
        use bevy_ecs::hierarchy::ChildOf;
        let mut w = World::new();
        // root(Linear) → mid(Inherit) → leaf(absent): leaf resolves to
        // the root's Linear.
        let root = w.spawn(TextureFilter(FilterMode::Linear)).id();
        let mid = w
            .spawn((ChildOf(root), TextureFilter(FilterMode::Inherit)))
            .id();
        let leaf = w.spawn(ChildOf(mid)).id();
        assert_eq!(
            resolve_texture_filter(&w, leaf, FilterMode::Nearest),
            FilterMode::Linear
        );
        // A concrete override closer to the leaf wins.
        let leaf2 = w
            .spawn((ChildOf(mid), TextureFilter(FilterMode::Nearest)))
            .id();
        assert_eq!(
            resolve_texture_filter(&w, leaf2, FilterMode::Linear),
            FilterMode::Nearest
        );
    }

    #[test]
    fn resolve_falls_back_to_project_default() {
        let mut w = World::new();
        let e = w.spawn_empty().id();
        assert_eq!(
            resolve_texture_filter(&w, e, FilterMode::Linear),
            FilterMode::Linear
        );
        assert_eq!(
            resolve_texture_repeat(&w, e, RepeatMode::Enabled),
            RepeatMode::Enabled
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
