//! Sorting / ordering components — Sprite Inspector v2 W3 (spec
//! [`05_ordering_sorting.md`](../../../docs/Sprite_projeto/05_ordering_sorting.md)).
//!
//! These are all **optional** ECS components (anti-god-struct, spec
//! §02): absence has a distinct meaning from any present value. A
//! sprite with no `ZIndexOverride` falls back to the DFS counter; one
//! with `ZIndexOverride(0)` is *forced* to absolute Z zero. The
//! composed sort key that reads them lives in
//! [`crate::sort_key`].
//!
//! Determinism (HR-5): every field is an integer / bool / quantizable
//! float, so the derived sort key is byte-identical cross-platform.

use bevy_ecs::component::Component;
use bevy_ecs::resource::Resource;
use ph2d_core::Vec2;
use serde::{Deserialize, Serialize};

/// Index into the project's named [`SortingLayers`] list (spec §5.2).
/// Capped at [`SortingLayers::MAX`] layers. Index `0` is the first
/// (furthest-back) layer; higher indices render on top.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u8);

/// Macro-camera nominal layer (Unity-style). Absence → the project's
/// default layer (the one whose index equals
/// [`SortingLayers::default_index`]). Spec §5.2.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortingLayer(pub LayerId);

/// Manual micro-ordering *within* a [`SortingLayer`] (Unity "Order in
/// Layer"). Refines the named-layer stage of the pipeline before
/// Y-sort / Z take over. Default `0`.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderInLayer(pub i32);

/// Manual Z override (Godot `z_index`). Presence forces ordering;
/// absence falls back to the DFS counter (spec §5.2 passo 4). Combine
/// with [`ZAsRelative`] for hierarchical cascade.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZIndexOverride(pub i32);

impl ZIndexOverride {
    /// Upper clamp for the *cascaded* effective Z (spec §5.9). Leaves
    /// headroom (`i32::MAX / 4`) so saturating hierarchical sums never
    /// overflow into wraparound.
    pub const Z_MAX: i32 = i32::MAX / 4;
    /// Lower clamp, mirror of [`Self::Z_MAX`].
    pub const Z_MIN: i32 = -(i32::MAX / 4);

    /// Clamp a raw authored value into the gateable range.
    pub const fn clamped(v: i32) -> i32 {
        if v > Self::Z_MAX {
            Self::Z_MAX
        } else if v < Self::Z_MIN {
            Self::Z_MIN
        } else {
            v
        }
    }
}

/// Whether [`ZIndexOverride`] cascades from the parent (spec §5.2
/// passo 4). `true` (default) → effective Z = own + parent's effective
/// Z. `false` → absolute, hierarchy-independent.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZAsRelative(pub bool);

impl Default for ZAsRelative {
    fn default() -> Self {
        // Godot default: z_as_relative = true.
        Self(true)
    }
}

/// Which point of a sprite the Y-sort projects (spec §5.2 passo 3 +
/// §3.7). A unit enum — the projection axis lives on [`YSort::axis`]
/// (the §3.7 "Custom Axis" widget), not inside this variant, so the two
/// are not double-represented.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortPoint {
    /// Centre of the sprite (default) — uses the world translation.
    #[default]
    Center,
    /// The sprite pivot (`anchor` applied). Recommended for top-down
    /// RPG (sort by the character's feet).
    Pivot,
    /// Project onto the arbitrary [`YSort::axis`]. Iso-45° = `Vec2(1, 1)`.
    Custom,
}

impl SortPoint {
    /// Tag for the Inspector segmented control / snapshot: 0 Center ·
    /// 1 Pivot · 2 Custom.
    pub const fn tag(self) -> u8 {
        match self {
            SortPoint::Center => 0,
            SortPoint::Pivot => 1,
            SortPoint::Custom => 2,
        }
    }

    /// Inverse of [`Self::tag`]; any out-of-range tag maps to `Center`.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => SortPoint::Pivot,
            2 => SortPoint::Custom,
            _ => SortPoint::Center,
        }
    }
}

/// Y-sort: when an *ancestor* carries `YSort { enabled: true, .. }`,
/// its descendants sort by projected world position (spec §5.2 passo
/// 3). Cascades recursively until a Z override breaks it (passo 4).
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YSort {
    pub enabled: bool,
    /// Projection axis. `Vec2(0, 1)` = plain vertical (more Y = more in
    /// front). Iso games use `Vec2(1, 1)`.
    pub axis: Vec2,
    pub sort_point: SortPoint,
}

impl Default for YSort {
    fn default() -> Self {
        Self {
            enabled: true,
            axis: Vec2::new(0.0, 1.0),
            sort_point: SortPoint::Center,
        }
    }
}

/// Multi-piece sort-as-a-block (Unity "Sorting Group"). On a node, the
/// whole subtree sorts as one unit at the group root's position (spec
/// §5.2 passo 5). `sort_at_root: true` on a *descendant* pops it out of
/// the enclosing block (escape hatch).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortingGroup {
    pub sort_at_root: bool,
}

/// Zero-size marker: this child draws *before* its parent (Godot
/// `show_behind_parent`). Spec §5.2 passo 6 — e.g. a shadow attached
/// as a child of a character.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowBehindParent;

/// Zero-size marker: break the transform/modulate cascade (Godot
/// `top_level`). The entity ignores its parent's accumulated transform
/// and tint — used for overlays parented for grouping convenience only.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopLevel;

/// Project-wide registry of named sorting layers (spec §5.2). Stored as
/// a resource; the Inspector dropdown and the sort pipeline read it.
/// Index order *is* the render order (index 0 furthest back).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct SortingLayers {
    /// Ordered layer names. Length ≤ [`Self::MAX`].
    names: Vec<String>,
    /// Index treated as "Default" for sprites with no [`SortingLayer`].
    default_index: u8,
}

impl SortingLayers {
    /// Hard cap on layer count (spec §5.7, FROZEN; bump = ADR
    /// amendment). Caps the comparison explosion; covers 95% of real
    /// scenes.
    pub const MAX: usize = 32;

    /// Canonical default set (spec §5.2): Background / Midground /
    /// Default / Foreground / UI, with "Default" as the fallback.
    pub fn canonical() -> Self {
        Self {
            names: vec![
                "Background".to_string(),
                "Midground".to_string(),
                "Default".to_string(),
                "Foreground".to_string(),
                "UI".to_string(),
            ],
            default_index: 2,
        }
    }

    /// The index a sprite with no [`SortingLayer`] component sorts at.
    pub fn default_index(&self) -> u8 {
        self.default_index
    }

    /// Number of registered layers.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Name of layer `id`, or `None` if out of range.
    pub fn name(&self, id: LayerId) -> Option<&str> {
        self.names.get(id.0 as usize).map(String::as_str)
    }

    /// Iterate `(LayerId, name)` in render order.
    pub fn iter(&self) -> impl Iterator<Item = (LayerId, &str)> {
        self.names
            .iter()
            .enumerate()
            .map(|(i, n)| (LayerId(i as u8), n.as_str()))
    }

    /// Append a named layer. Returns its [`LayerId`], or `None` if the
    /// list is already at [`Self::MAX`].
    pub fn push(&mut self, name: impl Into<String>) -> Option<LayerId> {
        if self.names.len() >= Self::MAX {
            return None;
        }
        let id = LayerId(self.names.len() as u8);
        self.names.push(name.into());
        Some(id)
    }
}

impl Default for SortingLayers {
    fn default() -> Self {
        Self::canonical()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z_clamp_saturates_symmetric() {
        assert_eq!(ZIndexOverride::clamped(i32::MAX), ZIndexOverride::Z_MAX);
        assert_eq!(ZIndexOverride::clamped(i32::MIN), ZIndexOverride::Z_MIN);
        assert_eq!(ZIndexOverride::clamped(5), 5);
        assert_eq!(ZIndexOverride::Z_MIN, -ZIndexOverride::Z_MAX);
    }

    #[test]
    fn defaults_match_godot_conventions() {
        assert!(ZAsRelative::default().0, "z_as_relative defaults true");
        assert!(YSort::default().enabled);
        assert_eq!(YSort::default().axis, Vec2::new(0.0, 1.0));
        assert!(!SortingGroup::default().sort_at_root);
    }

    #[test]
    fn canonical_layers_default_to_default_layer() {
        let layers = SortingLayers::canonical();
        assert_eq!(layers.len(), 5);
        assert_eq!(layers.name(LayerId(2)), Some("Default"));
        assert_eq!(layers.default_index(), 2);
        assert_eq!(layers.name(LayerId(0)), Some("Background"));
        assert_eq!(layers.name(LayerId(4)), Some("UI"));
        assert_eq!(layers.name(LayerId(99)), None);
    }

    #[test]
    fn push_caps_at_max() {
        let mut layers = SortingLayers {
            names: Vec::new(),
            default_index: 0,
        };
        for i in 0..SortingLayers::MAX {
            assert!(layers.push(format!("L{i}")).is_some());
        }
        assert_eq!(layers.len(), SortingLayers::MAX);
        assert!(layers.push("overflow").is_none(), "MAX is a hard cap");
    }

    #[test]
    fn sort_point_serde_round_trips_custom() {
        let p = SortPoint::Custom;
        let bytes = postcard::to_allocvec(&p).unwrap();
        let back: SortPoint = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, p);
    }
}
