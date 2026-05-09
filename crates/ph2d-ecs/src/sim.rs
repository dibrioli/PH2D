//! `SimWorld` — opaque newtype over `bevy_ecs::World` for simulation
//! state. ADR-0021 enforcement layer.

use bevy_ecs::component::Component;
use bevy_ecs::world::World;

/// Marker trait for components that belong in [`SimWorld`].
///
/// **Convention** (M4): a component implements `SimComponent` iff its
/// value affects simulation outcome (Position, Velocity, Health, FSM
/// state). Bundle-level enforcement (preventing accidental
/// `present.spawn(SimComp)`) lands in M5+ via clippy custom lint.
///
/// Components that derive `Saveable` (HR-14) MUST be `SimComponent`.
pub trait SimComponent: Component {}

/// Opaque newtype over `bevy_ecs::World` for simulation state.
///
/// API exposes:
/// - [`SimWorld::new`] — empty world.
/// - [`SimWorld::world`] — `&World` (immutable). Use for queries
///   that only need to read sim state (the canonical use is from
///   inside an [`crate::extract!`] macro body).
/// - [`SimWorld::world_mut`] — `&mut World`. Use this from inside
///   sim systems / sim schedule. Calling it from a presentation
///   system is the lint target of `ph2d-clippy::wrong-world-access`.
pub struct SimWorld {
    inner: World,
}

impl SimWorld {
    pub fn new() -> Self {
        Self {
            inner: World::new(),
        }
    }

    /// Read-only access to the underlying `bevy_ecs::World`. Used by
    /// the [`crate::extract!`] macro to expose sim state to
    /// presentation systems without letting them mutate it.
    pub fn world(&self) -> &World {
        &self.inner
    }

    /// Mutable access. Sim-internal systems use this; presentation
    /// systems must not (lint enforces in M5+).
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.inner
    }

    /// Convenience: count of all entities in the sim world (test/debug only).
    /// Takes `&mut self` because `bevy_ecs::World::query` mutates internal
    /// query cache state.
    pub fn entity_count(&mut self) -> usize {
        let mut q = self.inner.query::<bevy_ecs::entity::Entity>();
        q.iter(&self.inner).count()
    }
}

impl Default for SimWorld {
    fn default() -> Self {
        Self::new()
    }
}
