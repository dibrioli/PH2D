//! Plain-data descriptors for [`crate::world::PhysicsWorld`] — the body the
//! ECS bridge asks for, and the snapshot it reads back.
//!
//! Split out of `world.rs` so it stays under the LOC cap. `BodyDesc` names one
//! rapier type (`RigidBodyType`), so — unlike [`super::shape::ShapeDesc`] — it
//! is not rapier-free; that is why they live in separate siblings.

use rapier2d::dynamics::RigidBodyType;

use super::shape::ShapeDesc;

/// Snapshot of one rigid body for hashing / inspection. Sorted by
/// handle index in [`crate::world::PhysicsWorld::body_snapshots`] so cross-OS
/// hashing is stable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodySnapshot {
    pub handle_index: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub linvel_x: f32,
    pub linvel_y: f32,
    pub angvel: f32,
}

/// One body + its single collider, described in plain data for the ECS
/// bridge. All lengths are world units (meters); `rotation` is radians
/// CCW; `density` feeds rapier's mass computation (ignored for
/// non-dynamic bodies). **Append-only:** new fields go at the END.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyDesc {
    pub body_type: RigidBodyType,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub density: f32,
    pub shape: ShapeDesc,
    /// Bounciness `0..=1`. rapier's own default is `0.0`.
    pub restitution: f32,
    /// Coulomb friction. rapier's own default is `0.5`.
    pub friction: f32,
    /// Which collision layer this body belongs to (`0..MAX_LAYERS`).
    ///
    /// Only the LAYER travels here — never the resulting `InteractionGroups`.
    /// The filter is a function of `(layer, world matrix)`, and computing it in
    /// two places is how the two would come to disagree; [`crate::world::PhysicsWorld`]
    /// owns the matrix and is the single door (see `layers`).
    pub layer: u8,
    /// A **sensor** (trigger): the collider passes through — no contact forces,
    /// nothing is pushed — but the solver still reports which colliders overlap
    /// it. Default `false` is a solid collider, byte-identical to before sensors
    /// existed. The overlaps are read back through
    /// [`crate::world::PhysicsWorld::intersecting_body_pairs`]; a sensor with
    /// nothing reading them is inert, which is why this landed with the overlay
    /// + Inspector that make the detection visible (ADR-0131 W7).
    pub is_sensor: bool,
    /// Per-body multiplier on the world gravity. `1.0` = full gravity (rapier's
    /// own default, and what every body did before this field existed); `0.0` =
    /// weightless (a bullet, a top-down actor); `< 0` = floats up (a balloon);
    /// `> 1` = heavier. rapier ignores it for non-dynamic bodies, so it only
    /// bites a `Dynamic` one.
    ///
    /// It lives in `BodyDesc` — the spawn recipe the world stores per body —
    /// because `rewind_to` rebuilds the world FROM these descriptors, so a value
    /// that were not here would be silently dropped on the first scrub, and a
    /// gravity-scaled body would fall at full gravity after a rewind. The
    /// authored source is the optional `GravityScale` component in
    /// `ph2d-physics-ecs`; the bridge folds it in here (ADR-0131 W8).
    pub gravity_scale: f32,
}
