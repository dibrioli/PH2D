//! [`PhysicsWorld`] — Rapier2D wrapper sized to PH2D's 2D needs.
//!
//! All cross-frame state lives on this struct. Callers create it,
//! add bodies + colliders, then call [`PhysicsWorld::step`] once per
//! fixed-step tick (see [`ph2d_core::FixedStep`]).

pub mod blast;
pub mod buoyancy;
pub mod checkpoint;
pub mod collider_build;
pub mod contacts;
mod convenience;
pub mod damping;
pub mod defaults;
pub mod desc;
pub mod drag;
pub mod effector;
pub mod form_drag;
pub mod grab;
pub mod ik;
pub mod ik_coords;
pub mod joint_break;
pub mod joint_desc;
pub mod joint_gains;
pub mod joints;
pub mod kinematic;
pub mod layers;
pub mod oneway;
pub mod pulley;
pub mod queries;
pub mod rope_route;
pub mod sensors;
pub mod shape;
pub mod tuning;

use defaults::BodyDefaults;
use layers::LayerMatrix;
// The descriptors and the shape vocabulary live in sibling modules (LOC),
// re-exported so callers still see `ph2d_physics::{BodyDesc, ShapeDesc, …}`.
pub use desc::{AreaEffect, BodyDesc, BodySnapshot, CombineRules, DampingDesc};
use rapier2d::geometry::{Group, InteractionGroups};
pub use shape::{CAPSULE_CAP_SEGS, ELLIPSE_SEGS, ShapeDesc, capsule_vertices, ellipse_vertices};

use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, LockedAxes,
    MultibodyJointSet, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, RigidBodyType,
};
use rapier2d::geometry::{BroadPhaseBvh, ColliderHandle, ColliderSet, NarrowPhase};
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
    /// The registered **force zones** (W-Area): a sensor body and what it does to
    /// whatever overlaps it — a force in newtons, a drag that resists, or both.
    /// Derived from `BodyDesc` at spawn through the single door
    /// `effector::zone_effect`, so it is CONFIG —
    /// nothing in the step loop writes it, which is why a checkpoint restore
    /// (which swaps the body/collider arenas, not this) leaves it valid.
    ///
    /// **Sorted by handle**: a body standing in two overlapping zones sums their
    /// impulses, and the order of a float sum is exactly the kind of detail that
    /// makes a cross-OS hash drift (HR-5).
    ///
    /// The third element is the zone's own **silhouette**, copied from the same
    /// `BodyDesc` (W-AreaFalloff): the falloff measures how far out a body is *as a
    /// fraction of this shape*, so the zone has to remember what it looks like. It is
    /// carried here rather than re-read from the collider because this table is already
    /// the zone's config record, and asking rapier's `Shape` trait instead would be a
    /// second vocabulary for a silhouette `ShapeDesc` already names exactly.
    effectors: Vec<(RigidBodyHandle, desc::AreaEffect, shape::ShapeDesc)>,
    /// The live pulleys (W-Pulley) — rope constraints rapier does not have, so
    /// they are imposed from outside by a per-substep impulse pass, exactly like
    /// the drag and the force zones above it.
    ///
    /// CONFIG, not solver state: the bridge re-stamps the whole table every
    /// dispatch from the authored components, which is why it is absent from the
    /// checkpoint ring for the same reason `effectors` is — a restore keeps the
    /// table the bridge just installed rather than resurrecting one from a run
    /// that ended.
    pulleys: Vec<pulley::PulleyDesc>,
    /// As roldanas que as faixas de `pulleys` indexam — uma arena só para todas
    /// as cordas, pelo motivo escrito no [`pulley::PulleyDesc`]: um `Vec` por
    /// corda alocaria por frame e tiraria o `Copy` que deixa a tabela ser
    /// trocada com um `swap`.
    pulley_wheels: Vec<rope_route::RopeWheel>,
    /// Os trechos da rota da corda que o passe está resolvendo. Buffer PRÓPRIO
    /// e persistente porque a rota escreve `N+1` trechos por corda por sub-passo
    /// — alocá-los por chamada apareceria no gate de zero-alloc do caminho
    /// quente, do mesmo jeito que os scratches de contato apareceriam.
    route_scratch: Vec<rope_route::Tangent>,
    /// The Baumgarte fraction the pulley pass corrects per sub-step. A field
    /// rather than the constant read inline **so the measured table on
    /// [`pulley::PULLEY_BIAS`] is reproducible against the PRODUCT path** and not
    /// against a second copy of the pass living in a test file — the same reason
    /// (and the same shape) as `grab_body_with` and `spawn_joint_tuned`.
    pulley_bias: f32,
    /// The **impact peak** of each touching pair over the current tick's
    /// sub-steps (W-ImpactForce): body pair (lower handle first) → the hardest
    /// summed normal impulse it reached at any single sub-step.
    ///
    /// It is a READOUT, not simulation state — nothing in the solver reads it, so
    /// it is invisible to the C9 determinism hash (which is body poses), exactly
    /// like `contact_reports` is. It exists because the peak lives *between* the
    /// sub-steps and is gone by the time `step` returns: cleared at the start of
    /// each `step` and folded by `max` after each sub-step, so the readback finds
    /// the peak of the last tick. `BTreeMap` (not `HashMap`) so the readout is
    /// reproducible cross-OS, the same rule the whole module follows.
    ///
    /// A field and not a local for the same reason `kinematic_targets` is: `step`
    /// is the hot path with a zero-alloc gate, so the capacity is reached once and
    /// reused (`clear` keeps it). Empty for a scene where nothing touches.
    contact_peaks: std::collections::BTreeMap<contacts::PeakKey, contacts::PeakSample>,
    /// The **peak reaction** each joint carried over the current tick's sub-steps
    /// (W-J7): joint handle → the largest force and torque it was resisting at any
    /// single sub-step. Cleared and refilled exactly like [`Self::contact_peaks`],
    /// and for the same reason — the peak lives between the sub-steps.
    ///
    /// Unlike the contact ledger this one is *read by the simulation*: it is where
    /// a break is decided. A world with no finite threshold never compares
    /// anything, so it stays byte-identical.
    joint_peaks: std::collections::BTreeMap<joint_break::JointKey, joint_break::JointLoad>,
    /// The joints that gave way during the current tick — an EVENT channel, drained
    /// by the bridge (W-TickContacts' lesson: a transition is not derivable from the
    /// state that follows it).
    joint_breaks: Vec<joint_break::JointBreak>,
    /// **A mão** (W-Grab): o corpo que o artista está segurando enquanto a sim
    /// corre, e a tralha que o realiza. `None` no caso comum.
    ///
    /// ⚠️ Deliberadamente **fora** do [`checkpoint`](checkpoint) — e não por
    /// esquecimento: é a única entrada deste mundo que não vem do documento, então
    /// um checkpoint que a contivesse tornaria a resposta de um scrub dependente do
    /// cache. Quem garante que isso nunca acontece é a ponte (`bridge::grab`), com
    /// gate; ver [`grab`].
    grab: Option<grab::Grab>,
    /// **O campo de atração em voo** (W-Hand) — irmão exato do `grab` acima, e
    /// FORA do checkpoint pela mesma razão: um cutucão não está no documento,
    /// então nenhum replay o reproduz e um `restore` que o trouxesse de volta
    /// descreveria uma corrida que já acabou.
    attract: Option<blast::Attract>,
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
            effectors: Vec::new(),
            pulleys: Vec::new(),
            pulley_wheels: Vec::new(),
            route_scratch: Vec::new(),
            pulley_bias: pulley::PULLEY_BIAS,
            contact_peaks: std::collections::BTreeMap::new(),
            joint_peaks: std::collections::BTreeMap::new(),
            joint_breaks: Vec::new(),
            grab: None,
            attract: None,
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
            // Dominance group (collision priority). `0` is rapier's own default, so a
            // body authored before this is byte-identical; a higher value makes this
            // body bulldoze lower-dominance ones (infinite relative mass to them). It
            // rides the `BodyDesc`, so a rewind re-arms it.
            .dominance_group(desc.dominance)
            // Initial velocity (W9), applied at build. `[0,0]`/`0` is rapier's
            // own default, so a body authored before this is byte-identical; and
            // because it rides the `BodyDesc`, a rewind to t=0 re-arms the launch.
            //
            // ⚠️ A LOCKED translation axis drops its velocity component. rapier's
            // `LockedAxes` zeroes the axis's inverse mass, so no FORCE can move it
            // (gravity on a Y-locked body does nothing) — but `RigidBodyVelocity::
            // integrate` advances the body by its raw `linvel` WITHOUT projecting
            // out the locked axes, so an explicitly-set initial velocity would drift
            // a "frozen" body forever (measured: an X-locked body launched at 3 m/s
            // slid the full 1.5 m in 0.5 s). rapier special-cases only rotation. So
            // a frozen axis carries no velocity — Unity/Godot's Freeze Position
            // fully pins the axis, and this makes the lock authoritative.
            .linvel(Vector2::new(
                if desc.lock_x { 0.0 } else { desc.linvel[0] },
                if desc.lock_y { 0.0 } else { desc.linvel[1] },
            ))
            .angvel(desc.angvel)
            // Continuous collision detection. `false` is rapier's own default, so
            // a body authored before this is byte-identical; enabling it makes the
            // pipeline sweep this body's motion so a fast one does not tunnel
            // through thin geometry. It rides the `BodyDesc`, so a rewind re-arms it.
            .ccd_enabled(desc.ccd)
            // Constraints (Freeze Rotation / Position X / Position Y). Each flag ORs
            // in its own axis of the SAME `LockedAxes` bitmask; `empty()` (no flag
            // set) is rapier's default, so an unconstrained body is byte-identical.
            // `ROTATION_LOCKED` pins the angular DOF, `TRANSLATION_LOCKED_X/_Y` pin a
            // translation DOF — a body can freeze any combination. Rides the
            // `BodyDesc`, so a rewind re-arms every locked axis.
            .locked_axes({
                let mut axes = LockedAxes::empty();
                if desc.lock_rotation {
                    axes |= LockedAxes::ROTATION_LOCKED;
                }
                if desc.lock_x {
                    axes |= LockedAxes::TRANSLATION_LOCKED_X;
                }
                if desc.lock_y {
                    axes |= LockedAxes::TRANSLATION_LOCKED_Y;
                }
                axes
            })
            .build();
        let handle = self.bodies.insert(body);
        self.stamp_defaults(handle);
        // Per-body damping override (if any), stamped AFTER the global defaults so it
        // wins. `None` (the common case) leaves the body on the global drag, so an
        // un-overridden body is byte-identical to before this existed. Rides the
        // `BodyDesc`, so a rewind re-arms it.
        if let Some(d) = desc.damping {
            self.apply_damping_override(handle, d);
        }
        let collider = collider_build::build_collider(&desc);
        let collider_handle = self
            .colliders
            .insert_with_parent(collider, handle, &mut self.bodies);
        self.stamp_layer(collider_handle, desc.layer as usize);
        // Force zone (W-Area / W-AreaDrag). `zone_effect` is the single door — it
        // refuses a solid collider (an area you cannot enter is not an area, and the
        // narrow phase reports no overlap for it) and an INERT one (no force and no
        // drag: it would touch nothing and only WAKE bodies, so registering it would
        // not be byte-neutral). Kept sorted by handle so two overlapping zones apply
        // in a fixed order.
        if let Some(effect) = effector::zone_effect(&desc) {
            self.effectors.push((handle, effect, desc.shape));
            self.effectors
                .sort_unstable_by_key(|(h, _, _)| h.into_raw_parts());
        }
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
        // A removed body is no longer a zone. Left behind, the entry would keep
        // querying a dead collider handle every substep — harmless today (the
        // lookup fails), and exactly the kind of stale table that stops being
        // harmless the moment the arena reuses the index.
        self.effectors.retain(|(h, _, _)| *h != handle);
        // A pulley whose body is gone has no branch to pull on; `branch` would
        // already answer `None`, but a table that keeps naming dead handles is
        // the stale table the comment above is about.
        self.pulleys
            .retain(|p| p.body_a != handle && p.body_b != handle);
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
        // W-ImpactForce: the impact peak is the hardest a pair pushes at any
        // single sub-step, and it is gone by the time this returns. Start the tick
        // with an empty ledger and `max` into it after each sub-step below. Cleared
        // (capacity kept) rather than reallocated — the hot-path zero-alloc gate.
        self.contact_peaks.clear();
        // W-J7: the same shape, one field over — a joint's reaction also lives
        // between the sub-steps, and a break decided a frame late is a different
        // simulation. Capacity kept for the same zero-alloc reason.
        self.joint_peaks.clear();
        self.joint_breaks.clear();
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
            // Force zones (W-Area): a sensor's force, applied to whatever is inside
            // it. Per SUBSTEP for the same reason as the drag above. Empty in every
            // scene without a zone, which is what keeps this byte-identical.
            effector::apply(
                &mut self.bodies,
                &self.colliders,
                &self.narrow_phase,
                &self.effectors,
                self.gravity,
                self.integration_parameters.dt,
            );
            // The attract field (W-Hand): the artist holding the pointer down with
            // the Attract tool. Per SUBSTEP for the same reason as the two above,
            // and a no-op for every world nobody is poking.
            blast::apply_attract(
                &mut self.bodies,
                &self.attract,
                self.integration_parameters.dt,
            );
            // The pulleys (W-Pulley): a rope through two wheels, imposed as a
            // velocity projection. Per SUBSTEP for the same reason as the three
            // above, and a no-op for every world without one.
            pulley::apply(
                &mut self.bodies,
                &self.pulleys,
                &self.pulley_wheels,
                &mut self.route_scratch,
                self.integration_parameters.dt,
                self.pulley_bias,
            );
            // The one-way platform hook. Installing it is byte-neutral for every scene
            // without a one-way collider: rapier only calls `modify_solver_contacts`
            // for pairs where a collider carries `MODIFY_SOLVER_CONTACTS`, which only
            // a one-way collider sets.
            let physics_hooks = oneway::OneWayHooks;
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
            // W-ImpactForce: fold this sub-step's contact loads into the peak
            // ledger. Measured ≤ 1.93% of the HR-4 budget at 500 contact pairs
            // (`tests/measure_impact.rs`), so it is unconditional — a gating flag
            // that never toggled would be a dead flag. Empty for a scene where
            // nothing touches, which keeps it free there.
            contacts::accumulate_peaks(
                &self.narrow_phase,
                &self.colliders,
                &mut self.contact_peaks,
            );
            // W-J7: fold this sub-step's joint reactions into the peak ledger and
            // sever whatever just exceeded what it was told it could take. Inside
            // the loop, so the remaining sub-steps already run without the broken
            // joint. Empty for a world with no joints.
            let inv_dt = self.joint_impulse_to_force();
            joint_break::accumulate_and_break(
                &mut self.impulse_joints,
                &self.bodies,
                inv_dt,
                &mut self.joint_peaks,
                &mut self.joint_breaks,
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
pub(super) fn groups_for(layer: usize, matrix: LayerMatrix) -> InteractionGroups {
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
mod tests;
