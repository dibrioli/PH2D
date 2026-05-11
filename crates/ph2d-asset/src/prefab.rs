//! `PrefabDoc` — cooked prefab asset (ADR-0025 M14.3a).
//!
//! A prefab is a serialized bundle of components for a single
//! entity, plus a list of child prefab references. The wire format
//! is postcard (HR-6 content-addressed via blake3 of the encoded
//! bytes), versioned with a `u32` head per HR-14.
//!
//! # Why a separate doc type
//!
//! `PrefabDoc` is **not** the runtime ECS state — it's the on-disk
//! shape. Two reasons:
//! - Decouple wire format from in-memory layout. A future
//!   `Transform` field rename doesn't break old saves; the migration
//!   path is `migrate_v{N}_to_v{N+1}(old: V{N}) -> V{N+1}`.
//! - Cooker (offline) populates this; `ph2d-ecs::scene::spawn_prefab`
//!   (runtime) consumes it. They live in separate crates and only
//!   share `ph2d-asset` types.

use crate::AssetId;
use serde::{Deserialize, Serialize};

/// Stable identifier for a `Component` type at the cooked-wire level.
///
/// Computed offline as `blake3(stable_name).first_8_bytes()` so the
/// id is identical across rustc versions and host architectures
/// (unlike `std::any::TypeId`). See
/// `ph2d_ecs::scene::ComponentRegistry::stable_type_id`.
pub type ComponentTypeId = u64;

/// Postcard-encoded payload for one component on a prefab entity.
///
/// The pair `(type_id, data)` is the canonical wire representation;
/// the spawn-side registry knows how to map `type_id` back to a
/// concrete `Component` type and `postcard::from_bytes::<T>` the
/// `data` slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentBlob {
    pub type_id: ComponentTypeId,
    pub data: Vec<u8>,
}

/// One cooked prefab. Read from `Asset::Prefab(PrefabDoc)`.
///
/// `version: u32` lives at offset 0 (HR-14); a future schema change
/// bumps this and pairs with a migration function in
/// `crates/ph2d-asset/src/migration.rs` (lands when v2 ships).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefabDoc {
    /// Schema version. Always `PrefabDoc::VERSION` at cook time;
    /// older versions go through migration before reaching here.
    pub version: u32,
    /// Components installed on the root entity, in author order.
    /// Spawn-side iteration follows this order so the on-disk
    /// representation is the source of truth.
    pub components: Vec<ComponentBlob>,
    /// Child prefabs referenced by id. Each child is spawned as a
    /// descendant of the root and parented via `bevy_ecs::ChildOf`.
    pub children: Vec<PrefabRef>,
}

impl PrefabDoc {
    /// Current on-disk schema version. Bumped (alongside a migration
    /// function) when the postcard layout changes.
    pub const VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            components: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl Default for PrefabDoc {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference from a parent prefab to a child prefab, plus per-
/// instance overrides applied on top of the child's own component
/// set.
///
/// `overrides` is a Unity-style "Prefab Variant" mechanism: the
/// child prefab spawns first, then each blob in `overrides` is
/// `insert`-ed onto the spawned root, replacing the corresponding
/// component if the prefab already had one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefabRef {
    /// Content-addressed id of the child prefab (`Asset::Prefab`).
    pub prefab: AssetId,
    /// Components written after the child spawns. Empty for the
    /// common case "spawn the prefab as-authored".
    pub overrides: Vec<ComponentBlob>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty_v1() {
        let d = PrefabDoc::new();
        assert_eq!(d.version, 1);
        assert!(d.components.is_empty());
        assert!(d.children.is_empty());
    }

    #[test]
    fn postcard_round_trip() {
        let original = PrefabDoc {
            version: 1,
            components: vec![ComponentBlob {
                type_id: 0xDEAD_BEEF_CAFE_BABE,
                data: vec![1, 2, 3, 4, 5],
            }],
            children: vec![PrefabRef {
                prefab: AssetId::from_digest([7u8; 32]),
                overrides: vec![],
            }],
        };
        let bytes = postcard::to_allocvec(&original).unwrap();
        let decoded: PrefabDoc = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, original);
    }
}
