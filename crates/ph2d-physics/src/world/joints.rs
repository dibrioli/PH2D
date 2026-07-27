//! Joints — the articulations. *Two bodies, one constraint.*
//!
//! ## The anchor is authored in WORLD, stored in LOCAL, and that conversion
//! happens exactly once
//!
//! rapier wants the anchor twice, once in each body's local frame
//! (`local_anchor1`/`local_anchor2`) — because that is what stays meaningful
//! while the bodies move. The artist, on the other hand, points at *a place on
//! the screen*: the pin goes **there**, at the top of the plank.
//!
//! So [`JointDesc`] carries the world point and this module converts, using the
//! bodies' isometries as they stand at the moment of the spawn. Two consequences
//! worth stating out loud:
//!
//! * The conversion happens **exactly once, at authoring time**, and the LOCAL
//!   pair is what is stored — see [`JointDesc::anchor_a`]. An earlier version
//!   stored the world pair and re-converted on every spawn, which made a
//!   rebuild-from-rest produce a *different* constraint than the live spawn
//!   had; the module doc claimed joints were only ever spawned at rest, and
//!   that claim was simply false (nothing gated the first spawn).
//! * The inverse transform is rapier's own (`Isometry2::inverse_transform_point`),
//!   not trigonometry written here. The solver and the authoring path then agree
//!   about what "this point, in that body's frame" means by construction, and
//!   there is no second `sin`/`cos` convention to drift (HR-5).
//!
//! ## Why `ImpulseJointSet` and not `MultibodyJointSet`
//!
//! rapier ships both: impulse joints are solved with the contacts, multibody
//! joints are a reduced-coordinate formulation that cannot drift apart at all.
//! The reduced form sounds strictly better and is not: it models a **tree**, so
//! it cannot express a closed loop (a four-bar linkage, a chain whose end is
//! pinned back), and it has no spring. Impulse joints cover everything the
//! editor offers, and — the deciding fact — the checkpoint ring already clones
//! `ImpulseJointSet`, so a scrub backwards carries joints with no work at all.

use rapier2d::dynamics::{
    FixedJointBuilder, ImpulseJointHandle, JointAxis, PrismaticJointBuilder, RevoluteJointBuilder,
    RigidBodyHandle, RopeJointBuilder, SpringJointBuilder,
};
use rapier2d::na::{Isometry2, Point2, UnitVector2, Vector2};

// Re-exportado por conveniência dos irmãos que sempre importaram o vocabulário
// por aqui — a casa dele agora é [`super::joint_desc`], e o `pub use` existe
// para o corte não obrigar sete arquivos a mudar de `use` por uma mudança que
// não é deles.
pub use super::joint_desc::{JointDesc, JointKind, MotorDesc, MotorMode};
use super::joint_gains::{MOTOR_TRACKING, SERVO_DAMPING, SERVO_STIFFNESS};
use super::{PhysicsWorld, joint_break};

/// A direction as a unit vector, falling back to `+X` when it cannot be one.
///
/// ⚠️ **`UnitVector2::new_normalize` of a zero vector yields `NaN`**, and a `NaN`
/// axis does not fail loudly — it poisons the solver, and from there the poses,
/// the readback, and the determinism hash. A joint whose axis was never authored
/// (or arrived non-finite from a project file) gets the horizontal rail an
/// unrotated joint means, which is the same value `JointDesc::default` carries.
pub(super) fn unit_or_x(v: [f32; 2]) -> UnitVector2<f32> {
    let len = v[0].hypot(v[1]);
    if len.is_finite() && len > 1e-6 {
        UnitVector2::new_normalize(Vector2::new(v[0], v[1]))
    } else {
        UnitVector2::new_normalize(Vector2::new(1.0, 0.0))
    }
}

/// A rod's length, made safe to hand the solver.
///
/// Sibling of [`unit_or_x`], and for the same reason it exists: a `NaN` or a
/// negative does not fail loudly, it poisons the solver and from there the
/// poses, the readback and the determinism hash — and here it would land in a
/// motor's *target*, which the solver chases every sub-step. A rod of length
/// zero is a Pin written the hard way, so the floor is a millimetre rather than
/// zero, which keeps the constraint a *distance* instead of a degenerate point.
pub(super) fn rod_length(len: f32) -> f32 {
    if len.is_finite() && len > MIN_ROD_LENGTH {
        len
    } else {
        MIN_ROD_LENGTH
    }
}

/// The shortest rod the solver is asked to hold, metres. A millimetre: below it
/// the two anchors are the same point to every consumer that draws or measures
/// them, and *that* joint is a Pin.
const MIN_ROD_LENGTH: f32 = 0.001;

/// **Which degree of freedom a motor drives, per kind** — and `None` for the
/// kinds that have none.
///
/// In 2D rapier locks a revolute joint's two linear axes and leaves `AngX`, and
/// locks a prismatic joint's `LinY`+`AngX` and leaves `LinX`; a rope couples the
/// linear axes and drives the *distance* through that same `LinX`. So the whole
/// kind-dependence of a motor is this one axis, which is why the motor is
/// applied **once, after the builder**, instead of once per arm: three arms each
/// spelling out a motor is three places for a mode to be forgotten.
///
/// ⚠️ **A Spring gets `None` and that is not an omission.** rapier models a
/// spring *as* a motor on the coupled linear axis (`SpringJointBuilder` sets
/// one), so writing a second motor there would silently overwrite the spring the
/// artist authored — the stiffness and damping would vanish and the joint would
/// become a rate-driven rod. A Weld has no free axis at all.
#[must_use]
pub fn motor_axis(kind: JointKind) -> Option<JointAxis> {
    match kind {
        JointKind::Pin => Some(JointAxis::AngX),
        JointKind::Slider | JointKind::Rope => Some(JointAxis::LinX),
        // ⚠️ **A Rod gets `None` for the SAME mechanical reason as the Spring,
        // and it is not an oversight.** rapier models a rod *as* a motor on the
        // coupled linear axis (see [`JointKind::Rod`]), so writing a second
        // motor here would silently overwrite the constraint that makes it a rod
        // and leave a rate-driven rope behind. A driven length is a different
        // animal and already has two homes: a Slider with a motor (the ram) and
        // a Rope with one (the winch).
        JointKind::Spring | JointKind::Weld | JointKind::Rod => None,
    }
}

impl PhysicsWorld {
    /// Attach `a` to `b`. Returns the handle, or `None` when the joint cannot
    /// exist: a missing body, or a body joined **to itself**.
    ///
    /// Self-jointing is refused rather than clamped because there is no sensible
    /// reading of it — the constraint would be between a body and itself, which
    /// the solver satisfies trivially and forever. Refusing keeps a nonsense
    /// authoring gesture from becoming an invisible no-op joint the artist then
    /// tries to tune.
    pub fn spawn_joint(
        &mut self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        desc: JointDesc,
    ) -> Option<ImpulseJointHandle> {
        self.spawn_joint_with_gains(a, b, desc, SERVO_STIFFNESS, SERVO_DAMPING)
    }

    /// [`Self::spawn_joint`] with the servo gains handed in rather than taken
    /// from the measured constants.
    ///
    /// **It exists so the tables on [`SERVO_STIFFNESS`] and [`SERVO_DAMPING`]
    /// are reproducible against the PRODUCT path**, not against a second copy of
    /// it written in a test file. A sweep that builds its own rapier joint would
    /// measure a joint nobody ships; the numbers those tables carry are the
    /// reason two constants have the values they do, so they have to come from
    /// here. (`world::tests` runs the sweep.)
    pub(super) fn spawn_joint_with_gains(
        &mut self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        desc: JointDesc,
        servo_stiffness: f32,
        servo_damping: f32,
    ) -> Option<ImpulseJointHandle> {
        self.spawn_joint_tuned(a, b, desc, servo_stiffness, servo_damping, MOTOR_TRACKING)
    }

    /// The innermost door: every measured motor gain handed in. Its only
    /// non-test caller is [`Self::spawn_joint_with_gains`]; it exists so the
    /// TRACKING table can be swept on the product path too.
    pub(super) fn spawn_joint_tuned(
        &mut self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        desc: JointDesc,
        servo_stiffness: f32,
        servo_damping: f32,
        tracking: f32,
    ) -> Option<ImpulseJointHandle> {
        if a == b {
            return None;
        }
        // The anchors arrive ALREADY local (see [`JointDesc::anchor_a`]), so
        // this function has no opinion about where the bodies are — which is
        // exactly what makes a rebuild reproduce the same constraint.
        self.bodies.get(a)?;
        self.bodies.get(b)?;
        let anchor_a = Point2::new(desc.anchor_a[0], desc.anchor_a[1]);
        let anchor_b = Point2::new(desc.anchor_b[0], desc.anchor_b[1]);

        let mut joint: rapier2d::dynamics::GenericJoint = match desc.kind {
            JointKind::Pin => {
                let mut builder = RevoluteJointBuilder::new()
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b);
                if let Some([min, max]) = desc.limits {
                    builder = builder.limits([min, max]);
                }
                builder.into()
            }
            JointKind::Spring => {
                SpringJointBuilder::new(desc.rest_length, desc.stiffness, desc.damping)
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b)
                    .into()
            }
            JointKind::Rope => RopeJointBuilder::new(desc.max_length)
                .local_anchor1(anchor_a)
                .local_anchor2(anchor_b)
                .into(),
            // **The Rod is a Spring told to be rigid**, for the reason the enum
            // spells out: rapier's coupled linear *limit* is unilateral, so the
            // only two-sided distance constraint it offers is a motor. Same
            // builder as `Spring`, different numbers — and the numbers are the
            // whole difference, so they are named constants with the table that
            // chose them.
            JointKind::Rod => SpringJointBuilder::new(
                rod_length(desc.max_length),
                JointDesc::ROD_STIFFNESS,
                JointDesc::ROD_DAMPING,
            )
            .local_anchor1(anchor_a)
            .local_anchor2(anchor_b)
            .into(),
            // A rigid lock: the anchors coincide (shared-point policy) and no
            // relative rotation is allowed. No tunable parameters.
            JointKind::Weld => FixedJointBuilder::new()
                .local_anchor1(anchor_a)
                .local_anchor2(anchor_b)
                .into(),
            // The mirror of the Pin: one translational degree of freedom, no
            // rotation. The two local axes are the SAME world direction seen from
            // each body (see `JointDesc::axis_a`), so they are set separately —
            // `PrismaticJointBuilder::new` would put one vector in both frames,
            // which is only right when the bodies were authored at the same
            // rotation.
            JointKind::Slider => {
                let mut builder = PrismaticJointBuilder::new(unit_or_x(desc.axis_a))
                    .local_axis1(unit_or_x(desc.axis_a))
                    .local_axis2(unit_or_x(desc.axis_b))
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b);
                // Stroke limits, in METRES — the same `limits` field a Pin uses
                // for radians, because the limit belongs to whichever degree of
                // freedom the joint left free (rapier models it exactly so).
                if let Some([min, max]) = desc.limits {
                    builder = builder.limits([min, max]);
                }
                builder.into()
            }
        };
        // **The motor, applied ONCE for every kind that has one.** The builders
        // are three different types with three identical `motor_*` families, so
        // spelling the motor out per arm would be three chances to forget a
        // mode; the only kind-dependent part is the axis, and that is a
        // function (`motor_axis`).
        //
        // ⚠️ Byte-identical to the per-arm version it replaced, for the case
        // that existed before this: `RevoluteJointBuilder::motor_velocity(v, f)`
        // is `set_motor(AngX, target_pos, v, 0.0, f)` with the fresh builder's
        // `target_pos` still `0.0` — which is exactly the Velocity arm below.
        // (Pinned by the `physics_ecs_c9` hash and by a fingerprint gate.)
        if let (Some(axis), Some(m)) = (motor_axis(desc.kind), desc.motor) {
            match m.mode {
                // `damping` here is the measured tracking constant, NOT the
                // artist's `max_force`. The two stay separate on purpose: speed
                // is what the motor wants, max_force is what it may spend.
                MotorMode::Velocity => joint.set_motor(axis, 0.0, m.speed, 0.0, tracking),
                // A servo pulls towards a place, so it needs a stiffness; the
                // target velocity is zero because *arriving* is the instruction.
                MotorMode::Position => {
                    joint.set_motor(axis, m.target, 0.0, servo_stiffness, servo_damping)
                }
            };
            joint.set_motor_max_force(axis, m.max_force);
        }
        // ⚠️ **Jointed bodies do not collide with each other by DEFAULT**, and
        // rapier's default is the opposite (`contacts_enabled: true`). Box2D
        // (`collideConnected`) and Unity (`enableCollision`) both default to
        // false, and the reason is the canonical case: the links of a chain
        // OVERLAP at their pins by construction. Left enabled, every joint hands
        // the contact solver a permanent interpenetration to fight, and the
        // measurement that found this had a motor told to spin at 4 rad/s
        // reading -80 while the hub ball thrashed inside the plank it was
        // pinned to. It is a knob from W-J8 because the other case is real too
        // (a door and its frame); the DEFAULT is what the measurement bought.
        joint.contacts_enabled = desc.contacts_enabled;
        // W-J8: an inactive joint is BUILT and disabled, never skipped — see
        // `JointDesc::enabled`. `set_enabled` is rapier's own door, the same one
        // a break goes through.
        joint.set_enabled(desc.enabled);
        // W-J7: the break thresholds ride in the joint's own `user_data`, so a
        // checkpoint carries them and no side table can fall out of step. `(∞, ∞)`
        // packs to zero, which is what `GenericJoint::default` already holds.
        joint.user_data = joint_break::pack_thresholds(desc.break_force, desc.break_torque);
        Some(self.impulse_joints.insert(a, b, joint, true))
    }

    /// A world point → the same point in the frame of a body **at the pose you
    /// name**, which is not necessarily the pose it is in.
    ///
    /// That distinction is the whole reason this takes a pose instead of a
    /// handle: a joint's anchor has to be a function of the **authored** scene,
    /// so it is converted against the bodies' REST poses even when the artist
    /// creates it mid-swing. Converting against the live pose instead makes the
    /// same authored point land somewhere else on the body depending on when
    /// the gesture happened — measured at 1.771 m of walk across a Reset.
    ///
    /// `pose` is `[x, y, rotation]`, the same triple `BodyDesc` carries.
    pub fn local_anchor_at_pose(pose: [f32; 3], world: [f32; 2]) -> [f32; 2] {
        let iso = Isometry2::new(Vector2::new(pose[0], pose[1]), pose[2]);
        let p = iso.inverse_transform_point(&Point2::new(world[0], world[1]));
        [p.x, p.y]
    }

    /// **A Slider's axis in each body's local frame**, from the authored angles.
    ///
    /// The authored quantity is the joint entity's own rotation — a WORLD angle,
    /// exactly as its translation is a world point — so it is converted once,
    /// against the bodies' **REST** rotations. Same law as
    /// [`Self::local_anchor_at_pose`], and for the same measured reason:
    /// converting against a live pose makes one authored direction mean
    /// different things depending on when the conversion happened.
    ///
    /// ⚠️ **`libm::sincosf`, never `f32::sin_cos`** — this number reaches the
    /// solver and therefore the `physics_ecs_c9` cross-OS hash, and std trig is
    /// not pinned across platforms. It is the same rule (and the same crate) the
    /// ellipse collider tessellation follows.
    ///
    /// ⚠️ **Deliberate consequence, gated on the ECS side:** because the axis is
    /// world-authored, rotating body A does not re-aim the rail — the local axis
    /// changes to keep the world direction. That is the Godot/Unreal model: a
    /// prismatic axis is a direction in the scene (an elevator shaft), not a
    /// feature of the carriage. It is why the axis is DERIVED per reconcile
    /// instead of stored like the anchor, which *is* a feature of the body.
    #[must_use]
    pub fn axis_locals(joint_rot: f32, a_rot: f32, b_rot: f32) -> ([f32; 2], [f32; 2]) {
        fn dir(ang: f32) -> [f32; 2] {
            let (s, c) = libm::sincosf(ang);
            [c, s]
        }
        (dir(joint_rot - a_rot), dir(joint_rot - b_rot))
    }

    /// The inverse of [`Self::local_anchor_at_pose`]: a body-local anchor → the
    /// WORLD point it sits at, given the body's pose.
    ///
    /// This is how the ECS side derives the joint's DISPLAY pivot from the
    /// authored local anchor (`bridge::joints` syncs it into the joint's
    /// `Transform` so the anchor dot follows the body). Uses rapier's own
    /// `transform_point`, so it and `local_anchor_at_pose` round-trip exactly
    /// (`world_from_local(pose, local_anchor_at_pose(pose, w)) == w`).
    pub fn world_from_local_at_pose(pose: [f32; 3], local: [f32; 2]) -> [f32; 2] {
        let iso = Isometry2::new(Vector2::new(pose[0], pose[1]), pose[2]);
        let p = iso.transform_point(&Point2::new(local[0], local[1]));
        [p.x, p.y]
    }

    /// Two world points → the same two points in the bodies' own frames, using
    /// the poses the bodies are in **right now**.
    ///
    /// **The single door for that conversion**, and it is called exactly once
    /// per joint — at authoring time — after which the local pair is what is
    /// stored and replayed. Uses rapier's own inverse transform, so the solver
    /// and the authoring path cannot drift over what "this point, in that
    /// body's frame" means (HR-5: no second `sin`/`cos` convention).
    pub fn world_to_local_anchors(
        &self,
        a: RigidBodyHandle,
        b: RigidBodyHandle,
        world_a: [f32; 2],
        world_b: [f32; 2],
    ) -> Option<([f32; 2], [f32; 2])> {
        let pa = self
            .bodies
            .get(a)?
            .position()
            .inverse_transform_point(&Point2::new(world_a[0], world_a[1]));
        let pb = self
            .bodies
            .get(b)?
            .position()
            .inverse_transform_point(&Point2::new(world_b[0], world_b[1]));
        Some(([pa.x, pa.y], [pb.x, pb.y]))
    }

    /// Remove a joint. No-op if the handle is already gone.
    pub fn remove_joint(&mut self, handle: ImpulseJointHandle) {
        self.impulse_joints.remove(handle, true);
    }

    /// How many joints the world holds (tests, diagnostics).
    pub fn joint_count(&self) -> usize {
        self.impulse_joints.len()
    }

    /// Both anchors of a joint, in **world** meters — what the overlay draws.
    ///
    /// Two points and not one: they coincide for a pin at rest and separate the
    /// moment the constraint is strained, which is exactly the thing worth
    /// seeing. For a spring or a rope they are the two ends and are *supposed*
    /// to be apart.
    pub fn joint_anchors(&self, handle: ImpulseJointHandle) -> Option<([f32; 2], [f32; 2])> {
        let j = self.impulse_joints.get(handle)?;
        // ⚠️ `transform_point`, never `isometry * vector`. Multiplying an
        // isometry by a *vector* applies the rotation and drops the
        // translation — that is what a vector IS — so the first version of
        // this returned local coordinates wearing world coordinates' name.
        // The overlay would have drawn every joint near the origin.
        let pa = self
            .bodies
            .get(j.body1)?
            .position()
            .transform_point(&j.data.local_anchor1());
        let pb = self
            .bodies
            .get(j.body2)?
            .position()
            .transform_point(&j.data.local_anchor2());
        Some(([pa.x, pa.y], [pb.x, pb.y]))
    }
}
