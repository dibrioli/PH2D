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
