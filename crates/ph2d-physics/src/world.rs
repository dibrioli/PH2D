//! [`PhysicsWorld`] — Rapier2D wrapper sized to PH2D's 2D needs.
//!
//! All cross-frame state lives on this struct. Callers create it,
//! add bodies + colliders, then call [`PhysicsWorld::step`] once per
//! fixed-step tick (see [`ph2d_core::FixedStep`]).

use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, RigidBody,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet, RigidBodyType,
};
use rapier2d::geometry::{
    BroadPhaseBvh, ColliderBuilder, ColliderHandle, ColliderSet, NarrowPhase,
};
use rapier2d::na::{Isometry2, Vector2};
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::nalgebra;

/// Snapshot of one rigid body for hashing / inspection. Sorted by
/// handle index in [`PhysicsWorld::body_snapshots`] so cross-OS
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

/// Collider silhouette for [`PhysicsWorld::spawn_body`]. Kept as plain
/// data (no rapier types) so the ECS bridge can build one without a
/// direct `rapier2d` dependency — rapier stays confined to this crate
/// (SKILL §7 "don't couple public API to external types"). **Append-only:**
/// new variants go at the END (Capsule/Triangle/… land with W2/W3).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShapeDesc {
    /// Circle of `radius` (world units = meters).
    Ball { radius: f32 },
    /// Axis-aligned box with HALF-extents (rapier convention).
    Cuboid { half_x: f32, half_y: f32 },
}

/// One body + its single collider, described in plain data for the ECS
/// bridge. All lengths are world units (meters); `rotation` is radians
/// CCW; `density` feeds rapier's mass computation (ignored for
/// non-dynamic bodies). **Append-only:** restitution/friction/damping
/// land as new fields with W2 (defaults preserve today's behavior).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyDesc {
    pub body_type: RigidBodyType,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub density: f32,
    pub shape: ShapeDesc,
}

pub struct PhysicsWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    gravity: Vector2<f32>,
    /// Step counter — exposes a deterministic "tick number" so tests
    /// can advance N steps and report.
    step_count: u64,
}

impl PhysicsWorld {
    /// Default integration step. 60 Hz lockstep matches the rest of
    /// the engine ([`ph2d_core::FixedStep::DEFAULT_HZ`]).
    pub const DEFAULT_DT: f32 = 1.0 / 60.0;

    /// Default gravity (Y-up world per SKILL §11.1, so down is -y).
    /// 9.81 m/s² Earth-standard.
    pub const DEFAULT_GRAVITY_Y: f32 = -9.81;

    pub fn new() -> Self {
        let integration_parameters = IntegrationParameters {
            dt: Self::DEFAULT_DT,
            ..IntegrationParameters::default()
        };
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_pipeline: PhysicsPipeline::new(),
            integration_parameters,
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            gravity: Vector2::new(0.0, Self::DEFAULT_GRAVITY_Y),
            step_count: 0,
        }
    }

    /// Override gravity. Useful for top-down 2D (set to zero) or
    /// custom worlds.
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vector2::new(x, y);
    }

    /// Override the integration dt. Must match the caller's
    /// FixedStep — mismatched dt destroys both accuracy and
    /// determinism.
    pub fn set_dt(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
    }

    pub fn dt(&self) -> f32 {
        self.integration_parameters.dt
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Spawn a dynamic rigid body at `(x, y)` with a circle collider
    /// of `radius` and `density`. Convenience for fixtures + early
    /// games; full-fidelity callers reach for [`PhysicsWorld::bodies_mut`]
    /// and the rapier builders directly.
    pub fn add_dynamic_circle(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        density: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::dynamic()
            .translation(Vector2::new(x, y))
            .build();
        let body_handle = self.bodies.insert(body);
        let collider = ColliderBuilder::ball(radius).density(density).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        (body_handle, collider_handle)
    }

    /// Spawn a static cuboid (e.g. a floor or wall). `half_x` and
    /// `half_y` are HALF-EXTENTS (rapier convention).
    pub fn add_static_cuboid(
        &mut self,
        x: f32,
        y: f32,
        half_x: f32,
        half_y: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(Vector2::new(x, y))
            .build();
        let body_handle = self.bodies.insert(body);
        let collider = ColliderBuilder::cuboid(half_x, half_y).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        (body_handle, collider_handle)
    }

    /// Spawn a body of any [`RigidBodyType`] with one attached collider,
    /// from a plain [`BodyDesc`]. The general constructor the ECS bridge
    /// (`ph2d-physics-ecs`) drives — it covers every body×shape combo the
    /// two convenience helpers above don't (dynamic cuboid, static ball,
    /// kinematic, …). Returns the body handle; the bridge reads its pose
    /// back via [`PhysicsWorld::body_pose`].
    ///
    /// Additive — the existing helpers, `step`, and the C9 hash are
    /// untouched, so the cross-OS determinism gate stays byte-identical.
    pub fn spawn_body(&mut self, desc: BodyDesc) -> RigidBodyHandle {
        let body = RigidBodyBuilder::new(desc.body_type)
            .translation(Vector2::new(desc.x, desc.y))
            .rotation(desc.rotation)
            .build();
        let handle = self.bodies.insert(body);
        let shape = match desc.shape {
            ShapeDesc::Ball { radius } => ColliderBuilder::ball(radius),
            ShapeDesc::Cuboid { half_x, half_y } => ColliderBuilder::cuboid(half_x, half_y),
        };
        let collider = shape.density(desc.density).build();
        self.colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        handle
    }

    /// Teleport a body to `(x, y, rotation)` (world units, radians) and
    /// reset its velocity — the "settle to the authored pose" operation
    /// the bridge uses while paused, so a body sits exactly where the
    /// artist placed it before play. `wake` requests the body be woken
    /// (pass `true` when the pose actually changed).
    pub fn set_body_pose(
        &mut self,
        handle: RigidBodyHandle,
        x: f32,
        y: f32,
        rotation: f32,
        wake: bool,
    ) {
        if let Some(b) = self.bodies.get_mut(handle) {
            b.set_position(Isometry2::new(Vector2::new(x, y), rotation), wake);
            b.set_linvel(Vector2::zeros(), wake);
            b.set_angvel(0.0, wake);
        }
    }

    /// Remove a body and its attached colliders (used when the ECS entity
    /// carrying it is despawned). No-op if the handle is already gone.
    pub fn remove_body(&mut self, handle: RigidBodyHandle) {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    /// Spawn a body explicitly (advanced — used when you need a
    /// non-circle / non-cuboid shape or non-default body type).
    pub fn insert_body(&mut self, body: RigidBody) -> RigidBodyHandle {
        self.bodies.insert(body)
    }

    pub fn bodies(&self) -> &RigidBodySet {
        &self.bodies
    }
    pub fn bodies_mut(&mut self) -> &mut RigidBodySet {
        &mut self.bodies
    }
    pub fn colliders(&self) -> &ColliderSet {
        &self.colliders
    }
    pub fn colliders_mut(&mut self) -> &mut ColliderSet {
        &mut self.colliders
    }

    /// Advance one fixed step. Always uses
    /// [`PhysicsWorld::dt`] — never accept an external dt at the
    /// wrapper boundary (HR-5).
    pub fn step(&mut self) {
        let physics_hooks = ();
        let event_handler = ();
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &event_handler,
        );
        self.step_count += 1;
    }

    /// Convenience: read the position + orientation of a body.
    pub fn body_pose(&self, handle: RigidBodyHandle) -> Option<Isometry2<f32>> {
        self.bodies.get(handle).map(|b| *b.position())
    }

    /// Iterate every dynamic body's snapshot, sorted by handle index
    /// for stable order across runs / OSes.
    pub fn body_snapshots(&self) -> Vec<BodySnapshot> {
        let mut out: Vec<BodySnapshot> = self
            .bodies
            .iter()
            .map(|(handle, body)| {
                let pos = body.position();
                let lin = body.linvel();
                BodySnapshot {
                    handle_index: handle.into_raw_parts().0,
                    x: pos.translation.x,
                    y: pos.translation.y,
                    rotation: pos.rotation.angle(),
                    linvel_x: lin.x,
                    linvel_y: lin.y,
                    angvel: body.angvel(),
                }
            })
            .collect();
        out.sort_by_key(|s| s.handle_index);
        out
    }

    /// blake3 digest over the sorted body snapshots — the C9 cross-OS
    /// gate in CI compares this hash across Linux / macOS / Windows.
    /// MUST be byte-identical or HR-5 is violated.
    pub fn deterministic_hash(&self) -> [u8; 32] {
        let snapshots = self.body_snapshots();
        let mut hasher = blake3::Hasher::new();
        for s in &snapshots {
            hasher.update(&s.handle_index.to_le_bytes());
            hasher.update(&s.x.to_bits().to_le_bytes());
            hasher.update(&s.y.to_bits().to_le_bytes());
            hasher.update(&s.rotation.to_bits().to_le_bytes());
            hasher.update(&s.linvel_x.to_bits().to_le_bytes());
            hasher.update(&s.linvel_y.to_bits().to_le_bytes());
            hasher.update(&s.angvel.to_bits().to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

// `nalgebra` and `Isometry2` re-exports are part of the rapier surface;
// silence the "unused import" if the lib's surface ever stops needing
// them while keeping the surface stable.
#[allow(dead_code)]
fn _force_imports_alive() {
    let _ = std::mem::size_of::<nalgebra::Vector2<f32>>();
    let _ = std::mem::size_of::<RigidBodyType>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_world_has_zero_bodies() {
        let w = PhysicsWorld::new();
        assert_eq!(w.body_snapshots().len(), 0);
        assert_eq!(w.step_count(), 0);
    }

    #[test]
    fn step_advances_counter() {
        let mut w = PhysicsWorld::new();
        w.step();
        w.step();
        w.step();
        assert_eq!(w.step_count(), 3);
    }

    #[test]
    fn dt_default_is_60hz() {
        let w = PhysicsWorld::new();
        // 1/60 ≈ 0.01666...; allow tiny f32 rounding.
        assert!((w.dt() - 1.0 / 60.0).abs() < 1e-6);
    }

    #[test]
    fn falling_body_hits_floor() {
        let mut w = PhysicsWorld::new();
        // Floor at y=0, half-thickness 0.1.
        w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
        // Ball at y=10 (10 m above floor).
        let (ball, _) = w.add_dynamic_circle(0.0, 10.0, 0.5, 1.0);
        // Step long enough to settle (gravity 9.81 m/s²; free-fall
        // from 10m takes ~1.43s; settling takes a few more).
        for _ in 0..600 {
            w.step();
        }
        let pose = w.body_pose(ball).expect("ball still exists");
        // Ball center should be near floor + radius (~ 0.1 + 0.5 = 0.6).
        assert!(
            pose.translation.y >= 0.5 && pose.translation.y <= 1.0,
            "ball settled at y={}, expected ~0.6",
            pose.translation.y
        );
    }

    #[test]
    fn hash_is_stable_across_runs_in_same_process() {
        // Same fixture, same hash. Cross-OS test lives in the bin
        // (tests/spike-style C9 extension) — this is a sanity check
        // that the hashing function itself is deterministic on one
        // OS, not affected by allocation order or HashMap iteration.
        let h1 = run_50_body_fixture();
        let h2 = run_50_body_fixture();
        assert_eq!(h1, h2);
    }

    fn run_50_body_fixture() -> [u8; 32] {
        let mut w = PhysicsWorld::new();
        w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
        for i in 0..50 {
            let row = (i / 10) as f32;
            let col = (i % 10) as f32;
            w.add_dynamic_circle(col * 0.6 - 2.7, 5.0 + row * 0.6, 0.25, 1.0);
        }
        for _ in 0..120 {
            w.step();
        }
        w.deterministic_hash()
    }
}
