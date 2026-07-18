#![forbid(unsafe_code)]
//! ph2d-physics-ecs — the bridge that wires the dormant `ph2d-physics`
//! rapier wrapper (M10) into the ECS as a **global, runtime-truth** rigid
//! world (ADR-0131 W1).
//!
//! - [`RigidBody`] / [`Collider`] are ECS components (config only) that an
//!   entity carries to become a physics body.
//! - [`PhysicsBridge`] owns the transient rapier world + the entity↔handle
//!   map and is driven once per frame at the `Playhead` tick.
//! - [`register_physics_components`] plugs the components into the shared
//!   [`ComponentRegistry`] so they persist in the `WorldSnapshot` (undo +
//!   save) for free — call it at boot next to `register_render_components`.
//!
//! rapier stays confined to `ph2d-physics`; this crate holds only the
//! re-exported handle types, so the `enhanced-determinism` feature pin
//! lives in exactly one Cargo.toml (HR-5).

mod bridge;
mod components;
pub mod settings;

pub use bridge::PhysicsBridge;
pub use components::{BodyKind, Collider, ColliderShape, RigidBody};
pub use settings::{
    DEFAULT_SOLVER_ITERATIONS, GRAVITY_LIMIT, MAX_CONTACT_HZ, MAX_DAMPING, MAX_SLEEP_THRESHOLD,
    MAX_SOLVER_ITERATIONS, MAX_SUBSTEPS, MAX_TIME_UNTIL_SLEEP, MIN_CONTACT_HZ, PhysicsSettings,
};

use ph2d_ecs::scene::ComponentRegistry;

/// Register the components owned by `ph2d-physics-ecs` against the shared
/// [`ComponentRegistry`]. The shell calls this once at boot alongside
/// `register_ecs_components` and `register_render_components`.
///
/// Without it the `WorldSnapshot` **silently drops** `RigidBody`/`Collider`
/// (the `Locked`/`GroupedChildren`/`VecPathRef` bug). Registered here, they
/// round-trip through undo + save with zero snapshot-side code.
pub fn register_physics_components(reg: &mut ComponentRegistry) {
    reg.register::<RigidBody>("ph2d::physics::RigidBody");
    reg.register::<Collider>("ph2d::physics::Collider");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This count exists to hurt (mirrors `register_ecs_components_populates_registry`):
    /// a physics component that skips `register_physics_components` is
    /// dropped from every snapshot in silence. If you add one, bump this.
    #[test]
    fn registers_both_physics_components() {
        let mut reg = ComponentRegistry::new();
        register_physics_components(&mut reg);
        assert_eq!(reg.len(), 2);
        assert!(reg.get_by_name("ph2d::physics::RigidBody").is_some());
        assert!(reg.get_by_name("ph2d::physics::Collider").is_some());
    }
}
