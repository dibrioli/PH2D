//! **Contact readback — who hit whom, where, and how hard** (ADR-0131 W-Contacts).
//!
//! The solid counterpart of [`super::sensors`]. A sensor answers *"who is inside
//! me"* for a collider that passes through; this answers *"who am I touching"* for
//! one that does not — and, unlike an overlap, a contact has a **place** and a
//! **load**.
//!
//! ## Read-only, and that is the whole contract
//!
//! Nothing in [`super::PhysicsWorld::step`] calls this. It reads the narrow phase
//! the solver just filled and allocates a `Vec` for the caller; the world is not
//! touched, so installing it cannot move a single body — the C9 hash is unchanged
//! by construction, and there is a gate that asks the world for its hash before and
//! after a full read to keep it that way. That is why this wave adds **no** bodies
//! to the C9 harness: there is nothing new on the deterministic path to prove.
//!
//! ## Why one report per PAIR and not per contact point
//!
//! A box resting flat on the floor has **two** contact points (the two corners),
//! and a polygon can have more. Reporting each would answer *"how many corners are
//! touching"*, which is a fact about tessellation, not about the scene — two
//! objects touching is ONE event, and that is the question an artist (or, later, a
//! gameplay consumer) asks. The report carries the **deepest** point, which is where
//! the collision most is, and the **summed** normal impulse, which is how hard the
//! whole pair pushed.

use std::collections::BTreeMap;

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::{ColliderSet, NarrowPhase};

use super::PhysicsWorld;

/// A body pair keyed for the peak map — the two handles' raw parts, lower first,
/// so the key matches the order [`ContactReport`] reports the pair in. `BTreeMap`
/// wants `Ord`, and this derives it (unlike the handles themselves).
///
/// `pub` because the bridge diffs the per-tick union ([`PhysicsWorld::tick_contacts`])
/// against its standing set, and it already speaks handle raw parts (its own
/// handle→entity map is keyed the same way).
pub type PeakKey = ((u32, u32), (u32, u32));

/// What the world remembers about one pair over the sub-steps of a single tick — the
/// data an EVENT needs (W-TickContacts), which the settled end-of-tick state cannot
/// carry for a pair that lifted off before the last sub-step.
///
/// `impact` is the hardest the pair pushed at any sub-step (the peak, W-ImpactForce);
/// `point`/`impulse` are the LAST sub-step it was active in. For a pair still touching
/// at the tick's end, "last active" IS the end, so these agree with
/// [`PhysicsWorld::contact_reports`]; for a FAST touch — one caught in a mid-tick
/// sub-step and gone by the last — they are the closest instant the world saw it, which
/// is where its event's place and load come from.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PeakSample {
    /// The peak normal impulse over the tick's sub-steps, N·s.
    pub impact: f32,
    /// The deepest contact point at the last sub-step the pair was active in, world units.
    pub point: [f32; 2],
    /// The summed normal impulse at that same last-active sub-step, N·s.
    pub impulse: f32,
}

/// One touching pair, in plain data — no rapier type escapes except the body
/// handles the caller already speaks.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContactReport {
    /// The two bodies, in a fixed order (lower handle first) so a pair reads the
    /// same whichever way the narrow phase happened to report it.
    pub body1: RigidBodyHandle,
    pub body2: RigidBodyHandle,
    /// The **deepest** contact point, in world units. Where the touch most is.
    pub point: [f32; 2],
    /// The summed normal impulse over the pair's manifolds, in N·s — **the load this
    /// pair is carrying right now**.
    ///
    /// ⚠️ **Not the impact peak** — see [`Self::impact`], which is. A ball landing from
    /// 6 m reports the same *impulse* as the same ball sitting still (0.010032237 vs
    /// 0.010032236): `step` returns after the solver has already stopped the body, so
    /// what survives out here is the settled load, not the peak.
    ///
    /// What it IS is exactly and usefully physical: in a stack of four identical
    /// boxes the impulses come out **4 : 3 : 2 : 1** from the floor up, because the
    /// bottom contact holds four boxes and the top one holds one. It is a load meter,
    /// and that is what the overlay's standing cross size means.
    pub impulse: f32,
    /// The **peak** normal impulse this pair reached during the tick's sub-steps, in
    /// N·s — *how hard the hit was*, as opposed to [`Self::impulse`], which is the load
    /// it settled to (W-ImpactForce).
    ///
    /// This is the number a hit sound wants. It exists because [`Self::impulse`]
    /// **cannot** carry it: the impact happens *between* the sub-steps and is gone by
    /// the time `step` returns. Captured by a `max` over the sub-steps inside the step
    /// loop (measured: ≤ 1.93% of the HR-4 budget at 500 contact pairs, so always-on);
    /// see [`PhysicsWorld::step`].
    ///
    /// **`impact >= impulse` always** for a live pair: the tick's peak is at least its
    /// endpoint. For a pair that is touching now but whose peak was not captured (it
    /// began touching only on the readback, never during a stepped sub-step — which the
    /// current step loop cannot produce), this falls back to `impulse`.
    pub impact: f32,
}

/// The ordered body pair, deepest world point, and summed normal impulse of one
/// **actively touching** contact pair — or `None` for a near-miss (bounding
/// volumes overlap, no active manifold point).
///
/// The single door both readers go through, so `contact_reports` (the load meter)
/// and `accumulate_peaks` (the impact capture) cannot disagree about *which*
/// pairs are touching, in *what* order, at *what* load — a second copy of this
/// collider→body→order logic is exactly the kind that drifts.
fn active_pair(
    pair: &rapier2d::geometry::ContactPair,
    colliders: &ColliderSet,
) -> Option<(RigidBodyHandle, RigidBodyHandle, [f32; 2], f32)> {
    if !pair.has_any_active_contact {
        return None;
    }
    let c1 = colliders.get(pair.collider1)?;
    let c2 = colliders.get(pair.collider2)?;
    let (b1, b2) = (c1.parent()?, c2.parent()?);
    // ⚠️ `local_p1` is in **collider1's** frame, so it has to go through
    // collider1's world position — the same whose-frame-is-this care the one-way
    // hook pays, and for the same reason: the pair is not ordered for us.
    let (_, deepest) = pair.find_deepest_contact()?;
    let world = c1.position() * deepest.local_p1;
    let impulse = pair.total_impulse_magnitude();
    let (body1, body2) = if b1.into_raw_parts() <= b2.into_raw_parts() {
        (b1, b2)
    } else {
        (b2, b1)
    };
    Some((body1, body2, [world.x, world.y], impulse))
}

/// Fold this instant's contact loads into the peak map by `max` — the impact
/// capture, called once per sub-step from [`PhysicsWorld::step`]. Keyed by the
/// **body** pair (lower first), so it lines up with [`ContactReport`] when the
/// readback reads it back out.
///
/// The `impact` is a `max` and not a sum: it is the hardest the pair pushed at any
/// single instant of the tick, not the total over the tick (which would grow with the
/// sub-step count and mean nothing physical). The `point`/`impulse`, by contrast, are
/// **overwritten** each sub-step, so at the tick's end they hold the LAST sub-step the
/// pair was active in — which is what a fast touch's event needs (W-TickContacts): a
/// place and a load for a pair that is no longer touching when `step` returns.
pub(super) fn accumulate_peaks(
    narrow_phase: &NarrowPhase,
    colliders: &ColliderSet,
    out: &mut BTreeMap<PeakKey, PeakSample>,
) {
    for pair in narrow_phase.contact_pairs() {
        if let Some((b1, b2, point, impulse)) = active_pair(pair, colliders) {
            let key = (b1.into_raw_parts(), b2.into_raw_parts());
            let s = out.entry(key).or_insert(PeakSample {
                impact: 0.0,
                point,
                impulse,
            });
            s.impact = s.impact.max(impulse);
            s.point = point;
            s.impulse = impulse;
        }
    }
}

impl PhysicsWorld {
    /// Every pair of bodies **actually touching** right now, deepest point and
    /// summed impulse each.
    ///
    /// **The near-miss band is real, and it was measured.** The contact graph keeps a
    /// pair alive while the *bounding volumes* overlap: two circles 0.566 apart (radii
    /// 0.25) are 1 pair in the graph with 0 active contacts, while the same circles
    /// 0.003 apart on an axis are not in the graph at all. Reporting the graph as-is
    /// would call the first pair a collision — the same distinction
    /// `intersecting_body_pairs` draws with its `intersecting` flag, and the one a
    /// box-shaped fixture cannot see (a box's shape IS its AABB).
    ///
    /// ⚠️ **Two layers honour that, and each alone is enough today** — the flag, and
    /// the `?` on `find_deepest_contact` (no manifold point, no report). Mutating
    /// EITHER leaves all six gates green; mutating BOTH turns the near-miss gate red,
    /// which is how the gate was proven to be about something
    /// ([[feedback_layered_defenses_need_per_layer_gates]]). The flag stays as the
    /// primary predicate because it is rapier's own statement of the fact — the
    /// lookup merely happens to imply it, and would stop implying it the day
    /// speculative manifold points are kept.
    ///
    /// **Sorted by handle**, because the contact graph's own order is an internal
    /// detail and a readout built on it would flicker frame to frame. Empty for a
    /// world where nothing touches, so a scene in free fall pays one iteration of
    /// an empty graph.
    #[must_use]
    pub fn contact_reports(&self) -> Vec<ContactReport> {
        let mut out: Vec<ContactReport> = self
            .narrow_phase
            .contact_pairs()
            .filter_map(|pair| {
                let (body1, body2, point, impulse) = active_pair(pair, &self.colliders)?;
                // The impact peak the step loop captured for this pair. `impulse`
                // (the settled load) is the floor: the peak is at least the
                // endpoint, and a pair with no captured peak (impossible from the
                // current loop — a live pair touched the last sub-step) reads its
                // load.
                let key = (body1.into_raw_parts(), body2.into_raw_parts());
                let impact = self
                    .contact_peaks
                    .get(&key)
                    .map_or(impulse, |s| s.impact)
                    .max(impulse);
                Some(ContactReport {
                    body1,
                    body2,
                    point,
                    impulse,
                    impact,
                })
            })
            .collect();
        out.sort_unstable_by_key(|r| (r.body1.into_raw_parts(), r.body2.into_raw_parts()));
        out
    }

    /// The **per-tick union** of touching pairs — every pair that had an active contact
    /// in *any* sub-step of the last [`step`](PhysicsWorld::step), with its peak, its
    /// last-active place, and its last-active load ([`PeakSample`]).
    ///
    /// This is the SUPERSET of [`contact_reports`](Self::contact_reports) (the end-of-tick
    /// live state): a pair that lands and rebounds within one tick is here, and is not
    /// there. The bridge diffs this against its standing set to report a fast touch that
    /// the end-of-tick state can never show (W-TickContacts). It is the same ledger the
    /// impact peak already accumulates — no extra work on the stepping path, and cleared
    /// (capacity kept) at the start of every `step`.
    #[must_use]
    pub fn tick_contacts(&self) -> &BTreeMap<PeakKey, PeakSample> {
        &self.contact_peaks
    }
}
