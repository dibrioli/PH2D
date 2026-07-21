//! **Force zones** — an area that pushes whatever is inside it (ADR-0131 W-Area).
//!
//! Wind, an updraft, a conveyor, a current: a sensor collider carrying a force
//! vector, applied each substep to every dynamic body overlapping it. Unity's
//! `AreaEffector2D` and Godot's `Area2D` overrides are the same shape.
//!
//! ## A force, never an acceleration — and why that decides the model
//!
//! The impulse `F·dt` is resisted by the body's mass, so a leaf is carried by a
//! wind a crate barely feels. That asymmetry IS the feature: it is the intuition
//! every artist brings, and it is the half of "make this thing move" that cannot
//! be authored per body today. An *acceleration* zone would be mass-independent,
//! and it would also be a second answer to a question `gravity_scale` (W8) already
//! answers per body — two doors to one quantity, the recurring bug of this line.
//!
//! ## The sensor coupling
//!
//! A force zone must be a **sensor**. Two independent reasons, and they agree:
//! the narrow phase only records an *intersection* when one side is a sensor (a
//! solid pair produces a contact instead), so a solid zone would see nobody; and
//! a solid collider pushes bodies OUT rather than letting them in, so an area you
//! cannot enter is not an area. [`zone_force`] is the single door that answers
//! "is this body a force zone" — the world registers through it and the §11 rows
//! are offered under the same condition, so the two cannot drift.
//!
//! ## Impulse, and awake
//!
//! `apply_impulse`, never `add_force` — the lesson `drag` paid for: rapier's
//! `add_force` persists until `reset_forces`, which the pipeline never calls, so
//! it accumulates over every substep of every tick.
//!
//! ⚠️ **The impulse WAKES the body** (unlike drag, which passes `false`). A zone
//! that cannot start a body already resting on the floor is broken in its main
//! use — the updraft under a crate, the conveyor under a box. The price is that a
//! body inside an active zone does not sleep, which is honest: it is being pushed.
//! A zone with a ZERO force is never registered at all, precisely so it cannot
//! wake anything — that is what keeps an inert zone byte-identical (there is a
//! gate on it).

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::geometry::{ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;

use super::desc::{AreaEffect, BodyDesc};

/// **Is this body a force zone, and with what effect?** The single door.
///
/// `None` for a body with no effector, for a zero force, and for a **solid**
/// collider.
///
/// ⚠️ **Neither refusal is load-bearing for the simulation, and both were measured
/// to find that out.** rapier's own `apply_impulse` opens with `if !impulse.is_zero()`,
/// so a zero force could not wake anything even if it were registered; and the
/// narrow phase records an *intersection* only when one side is a sensor, so a solid
/// zone would see nobody. Deleting either line leaves every wrapper gate green, and
/// that is expected — the same shape as `drag`'s early-out and the impasto light's
/// flat-paint branch ([[feedback_layered_defenses_need_per_layer_gates]]: ask what
/// each layer protects ALONE, and be willing to answer "nothing").
///
/// What they DO buy: an inert zone never enters `effectors`, so the substep walk is
/// skipped entirely rather than iterated to no effect; and the sensor half is the
/// single door the §11 rows mirror — the panel offers Force only for a sensor, and
/// its seam gate is where that rule IS observable.
#[must_use]
pub(crate) fn zone_effect(desc: &BodyDesc) -> Option<AreaEffect> {
    let e = desc.effector?;
    let inert = e.force[0] == 0.0 && e.force[1] == 0.0 && e.drag <= 0.0;
    if !desc.is_sensor || inert {
        return None;
    }
    Some(e)
}

/// Apply one substep of every registered zone's force to the dynamic bodies
/// overlapping it.
///
/// Called from inside [`super::PhysicsWorld::step`]'s substep loop, beside
/// `drag::apply` and for the same reason: the integrator sees smaller slices, and
/// an impulse is exactly "this force, for this slice of time".
///
/// ⚠️ The overlap read is the narrow phase's state from the PREVIOUS pipeline
/// step, so a body entering a zone is pushed one substep later than it geometrically
/// arrives. That is inherent to reading a solver's own broad/narrow phase rather
/// than re-deriving overlap here (which would be a second, disagreeing answer to
/// "who is inside"), and at 240 Hz of substeps it is not a visible delay.
///
/// `zones` is kept sorted by handle by the world, so a body standing in TWO
/// overlapping zones accumulates their impulses in a fixed order — float addition
/// is not associative, and the ORDER of a sum is exactly the kind of thing that
/// makes a cross-OS hash drift (HR-5). Within one zone the order cannot matter:
/// each overlapping body is a distinct entry, so each gets exactly one impulse.
pub(crate) fn apply(
    bodies: &mut RigidBodySet,
    colliders: &ColliderSet,
    narrow_phase: &NarrowPhase,
    zones: &[(RigidBodyHandle, AreaEffect)],
    dt: f32,
) {
    // Fast path AND contract: a world with no zone never touches a body, so it is
    // byte-identical to one built before this module existed.
    if zones.is_empty() {
        return;
    }
    for &(zone_body, effect) in zones {
        let force = Vector2::new(effect.force[0], effect.force[1]);
        // The zone's own collider handle, copied out so the immutable borrow of
        // `bodies` ends before the impulses below need it mutably.
        let Some(zone_collider) = bodies
            .get(zone_body)
            .and_then(|b| b.colliders().first().copied())
        else {
            continue;
        };
        for (c1, c2, intersecting) in narrow_phase.intersection_pairs_with(zone_collider) {
            // `intersecting` is the real shape overlap; the pair exists as soon as
            // the bounding volumes touch, which is a bigger region than the area.
            if !intersecting {
                continue;
            }
            let other = if c1 == zone_collider { c2 } else { c1 };
            // ONE body, ONE collider: `spawn_body` inserts exactly one, so "the
            // other collider of the pair" is always a different BODY, and there is
            // no self-push to guard against. A guard for it was written, no gate
            // could reach it, and code no gate can reach is a claim that the model
            // is looser than it is. If a multi-collider body ever arrives, the
            // guard comes back — with a fixture that contains the case.
            let Some(parent) = colliders.get(other).and_then(|c| c.parent()) else {
                continue;
            };
            let Some(b) = bodies.get_mut(parent) else {
                continue;
            };
            // Only a dynamic body has a velocity the solver owns. rapier would
            // ignore the impulse on the others anyway, but asking here keeps the
            // wake-up (which is NOT a no-op) off a body that cannot move.
            if !b.is_dynamic() {
                continue;
            }
            b.apply_impulse(force * dt, true);
            // The medium's resistance. ⚠️ Written as `v /= 1 + d·dt`, which is
            // *exactly* what rapier's own `linear_damping` integrator does — so the
            // word "drag" means one thing across the world default, the per-body
            // override and this. Applying it as another impulse would be a second law
            // for the same word, and the two would disagree at large `d` (an impulse
            // can overshoot through zero and push the body BACKWARDS; this cannot).
            if effect.drag > 0.0 {
                let k = 1.0 / (1.0 + effect.drag * dt);
                let v = *b.linvel();
                let w = b.angvel();
                b.set_linvel(v * k, true);
                b.set_angvel(w * k, true);
            }
        }
    }
}
