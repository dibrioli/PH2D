//! `SceneDoc` — cooked scene asset (ADR-0025 M14.3a).
//!
//! A scene is a collection of `PrefabInstance`s plus a list of
//! parent/child relations to restore on load. Like
//! [`crate::prefab::PrefabDoc`], scenes are postcard-encoded and
//! content-addressed (HR-6), with a `u32` schema-version head
//! (HR-14).
//!
//! # vs PrefabDoc
//!
//! `PrefabDoc` describes a single entity (plus its sub-prefab
//! children). `SceneDoc` describes a flat list of root-level prefab
//! instances and the hierarchy that links them. Spawn-side:
//!
//! ```text
//! let entities: Vec<Entity> = scene.instances
//!     .iter()
//!     .map(|i| spawn_prefab(world, i.prefab))
//!     .collect();
//! for r in &scene.relations {
//!     world.entity_mut(entities[r.child_index as usize])
//!          .insert(ChildOf(entities[r.parent_index as usize]));
//! }
//! ```

use crate::AssetId;
use crate::prefab::ComponentBlob;
use serde::{Deserialize, Serialize};

/// One root-level entity in a scene: which prefab to spawn + any
/// overrides applied on top.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefabInstance {
    pub prefab: AssetId,
    /// Components written after the prefab spawns. Same semantics as
    /// `PrefabRef::overrides` — Unity-style "Prefab Variant"
    /// mechanism.
    pub overrides: Vec<ComponentBlob>,
}

/// A parent/child relation between two `PrefabInstance` indices.
///
/// `parent_index` and `child_index` both index into
/// [`SceneDoc::instances`]; the spawn loop maps them to the actual
/// `bevy_ecs::Entity` values once the prefabs are spawned.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildOfPair {
    pub parent_index: u32,
    pub child_index: u32,
}

/// One cooked scene asset. Read from `Asset::Scene(SceneDoc)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneDoc {
    /// Schema version. Always `SceneDoc::VERSION` at cook time.
    pub version: u32,
    /// Top-level instances in spawn order.
    pub instances: Vec<PrefabInstance>,
    /// Hierarchy: `(parent_index, child_index)` pairs in the
    /// `instances` array. Indices outside the array are rejected at
    /// load time.
    pub relations: Vec<ChildOfPair>,
}

impl SceneDoc {
    pub const VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            instances: Vec::new(),
            relations: Vec::new(),
        }
    }
}

impl Default for SceneDoc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty_v1() {
        let d = SceneDoc::new();
        assert_eq!(d.version, 1);
        assert!(d.instances.is_empty());
        assert!(d.relations.is_empty());
    }

    #[test]
    fn postcard_round_trip() {
        let original = SceneDoc {
            version: 1,
            instances: vec![
                PrefabInstance {
                    prefab: AssetId::from_digest([1u8; 32]),
                    overrides: vec![],
                },
                PrefabInstance {
                    prefab: AssetId::from_digest([2u8; 32]),
                    overrides: vec![ComponentBlob {
                        type_id: 0x1234,
                        data: vec![9, 9, 9],
                    }],
                },
            ],
            relations: vec![ChildOfPair {
                parent_index: 0,
                child_index: 1,
            }],
        };
        let bytes = postcard::to_allocvec(&original).unwrap();
        let decoded: SceneDoc = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, original);
    }
}
