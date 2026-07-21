//! **Contact state** (W-Contacts) — which entities are touching, where, and under
//! how much load.
//!
//! The solid sibling of [`super::triggers`], and deliberately the same shape: the
//! world reports body HANDLES ([`ph2d_physics::PhysicsWorld::contact_reports`]) and
//! this turns them into entities the overlay can draw. Split out of `bridge.rs` for
//! the same LOC reason, `impl PhysicsBridge` here like the other submodules.
//!
//! ## Why a flat list and not a map
//!
//! `triggers` is a `BTreeMap<sensor, Vec<inside>>` because a trigger is asked about
//! ONE entity ("is this sensor firing?"). A contact has no owner — it is a
//! *relationship*, symmetric by construction — so the honest shape is the list of
//! relationships, in the deterministic order the world already sorted them into. The
//! per-entity question is answered by scanning it, which is what
//! [`PhysicsBridge::contact_count`] does; at the counts this module sees that is
//! cheaper than maintaining a second index of the same fact.

use ph2d_ecs::Entity;

use super::PhysicsBridge;

/// One touching pair, in entity space — what the overlay draws.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyContact {
    /// The two entities, in the world's handle order (so a pair reads the same
    /// whichever way the narrow phase reported it).
    pub a: Entity,
    pub b: Entity,
    /// The deepest contact point, in world units.
    pub point: [f32; 2],
    /// The load this pair is carrying right now, in N·s. **Not an impact peak** —
    /// see [`ph2d_physics::ContactReport::impulse`], where that was measured.
    pub impulse: f32,
}

impl PhysicsBridge {
    /// Rebuild [`contacts`](Self::contacts) from the world's current touching pairs.
    ///
    /// Returns early — before building the handle→entity map — when nothing is
    /// touching, which is every frame of a scene in free fall. The map is built here
    /// rather than maintained every frame for the same reason `triggers` does it: it
    /// is only needed when there is something to translate.
    pub(super) fn rebuild_contacts(&mut self) {
        self.contacts.clear();
        let reports = self.world.contact_reports();
        if reports.is_empty() {
            return;
        }
        let mut by_handle: std::collections::BTreeMap<(u32, u32), Entity> = Default::default();
        for (e, b) in &self.bodies {
            by_handle.insert(b.handle.into_raw_parts(), *e);
        }
        for r in reports {
            let (Some(&a), Some(&b)) = (
                by_handle.get(&r.body1.into_raw_parts()),
                by_handle.get(&r.body2.into_raw_parts()),
            ) else {
                continue;
            };
            self.contacts.push(BodyContact {
                a,
                b,
                point: r.point,
                impulse: r.impulse,
            });
        }
    }

    /// Every pair of entities touching right now — what the overlay draws a spark on.
    /// Sorted (the world sorted the handles, and the map preserves that order). Empty
    /// in a scene where nothing touches.
    #[must_use]
    pub fn contacts(&self) -> &[BodyContact] {
        &self.contacts
    }

    /// How many bodies `entity` is touching right now. A scan, not an index — see the
    /// module header for why a contact has no owner to key a map on.
    #[must_use]
    pub fn contact_count(&self, entity: Entity) -> usize {
        self.contacts
            .iter()
            .filter(|c| c.a == entity || c.b == entity)
            .count()
    }
}
