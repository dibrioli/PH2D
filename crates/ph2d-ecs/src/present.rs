//! `PresentWorld` — opaque newtype over `bevy_ecs::World` for
//! presentation state (render, animation, particles, editor).
//! ADR-0021.

use bevy_ecs::component::Component;
use bevy_ecs::world::World;

/// Marker trait for components that belong in [`PresentWorld`].
///
/// **Convention** (M4): a component implements `PresentComponent`
/// iff it's derived state for rendering / animation / editor and is
/// rebuilt each frame from `SimWorld` via the `extract!` macro.
/// `PresentComponent` types are NOT serialized to save (HR-14) and
/// are NOT part of replay determinism (HR-5) — they're free to use
/// non-deterministic data (random jitter, timestamps, GPU handles).
pub trait PresentComponent: Component {}

/// Opaque newtype over `bevy_ecs::World` for presentation state.
///
/// Mirror of [`crate::SimWorld`] — same API shape; semantically
/// distinct so type errors catch accidental cross-pollination.
pub struct PresentWorld {
    inner: World,
}

impl PresentWorld {
    pub fn new() -> Self {
        Self {
            inner: World::new(),
        }
    }

    pub fn world(&self) -> &World {
        &self.inner
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.inner
    }

    pub fn entity_count(&mut self) -> usize {
        let mut q = self.inner.query::<bevy_ecs::entity::Entity>();
        q.iter(&self.inner).count()
    }
}

impl Default for PresentWorld {
    fn default() -> Self {
        Self::new()
    }
}
