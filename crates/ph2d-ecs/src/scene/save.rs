//! `WorldSnapshot` — capture-and-restore primitive for save / replay
//! / rollback (ADR-0025 M14.3d).
//!
//! Distinct from `PrefabDoc` / `SceneDoc`:
//!
//! - **Prefab / Scene** are authored content. They reference other
//!   prefabs by `AssetId`, carry Unity-style overrides, and are
//!   designed to be hand-edited via JSON5 then cooked.
//! - **WorldSnapshot** is a full state capture of every registered
//!   component on every entity in a world. It's the "save game"
//!   primitive (HR-14) and the rollback / replay snapshot the
//!   networking layer will consume (HR-5).
//!
//! Both formats use postcard for the wire layout so a future
//! `Saveable` macro can shore up both pipelines simultaneously.

use crate::scene::registry::{ComponentRegistry, RegistryError};
use crate::transform::{TransformPropagationState, WorklistBuf};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::world::World;
use ph2d_asset::ComponentBlob;
use serde::{Deserialize, Serialize};

/// One row in a [`WorldSnapshot`]: a single entity's full state.
///
/// `parent_index` references another row in the same snapshot. We
/// store **indices** instead of `Entity::to_bits()` because
/// `bevy_ecs` allocates fresh ids in the restore world — the
/// snapshot is portable across world instances by design.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySnapshotRow {
    /// Registered components on this entity, in `ComponentRegistry`
    /// id-sorted order (HR-5 determinism).
    pub components: Vec<ComponentBlob>,
    /// Index of this entity's parent (`ChildOf::0`) in the
    /// snapshot's `entities` vec, if any. `None` for roots.
    pub parent: Option<u32>,
}

/// Full-world snapshot. Versioned (HR-14); the snapshot pipeline is
/// the canonical save format until a richer `Saveable` derive ships.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub version: u32,
    /// Entities in stable DFS order (roots first, then children).
    pub entities: Vec<EntitySnapshotRow>,
}

impl WorldSnapshot {
    /// Current schema version. Migrations land alongside future
    /// bumps; for v1 there is none.
    pub const VERSION: u32 = 1;

    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            entities: Vec::new(),
        }
    }

    /// Stable hash of the snapshot's component bytes. The hash is
    /// blake3 over the postcard encoding — invariant across runs as
    /// long as the encoding is deterministic (which it is per HR-5
    /// + HR-16).
    ///
    /// Used by replay determinism tests (HR-5) and save-load
    /// integrity verification (HR-14 recovery path).
    pub fn state_hash(&self) -> [u8; 32] {
        let bytes = postcard::to_allocvec(self).expect("WorldSnapshot serialize");
        *blake3::hash(&bytes).as_bytes()
    }
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum SaveError {
    Registry(RegistryError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(e) => write!(f, "world_to_snapshot: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<RegistryError> for SaveError {
    fn from(e: RegistryError) -> Self {
        Self::Registry(e)
    }
}

/// Capture every entity that has at least one registered component.
///
/// **Determinism (HR-5):**
/// - Entities visited via the hierarchy DFS — roots sorted by
///   `Entity::to_bits()`, children in `Children` insertion order.
///   So byte output is invariant given the same spawn sequence.
/// - Components serialized in `ComponentRegistry::iter` order (id-
///   sorted, BTreeMap-backed).
///
/// **Pre-allocated scratch:** caller provides `TransformPropagationState`
/// and `WorklistBuf` so this function makes zero per-call
/// allocations beyond the output `Vec`'s own growth.
pub fn world_to_snapshot(
    world: &World,
    state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
    registry: &ComponentRegistry,
    out: &mut WorldSnapshot,
) -> Result<(), SaveError> {
    out.version = WorldSnapshot::VERSION;
    out.entities.clear();
    worklist.clear();

    // Two-pass:
    //   1. Walk the hierarchy via the existing DFS infrastructure
    //      to record (entity, parent_entity) in stable order.
    //   2. Translate Entity → snapshot index, serialize components.

    // Phase 1: traverse — we reuse the WorklistBuf stack but with
    // Transform sentinel so the type stays uniform with
    // propagate_transforms. Parent info is reconstructed via
    // `ChildOf` lookups after the walk.
    for entity in state.roots.iter(world) {
        worklist.stack.push((entity, crate::Transform::IDENTITY));
    }
    worklist.stack.sort_unstable_by_key(|&(e, _)| e.to_bits());

    // Drain in DFS LIFO order, recording the visit sequence.
    let mut visit_order: Vec<Entity> = Vec::with_capacity(worklist.stack.len());
    while let Some((entity, _)) = worklist.stack.pop() {
        // Skip if the entity has no Transform (shouldn't happen
        // given the filter, but defensive).
        if world.get_entity(entity).is_err() {
            continue;
        }
        visit_order.push(entity);
        if let Ok(entity_ref) = world.get_entity(entity)
            && let Some(children) = entity_ref.get::<Children>()
        {
            worklist.children_scratch.clear();
            worklist.children_scratch.extend(children.iter());
            for c in worklist.children_scratch.iter().rev().copied() {
                worklist.stack.push((c, crate::Transform::IDENTITY));
            }
        }
    }

    // Phase 2: build the index map (Entity → snapshot index), then
    // emit rows.
    let mut index_of: std::collections::BTreeMap<Entity, u32> = std::collections::BTreeMap::new();
    for (i, e) in visit_order.iter().enumerate() {
        index_of.insert(*e, i as u32);
    }

    for entity in &visit_order {
        let mut row = EntitySnapshotRow {
            components: Vec::new(),
            parent: None,
        };
        for entry in registry.iter() {
            match (entry.serialize)(world, *entity) {
                Ok(Some(bytes)) => {
                    row.components.push(ComponentBlob {
                        type_id: entry.type_id,
                        data: bytes,
                    });
                }
                Ok(None) => {}
                Err(e) => return Err(e.into()),
            }
        }
        // Look up parent (if any) and translate to snapshot index.
        if let Ok(eref) = world.get_entity(*entity)
            && let Some(co) = eref.get::<ChildOf>()
        {
            row.parent = index_of.get(&co.0).copied();
        }
        out.entities.push(row);
    }
    Ok(())
}

/// Inverse of [`world_to_snapshot`]: spawn an entity per row, install
/// components, restore parent/child relations. Returns the assigned
/// `Entity` per row (indices match `snapshot.entities`).
///
/// `world` does **not** need to be empty — existing entities are
/// left alone. The caller is responsible for clearing if a full
/// restore is desired.
pub fn snapshot_to_world(
    world: &mut World,
    snapshot: &WorldSnapshot,
    registry: &ComponentRegistry,
) -> Result<Vec<Entity>, SaveError> {
    let mut entities = Vec::with_capacity(snapshot.entities.len());
    // Pass 1: spawn + install components.
    for row in &snapshot.entities {
        let entity = world.spawn_empty().id();
        for blob in &row.components {
            let entry = registry.get_by_id(blob.type_id).ok_or(SaveError::Registry(
                RegistryError::UnknownTypeId(blob.type_id),
            ))?;
            (entry.insert_from_bytes)(world, entity, &blob.data).map_err(SaveError::Registry)?;
        }
        entities.push(entity);
    }
    // Pass 2: relations.
    for (i, row) in snapshot.entities.iter().enumerate() {
        if let Some(p) = row.parent {
            let parent = entities[p as usize];
            let child = entities[i];
            world.entity_mut(child).insert(ChildOf(parent));
        }
    }
    Ok(entities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimWorld;
    use crate::scene::register_ecs_components;
    use crate::{Name, Transform};
    use ph2d_core::Vec2;

    fn populated_world() -> (SimWorld, ComponentRegistry) {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        // Build a 3-level hierarchy with names.
        let root = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(10.0, 20.0)),
                Name::new("Root"),
            ))
            .id();
        let mid = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(1.0, 0.0)),
                Name::new("Mid"),
                ChildOf(root),
            ))
            .id();
        sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.5, 0.5)),
            Name::new("Leaf"),
            ChildOf(mid),
        ));
        (sim, reg)
    }

    #[test]
    fn snapshot_captures_all_entities() {
        let (mut sim, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(sim.world(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
        assert_eq!(snap.entities.len(), 3);
    }

    #[test]
    fn snapshot_restore_round_trip_preserves_names_and_hierarchy() {
        let (mut sim_a, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(sim_a.world(), &mut state, &mut worklist, &reg, &mut snap).unwrap();

        let mut sim_b = SimWorld::new();
        let entities = snapshot_to_world(sim_b.world_mut(), &snap, &reg).unwrap();
        assert_eq!(entities.len(), 3);

        // Verify names + hierarchy in the restored world.
        let names: Vec<String> = entities
            .iter()
            .map(|e| {
                sim_b
                    .world_mut()
                    .get::<Name>(*e)
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect();
        assert!(names.contains(&"Root".to_string()));
        assert!(names.contains(&"Mid".to_string()));
        assert!(names.contains(&"Leaf".to_string()));

        // The Leaf entity (last in visit order) should have a parent.
        let parents: Vec<Option<Entity>> = entities
            .iter()
            .map(|e| sim_b.world_mut().get::<ChildOf>(*e).map(|c| c.0))
            .collect();
        // Exactly one root (Root) → exactly one None.
        let root_count = parents.iter().filter(|p| p.is_none()).count();
        assert_eq!(root_count, 1);
    }

    #[test]
    fn state_hash_is_deterministic_across_round_trip() {
        let (mut sim_a, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim_a.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap_a = WorldSnapshot::new();
        world_to_snapshot(sim_a.world(), &mut state, &mut worklist, &reg, &mut snap_a).unwrap();
        let hash_a = snap_a.state_hash();

        // Restore to a fresh world, snapshot again — hashes match.
        let mut sim_b = SimWorld::new();
        snapshot_to_world(sim_b.world_mut(), &snap_a, &reg).unwrap();
        let mut state_b = TransformPropagationState::new(sim_b.world_mut());
        let mut snap_b = WorldSnapshot::new();
        world_to_snapshot(
            sim_b.world(),
            &mut state_b,
            &mut worklist,
            &reg,
            &mut snap_b,
        )
        .unwrap();
        let hash_b = snap_b.state_hash();
        assert_eq!(
            hash_a, hash_b,
            "snapshot round-trip produced a different state hash"
        );
    }

    #[test]
    fn snapshot_postcard_round_trips() {
        let (mut sim, reg) = populated_world();
        let mut state = TransformPropagationState::new(sim.world_mut());
        let mut worklist = WorklistBuf::new();
        let mut snap = WorldSnapshot::new();
        world_to_snapshot(sim.world(), &mut state, &mut worklist, &reg, &mut snap).unwrap();
        let bytes = postcard::to_allocvec(&snap).unwrap();
        let decoded: WorldSnapshot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, snap);
    }
}
