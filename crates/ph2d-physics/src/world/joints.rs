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

use super::joint_gains::{MOTOR_TRACKING, SERVO_DAMPING, SERVO_STIFFNESS};
use super::{PhysicsWorld, joint_break};

/// Which constraint. **Fieldless on purpose** — the parameters live beside it in
/// [`JointDesc`], flat, so the ECS component that mirrors this is flat too and
/// its postcard layout can grow by appending (the same rule `Collider` follows).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JointKind {
    /// **Pin** — the two bodies share a point and are free to rotate about it.
    /// The hinge, the pendulum's pivot, the ragdoll's elbow. Optionally limited
    /// to an angular range, and optionally driven by a motor.
    Pin,
    /// **Spring** — a damped spring between the anchors. The only joint here
    /// that is *soft*: the distance is a target, not a law.
    Spring,
    /// **Rope** — the anchors may come as close as they like but never further
    /// apart than `max_length`. Slack below it, rigid at it.
    Rope,
    /// **Weld** — the two bodies are locked rigidly at the anchor: no relative
    /// translation OR rotation. rapier's `FixedJoint`.
    Weld,
    /// **Slider** — the bodies may only slide along one AXIS, and never rotate
    /// relative to each other. The elevator shaft, the sliding door, the piston.
    /// rapier's `PrismaticJoint`.
    ///
    /// It is the mirror image of the Pin: a Pin allows rotation and forbids
    /// translation, a Slider allows translation along one direction and forbids
    /// everything else. That is why its `limits` are a range in **metres** —
    /// the stroke — where a Pin's are radians. rapier expresses both through the
    /// same `limits` field for the same reason: the limit belongs to whichever
    /// degree of freedom the joint left free.
    Slider,
}

/// What a motor is *aiming at*. The two things a driven joint can be told, and
/// they are genuinely different instructions rather than two settings of one.
///
/// rapier expresses both through the same `set_motor(target_pos, target_vel,
/// stiffness, damping)`, and the mode is which pair carries the signal:
/// velocity leaves `stiffness` at zero (there is no place to pull towards),
/// position leaves `target_vel` at zero (the *place* is the instruction).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum MotorMode {
    /// **Keep turning / keep sliding at this rate.** A wheel, a conveyor, a
    /// winch paying out. Has no notion of "arrived" — it is a rate, forever.
    #[default]
    Velocity,
    /// **Go to this place and HOLD it.** The servo: an arm that stops at 45°
    /// and stays there under load, a lift that parks at a floor, a winch that
    /// reels to a length. This is the mode that needs a stiffness, because
    /// holding against gravity is a force proportional to how far off it is.
    Position,
}

/// A motor driving whichever degree of freedom the joint left free — the hinge
/// of a [`JointKind::Pin`], the rail of a [`JointKind::Slider`], the distance of
/// a [`JointKind::Rope`] (a winch).
///
/// ⚠️ **The unit follows the joint, not this struct.** `speed` and `target` are
/// radians and radians/s on a Pin, metres and metres/s on a Slider or a Rope —
/// exactly as `JointDesc::limits` is radians on one and metres on the other,
/// and for the same reason: the number belongs to the free degree of freedom.
/// The caller that authored it knows which; this one does not have to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MotorDesc {
    /// Which instruction this motor carries.
    pub mode: MotorMode,
    /// [`MotorMode::Velocity`]: the target rate. Sign picks the direction.
    pub speed: f32,
    /// [`MotorMode::Position`]: the place to hold. Measured along the free
    /// degree of freedom from the joint's own zero — the anchor for a rail, the
    /// authored angle for a hinge, the anchor distance for a winch.
    pub target: f32,
    /// Ceiling on the force the motor may use to get there. This is what makes
    /// a motor *stoppable*: a weak motor stalls against a heavy load instead of
    /// teleporting it — and it is what makes a servo *yield*, which is the
    /// difference between a held arm and a welded one.
    pub max_force: f32,
}

/// One joint, in plain data — no rapier types, like [`super::BodyDesc`], so the
/// ECS bridge can describe a joint without depending on rapier.
///
/// Fields that do not apply to the chosen [`JointKind`] are ignored, exactly as
/// `BodyDesc::density` is ignored for a static body.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointDesc {
    pub kind: JointKind,
    /// Where the joint attaches **on body A**, in that body's own LOCAL frame.
    ///
    /// ⚠️ **Local, not world, and that is the whole point.** The artist points
    /// at a place on screen, so the caller converts once — through
    /// [`PhysicsWorld::world_to_local_anchors`] — and then *keeps the local
    /// pair*. Storing the world point instead means the conversion is redone
    /// against whatever pose the bodies happen to have later, so the live
    /// spawn and a rebuild-from-rest answer *"where on the body is this
    /// pinned?"* differently: measured, a joint made mid-swing pinned at
    /// 1.611 m and replayed at 0.642 m after a Reset — the pin walked 0.969 m
    /// along the body with no user action.
    pub anchor_a: [f32; 2],
    /// Where it attaches **on body B**, likewise in B's local frame.
    ///
    /// Two points and not one, because a pin and a rope are different animals:
    /// a pin's two anchors are the *same place* (that is what a pin is), while
    /// a rope's are the two ends of the rope and start apart. Box2D and Unity
    /// both take the pair for exactly this reason. Collapsing them would make a
    /// 2 m rope hang its ball 2.5 m down whenever the authored point happened
    /// not to be the ball's centre — a number the artist typed, silently not
    /// meaning what it says.
    ///
    /// Which points these *are* is the caller's policy, not the engine's; the
    /// ECS bridge states it in one sentence.
    pub anchor_b: [f32; 2],
    /// [`JointKind::Pin`]: angular range `[min, max]` in radians, or `None` for
    /// a free hinge. `None` and a range covering a full turn are *not* the same
    /// thing to the solver, so the option is real state, not a sentinel.
    pub limits: Option<[f32; 2]>,
    /// The motor, or `None` for a passive joint. Applies to whichever kinds have
    /// a free degree of freedom to drive — see [`motor_axis`]: a Pin's hinge, a
    /// Slider's rail, a Rope's distance. Ignored by a Spring (which *is* a motor
    /// in rapier's model) and a Weld (which has no free axis).
    pub motor: Option<MotorDesc>,
    /// [`JointKind::Spring`]: the length the spring pulls towards, meters.
    pub rest_length: f32,
    /// [`JointKind::Spring`]: spring constant.
    pub stiffness: f32,
    /// [`JointKind::Spring`]: damping constant. Zero oscillates forever.
    pub damping: f32,
    /// [`JointKind::Rope`]: the distance the anchors may not exceed, meters.
    pub max_length: f32,
    /// [`JointKind::Slider`]: the sliding direction in **body A's local frame**.
    ///
    /// Local, and TWO of them, for exactly the reasons [`Self::anchor_a`] gives:
    /// the artist aims a direction in the world, the caller converts once
    /// against the bodies' REST poses, and the pair is what gets stored — so a
    /// rebuild reproduces the same constraint instead of re-deriving the axis
    /// against whatever pose the bodies drifted into.
    ///
    /// Need not be normalised; [`PhysicsWorld::spawn_joint`] normalises, and a
    /// degenerate (zero / non-finite) axis falls back to `+X` rather than
    /// handing rapier a `NaN` direction.
    pub axis_a: [f32; 2],
    /// The same direction in **body B's** local frame.
    ///
    /// Two fields and not one because the bodies can be authored at different
    /// rotations: one vector cannot be the same direction in both frames unless
    /// they happen to agree. (`anchor_a`/`anchor_b` are two fields for the same
    /// reason, and a Pin's *are* usually the same point.)
    pub axis_b: [f32; 2],
    /// The linear reaction, in **newtons**, above which this joint gives way —
    /// `f32::INFINITY` for a joint that never breaks, which is the default (P7).
    ///
    /// A force and not an impulse on purpose; [`super::joint_break`] gives the
    /// reason (an impulse threshold would change meaning with the sub-step count).
    /// Applies to every kind: what tears a rope apart also tears a pin out.
    pub break_force: f32,
    /// The angular reaction, in **newton-metres**, above which it gives way.
    ///
    /// Separate from [`Self::break_force`] because they are separate failures —
    /// a hinge can be twisted off without ever being pulled apart, and Unity ships
    /// the pair separate for exactly that reason.
    pub break_torque: f32,
    /// **Is this constraint in force at all?** `false` builds the joint and hands
    /// it to the solver `JointEnabled::Disabled` — present, parameterised, and
    /// imposing nothing.
    ///
    /// The joint is still *built* rather than skipped, and that is the whole
    /// point: skipping it would take it out of `joint_anchors`, `joint_load` and
    /// therefore off the canvas, so *disabled* would be indistinguishable from
    /// *deleted* to everything downstream — which is the one thing this switch
    /// exists not to be.
    ///
    /// ⚠️ **The same rapier flag a BREAK writes** ([`super::joint_break`]), and
    /// that is not a collision: one is authored (it rides in the descriptor, so a
    /// rebuild reproduces it) and the other is runtime (the solver sets it, and a
    /// rebuild clears it). A joint that is authored inactive comes back inactive
    /// from a Reset; a broken one comes back holding.
    pub enabled: bool,
    /// **Do the two jointed bodies collide with each other?**
    ///
    /// `false` — the default, and the right one — because the canonical case is a
    /// chain link, which OVERLAPS its neighbour at the pin by construction. rapier
    /// defaults the opposite way; Box2D (`collideConnected`) and Unity
    /// (`enableCollision`) default as we do. MEASURED on the rig that found it: a
    /// hub pinned inside a plank and told to spin at 4 rad/s reads **−80** with
    /// contacts on, while the ball thrashes inside the plank it is pinned to.
    ///
    /// Exposed as a knob rather than hardcoded because the *other* case is real
    /// too: two bodies pinned side by side that should still bump into each other
    /// (a door and its frame, a two-link limb that must not fold through itself).
    pub contacts_enabled: bool,
}

impl Default for JointDesc {
    /// A free pin at the origin. Every other field is the neutral value of the
    /// kind it belongs to, so `..Default::default()` in a fixture never smuggles
    /// in a spring that a Pin test did not ask for.
    fn default() -> Self {
        Self {
            kind: JointKind::Pin,
            anchor_a: [0.0, 0.0],
            anchor_b: [0.0, 0.0],
            limits: None,
            motor: None,
            rest_length: 1.0,
            stiffness: Self::DEFAULT_STIFFNESS,
            damping: Self::DEFAULT_DAMPING,
            max_length: 1.0,
            // `+X` — a horizontal rail, which is what an unrotated joint means.
            axis_a: [1.0, 0.0],
            axis_b: [1.0, 0.0],
            // ∞ = off. A joint holds no matter what until someone says otherwise,
            // and it is what every existing fixture inherits — which is what keeps
            // this wave byte-identical for every scene that predates it.
            break_force: f32::INFINITY,
            break_torque: f32::INFINITY,
            // A joint holds; that is what a joint is for.
            enabled: true,
            // Jointed bodies do not collide — see the field, where the number
            // that decided it lives.
            contacts_enabled: false,
        }
    }
}

impl JointDesc {
    /// Spring constant a new spring is born with.
    ///
    /// **MEASURED**, and the first guess (100) was refuted: a 0.2 kg body hung
    /// on it sagged 1.9 cm past a 1 m rest length — 2%, which reads as a rod,
    /// not a spring. A body on a new spring has to visibly hang.
    ///
    /// | stiffness | sag | rebound |
    /// |---|---|---|
    /// | 10 | 0.19 m | 0.17 m |  ← nearly doubles its length; floppy
    /// | **30** | **0.065 m** | **0.077 m** |
    /// | 100 | 0.019 m | 0.028 m |  ← reads as a rod
    ///
    /// Sag scales with the hanging mass, so this is the value that stays
    /// sensible as bodies get heavier: a 1 kg body on it sags ~0.3 m, where
    /// stiffness 10 would nearly double the spring's length.
    pub const DEFAULT_STIFFNESS: f32 = 30.0;
    /// Damping a new spring is born with. Under-damped on purpose: a spring
    /// that does not bounce is indistinguishable from a rod, and the artist
    /// reaches for a spring precisely to see it bounce.
    ///
    /// Also measured, and also refuted at the first guess (5.0), which left a
    /// rebound of 2 mm — the ball sank and stayed. At stiffness 30: damping
    /// 0.1 rebounds 0.113 m, **0.5 rebounds 0.077 m**, 1.0 rebounds 0.049 m.
    pub const DEFAULT_DAMPING: f32 = 0.5;
}

/// A direction as a unit vector, falling back to `+X` when it cannot be one.
///
/// ⚠️ **`UnitVector2::new_normalize` of a zero vector yields `NaN`**, and a `NaN`
/// axis does not fail loudly — it poisons the solver, and from there the poses,
/// the readback, and the determinism hash. A joint whose axis was never authored
/// (or arrived non-finite from a project file) gets the horizontal rail an
/// unrotated joint means, which is the same value `JointDesc::default` carries.
fn unit_or_x(v: [f32; 2]) -> UnitVector2<f32> {
    let len = v[0].hypot(v[1]);
    if len.is_finite() && len > 1e-6 {
        UnitVector2::new_normalize(Vector2::new(v[0], v[1]))
    } else {
        UnitVector2::new_normalize(Vector2::new(1.0, 0.0))
    }
}

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
        JointKind::Spring | JointKind::Weld => None,
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
