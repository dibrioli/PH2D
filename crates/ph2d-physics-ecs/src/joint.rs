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
}

impl JointKind {
    /// Does this kind swing about a pivot? Only a [`JointKind::Pin`] does, and
    /// it is the only one with limits or a motor.
    ///
    /// **One door.** The Inspector asks it to decide which rows to paint, and
    /// the bridge asks it to decide which parameters to hand the solver. Two
    /// answers to *"does this joint have a motor?"* is how a knob comes to be
    /// painted for a kind that ignores it.
    pub fn is_hinge(self) -> bool {
        matches!(self, JointKind::Pin)
    }

    /// Does this kind have a length the artist tunes? Only Spring and Rope do —
    /// a Pin's length is zero (the anchors coincide) and a Weld's is too (it is
    /// rigid). **Not** `!is_hinge()`: a Weld is not a hinge but still has no
    /// length, so the two questions had to stop sharing an answer.
    pub fn has_length(self) -> bool {
        matches!(self, JointKind::Spring | JointKind::Rope)
    }

    /// Do the two bodies share one point (a Pin or a Weld), rather than have two
    /// separate ends (a Spring or a Rope)? The anchor policy reads this: a
    /// shared-point joint anchors both bodies at the same world point, a
    /// two-ended one anchors body B at its own centre.
    pub fn shares_a_point(self) -> bool {
        !self.has_length()
    }
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
    /// Drive the hinge? (Pin only.)
    pub motor_enabled: bool,
    /// Target angular velocity, radians/s. Sign picks the direction.
    pub motor_speed: f32,
    /// The force ceiling that makes a motor stoppable — a weak one stalls
    /// against a heavy load instead of winning.
    pub motor_max_force: f32,
    /// Spring: the length it pulls towards, meters.
    pub rest_length: f32,
    /// Spring: how hard it pulls.
    pub stiffness: f32,
    /// Spring: how fast it stops bouncing.
    pub damping: f32,
    /// Rope: the distance the anchors may not exceed, meters.
    pub max_length: f32,
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
        self.motor_max_force = finite(self.motor_max_force, d.motor_max_force).max(0.0);
        self.rest_length = finite(self.rest_length, d.rest_length).max(0.0);
        self.stiffness = finite(self.stiffness, d.stiffness).max(0.0);
        self.damping = finite(self.damping, d.damping).max(0.0);
        // rapier's own docs require a rope's distance to be strictly positive.
        self.max_length = finite(self.max_length, d.max_length).max(Self::MIN_LENGTH);
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
