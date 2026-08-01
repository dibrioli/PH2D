//! Sensor (trigger) overlap readback — ADR-0131 W7.
//!
//! A sensor collider passes through (no contact forces) but the narrow phase
//! still records which colliders overlap it, in its intersection graph. This
//! reads that graph back as body pairs, so the ECS bridge can turn handles into
//! entities and publish a trigger state without ever seeing a rapier type.

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::ColliderHandle;

use super::PhysicsWorld;

/// A pair of handles in a fixed order, so `(a, b)` and `(b, a)` collapse to one
/// entry no matter which way the narrow phase reported the intersection.
fn ordered<T: Copy>(a: T, b: T, key: impl Fn(T) -> (u32, u32)) -> (T, T) {
    if key(a) <= key(b) { (a, b) } else { (b, a) }
}

impl PhysicsWorld {
    /// Every pair of **colliders** currently overlapping through a sensor. At
    /// least one side is a sensor (a solid-vs-solid pair produces a contact,
    /// never an intersection).
    ///
    /// ⚠️ **This is the primitive, and the collider is the right unit** — being
    /// a sensor is a property of a *collider*, never of a body. While a body had
    /// exactly one collider the two statements were interchangeable, and the
    /// body-shaped answer below was written when that was true. W-Compound made
    /// it false: a character with a solid torso and a **sensor foot** reports the
    /// pair `(torso body, ground body)`, and *which* of the torso's shapes was
    /// the sensor is exactly the fact that was being thrown away — measured, the
    /// foot passed through correctly and no channel could name it
    /// ([`intersecting_body_pairs`](Self::intersecting_body_pairs) callers saw a
    /// body whose own collider is solid, and dropped the overlap).
    ///
    /// The result is **sorted and de-duplicated** — the intersection graph's own
    /// order is an internal detail, and a trigger state built from an unstable
    /// order would flicker frame to frame. (It does not feed the C9 hash — only
    /// poses do — but a reproducible trigger readout is still worth the one
    /// sort.) Empty in a world with no sensors, so a scene without triggers pays
    /// nothing.
    #[must_use]
    pub fn intersecting_collider_pairs(&self) -> Vec<(ColliderHandle, ColliderHandle)> {
        let mut pairs: Vec<(ColliderHandle, ColliderHandle)> = self
            .narrow_phase
            .intersection_pairs()
            .filter(|(_, _, intersecting)| *intersecting)
            .map(|(c1, c2, _)| ordered(c1, c2, ColliderHandle::into_raw_parts))
            .collect();
        pairs.sort_unstable_by_key(|(a, b)| (a.into_raw_parts(), b.into_raw_parts()));
        pairs.dedup();
        pairs
    }

    /// The same overlaps projected onto the **bodies** that own them.
    ///
    /// ⚠️ **Derived from [`intersecting_collider_pairs`](Self::intersecting_collider_pairs)
    /// rather than read a second time from the narrow phase** — two walks of the
    /// same graph are two answers to *what overlaps what*, and they would drift
    /// the day one of them learns a filter the other does not.
    ///
    /// ⚠️ **Lossy on purpose, and the loss is what a caller must weigh:** a body
    /// with several shapes appears here without saying which one is the sensor.
    /// Anything that needs to name the sensor asks the collider version.
    #[must_use]
    pub fn intersecting_body_pairs(&self) -> Vec<(RigidBodyHandle, RigidBodyHandle)> {
        let mut pairs: Vec<(RigidBodyHandle, RigidBodyHandle)> = self
            .intersecting_collider_pairs()
            .into_iter()
            .filter_map(|(c1, c2)| {
                let b1 = self.colliders.get(c1)?.parent()?;
                let b2 = self.colliders.get(c2)?.parent()?;
                Some(ordered(b1, b2, RigidBodyHandle::into_raw_parts))
            })
            .collect();
        pairs.sort_unstable_by_key(|(a, b)| (a.into_raw_parts(), b.into_raw_parts()));
        pairs.dedup();
        pairs
    }

    /// The body a collider hangs from, or `None` if the handle is dead or the
    /// collider is parentless.
    ///
    /// The bridge needs it to resolve *whose shape is this?* on the far side of
    /// an overlap: a collider that is not a known part must belong to a body's
    /// own shape, and this is how that body is named.
    #[must_use]
    pub fn collider_body(&self, handle: ColliderHandle) -> Option<RigidBodyHandle> {
        self.colliders.get(handle)?.parent()
    }
}
