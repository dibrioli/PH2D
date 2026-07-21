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

use rapier2d::dynamics::RigidBodyHandle;

use super::PhysicsWorld;

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
    /// ⚠️ **Not the impact peak, and that was measured rather than assumed.** A ball
    /// landing from 6 m reports the same impulse as the same ball sitting still
    /// (0.010032237 vs 0.010032236): `step` returns after the solver has already
    /// stopped the body, so the peak lives *between* the substeps and is gone before
    /// anyone can read it. Reporting an "impact strength" from out here would be a
    /// number that never gets big.
    ///
    /// What it IS is exactly and usefully physical: in a stack of four identical
    /// boxes the impulses come out **4 : 3 : 2 : 1** from the floor up, because the
    /// bottom contact holds four boxes and the top one holds one. It is a load meter,
    /// and that is what the overlay's spark size means.
    pub impulse: f32,
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
            .filter(|pair| pair.has_any_active_contact)
            .filter_map(|pair| {
                let c1 = self.colliders.get(pair.collider1)?;
                let c2 = self.colliders.get(pair.collider2)?;
                let (b1, b2) = (c1.parent()?, c2.parent()?);
                // ⚠️ `local_p1` is in **collider1's** frame, so it has to go
                // through collider1's world position — the same
                // whose-frame-is-this care the one-way hook pays, and for the same
                // reason: the pair is not ordered for us.
                let (_, deepest) = pair.find_deepest_contact()?;
                let world = c1.position() * deepest.local_p1;
                let (body1, body2) = if b1.into_raw_parts() <= b2.into_raw_parts() {
                    (b1, b2)
                } else {
                    (b2, b1)
                };
                Some(ContactReport {
                    body1,
                    body2,
                    point: [world.x, world.y],
                    impulse: pair.total_impulse_magnitude(),
                })
            })
            .collect();
        out.sort_unstable_by_key(|r| (r.body1.into_raw_parts(), r.body2.into_raw_parts()));
        out
    }
}
