//! **O VOCABULÁRIO de um joint** — que espécies existem, o que cada uma carrega
//! e com que números elas nascem.
//!
//! Irmão de [`super::joints`], separado dele quando os dois juntos passaram do
//! cap de 700 LOC, e o corte é o mesmo que `world/desc.rs` já fez para os
//! corpos: aqui *o que um joint É*, lá *como um é CONSTRUÍDO no rapier*. É por
//! isso que nenhum tipo do rapier aparece neste arquivo — o descritor é plain
//! data, exatamente para a ponte do ECS poder descrever um joint sem depender do
//! solver.

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
    /// **Rod** — the anchors are held at `max_length`, and both bodies are free
    /// to turn. The connecting rod, the tie bar, the strut of a four-bar
    /// linkage.
    ///
    /// ⚠️ **It is the one thing this kit could not express**, and the gap is
    /// narrow enough to be easy to miss: a Weld holds the distance but *also*
    /// freezes the rotation, a Rope holds only the ceiling (it goes slack), and
    /// a Spring is deliberately bouncy. A linkage needs the distance held and
    /// the ends free — which is none of the three.
    ///
    /// ⛔ **The obvious construction is MEASURED and DEAD — do not try it
    /// again.** *"A rope with `set_limits(LinX, [d, d])`"* does not hold: in
    /// rapier 0.28 `limit_linear_coupled` carries the literal comment
    /// `// FIXME: handle min limit too.`, reads only `limits[1]` and leaves
    /// `impulse_bounds = [0, INFINITY]` — the constraint is **unilateral**, and
    /// the minimum of a coupled linear limit simply is not implemented. Built
    /// that way, a rod measures **min 0.0293 m** on the inverted pendulum, which
    /// is the rope's own number to four decimals.
    ///
    /// **What it is instead:** a **position motor on the coupled linear axis**
    /// at `target = length`, stiff and critically damped
    /// ([`JointDesc::ROD_STIFFNESS`]). Mechanically that is the same family as a
    /// [`JointKind::Spring`] — and it is a separate *kind* for the reason the
    /// artist cares about: a Spring ships bouncy on purpose (stiffness 30, sag
    /// 6.5 cm under 0.2 kg) and exposes three numbers, of which the damping must
    /// be a *function* of the stiffness to not oscillate. A Rod exposes **one
    /// number, the length**, and derives the rest.
    Rod,
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

    /// How stiff a [`JointKind::Rod`] is. **MEASURED**, hanging a growing load
    /// on a 2 m rod and reading both how far it stretches and whether the
    /// settled tail ripples (a motor too stiff for the sub-step count buzzes,
    /// and buzzing is worse than stretching a millimetre):
    ///
    /// | stiffness | stretch @ 12.6 kg | tail ripple |
    /// |---|---|---|
    /// | 1e4 | 12.3 mm | 0.0000 |
    /// | 1e5 | 1.2 mm | 0.0000 |
    /// | **1e6** | **0.1 mm** | 0.0000 |
    /// | 1e7 | 0.0 mm | 0.0000 |
    ///
    /// **1e6**, because it puts the stretch an order of magnitude *below the
    /// engine's own resting contact tolerance* (1.3 mm, W2a) — under that, the
    /// solver itself cannot tell the rod from rigid — and 1e7 buys nothing this
    /// probe can measure. Nothing here is limited by stability: the ripple is
    /// zero at every value tried, because rapier solves a motor as a soft
    /// constraint rather than an explicit spring.
    pub const ROD_STIFFNESS: f32 = 1.0e6;

    /// Damping of a [`JointKind::Rod`] — critical (`2·√k`), derived rather than
    /// authored precisely because it is a *function* of the stiffness, and a
    /// knob whose right value is computed from another knob is the ergonomics
    /// failure this project names by hand.
    ///
    /// ⚠️ **MEASURED INERT at [`Self::ROD_STIFFNESS`], and this note exists so
    /// nobody claims otherwise.** A mutation setting it to `0` survives every
    /// gate, and that is a fact about the solver rather than a hole: rapier
    /// integrates a motor as an *implicit soft constraint*, not an explicit
    /// spring, so at 1e6 the constraint is met every sub-step and there is no
    /// oscillation left to damp — transient peak **8 µm at damping 0 and at
    /// 20000**, settled ripple **exactly 0** at both. It stays critical so the
    /// constant is still right if a future rod is made softer; it gets **no
    /// gate**, because a bar on a quantity that does not move could not fail for
    /// the reason it would allege.
    pub const ROD_DAMPING: f32 = 2000.0;
}
