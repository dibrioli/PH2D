//! [`PhysicsWorld`] — Rapier2D wrapper sized to PH2D's 2D needs.
//!
//! All cross-frame state lives on this struct. Callers create it,
//! add bodies + colliders, then call [`PhysicsWorld::step`] once per
//! fixed-step tick (see [`ph2d_core::FixedStep`]).

pub mod checkpoint;
pub mod defaults;
pub mod desc;
pub mod drag;
pub mod joints;
pub mod kinematic;
pub mod layers;
pub mod sensors;
pub mod shape;

use defaults::BodyDefaults;
use layers::LayerMatrix;
// The descriptors and the shape vocabulary live in sibling modules (LOC),
// re-exported so callers still see `ph2d_physics::{BodyDesc, ShapeDesc, …}`.
pub use desc::{BodyDesc, BodySnapshot};
use rapier2d::geometry::{Group, InteractionGroups};
pub use shape::{ELLIPSE_SEGS, ShapeDesc, ellipse_vertices};

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
    /// Sub-steps per `step()` (see [`PhysicsWorld::set_substeps`]).
    substeps: u32,
    /// The caller's tick length; `integration_parameters.dt` is this divided
    /// by [`Self::substeps`].
    base_dt: f32,
    /// World-level values every new body is born with (damping, sleep). See
    /// [`BodyDefaults`] for why a per-body rapier concept is a world setting
    /// here — and for the one rule that keeps it a single door.
    body_defaults: BodyDefaults,
    /// Which layers collide with which. See [`layers`] for why this lives on
    /// the world and why it cannot be asymmetric.
    layer_matrix: LayerMatrix,
    /// Air-drag coefficient (lumps `½·ρ·Cd`). `0.0` = vacuum, and byte-identical
    /// to a build without the feature. See [`drag`] for why this is a separate
    /// model from `body_defaults.linear_damping` rather than a tuning of it.
    air_drag: f32,
    /// Where each kinematic body was told to be by the END of this tick:
    /// `(handle, pose at the start of the tick, pose to arrive at)`.
    ///
    /// It is a field and not a local because [`PhysicsWorld::step`] is the hot
    /// path with a zero-alloc gate on it — cleared per tick, so the capacity is
    /// reached once and never allocated again. Empty in a world with no
    /// kinematic bodies, which is what keeps `step` byte-identical to the one
    /// that shipped before this existed.
    kinematic_targets: Vec<(RigidBodyHandle, Isometry2<f32>, Isometry2<f32>)>,
}

impl PhysicsWorld {
    /// Default integration step. 60 Hz lockstep matches the rest of
    /// the engine ([`ph2d_core::FixedStep::DEFAULT_HZ`]).
    pub const DEFAULT_DT: f32 = 1.0 / 60.0;

    /// Default gravity (Y-up world per SKILL §11.1, so down is -y).
    /// 9.81 m/s² Earth-standard.
    pub const DEFAULT_GRAVITY_Y: f32 = -9.81;

    /// Integration sub-steps per tick. **Not rapier's default of 1** —
    /// chosen by measurement (Enio, 2026-07-18: *"observa-se alguma
    /// interpenetração dos objetos dinâmicos com o chão"*).
    ///
    /// A body landing at 9.4 m/s travels 157 mm in a 60 Hz tick, so on the
    /// tick it first touches it is **already 83 mm inside the floor** — and
    /// no solver knob moves that number, because it is not a solver failure,
    /// it is `velocity × dt`. Measured: contact damping, corrective-velocity
    /// ceiling, extra solver iterations and even CCD all left the depth at
    /// 83.2 mm. (CCD does nothing here because nothing *tunnels* — 83 mm of
    /// overlap on a 560 mm body is not a missed collision.)
    ///
    /// Sub-stepping is the only lever on the depth, and it is linear:
    /// 1→83 mm, 2→73, 4→31, 8→8.8. Four is the knee — Box2D v3 ships the
    /// same default for the same reason — and costs 264 µs for 500 bodies
    /// against HR-4's 1.5 ms.
    pub const DEFAULT_SUBSTEPS: u32 = 4;

    /// Contact spring frequency, Hz. **Not rapier's default of 30** — that
    /// governs how fast the remaining overlap is pushed back out, which is
    /// the other half of what the artist sees: 30 Hz took 9 frames to become
    /// invisible, 120 Hz takes 1.
    ///
    /// Raising *damping* instead (rapier's usual advice for a stiffer look)
    /// was measured and goes the wrong way here — 5.0 is already overdamped,
    /// and 20 stretched the recovery from 9 frames to 30. Damping is left at
    /// rapier's tuned default.
    pub const DEFAULT_CONTACT_HZ: f32 = 120.0;

    pub fn new() -> Self {
        let integration_parameters = IntegrationParameters {
            dt: Self::DEFAULT_DT / Self::DEFAULT_SUBSTEPS as f32,
            contact_natural_frequency: Self::DEFAULT_CONTACT_HZ,
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
            substeps: Self::DEFAULT_SUBSTEPS,
            base_dt: Self::DEFAULT_DT,
            body_defaults: BodyDefaults::rapier(),
            layer_matrix: LayerMatrix::all(),
            air_drag: 0.0,
            kinematic_targets: Vec::new(),
        }
    }

    /// Override gravity. Useful for top-down 2D (set to zero) or
    /// custom worlds.
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vector2::new(x, y);
    }

    /// Contact response tuning, as plain numbers (rapier's
    /// `IntegrationParameters` stays inside this crate).
    ///
    /// - `damping_ratio` — rapier default `5.0`. Its own docs name this as
    ///   the knob to reach for when the simulation should *look stiffer*,
    ///   in preference to raising the contact natural frequency (which
    ///   overshoots and jitters).
    /// - `max_corrective_velocity` — rapier default `10.0` m/s. The ceiling
    ///   on how fast the solver is allowed to push accumulated penetration
    ///   back out.
    ///
    /// ⚠️ These feed the solver, so changing them **changes every
    /// simulation** — including the cross-OS C9 hash. That is a deliberate
    /// product decision, not a free knob.
    pub fn set_contact_response(&mut self, damping_ratio: f32, max_corrective_velocity: f32) {
        self.integration_parameters.contact_damping_ratio = damping_ratio;
        self.integration_parameters
            .normalized_max_corrective_velocity = max_corrective_velocity;
    }

    /// Contact spring frequency, Hz (rapier default `30.0`). rapier's docs:
    /// *"increasing this value will make it so that penetrations get fixed
    /// more quickly at the expense of potential jitter due to overshooting"*.
    pub fn set_contact_frequency(&mut self, hz: f32) {
        self.integration_parameters.contact_natural_frequency = hz;
    }

    /// How many integration sub-steps one [`PhysicsWorld::step`] runs.
    ///
    /// The **only** lever on how deep a fast body is already overlapping the
    /// frame it first touches: that depth is `velocity × dt` and no solver
    /// can undo it after the fact. Halving the sub-step halves the overlap,
    /// at a proportional cost.
    pub fn set_substeps(&mut self, n: u32) {
        self.substeps = n.max(1);
        self.integration_parameters.dt = self.base_dt / self.substeps as f32;
    }

    /// Number of solver iterations per step (rapier default `4`). More
    /// iterations resolve a stack's contacts more completely, at linear cost.
    pub fn set_solver_iterations(&mut self, n: usize) {
        if let Some(n) = std::num::NonZeroUsize::new(n) {
            self.integration_parameters.num_solver_iterations = n;
        }
    }

    /// The world-level values new bodies are born with (damping, sleep).
    pub fn body_defaults(&self) -> BodyDefaults {
        self.body_defaults
    }

    /// Replace the world-level body defaults.
    ///
    /// **Applies to the bodies that already exist, not only to future ones.**
    /// The artist is describing the world in front of them; a drag value that
    /// only reached the next body spawned would be a number that appears to do
    /// nothing. See [`BodyDefaults`] for why these are world settings at all.
    pub fn set_body_defaults(&mut self, d: BodyDefaults) {
        self.body_defaults = d;
        d.apply_to_all(&mut self.bodies);
    }

    /// Which layers collide with which.
    pub fn layer_matrix(&self) -> LayerMatrix {
        self.layer_matrix
    }

    /// Replace the collision-layer matrix.
    ///
    /// **Applies to the colliders that already exist**, for the same reason
    /// [`Self::set_body_defaults`] does: the artist is describing the scene in
    /// front of them, and a rule that only reached the next body spawned would
    /// look like a dead checkbox.
    ///
    /// A collider already carries its own layer — it is `memberships`, a single
    /// bit — so the layer never has to be stored twice or looked up elsewhere.
    pub fn set_layer_matrix(&mut self, matrix: LayerMatrix) {
        self.layer_matrix = matrix;
        for (_, collider) in self.colliders.iter_mut() {
            let membership_bits = collider.collision_groups().memberships.bits();
            let layer = membership_bits.trailing_zeros() as usize;
            collider.set_collision_groups(groups_for(layer, matrix));
        }
    }

    /// The air-drag coefficient. `0.0` = vacuum.
    pub fn air_drag(&self) -> f32 {
        self.air_drag
    }

    /// Set the air-drag coefficient (lumps `½·ρ·Cd` — "how thick is the air").
    ///
    /// Unlike [`Self::set_body_defaults`], this touches no body: drag is a
    /// property of the WORLD, applied as a force each substep, so there is
    /// nothing to stamp and nothing to wake.
    pub fn set_air_drag(&mut self, k: f32) {
        self.air_drag = k.max(0.0);
    }

    /// Put a freshly inserted collider on its layer — the one place a `(layer,
    /// matrix)` pair becomes rapier `InteractionGroups`, shared with
    /// [`Self::set_layer_matrix`] so spawning and re-filtering can never
    /// disagree about what a layer means.
    fn stamp_layer(&mut self, handle: ColliderHandle, layer: usize) {
        let groups = groups_for(layer, self.layer_matrix);
        if let Some(c) = self.colliders.get_mut(handle) {
            c.set_collision_groups(groups);
        }
    }

    /// Stamp the world defaults onto a body that was just inserted — the single
    /// place every spawn path funnels through, so "every body in this world
    /// carries this world's defaults" is true by construction rather than by a
    /// list of call sites someone has to keep in sync.
    fn stamp_defaults(&mut self, handle: RigidBodyHandle) {
        if let Some(body) = self.bodies.get_mut(handle) {
            self.body_defaults.apply_to(body);
        }
    }

    /// Override the integration dt. Must match the caller's
    /// FixedStep — mismatched dt destroys both accuracy and
    /// determinism.
    pub fn set_dt(&mut self, dt: f32) {
        self.base_dt = dt;
        self.integration_parameters.dt = dt / self.substeps as f32;
    }

    /// The **tick** length — what one [`PhysicsWorld::step`] advances, and
    /// what must match the caller's `FixedStep`.
    ///
    /// Deliberately not the integrator's dt: `step()` runs
    /// [`PhysicsWorld::substeps`] internal integrations of `dt/substeps`
    /// each. One name, one meaning — a `dt()` that silently became the
    /// sub-step length would quietly disagree with the Playhead.
    pub fn dt(&self) -> f32 {
        self.base_dt
    }

    /// The integrator's own step: [`Self::dt`] divided by [`Self::substeps`].
    pub fn substep_dt(&self) -> f32 {
        self.integration_parameters.dt
    }

    /// Integration sub-steps per [`PhysicsWorld::step`].
    pub fn substeps(&self) -> u32 {
        self.substeps
    }

    /// How many kinematic aims are queued for the next [`PhysicsWorld::step`]
    /// — the per-sub-step replay list. Exists so the refusal in
    /// [`PhysicsWorld::set_next_kinematic_pose`] is a claim a test can check
    /// rather than a comment; the effect it guards is cost, and cost is not
    /// visible in a pose.
    ///
    /// [`step`]: PhysicsWorld::step
    #[doc(hidden)]
    #[must_use]
    pub fn kinematic_aim_count(&self) -> usize {
        self.kinematic_targets.len()
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
        self.stamp_defaults(body_handle);
        let collider = ColliderBuilder::ball(radius).density(density).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.stamp_layer(collider_handle, 0);
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
        self.stamp_defaults(body_handle);
        let collider = ColliderBuilder::cuboid(half_x, half_y).build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);
        self.stamp_layer(collider_handle, 0);
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
            // Per-body gravity multiplier (W8). Setting `1.0` explicitly is
            // rapier's own default, so an unscaled body is byte-identical to
            // before this existed; the value survives rewind because it rides
            // the `BodyDesc` the world rebuilds from.
            .gravity_scale(desc.gravity_scale)
            .build();
        let handle = self.bodies.insert(body);
        self.stamp_defaults(handle);
        let shape = match desc.shape {
            ShapeDesc::Ball { radius } => ColliderBuilder::ball(radius),
            ShapeDesc::Cuboid { half_x, half_y } => ColliderBuilder::cuboid(half_x, half_y),
            ShapeDesc::Ellipse { rx, ry } => {
                let pts: Vec<_> = ellipse_vertices(rx, ry)
                    .into_iter()
                    .map(|[x, y]| nalgebra::Point2::new(x, y))
                    .collect();
                // `convex_polyline` returns None only on a degenerate ring
                // (an axis scaled to ~0). That is not a shape a real sprite
                // produces, but a `None` here must not panic the spawn — fall
                // back to a ball of the larger half-extent so the body still
                // exists and collides.
                ColliderBuilder::convex_polyline(pts)
                    .unwrap_or_else(|| ColliderBuilder::ball(rx.max(ry).max(f32::MIN_POSITIVE)))
            }
        };
        let collider = shape
            .density(desc.density)
            .restitution(desc.restitution)
            .friction(desc.friction)
            // A sensor passes through (no contact forces) but the narrow phase
            // still records its overlaps — read back by `intersecting_body_pairs`.
            .sensor(desc.is_sensor)
            .build();
        let collider_handle = self
            .colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        self.stamp_layer(collider_handle, desc.layer as usize);
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
        let handle = self.bodies.insert(body);
        self.stamp_defaults(handle);
        handle
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
        for sub in 0..self.substeps {
            // Kinematic bodies advance a SLICE of their tick per sub-step, for
            // the same reason the drag below is applied per sub-step: rapier
            // derives a kinematic body's velocity from how far it was told to
            // move THIS sub-step, so handing it the whole tick at once reports
            // a speed the body does not have. Empty unless something is
            // kinematic, which is what keeps this loop byte-identical for every
            // world that has none.
            let f = (sub + 1) as f32 / self.substeps as f32;
            for i in 0..self.kinematic_targets.len() {
                let (handle, start, target) = self.kinematic_targets[i];
                if let Some(b) = self.bodies.get_mut(handle) {
                    b.set_next_kinematic_position(Self::kinematic_slice(&start, &target, f));
                }
            }
            // Per SUBSTEP, not per tick: a force applied once per tick would be
            // wrong by the substep count.
            drag::apply(
                &mut self.bodies,
                &self.colliders,
                self.air_drag,
                self.integration_parameters.dt,
            );
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
        }
        // The aim is spent. Retaining the capacity is why this is a field:
        // the next tick refills it without allocating (the zero-alloc gate on
        // the hot path measures exactly that).
        self.kinematic_targets.clear();
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

/// `(layer, matrix)` → rapier's `InteractionGroups`. **The single door.**
///
/// `memberships` is the body's own layer bit — which is also how a collider
/// remembers its layer, so nothing else has to store it. `filter` is that
/// layer's row of the matrix.
fn groups_for(layer: usize, matrix: LayerMatrix) -> InteractionGroups {
    let layer = layer.min(layers::MAX_LAYERS - 1);
    InteractionGroups::new(
        Group::from_bits_truncate(1 << layer),
        Group::from_bits_truncate(u32::from(matrix.row(layer))),
    )
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
