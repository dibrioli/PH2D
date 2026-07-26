//! [`PhysicsJoint`] — a joint is an **entity**, and it names its two bodies.
//!
//! # Why an entity and not a field on a body
//!
//! Because that is what this editor already decided every object is
//! (ADR-0110: *vector nodes are ECS entities, one hierarchy*). Making the
//! joint an entity means it shows up in the Hierarchy, is selectable,
//! nameable, deletable, undoable and saveable — all of it machinery that
//! already exists and none of it written twice. Deleting a joint is deleting
//! an object; there is no "remove joint" button to invent.
//!
//! It also fixes the ceiling: a joint stored *on* a body can only be one per
//! body (bevy holds one component of a type per entity), which forbids closed
//! loops and makes a ragdoll's pelvis impossible to attach three ways.
//!
//! And the anchor comes free. The joint entity carries a `Transform`, and the
//! Inspector's Position fields already "land on every entity that has a
//! `Transform`, not just sprites" — so the pivot is authorable in numbers on
//! day one, with no new UI. A point-handle gizmo on canvas is a *different*
//! thing (the three `GizmoView` publishers are all boxes with scale handles),
//! and it landed later as `ph2d_editor::PointGizmoView` / `paint_point_gizmo`:
//! a grabbable dot at the anchor that opens a plain `Translate` drag of this
//! entity, so the pivot is draggable too, not only typeable.
//!
//! # The two bodies are named, never pointed at
//!
//! [`PhysicsJoint::body_a`] and `body_b` hold [`ph2d_ecs::stable_name_id`] —
//! the hash of the body's `Name` — and **not** `Entity::to_bits()`. Bits are
//! an allocation id: the undo respawns every entity with new ones, so a joint
//! holding bits would come undone on the first Ctrl+Z. Worse, bits inside a
//! component's serialized bytes make two logically identical states compare
//! different, which is the spurious-undo-step bug `canonicalize` exists to
//! kill. See the docs on `stable_name_id` for the whole argument, including
//! what it costs (renaming a body detaches its joints, exactly as it detaches
//! its timeline tracks).
//!
//! # The anchor is authored BODY-LOCAL, so it follows the body (the gold standard)
//!
//! [`PhysicsJoint::local_a`] / `local_b` hold the anchor in **each body's own
//! local frame** — the same pair rapier stores (`local_anchor1`/`local_anchor2`)
//! and the pair Box2D's `Initialize` and Unity's `anchor`/`connectedAnchor`
//! store. Because they are local, a body carries its anchor by construction:
//! move a body on the canvas and the pin stays glued where the artist put it.
//!
//! This replaces the old model, where the anchor was a single WORLD point (the
//! joint's `Transform.translation`) that the bridge re-derived to local **every
//! reconcile** against the bodies' current poses. That re-derivation was the
//! slide: moving a body left the world point put and shifted the local anchor
//! along the body. The world point is now a DERIVED display value —
//! `bridge::joints` syncs the joint's `Transform.translation = bodyA · local_a`
//! at rest so the anchor dot and the Inspector Position follow the body.
//!
//! [`PhysicsJoint::anchored`] is the seed sentinel. A joint arriving with only a
//! world `Transform` (a fresh `create_joint`, a bare test fixture, a re-pick)
//! carries `anchored: false`; the first reconcile derives `local_a`/`local_b`
//! from that `Transform` — the SAME conversion the old model did, once — and
//! flips it true. An authoring gesture that REPOSITIONS the pivot (the anchor
//! dot, the Inspector Position, re-picking a body) sets it back to `false` to
//! request one more re-derive; a body MOVE never touches it, which is the whole
//! fix.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// Which constraint this joint applies. **Append-only** — postcard encodes the
/// discriminant positionally, so new kinds go at the END or every saved
/// project reads its joints as the wrong type.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JointKind {
    /// The two bodies share a point and turn freely about it. The hinge, the
    /// pendulum's pivot, the ragdoll's elbow.
    #[default]
    Pin,
    /// A damped spring between the anchors — the distance is a target, not a
    /// law.
    Spring,
    /// The anchors may come as close as they like but never further apart than
    /// [`PhysicsJoint::max_length`].
    Rope,
    /// **Weld** — the two bodies are locked rigidly together at the anchor:
    /// no relative motion, no rotation. A pin with its rotation frozen. Useful
    /// for compound bodies and (later) breakable structures. It shares a point
    /// like a pin but has no motor, no limits, no length.
    Weld,
    /// **Slider** — the bodies may only slide along ONE axis and never rotate
    /// relative to each other. The elevator, the sliding door, the piston.
    ///
    /// The **axis is the joint entity's own `Transform::rotation`** — no new
    /// field, and no second place to keep it. That is the model Godot and
    /// Unreal use, and it is the one this component already implies: the
    /// entity's `Transform` is where a joint's *placement* lives (its
    /// translation is the anchor), so its rotation is where a placement's
    /// direction lives. It also means the axis is authorable on day one, in the
    /// Inspector's own Rotation field, with zero widgets added.
    ///
    /// Like a Pin it has **limits** — but they are a STROKE, in metres. See
    /// [`JointKind::limits_in_metres`].
    Slider,
}

impl JointKind {
    /// Does this kind swing about a pivot? Only a [`JointKind::Pin`] does.
    ///
    /// ⚠️ **No longer the same question as *"does it have a motor?"*** — that is
    /// [`Self::has_motor`], and it grew a Slider and a Rope in W-J6. This one is
    /// still asked wherever the *angular* nature is the point.
    pub fn is_hinge(self) -> bool {
        matches!(self, JointKind::Pin)
    }

    /// **Is this kind's free degree of freedom a TRANSLATION?** A Slider slides
    /// along its axis; a Rope's is the distance between the anchors. A Pin's is
    /// an angle.
    ///
    /// The one underlying fact behind every "metres or radians?" in this file:
    /// [`Self::limits_in_metres`] and [`Self::motor_in_metres`] both read it
    /// rather than re-listing the kinds, so a sixth kind states its unit once.
    pub fn translates(self) -> bool {
        matches!(self, JointKind::Slider | JointKind::Rope)
    }

    /// Does this kind have a limit RANGE the artist tunes? A Pin (an angular
    /// range) and a Slider (a stroke) do.
    ///
    /// ⚠️ **Split out of `is_hinge` when the Slider arrived**, for the same
    /// reason `has_length` had to be split out of `!is_hinge` when the Weld did:
    /// *"is it a hinge?"* and *"does it have limits?"* had the same answer while
    /// the Pin was the only limited kind, and one of the two questions was
    /// always going to grow a second member. Collapsed, a Slider would either
    /// get a motor it has no model for or lose the limits that make it a rail.
    pub fn has_limits(self) -> bool {
        matches!(self, JointKind::Pin | JointKind::Slider)
    }

    /// **Can this kind be DRIVEN?** A Pin's hinge, a Slider's rail and a Rope's
    /// distance (a winch) all have a free degree of freedom a motor can push
    /// along. A Spring and a Weld do not.
    ///
    /// **One door.** The Inspector asks it to decide whether to paint the Motor
    /// card, and the bridge asks it before handing a motor to the solver — two
    /// answers to *"does this joint have a motor?"* is how a knob comes to be
    /// painted for a kind that ignores it.
    ///
    /// ⚠️ **A Spring is excluded for a MECHANICAL reason, not a UI one:** rapier
    /// models a spring *as* a motor on the same axis, so a second motor there
    /// would overwrite the stiffness and damping the artist authored and turn the
    /// spring into a rate-driven rod. The wrapper states the same exclusion in
    /// `ph2d_physics::motor_axis`, which is what the solver actually reads.
    pub fn has_motor(self) -> bool {
        matches!(self, JointKind::Pin | JointKind::Slider | JointKind::Rope)
    }

    /// Are this kind's limits a distance rather than an angle?
    ///
    /// ⚠️ **`limit_min`/`limit_max` carry the unit of the KIND** — radians for a
    /// Pin, metres for a Slider — which is exactly how rapier models it (one
    /// `limits` field, belonging to whichever degree of freedom the joint left
    /// free). One door so the Inspector's label, its conversion, and the
    /// bridge's hand-off to the solver cannot disagree about what the number
    /// means; and the kind-change re-seeds the range, so `±0.785` rad never
    /// silently becomes `±0.785` m.
    ///
    /// Derived from [`Self::translates`] and [`Self::has_limits`] rather than
    /// listing kinds again: a Rope translates but has no limit range, so it
    /// answers `false` here and `true` to [`Self::motor_in_metres`] — the two
    /// questions genuinely differ for it, and deriving both from one fact is
    /// what keeps them able to.
    pub fn limits_in_metres(self) -> bool {
        self.has_limits() && self.translates()
    }

    /// Is this kind's MOTOR linear rather than angular — metres and metres per
    /// second rather than degrees and degrees per second?
    ///
    /// ⚠️ **Not the same question as [`Self::limits_in_metres`], and a Rope is
    /// the case that proves it:** a Rope has no limit range at all (so that one
    /// says `false`) and its motor is a winch reeling a distance (so this one
    /// says `true`). Sharing one door would have given the winch a target in
    /// degrees.
    pub fn motor_in_metres(self) -> bool {
        self.translates()
    }

    /// Does this kind have a length the artist tunes? Only Spring and Rope do —
    /// a Pin's length is zero (the anchors coincide) and a Weld's is too (it is
    /// rigid). **Not** `!is_hinge()`: a Weld is not a hinge but still has no
    /// length, so the two questions had to stop sharing an answer.
    pub fn has_length(self) -> bool {
        matches!(self, JointKind::Spring | JointKind::Rope)
    }

    /// Do the two bodies share one point (a Pin, a Weld or a Slider), rather than
    /// have two separate ends (a Spring or a Rope)? The anchor policy reads this:
    /// a shared-point joint anchors both bodies at the same world point, a
    /// two-ended one anchors body B at its own centre.
    ///
    /// A Slider shares a point **and that point is the zero of its stroke** —
    /// rapier measures the prismatic displacement from where the two anchors
    /// coincide, so authoring them apart would offset the whole range by the
    /// gap. It falls out of `!has_length()` correctly, but it is stated here
    /// rather than left to fall out by accident.
    pub fn shares_a_point(self) -> bool {
        !self.has_length()
    }
}

/// What a driven joint is *aiming at* — the editor's mirror of
/// [`ph2d_physics::MotorMode`]. **Append-only**, for the same postcard reason
/// [`JointKind`] is.
///
/// Its own enum rather than a re-export because this one is a **file format**:
/// it is `serde`, it travels inside the project, and its discriminants are
/// pinned by a gate. The wrapper's is a solver adapter with no such promise, and
/// `joint_desc` maps between them in one place — the same split `JointKind`
/// already makes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotorMode {
    /// Keep turning / keep sliding at [`PhysicsJoint::motor_speed`], forever.
    #[default]
    Velocity,
    /// Go to [`PhysicsJoint::motor_target`] and HOLD it — the servo.
    Position,
}

/// A joint between two named bodies. The entity carrying it also carries a
/// `Transform`, whose translation is the anchor (module docs).
///
/// Parameters that do not apply to the chosen [`JointKind`] are ignored and
/// keep their values — switching kind and switching back returns the joint you
/// had, the same way a `Collider` preserves its footprint across a shape
/// change.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsJoint {
    /// [`ph2d_ecs::stable_name_id`] of the first body's `Name`. `0` = unset.
    pub body_a: u64,
    /// The second body's, likewise.
    pub body_b: u64,
    pub kind: JointKind,
    /// Confine the hinge to an angular range? (Pin only.)
    pub limits_enabled: bool,
    /// The range, **radians**. The engine's unit; the Inspector shows degrees,
    /// converting at the paint/commit boundary like `Transform::rotation_rad`.
    pub limit_min: f32,
    pub limit_max: f32,
    /// Drive the free degree of freedom? ([`JointKind::has_motor`] kinds only:
    /// Pin, Slider, Rope.)
    pub motor_enabled: bool,
    /// Target velocity — **radians/s on a hinge, metres/s on a rail or a
    /// winch** ([`JointKind::motor_in_metres`]). Sign picks the direction. Read
    /// only in [`MotorMode::Velocity`].
    pub motor_speed: f32,
    /// The force ceiling that makes a motor stoppable — a weak one stalls
    /// against a heavy load instead of winning. In [`MotorMode::Position`] it is
    /// also what makes a servo *yield* rather than weld.
    pub motor_max_force: f32,
    /// Spring: the length it pulls towards, meters.
    pub rest_length: f32,
    /// Spring: how hard it pulls.
    pub stiffness: f32,
    /// Spring: how fast it stops bouncing.
    pub damping: f32,
    /// Rope: the distance the anchors may not exceed, meters.
    pub max_length: f32,
    /// The anchor on body A, in **body A's local frame** (module docs). Where
    /// the pin attaches ON the body, so it follows the body when the body moves.
    pub local_a: [f32; 2],
    /// The anchor on body B, likewise in B's local frame. For a two-ended joint
    /// (Spring/Rope) this is body B's centre (`[0, 0]`).
    pub local_b: [f32; 2],
    /// Have `local_a`/`local_b` been derived yet? `false` on a joint that
    /// arrives with only a world `Transform` (fresh create, bare fixture,
    /// re-pick) — the first reconcile seeds the locals from the `Transform` and
    /// flips this true. A reposition gesture sets it false again to re-seed; a
    /// body move never does (the slide fix).
    pub anchored: bool,
    /// Is the motor chasing a RATE or a PLACE? (W-J6.)
    pub motor_mode: MotorMode,
    /// [`MotorMode::Position`]: the place to hold, in the free degree of
    /// freedom's own unit — **radians on a hinge, metres on a rail or a winch**
    /// ([`JointKind::motor_in_metres`]), measured from the joint's zero (the
    /// anchor).
    ///
    /// ⚠️ Appended at the END with `motor_mode`, like `local_a`/`local_b`/
    /// `anchored` before them, because postcard encodes this struct
    /// **positionally**: a field inserted among the motor fields it reads with
    /// would shift every field after it, and every joint in every saved project
    /// would decode as something else. (`PROJECT_SCHEMA` bumps regardless — the
    /// two new fields make an old blob the wrong length — but the discipline is
    /// what keeps a future field from being inserted in the middle *without* a
    /// bump.)
    pub motor_target: f32,
}

impl Default for PhysicsJoint {
    /// A free pin between nothing and nothing. Every numeric default is the
    /// engine's measured one, so a joint the artist creates and never touches
    /// behaves the way the wrapper's tests describe.
    fn default() -> Self {
        Self {
            body_a: 0,
            body_b: 0,
            kind: JointKind::Pin,
            limits_enabled: false,
            limit_min: -Self::DEFAULT_LIMIT,
            limit_max: Self::DEFAULT_LIMIT,
            motor_enabled: false,
            motor_speed: Self::DEFAULT_MOTOR_SPEED,
            motor_max_force: Self::DEFAULT_MOTOR_MAX_FORCE,
            rest_length: 1.0,
            stiffness: ph2d_physics::JointDesc::DEFAULT_STIFFNESS,
            damping: ph2d_physics::JointDesc::DEFAULT_DAMPING,
            max_length: 1.0,
            // A fresh joint has no anchor yet: the first reconcile seeds the
            // locals from the entity's `Transform`. `[0, 0]` is a valid local
            // anchor (a body's centre), which is exactly why `anchored` — not a
            // sentinel value — records whether the seed has happened.
            local_a: [0.0, 0.0],
            local_b: [0.0, 0.0],
            anchored: false,
            motor_mode: MotorMode::Velocity,
            // Zero is the joint's own zero in EITHER unit — the authored angle of
            // a hinge, the anchor of a rail, a fully-reeled winch — so unlike
            // `motor_speed` this default needs no per-kind twin.
            motor_target: 0.0,
        }
    }
}

impl PhysicsJoint {
    /// Half-range a newly limited hinge gets: ±45°, wide enough to read as a
    /// hinge and narrow enough that switching limits on is visibly different
    /// from leaving them off.
    pub const DEFAULT_LIMIT: f32 = std::f32::consts::FRAC_PI_4;
    /// A new motor's speed, radians/s — about a third of a turn per second,
    /// slow enough to watch.
    pub const DEFAULT_MOTOR_SPEED: f32 = 2.0;
    /// A new motor's force ceiling. Strong enough to lift a small arm (the
    /// wrapper measured a 0.2 kg arm needing ~1 N·m), so a motor switched on
    /// visibly does something rather than looking broken.
    pub const DEFAULT_MOTOR_MAX_FORCE: f32 = 10.0;
    /// The shortest rope the solver accepts. A rope of zero length is a weld
    /// nobody asked for.
    pub const MIN_LENGTH: f32 = 1e-3;
    /// Half-stroke a newly limited **Slider** gets, metres — so its range is
    /// ±0.5 m, a metre of travel centred where the artist put the joint.
    ///
    /// Reads the same way `DEFAULT_LIMIT` does for a hinge: wide enough to be
    /// obviously a rail (the smoke's bodies are ~0.5-1 m, so the carriage
    /// travels about its own length each way) and narrow enough that switching
    /// limits on visibly differs from leaving them off.
    pub const DEFAULT_STROKE: f32 = 0.5;

    /// The default limit range for a kind, in **that kind's unit**.
    ///
    /// The door the kind-change re-seeds through: a Pin's ±45° reinterpreted as
    /// a Slider's stroke is ±0.785 **metres**, a number nobody typed, and the
    /// reverse turns a 0.5 m rail into a 28.6° hinge. Same field, two units, so
    /// the switch has to restate the range.
    pub fn default_limits(kind: JointKind) -> [f32; 2] {
        let half = if kind.limits_in_metres() {
            Self::DEFAULT_STROKE
        } else {
            Self::DEFAULT_LIMIT
        };
        [-half, half]
    }

    /// A new LINEAR motor's speed, metres/s — the rail and the winch.
    ///
    /// Chosen by the same rule [`Self::DEFAULT_MOTOR_SPEED`] states for the
    /// hinge (*"slow enough to watch"*), applied to the stroke a new Slider
    /// gets: `2 · DEFAULT_STROKE` is a metre of travel, so at 0.5 m/s the
    /// carriage crosses its whole range in **2 s** — long enough to see it go
    /// and come back rather than blink between the ends.
    pub const DEFAULT_LINEAR_MOTOR_SPEED: f32 = 0.5;

    /// The default motor speed for a kind, in **that kind's unit** — rad/s for a
    /// hinge, m/s for a rail or a winch.
    ///
    /// The twin of [`Self::default_limits`], and it exists for the same hazard:
    /// `motor_speed` carries the unit of the free degree of freedom, so a Pin's
    /// 2 rad/s reinterpreted as a Slider's is **2 m/s** — a carriage crossing
    /// its default stroke four times a second, a number nobody typed. The kind
    /// change re-seeds through here whenever [`JointKind::motor_in_metres`]
    /// flips.
    ///
    /// ⚠️ **A winch's default is NEGATIVE, and the reason is in rapier's own
    /// solver rather than in taste.** Read from
    /// `joint_constraint_builder::motor_linear_coupled` — the constraint a Rope's
    /// motor is actually built into, because a rope couples its linear axes:
    ///
    /// * the quantity the motor drives is `dist = ‖lin_jac‖`, **the distance
    ///   between the two anchors** — a non-negative magnitude, with the jacobian
    ///   the unit vector between them. So `target_pos` is a target *length* and
    ///   `target_vel` is a target *rate of change of length*: **positive pays
    ///   out, negative reels in**;
    /// * and **the limits CLAMP the motor's own target velocity**, in that
    ///   function, before any force is computed:
    ///   `target_vel = clamp(target_vel, (min − dist)/dt, (max − dist)/dt)`.
    ///
    /// A rope carries `limits = [0, max_length]` and a hanging load sits at
    /// **exactly** `max_length`, so the upper bound is `(max_length − dist)/dt`
    /// = **0**: every positive target is clamped to zero. That is also why
    /// `max_force` looked inert — `max_impulse = max_force · dt` bounds the
    /// impulse, and with a target of zero there is no error to spend it on.
    /// **One cause, both knobs dead.** (Enio, 2026-07-26: *"Velocity parâmetros
    /// Speed e Max Force não afetam a simulação"*.)
    ///
    /// Measured on that rig, told `+0.5`: `y = 4.000` for five seconds at every
    /// `max_force`. Told `-0.5`: 4.489 → 4.980 → 5.470 → 5.960 → 6.000, a metre
    /// every two seconds until the rope runs out. A default that is a no-op from
    /// the state the artist starts in is a knob that looks dead.
    ///
    /// ⚠️ **The same clamp applies to a Slider** parked at an end of its stroke,
    /// and there it is right and visible: the rail stops, and the artist can see
    /// the end-of-travel tick it stopped at. On a rope the boundary is where the
    /// load *hangs*, which is why only this kind needed its default moved.
    pub fn default_motor_speed(kind: JointKind) -> f32 {
        match kind {
            // Reel IN: see above.
            JointKind::Rope => -Self::DEFAULT_LINEAR_MOTOR_SPEED,
            k if k.motor_in_metres() => Self::DEFAULT_LINEAR_MOTOR_SPEED,
            _ => Self::DEFAULT_MOTOR_SPEED,
        }
    }

    /// **A joint of `kind`, with every unit-carrying number seeded in THAT
    /// kind's own unit.**
    ///
    /// [`Self::default`] cannot do this — it has no kind to ask — so it seeds the
    /// Pin's: `±45°` of range and `2 rad/s` of motor. Read as a Slider those are
    /// **±0.785 metres** of stroke and **2 m/s** of carriage, and read as a Rope
    /// the speed is 2 m/s of winch. Numbers nobody typed, in a joint the artist
    /// just created.
    ///
    /// ⚠️ **The kind CHANGE has re-seeded through the same doors since W-J5, and
    /// creation did not** — `create_joint` built `PhysicsJoint { kind,
    /// ..default() }`, so "Join As = Slider" and "make a Pin then switch it to
    /// Slider" produced different joints. Two ways to reach one state that
    /// disagree is the shape this whole file keeps refusing; this is the one
    /// door both now take.
    #[must_use]
    pub fn of_kind(kind: JointKind) -> Self {
        let [lo, hi] = Self::default_limits(kind);
        Self {
            kind,
            limit_min: lo,
            limit_max: hi,
            motor_speed: Self::default_motor_speed(kind),
            ..Self::default()
        }
    }

    /// This joint with every number forced back into a range the solver can
    /// use. **The door a loaded project file comes through.**
    ///
    /// The Inspector already sanitises what it writes, but a component is
    /// `serde` and travels in the project file, so the Inspector is not the
    /// only way values arrive — and this is the last place before rapier.
    /// `PhysicsSettings::clamped` exists for exactly this reason on the world
    /// side; without the twin, joints were the one loader-facing surface in
    /// the line that did not clamp.
    ///
    /// Measured on the unclamped version: `stiffness = NaN` drove the body's
    /// pose to `(NaN, NaN)` within 120 steps, and `readback` then wrote NaN
    /// straight into the entity's `Transform` — where it flows into the
    /// cross-OS determinism hash. `max_length = -1` behaved as an unrelated
    /// length, silently.
    ///
    /// ⚠️ **Inverted limits are a WELD, not a wide hinge.** `limit_min` and
    /// `limit_max` are authored independently, so `min > max` is one keystroke
    /// away — and rapier, handed `[min, max]` that way, froze the plank solid
    /// (measured: `rot 0.000` after 180 steps). A hinge the artist believes is
    /// limited to ±45° being a weld is the kind of wrong that has no symptom
    /// to search for, so the pair is ordered here.
    pub fn clamped(mut self) -> Self {
        fn finite(v: f32, fallback: f32) -> f32 {
            if v.is_finite() { v } else { fallback }
        }
        let d = Self::default();
        self.limit_min = finite(self.limit_min, d.limit_min);
        self.limit_max = finite(self.limit_max, d.limit_max);
        if self.limit_min > self.limit_max {
            std::mem::swap(&mut self.limit_min, &mut self.limit_max);
        }
        self.motor_speed = finite(self.motor_speed, d.motor_speed);
        // A servo's target flows into rapier's `target_pos`; a NaN there poisons
        // the pose exactly as a NaN stiffness does (measured, above).
        self.motor_target = finite(self.motor_target, d.motor_target);
        self.motor_max_force = finite(self.motor_max_force, d.motor_max_force).max(0.0);
        self.rest_length = finite(self.rest_length, d.rest_length).max(0.0);
        self.stiffness = finite(self.stiffness, d.stiffness).max(0.0);
        self.damping = finite(self.damping, d.damping).max(0.0);
        // rapier's own docs require a rope's distance to be strictly positive.
        self.max_length = finite(self.max_length, d.max_length).max(Self::MIN_LENGTH);
        // A local anchor flows straight into rapier's `local_anchor1/2`; a NaN
        // there poisons the body's pose (the same failure `stiffness = NaN`
        // caused above). Guard each component back to the body's centre.
        self.local_a = [finite(self.local_a[0], 0.0), finite(self.local_a[1], 0.0)];
        self.local_b = [finite(self.local_b[0], 0.0), finite(self.local_b[1], 0.0)];
        self
    }

    /// Is this joint fully specified — does it name two *different* bodies?
    ///
    /// A joint naming one body twice, or naming none, is not a joint the
    /// solver can build. Asked here so the bridge and the Inspector agree
    /// about what "incomplete" means.
    pub fn names_two_bodies(&self) -> bool {
        self.body_a != 0 && self.body_b != 0 && self.body_a != self.body_b
    }
}

impl SimComponent for PhysicsJoint {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_joint_that_names_one_body_twice_is_not_complete() {
        let j = PhysicsJoint {
            body_a: 7,
            body_b: 7,
            ..PhysicsJoint::default()
        };
        assert!(!j.names_two_bodies());
    }

    #[test]
    fn a_fresh_joint_names_nobody() {
        assert!(!PhysicsJoint::default().names_two_bodies());
    }

    #[test]
    fn only_a_pin_is_a_hinge_and_only_the_others_have_a_length() {
        assert!(JointKind::Pin.is_hinge());
        assert!(!JointKind::Pin.has_length());
        for k in [JointKind::Spring, JointKind::Rope] {
            assert!(!k.is_hinge(), "{k:?} is not a hinge");
            assert!(k.has_length(), "{k:?} has a length");
        }
        // A Weld is neither a hinge nor length-tuned — the case that broke the
        // old `has_length == !is_hinge` shortcut — but it DOES share a point.
        assert!(!JointKind::Weld.is_hinge());
        assert!(!JointKind::Weld.has_length());
        assert!(JointKind::Weld.shares_a_point());
        assert!(JointKind::Pin.shares_a_point());
        assert!(!JointKind::Spring.shares_a_point());
    }

    /// `JointKind` goes into the project file, where postcard encodes the
    /// discriminant **positionally**. Reordering these — or inserting a kind
    /// anywhere but the end — silently reads every saved Spring as a Rope.
    #[test]
    fn the_kind_discriminants_are_pinned_in_order() {
        let all = [
            JointKind::Pin,
            JointKind::Spring,
            JointKind::Rope,
            JointKind::Weld,
        ];
        for (i, k) in all.iter().enumerate() {
            let bytes = postcard::to_allocvec(k).expect("encode");
            assert_eq!(
                bytes[0] as usize, i,
                "{k:?} must stay at discriminant {i} — postcard is positional"
            );
        }
    }
}
