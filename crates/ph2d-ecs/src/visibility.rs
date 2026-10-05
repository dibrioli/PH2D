//! `Visibility` — runtime hide/show flag (M14.6A).
//!
//! Attached to any entity that participates in the render extract.
//! When `hidden == true` the extract path skips emitting a
//! `RenderInstance` for that entity (sprite vanishes from the
//! canvas) but the entity itself stays in `SimWorld` — name,
//! transform, components, ChildOf hierarchy all intact. Re-enabling
//! visibility brings it back without a re-spawn.
//!
//! **Invariant** (HR-5): absence of the component equals visible.
//! New entities default to visible; the editor only writes
//! `Visibility { hidden: true }` when the user toggles the eye icon
//! in the Hierarchy panel.
//!
//! Why a dedicated component instead of a `Sprite::hidden` field:
//! - **Extensible**: future render kinds (lights, particles, shapes)
//!   can opt-in to the same toggle without each duplicating the flag.
//! - **Schema-friendly**: `Sprite::VERSION` stays at 1 so save/replay
//!   fixtures don't need a bake refresh.
//! - **Lookup-cheap**: a bevy_ecs sparse component query is faster
//!   than reading + branching on a per-archetype field.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visibility {
    pub hidden: bool,
}

impl Visibility {
    pub const fn hidden() -> Self {
        Self { hidden: true }
    }

    pub const fn visible() -> Self {
        Self { hidden: false }
    }
}

/// Hidden by its own eye or by any ancestor's (hiding a group hides its children) —
/// the same `ChildOf` walk as [`crate::parent_world_transform`]. Readers: the skeleton
/// overlay and bone picking. ⚠️ The sprite extract reads the eye PER ENTITY
/// (`ph2d_entity_visibility::off_canvas::is_off_canvas`); the Flip gizmo walks the tree.
#[must_use]
pub fn is_hidden_in_tree(world: &bevy_ecs::world::World, entity: bevy_ecs::entity::Entity) -> bool {
    let mut cur = Some(entity);
    while let Some(e) = cur {
        if world.get::<Visibility>(e).is_some_and(|v| v.hidden) {
            return true;
        }
        cur = world.get::<crate::ChildOf>(e).map(|c| c.parent());
    }
    false
}
