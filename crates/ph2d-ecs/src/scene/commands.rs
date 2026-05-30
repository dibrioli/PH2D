//! `EditorCommandQueue` — substrate for editor-driven sim mutations
//! (ADR-0025 M14.3c, separate from script's `WriteQueue`).
//!
//! M14.3 ships the data structure + drain helper without wiring it
//! into `ph2d-editor` (hero screen is in dev/test per the user
//! constraint). When the editor stabilizes, it pushes commands here
//! and the shell drains them pre-tick — exact analog to how Luau
//! scripts drive `WriteQueue` / `SpawnQueue` in M14.2.
//!
//! # Why a separate queue from `WriteQueue`
//!
//! - **HR-7 separation of concerns:** scripts and editor have
//!   different audit trails, different rate limits, different
//!   authorization stories (HR-11 destructive ops). Mixing them in
//!   one queue blurs ownership.
//! - **Richer command shape:** editor cares about component-level
//!   set/replace, while scripts deal with primitive fields.
//!
//! The queue uses the same `Arc<Mutex<Vec<…>>>` pattern as the rest
//! of `ph2d-script::io` for consistency.

use crate::scene::registry::{ComponentRegistry, RegistryError};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use std::sync::{Arc, Mutex};

/// One queued editor mutation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommand {
    /// Replace (or insert) a component blob on an existing entity.
    /// Wire format matches `ph2d_asset::ComponentBlob` so the editor
    /// + cooker share a representation.
    SetComponent {
        entity: u64,
        type_id: u64,
        data: Vec<u8>,
    },
    /// Spawn a new entity carrying the listed component blobs. The
    /// inspector populates this when the user clicks "Add Entity"
    /// with initial Transform/Name values.
    Spawn {
        components: Vec<(u64 /*type_id*/, Vec<u8>)>,
    },
    /// Remove an optional component from an existing entity. Used by the
    /// Sprite Inspector v2 W3 to detach an optional sorting / visibility
    /// / sampling component (toggling a marker off, unsetting a Z
    /// override). Idempotent: a no-op if the component is absent.
    RemoveComponent { entity: u64, type_id: u64 },
    /// Despawn an entity (with HR-11 confirmation handled at the
    /// editor UI level — this queue is post-confirmation).
    Despawn { entity: u64 },
    /// Re-parent `entity` under `new_parent`. `None` for
    /// `new_parent` makes it a root.
    Reparent {
        entity: u64,
        new_parent: Option<u64>,
    },
}

/// Backpressure error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorQueueFull {
    pub cap: usize,
}

impl std::fmt::Display for EditorQueueFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EditorCommandQueue full ({} pending commands)", self.cap)
    }
}

impl std::error::Error for EditorQueueFull {}

/// Cloneable handle to a shared queue of editor commands. Same
/// `Arc<Mutex<…>>` shape as the script-side `WriteQueue` /
/// `SpawnQueue`.
#[derive(Clone)]
pub struct EditorCommandQueue {
    inner: Arc<Mutex<Vec<EditorCommand>>>,
    cap: usize,
}

impl EditorCommandQueue {
    /// 4096 pending commands — comfortably above any plausible
    /// per-frame inspector edit rate. Beyond this we surface the
    /// `EditorQueueFull` error rather than OOM.
    pub const DEFAULT_CAP: usize = 4096;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            cap,
        }
    }

    pub fn push(&self, cmd: EditorCommand) -> Result<(), EditorQueueFull> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if g.len() >= self.cap {
            return Err(EditorQueueFull { cap: self.cap });
        }
        g.push(cmd);
        Ok(())
    }

    pub fn drain(&self) -> Vec<EditorCommand> {
        std::mem::take(&mut *self.inner.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl Default for EditorCommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Drain `queue` and apply each command to `sim_w`. The host calls
/// this between ticks (HR-7 editor ↔ sim boundary).
///
/// Returns the list of newly-spawned entities (in `Spawn` command
/// order) so the caller can echo them back to the editor for
/// selection.
pub fn apply_editor_commands(
    sim_w: &mut World,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Result<Vec<Entity>, ApplyError> {
    let cmds = queue.drain();
    let mut spawned = Vec::new();
    for cmd in cmds {
        match cmd {
            EditorCommand::SetComponent {
                entity,
                type_id,
                data,
            } => {
                let entry = registry
                    .get_by_id(type_id)
                    .ok_or(ApplyError::Registry(RegistryError::UnknownTypeId(type_id)))?;
                let bevy_entity = entity_from_bits(entity);
                (entry.insert_from_bytes)(sim_w, bevy_entity, &data)
                    .map_err(ApplyError::Registry)?;
            }
            EditorCommand::Spawn { components } => {
                let entity = sim_w.spawn_empty().id();
                for (type_id, data) in components {
                    let entry = registry
                        .get_by_id(type_id)
                        .ok_or(ApplyError::Registry(RegistryError::UnknownTypeId(type_id)))?;
                    (entry.insert_from_bytes)(sim_w, entity, &data)
                        .map_err(ApplyError::Registry)?;
                }
                spawned.push(entity);
            }
            EditorCommand::RemoveComponent { entity, type_id } => {
                let entry = registry
                    .get_by_id(type_id)
                    .ok_or(ApplyError::Registry(RegistryError::UnknownTypeId(type_id)))?;
                let bevy_entity = entity_from_bits(entity);
                (entry.remove)(sim_w, bevy_entity);
            }
            EditorCommand::Despawn { entity } => {
                let bevy_entity = entity_from_bits(entity);
                let _ = sim_w.despawn(bevy_entity);
            }
            EditorCommand::Reparent { entity, new_parent } => {
                let bevy_entity = entity_from_bits(entity);
                let mut e = sim_w
                    .get_entity_mut(bevy_entity)
                    .map_err(|_| ApplyError::EntityMissing(bevy_entity))?;
                match new_parent {
                    Some(p) => {
                        e.insert(ChildOf(entity_from_bits(p)));
                    }
                    None => {
                        e.remove::<ChildOf>();
                    }
                }
            }
        }
    }
    Ok(spawned)
}

#[derive(Debug)]
pub enum ApplyError {
    Registry(RegistryError),
    EntityMissing(Entity),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(e) => write!(f, "apply_editor_commands: {e}"),
            Self::EntityMissing(e) => {
                write!(f, "apply_editor_commands: entity {:?} missing", e)
            }
        }
    }
}

impl std::error::Error for ApplyError {}

fn entity_from_bits(bits: u64) -> Entity {
    // bevy_ecs 0.18: `Entity::from_bits(u64) -> Entity` (panic-free).
    Entity::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimWorld;
    use crate::scene::register_ecs_components;
    use crate::{Name, Transform};
    use ph2d_core::Vec2;

    fn name_blob_bytes(name: &str) -> Vec<u8> {
        let n = Name::new(name);
        postcard::to_allocvec(&n).unwrap()
    }

    fn transform_blob_bytes(t: Transform) -> Vec<u8> {
        postcard::to_allocvec(&t).unwrap()
    }

    #[test]
    fn queue_push_drain() {
        let q = EditorCommandQueue::new();
        q.push(EditorCommand::Despawn { entity: 7 }).unwrap();
        q.push(EditorCommand::Despawn { entity: 8 }).unwrap();
        let drained = q.drain();
        assert_eq!(drained.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn queue_caps_at_max() {
        let q = EditorCommandQueue::with_cap(2);
        q.push(EditorCommand::Despawn { entity: 1 }).unwrap();
        q.push(EditorCommand::Despawn { entity: 2 }).unwrap();
        let err = q.push(EditorCommand::Despawn { entity: 3 }).unwrap_err();
        assert_eq!(err.cap, 2);
    }

    #[test]
    fn apply_spawn_creates_entity_with_components() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let queue = EditorCommandQueue::new();
        let transform_type_id = crate::scene::stable_type_id("ph2d::ecs::Transform");
        let name_type_id = crate::scene::stable_type_id("ph2d::ecs::Name");
        queue
            .push(EditorCommand::Spawn {
                components: vec![
                    (
                        transform_type_id,
                        transform_blob_bytes(Transform::from_translation(Vec2::new(1.0, 2.0))),
                    ),
                    (name_type_id, name_blob_bytes("New")),
                ],
            })
            .unwrap();
        let spawned = apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        assert_eq!(spawned.len(), 1);
        let e = spawned[0];
        let t = sim.world_mut().get::<Transform>(e).unwrap();
        assert_eq!(t.translation, Vec2::new(1.0, 2.0));
        let n = sim.world_mut().get::<Name>(e).unwrap();
        assert_eq!(n.as_str(), "New");
    }

    #[test]
    fn apply_set_component_overrides_existing() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let entity = sim.world_mut().spawn(Name::new("Old")).id();
        let queue = EditorCommandQueue::new();
        queue
            .push(EditorCommand::SetComponent {
                entity: entity.to_bits(),
                type_id: crate::scene::stable_type_id("ph2d::ecs::Name"),
                data: name_blob_bytes("New"),
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        let n = sim.world_mut().get::<Name>(entity).unwrap();
        assert_eq!(n.as_str(), "New");
    }

    #[test]
    fn apply_remove_component_detaches_and_is_idempotent() {
        use crate::ZIndexOverride;
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let z_type_id = crate::scene::stable_type_id("ph2d::ecs::ZIndexOverride");

        // Attach via SetComponent, then detach via RemoveComponent.
        let entity = sim.world_mut().spawn(ZIndexOverride(5)).id();
        let queue = EditorCommandQueue::new();
        queue
            .push(EditorCommand::RemoveComponent {
                entity: entity.to_bits(),
                type_id: z_type_id,
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        assert!(
            sim.world_mut().get::<ZIndexOverride>(entity).is_none(),
            "RemoveComponent detaches the optional component"
        );

        // Removing an already-absent component is a no-op (idempotent),
        // and the entity survives.
        queue
            .push(EditorCommand::RemoveComponent {
                entity: entity.to_bits(),
                type_id: z_type_id,
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        assert!(
            sim.world().get_entity(entity).is_ok(),
            "entity still exists"
        );
    }

    #[test]
    fn apply_despawn_removes_entity() {
        let mut sim = SimWorld::new();
        let reg = ComponentRegistry::new();
        let entity = sim.world_mut().spawn(Name::new("Bye")).id();
        let queue = EditorCommandQueue::new();
        queue
            .push(EditorCommand::Despawn {
                entity: entity.to_bits(),
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        assert!(sim.world_mut().get_entity(entity).is_err());
    }

    #[test]
    fn apply_reparent_installs_child_of() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let parent = sim.world_mut().spawn(Transform::IDENTITY).id();
        let child = sim.world_mut().spawn(Transform::IDENTITY).id();
        let queue = EditorCommandQueue::new();
        queue
            .push(EditorCommand::Reparent {
                entity: child.to_bits(),
                new_parent: Some(parent.to_bits()),
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        let c = sim.world_mut().get::<ChildOf>(child).unwrap();
        assert_eq!(c.0, parent);
    }

    #[test]
    fn apply_reparent_none_removes_child_of() {
        let mut sim = SimWorld::new();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let parent = sim.world_mut().spawn(Transform::IDENTITY).id();
        let child = sim
            .world_mut()
            .spawn((Transform::IDENTITY, ChildOf(parent)))
            .id();
        let queue = EditorCommandQueue::new();
        queue
            .push(EditorCommand::Reparent {
                entity: child.to_bits(),
                new_parent: None,
            })
            .unwrap();
        apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        assert!(sim.world_mut().get::<ChildOf>(child).is_none());
    }
}
