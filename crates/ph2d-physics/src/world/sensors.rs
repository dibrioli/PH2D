//! Sensor (trigger) overlap readback — ADR-0131 W7.
//!
//! A sensor collider passes through (no contact forces) but the narrow phase
//! still records which colliders overlap it, in its intersection graph. This
//! reads that graph back as body pairs, so the ECS bridge can turn handles into
//! entities and publish a trigger state without ever seeing a rapier type.

use rapier2d::dynamics::RigidBodyHandle;

use super::PhysicsWorld;

/// A pair of body handles in a fixed order, so `(a, b)` and `(b, a)` collapse to
/// one entry no matter which way the narrow phase reported the intersection.
fn ordered(a: RigidBodyHandle, b: RigidBodyHandle) -> (RigidBodyHandle, RigidBodyHandle) {
    if a.into_raw_parts() <= b.into_raw_parts() {
        (a, b)
    } else {
        (b, a)
    }
}

impl PhysicsWorld {
    /// Every pair of **bodies** whose colliders currently overlap through a
    /// sensor. At least one side is a sensor (a solid-vs-solid pair produces a
    /// contact, never an intersection), and each body owns one collider, so the
    /// pair is reported by body handle.
    ///
    /// The result is **sorted and de-duplicated** — the intersection graph's
    /// own order is an internal detail, and a trigger state built from an
    /// unstable order would flicker frame to frame. (It does not feed the C9
    /// hash — only poses do — but a reproducible trigger readout is still worth
    /// the one sort.) Empty in a world with no sensors, so a scene without
    /// triggers pays nothing.
    #[must_use]
    pub fn intersecting_body_pairs(&self) -> Vec<(RigidBodyHandle, RigidBodyHandle)> {
        let mut pairs: Vec<(RigidBodyHandle, RigidBodyHandle)> = self
            .narrow_phase
            .intersection_pairs()
            .filter(|(_, _, intersecting)| *intersecting)
            .filter_map(|(c1, c2, _)| {
                let b1 = self.colliders.get(c1)?.parent()?;
                let b2 = self.colliders.get(c2)?.parent()?;
                Some(ordered(b1, b2))
            })
            .collect();
        pairs.sort_unstable_by_key(|(a, b)| (a.into_raw_parts(), b.into_raw_parts()));
        pairs.dedup();
        pairs
    }
}
