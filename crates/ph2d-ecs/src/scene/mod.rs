//! Scene/Prefab spawn + ComponentRegistry — ADR-0025 M14.3.
//!
//! Bridges `ph2d-asset`'s cooked `PrefabDoc` / `SceneDoc` (postcard
//! wire format) to live `bevy_ecs::World` state via a manual
//! [`registry::ComponentRegistry`].

pub mod commands;
pub mod registry;
pub mod save;
pub mod snapshot;
pub mod spawn;

pub use commands::{
    ApplyError, EditorCommand, EditorCommandQueue, EditorQueueFull, apply_editor_commands,
};
pub use registry::{
    ComponentRegistry, ComponentTypeEntry, ComponentTypeId, RegistryError, register_ecs_components,
    stable_type_id,
};
pub use save::{EntitySnapshotRow, SaveError, WorldSnapshot, snapshot_to_world, world_to_snapshot};
pub use snapshot::{
    ComponentEntry, ComponentSnapshot, HierarchyEntry, HierarchySnapshot, HierarchyWalkState,
    build_hierarchy_snapshot, extract_component_snapshot,
};
pub use spawn::{SpawnError, spawn_prefab, spawn_scene};
