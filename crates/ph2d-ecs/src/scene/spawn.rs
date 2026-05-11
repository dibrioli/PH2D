//! `spawn_prefab` / `spawn_scene` — instantiate cooked
//! `PrefabDoc` / `SceneDoc` assets into a live `bevy_ecs::World`
//! (ADR-0025 M14.3b).
//!
//! Both functions are pure given `(doc, registry)` — same input
//! produces an equivalent set of entities + components every time.
//! The only thing that differs across calls is the `Entity` ids the
//! `bevy_ecs::World` allocates, which are intentionally opaque
//! (HR-8).
//!
//! # Error model
//!
//! Spawn errors are user-facing: an unknown `type_id` in the cooked
//! prefab means the runtime's `ComponentRegistry` is missing the
//! type the cooker registered. That's a configuration bug — surface
//! it loudly via `SpawnError` rather than silently dropping
//! components.

use crate::scene::registry::{ComponentRegistry, RegistryError};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_asset::{AssetDb, AssetId, ComponentBlob, PrefabDoc, SceneDoc};

#[derive(Debug)]
pub enum SpawnError {
    /// Cooked prefab references a child by `AssetId` that isn't in
    /// the supplied `AssetDb`.
    MissingChildPrefab(AssetId),
    /// Cooked asset references the same `AssetId` but it isn't a
    /// `PrefabDoc` (e.g. a `SceneDoc` or `ImageRgba8`).
    AssetIsNotPrefab(AssetId),
    /// Scene instance references an `AssetId` not present in the db.
    MissingInstancePrefab(AssetId),
    /// Component blob's `type_id` isn't registered. Add it to the
    /// runtime's `register_*_components()` call set.
    Registry(RegistryError),
    /// Scene relation references an `instance` index out of bounds.
    RelationOutOfBounds {
        instances: usize,
        index: u32,
    },
}

impl std::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingChildPrefab(id) => write!(
                f,
                "spawn_prefab: child prefab {id} missing from AssetDb"
            ),
            Self::AssetIsNotPrefab(id) => {
                write!(f, "spawn_prefab: asset {id} is not Asset::Prefab")
            }
            Self::MissingInstancePrefab(id) => write!(
                f,
                "spawn_scene: instance prefab {id} missing from AssetDb"
            ),
            Self::Registry(e) => write!(f, "spawn: {e}"),
            Self::RelationOutOfBounds { instances, index } => write!(
                f,
                "spawn_scene: relation index {index} ≥ instances.len() {instances}"
            ),
        }
    }
}

impl std::error::Error for SpawnError {}

impl From<RegistryError> for SpawnError {
    fn from(e: RegistryError) -> Self {
        Self::Registry(e)
    }
}

/// Spawn a single `PrefabDoc` into `world`, returning the root
/// entity. Each child prefab is resolved via `prefab_db` and spawned
/// recursively, attached to the parent with `bevy_ecs::ChildOf`.
///
/// Component installation order: the parent's `components` first,
/// then `children` recursively. Override blobs on each `PrefabRef`
/// are inserted *after* the child's own components, so they win.
pub fn spawn_prefab(
    world: &mut World,
    doc: &PrefabDoc,
    prefab_db: &AssetDb,
    registry: &ComponentRegistry,
) -> Result<Entity, SpawnError> {
    let entity = world.spawn_empty().id();
    for blob in &doc.components {
        insert_blob(world, entity, blob, registry)?;
    }
    for child_ref in &doc.children {
        let child_asset = prefab_db
            .get(&child_ref.prefab)
            .ok_or(SpawnError::MissingChildPrefab(child_ref.prefab))?;
        let child_doc = match &*child_asset {
            ph2d_asset::Asset::Prefab(p) => std::sync::Arc::clone(p),
            _ => return Err(SpawnError::AssetIsNotPrefab(child_ref.prefab)),
        };
        let child_entity = spawn_prefab(world, &child_doc, prefab_db, registry)?;
        for ov in &child_ref.overrides {
            insert_blob(world, child_entity, ov, registry)?;
        }
        world.entity_mut(child_entity).insert(ChildOf(entity));
    }
    Ok(entity)
}

/// Spawn every instance in `scene`, then apply `relations` as
/// `ChildOf` between the spawned root entities. Returns the list of
/// root entities in `scene.instances` order — index `i` of the
/// return matches index `i` in `scene.instances`.
pub fn spawn_scene(
    world: &mut World,
    scene: &SceneDoc,
    prefab_db: &AssetDb,
    registry: &ComponentRegistry,
) -> Result<Vec<Entity>, SpawnError> {
    let mut roots = Vec::with_capacity(scene.instances.len());
    for instance in &scene.instances {
        let asset = prefab_db
            .get(&instance.prefab)
            .ok_or(SpawnError::MissingInstancePrefab(instance.prefab))?;
        let doc = match &*asset {
            ph2d_asset::Asset::Prefab(p) => std::sync::Arc::clone(p),
            _ => return Err(SpawnError::AssetIsNotPrefab(instance.prefab)),
        };
        let entity = spawn_prefab(world, &doc, prefab_db, registry)?;
        for ov in &instance.overrides {
            insert_blob(world, entity, ov, registry)?;
        }
        roots.push(entity);
    }
    // Apply relations. We validate indices then apply — partial
    // application would leave the world in an inconsistent state.
    for rel in &scene.relations {
        let n = roots.len();
        if rel.parent_index as usize >= n {
            return Err(SpawnError::RelationOutOfBounds {
                instances: n,
                index: rel.parent_index,
            });
        }
        if rel.child_index as usize >= n {
            return Err(SpawnError::RelationOutOfBounds {
                instances: n,
                index: rel.child_index,
            });
        }
    }
    for rel in &scene.relations {
        let parent = roots[rel.parent_index as usize];
        let child = roots[rel.child_index as usize];
        world.entity_mut(child).insert(ChildOf(parent));
    }
    Ok(roots)
}

fn insert_blob(
    world: &mut World,
    entity: Entity,
    blob: &ComponentBlob,
    registry: &ComponentRegistry,
) -> Result<(), SpawnError> {
    let entry = registry
        .get_by_id(blob.type_id)
        .ok_or(SpawnError::Registry(RegistryError::UnknownTypeId(
            blob.type_id,
        )))?;
    (entry.insert_from_bytes)(world, entity, &blob.data).map_err(SpawnError::Registry)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::register_ecs_components;
    use crate::{Name, Transform};
    use ph2d_asset::{ChildOfPair, PrefabInstance, PrefabRef};
    use ph2d_core::Vec2;

    fn make_registry() -> ComponentRegistry {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        reg
    }

    fn make_db() -> AssetDb {
        AssetDb::new()
    }

    fn transform_blob(t: Transform) -> ComponentBlob {
        let bytes = postcard::to_allocvec(&t).unwrap();
        ComponentBlob {
            type_id: crate::scene::stable_type_id("ph2d::ecs::Transform"),
            data: bytes,
        }
    }

    fn name_blob(name: &str) -> ComponentBlob {
        let n = Name::new(name);
        let bytes = postcard::to_allocvec(&n).unwrap();
        ComponentBlob {
            type_id: crate::scene::stable_type_id("ph2d::ecs::Name"),
            data: bytes,
        }
    }

    #[test]
    fn spawn_prefab_installs_components() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        let doc = PrefabDoc {
            version: 1,
            components: vec![
                transform_blob(Transform::from_translation(Vec2::new(3.0, 4.0))),
                name_blob("Player"),
            ],
            children: vec![],
        };
        let entity = spawn_prefab(&mut world, &doc, &db, &reg).unwrap();
        let t = world.get::<Transform>(entity).unwrap();
        assert_eq!(t.translation, Vec2::new(3.0, 4.0));
        let n = world.get::<Name>(entity).unwrap();
        assert_eq!(n.as_str(), "Player");
    }

    #[test]
    fn spawn_prefab_with_child_creates_child_of_relation() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        // Cook a leaf prefab manually and insert into the db so the
        // parent can reference it.
        let leaf = PrefabDoc {
            version: 1,
            components: vec![name_blob("Leaf")],
            children: vec![],
        };
        let leaf_bytes = postcard::to_allocvec(&leaf).unwrap();
        let leaf_id = db.insert_prefab_bytes(&leaf_bytes).unwrap();

        let parent = PrefabDoc {
            version: 1,
            components: vec![name_blob("Parent")],
            children: vec![PrefabRef {
                prefab: leaf_id,
                overrides: vec![],
            }],
        };
        let parent_entity = spawn_prefab(&mut world, &parent, &db, &reg).unwrap();

        // The world should have 2 entities; the leaf has ChildOf(parent).
        let mut q = world.query::<(Entity, &Name, Option<&ChildOf>)>();
        let mut leaves: Vec<(Entity, String, Option<Entity>)> = q
            .iter(&world)
            .map(|(e, n, c)| (e, n.as_str().to_owned(), c.map(|c| c.0)))
            .collect();
        leaves.sort_by_key(|(e, _, _)| e.to_bits());
        assert_eq!(leaves.len(), 2, "expected parent + 1 child");
        let leaf_row = leaves
            .iter()
            .find(|(_, name, _)| name == "Leaf")
            .expect("child entity present");
        assert_eq!(leaf_row.2, Some(parent_entity));
    }

    #[test]
    fn spawn_prefab_override_wins_over_inner_component() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        let inner = PrefabDoc {
            version: 1,
            components: vec![name_blob("Original")],
            children: vec![],
        };
        let inner_bytes = postcard::to_allocvec(&inner).unwrap();
        let inner_id = db.insert_prefab_bytes(&inner_bytes).unwrap();

        let outer = PrefabDoc {
            version: 1,
            components: vec![],
            children: vec![PrefabRef {
                prefab: inner_id,
                overrides: vec![name_blob("Overridden")],
            }],
        };
        spawn_prefab(&mut world, &outer, &db, &reg).unwrap();
        let mut q = world.query::<&Name>();
        let names: Vec<&str> = q.iter(&world).map(|n| n.as_str()).collect();
        assert!(names.contains(&"Overridden"));
        assert!(!names.contains(&"Original"));
    }

    #[test]
    fn spawn_prefab_unknown_type_id_errors() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        let doc = PrefabDoc {
            version: 1,
            components: vec![ComponentBlob {
                type_id: 0xDEAD_BEEF_FFFF_FFFF,
                data: vec![],
            }],
            children: vec![],
        };
        let err = spawn_prefab(&mut world, &doc, &db, &reg).unwrap_err();
        match err {
            SpawnError::Registry(RegistryError::UnknownTypeId(id)) => {
                assert_eq!(id, 0xDEAD_BEEF_FFFF_FFFF);
            }
            other => panic!("expected UnknownTypeId, got {other:?}"),
        }
    }

    #[test]
    fn spawn_scene_creates_relations() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        // Build a simple prefab and insert it.
        let leaf = PrefabDoc {
            version: 1,
            components: vec![name_blob("X")],
            children: vec![],
        };
        let leaf_id = db
            .insert_prefab_bytes(&postcard::to_allocvec(&leaf).unwrap())
            .unwrap();
        let scene = SceneDoc {
            version: 1,
            instances: vec![
                PrefabInstance {
                    prefab: leaf_id,
                    overrides: vec![name_blob("A")],
                },
                PrefabInstance {
                    prefab: leaf_id,
                    overrides: vec![name_blob("B")],
                },
                PrefabInstance {
                    prefab: leaf_id,
                    overrides: vec![name_blob("C")],
                },
            ],
            relations: vec![
                ChildOfPair {
                    parent_index: 0,
                    child_index: 1,
                },
                ChildOfPair {
                    parent_index: 0,
                    child_index: 2,
                },
            ],
        };
        let roots = spawn_scene(&mut world, &scene, &db, &reg).unwrap();
        assert_eq!(roots.len(), 3);

        // A has no ChildOf; B and C have ChildOf(roots[0]).
        let parent_of = |e: Entity| world.get::<ChildOf>(e).map(|c| c.0);
        assert_eq!(parent_of(roots[0]), None);
        assert_eq!(parent_of(roots[1]), Some(roots[0]));
        assert_eq!(parent_of(roots[2]), Some(roots[0]));
    }

    #[test]
    fn spawn_scene_relation_out_of_bounds_errors() {
        let mut world = World::new();
        let db = make_db();
        let reg = make_registry();
        let leaf = PrefabDoc::new();
        let leaf_id = db
            .insert_prefab_bytes(&postcard::to_allocvec(&leaf).unwrap())
            .unwrap();
        let scene = SceneDoc {
            version: 1,
            instances: vec![PrefabInstance {
                prefab: leaf_id,
                overrides: vec![],
            }],
            relations: vec![ChildOfPair {
                parent_index: 0,
                child_index: 5, // out of bounds
            }],
        };
        let err = spawn_scene(&mut world, &scene, &db, &reg).unwrap_err();
        match err {
            SpawnError::RelationOutOfBounds { instances, index } => {
                assert_eq!(instances, 1);
                assert_eq!(index, 5);
            }
            other => panic!("expected RelationOutOfBounds, got {other:?}"),
        }
    }
}
