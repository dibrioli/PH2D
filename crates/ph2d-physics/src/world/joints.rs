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
    ImpulseJointHandle, RevoluteJointBuilder, RigidBodyHandle, RopeJointBuilder, SpringJointBuilder,
};
use rapier2d::na::{Isometry2, Point2, Vector2};

use super::PhysicsWorld;

/// How hard a motor corrects its velocity error — rapier's motor `damping`,
/// with `stiffness` at zero, which is what makes it a *velocity* motor.
///
/// **Not a knob, and MEASURED.** It is not exposed because it is not a
/// question the artist has: they say how fast and how strong, and this is the
/// number that makes those two mean what they say. Too low and the motor
/// cannot hold its speed under load — it is a suggestion, not a motor.
///
/// A 0.2 kg arm hanging straight down, told to turn at 4 rad/s for 5 s. A
/// motor that keeps its word travels 20 rad; one that cannot lift the arm past
/// horizontal stalls near zero:
///
/// | tracking | `max_force` 0.1 | 1.0 | 100 |
/// |---|---|---|---|
/// | 10  | 0.49 | 1.39 | 1.39 |  ← cannot lift its own arm
/// | 50  | 0.49 | 18.57 | 18.90 |
/// | **100** | **0.49** | **19.21** | **19.62** |  ← the knee
/// | 300 | 0.49 | 19.37 | 19.90 |
/// | 1000 | 0.49 | 19.39 | 19.97 |
///
/// The first column is the point: at every tracking value a motor capped at
/// 0.1 N·m still stalls. Raising this does not make motors unstoppable — it
/// makes a motor that *is* strong enough actually reach its speed.
const MOTOR_TRACKING: f32 = 100.0;

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
}

/// A motor driving a [`JointKind::Pin`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MotorDesc {
    /// Target angular velocity, radians/s. Sign picks the direction.
    pub speed: f32,
    /// Ceiling on the force the motor may use to reach that speed. This is what
    /// makes a motor *stoppable*: a weak motor stalls against a heavy load
    /// instead of teleporting it.
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
    /// [`JointKind::Pin`]: the motor, or `None` for a passive hinge.
    pub motor: Option<MotorDesc>,
    /// [`JointKind::Spring`]: the length the spring pulls towards, meters.
    pub rest_length: f32,
    /// [`JointKind::Spring`]: spring constant.
    pub stiffness: f32,
    /// [`JointKind::Spring`]: damping constant. Zero oscillates forever.
    pub damping: f32,
    /// [`JointKind::Rope`]: the distance the anchors may not exceed, meters.
    pub max_length: f32,
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
                if let Some(m) = desc.motor {
                    // `factor` is the motor's damping — how hard it corrects
                    // the velocity error — and it is the measured constant
                    // `MOTOR_TRACKING`, NOT the artist's `max_force`. (An
                    // earlier comment here claimed the two were tied; they
                    // never were, and the constant's own docs said so three
                    // screens up. One of the two had to go.) The two knobs stay
                    // separate on purpose: speed is what the motor wants,
                    // max_force is what it is allowed to spend getting there.
                    builder = builder
                        .motor_velocity(m.speed, MOTOR_TRACKING)
                        .motor_max_force(m.max_force);
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
        };
        // ⚠️ **Jointed bodies do not collide with each other**, and rapier's
        // default is the opposite (`contacts_enabled: true`). Box2D
        // (`collideConnected`) and Unity (`enableCollision`) both default to
        // false, and the reason is the canonical case: the links of a chain
        // OVERLAP at their pins by construction. Left enabled, every joint hands
        // the contact solver a permanent interpenetration to fight, and the
        // measurement that found this had a motor told to spin at 4 rad/s
        // reading -80 while the hub ball thrashed inside the plank it was
        // pinned to.
        joint.contacts_enabled = false;
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
