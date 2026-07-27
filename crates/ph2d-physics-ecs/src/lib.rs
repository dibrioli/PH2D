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

pub mod bake;
mod bridge;
mod components;
pub mod interaction;
mod joint;
mod joint_group;
pub mod joint_tool;
mod scale;
pub mod settings;

pub use bake::{BakedTrajectory, PoseChannel, bake_trajectories, bake_trajectories_with_scene};
pub use bridge::anchors::JointSide;
pub use bridge::contacts::{
    BodyContact, CONTACT_FLASH_TICKS, ContactEvent, ContactFlash, ContactPhase,
};
pub use bridge::joint_break::JointBreakEvent;
// O par de números que um readout de joint mostra. Re-exportado porque a shell
// não depende de `ph2d-physics` direto.
pub use bridge::fk::FkSession;
pub use bridge::ik::{IkPlan, IkSession};
pub use bridge::joints::joint_desc;
pub use bridge::views::JointView;
pub use bridge::{FrozenScene, PhysicsBridge, SceneAtTick};
pub use components::{
    AreaBuoyancy, AreaDrag, AreaEffector, AreaFalloff, AreaForceWorldAxes, AreaFormDrag,
    AreaTorque, BodyKind, Ccd, Collider, ColliderShape, CombineRule, DampMode, DampingOverride,
    Dominance, GravityScale, InitialVelocity, LockPositionX, LockPositionY, LockRotation,
    MassOverride, MaterialCombine, OneWayPlatform, RigidBody,
};
pub use interaction::{
    HoldMode, InteractionSettings, InteractionTool, MAX_ATTRACT_FORCE, MAX_BLAST_IMPULSE,
    MAX_HOLD_DAMPING_RATIO, MAX_HOLD_STIFFNESS, MIN_HOLD_STIFFNESS, WORLD_REACH_M,
};
pub use joint::{JointKind, MotorMode, PhysicsJoint};
pub use joint_group::{jointed_by, jointed_group, jointed_rig};
pub use joint_tool::{DragReach, JointGesture, JointTool};
pub use ph2d_physics::{IkOptions, JointLoad};
pub use scale::scaled_shape;
// `ShapeDesc` + the ellipse tessellation are re-exported so the overlay (in
// the shell, which only deps this crate) draws the SAME resolved shape the
// bridge simulates — one import path, one answer. `zone_force_world_at` is there
// for the same reason and it matters more: the arrow IS the only place the wind's
// direction is ever read by a person, so a second answer would be a picture of a
// blow that does not happen, and no gate reads a screenshot.
pub use ph2d_physics::{
    CAPSULE_CAP_SEGS, ELLIPSE_SEGS, LayerMatrix, MAX_LAYERS, PhysicsWorld, ShapeDesc,
    capsule_vertices, ellipse_vertices, zone_force_world_at, zone_spin_sign,
};
pub use settings::{
    DEFAULT_SOLVER_ITERATIONS, GRAVITY_LIMIT, MAX_AIR_DRAG, MAX_CONTACT_HZ, MAX_DAMPING,
    MAX_SLEEP_THRESHOLD, MAX_SOLVER_ITERATIONS, MAX_SUBSTEPS, MAX_TIME_UNTIL_SLEEP, MIN_CONTACT_HZ,
    PhysicsSettings,
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
    reg.register::<PhysicsJoint>("ph2d::physics::PhysicsJoint");
    reg.register::<GravityScale>("ph2d::physics::GravityScale");
    reg.register::<InitialVelocity>("ph2d::physics::InitialVelocity");
    reg.register::<Ccd>("ph2d::physics::Ccd");
    reg.register::<LockRotation>("ph2d::physics::LockRotation");
    reg.register::<LockPositionX>("ph2d::physics::LockPositionX");
    reg.register::<LockPositionY>("ph2d::physics::LockPositionY");
    reg.register::<MassOverride>("ph2d::physics::MassOverride");
    reg.register::<Dominance>("ph2d::physics::Dominance");
    reg.register::<MaterialCombine>("ph2d::physics::MaterialCombine");
    reg.register::<DampingOverride>("ph2d::physics::DampingOverride");
    reg.register::<OneWayPlatform>("ph2d::physics::OneWayPlatform");
    reg.register::<AreaEffector>("ph2d::physics::AreaEffector");
    reg.register::<AreaDrag>("ph2d::physics::AreaDrag");
    reg.register::<AreaBuoyancy>("ph2d::physics::AreaBuoyancy");
    reg.register::<AreaFormDrag>("ph2d::physics::AreaFormDrag");
    reg.register::<AreaTorque>("ph2d::physics::AreaTorque");
    reg.register::<AreaForceWorldAxes>("ph2d::physics::AreaForceWorldAxes");
    reg.register::<AreaFalloff>("ph2d::physics::AreaFalloff");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This count exists to hurt (mirrors `register_ecs_components_populates_registry`):
    /// a physics component that skips `register_physics_components` is
    /// dropped from every snapshot in silence. If you add one, bump this.
    #[test]
    fn registers_every_physics_component() {
        let mut reg = ComponentRegistry::new();
        register_physics_components(&mut reg);
        assert_eq!(reg.len(), 21);
        assert!(reg.get_by_name("ph2d::physics::RigidBody").is_some());
        assert!(reg.get_by_name("ph2d::physics::Collider").is_some());
        assert!(reg.get_by_name("ph2d::physics::PhysicsJoint").is_some());
        assert!(reg.get_by_name("ph2d::physics::GravityScale").is_some());
        assert!(reg.get_by_name("ph2d::physics::InitialVelocity").is_some());
        assert!(reg.get_by_name("ph2d::physics::Ccd").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockRotation").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockPositionX").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockPositionY").is_some());
        assert!(reg.get_by_name("ph2d::physics::MassOverride").is_some());
        assert!(reg.get_by_name("ph2d::physics::Dominance").is_some());
        assert!(reg.get_by_name("ph2d::physics::MaterialCombine").is_some());
        assert!(reg.get_by_name("ph2d::physics::DampingOverride").is_some());
        assert!(reg.get_by_name("ph2d::physics::OneWayPlatform").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaEffector").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaDrag").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaBuoyancy").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaFormDrag").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaTorque").is_some());
        assert!(
            reg.get_by_name("ph2d::physics::AreaForceWorldAxes")
                .is_some()
        );
        assert!(reg.get_by_name("ph2d::physics::AreaFalloff").is_some());
    }
}
