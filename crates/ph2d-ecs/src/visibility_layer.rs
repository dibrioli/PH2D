//! Visibility-layer & on-screen culling components — Sprite Inspector
//! v2 W3 (spec [`02`](../../../docs/Sprite_projeto/02_components_ortogonais.md)
//! + [`06 §6.5`](../../../docs/Sprite_projeto/06_mask_clip.md)).
//!
//! [`VisibilityLayer`] is a per-entity bitmask culled against a
//! camera's `cull_mask` (Godot visibility layers). [`OnScreenEnabler`]
//! auto-disables processing when the entity leaves a world rect (Godot
//! `VisibleOnScreenEnabler2D`).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// Per-entity visibility bitmask (Godot visibility layers). The entity
/// renders for a camera when `layers & camera.cull_mask != 0`. Absence
/// → [`VisibilityLayer::ALL`] (visible to every camera). Spec §02.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibilityLayer(pub u32);

impl VisibilityLayer {
    /// Visible to every camera (all 32 bits set).
    pub const ALL: u32 = u32::MAX;

    /// Layer 1 only (bit 0) — the common single-layer default.
    pub const LAYER_1: u32 = 1;

    /// Does this entity's mask intersect `cull_mask`?
    pub const fn visible_to(self, cull_mask: u32) -> bool {
        self.0 & cull_mask != 0
    }
}

impl Default for VisibilityLayer {
    fn default() -> Self {
        Self(Self::ALL)
    }
}

/// What [`OnScreenEnabler`] does when the entity leaves its rect.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnableMode {
    /// Resume processing on enter, pause on exit (Godot default).
    #[default]
    InheritPause,
    /// Pause only this node's processing.
    PauseProcessing,
    /// Make the node (and subtree) invisible off-screen.
    HideVisible,
}

/// Auto-disable processing when the entity leaves a world-space rect
/// (Godot `VisibleOnScreenEnabler2D`, spec §02). The rect is
/// `[x, y, w, h]` in world meters, matching `Sprite::region_rect`'s
/// `[f32; 4]` convention.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnScreenEnabler {
    pub rect: [f32; 4],
    pub mode: EnableMode,
}

impl OnScreenEnabler {
    pub const fn new(rect: [f32; 4], mode: EnableMode) -> Self {
        Self { rect, mode }
    }

    /// Is world-space `point` inside the enabler rect (`[x, y, w, h]`)?
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let [rx, ry, rw, rh] = self.rect;
        x >= rx && x <= rx + rw && y >= ry && y <= ry + rh
    }
}

impl Default for OnScreenEnabler {
    fn default() -> Self {
        Self {
            rect: [0.0, 0.0, 0.0, 0.0],
            mode: EnableMode::InheritPause,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absence_default_is_all_visible() {
        assert_eq!(VisibilityLayer::default().0, VisibilityLayer::ALL);
        assert!(VisibilityLayer::default().visible_to(VisibilityLayer::LAYER_1));
    }

    #[test]
    fn bitmask_culls_on_disjoint_layers() {
        let on_layer_2 = VisibilityLayer(0b10);
        assert!(on_layer_2.visible_to(0b10));
        assert!(on_layer_2.visible_to(0b11));
        assert!(!on_layer_2.visible_to(0b01));
        assert!(!on_layer_2.visible_to(0));
    }

    #[test]
    fn enabler_rect_contains_inclusive() {
        let e = OnScreenEnabler::new([0.0, 0.0, 10.0, 10.0], EnableMode::HideVisible);
        assert!(e.contains(5.0, 5.0));
        assert!(e.contains(0.0, 0.0));
        assert!(e.contains(10.0, 10.0));
        assert!(!e.contains(11.0, 5.0));
        assert!(!e.contains(-1.0, 5.0));
    }

    #[test]
    fn enable_mode_serde_round_trip() {
        for m in [
            EnableMode::InheritPause,
            EnableMode::PauseProcessing,
            EnableMode::HideVisible,
        ] {
            let b = postcard::to_allocvec(&m).unwrap();
            assert_eq!(postcard::from_bytes::<EnableMode>(&b).unwrap(), m);
        }
    }
}
