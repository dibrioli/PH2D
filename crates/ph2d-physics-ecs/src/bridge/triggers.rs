//! **Trigger state** (W7) — which sensor has which body inside it.
//!
//! Split out of `bridge.rs` for LOC. A sensor collider passes through but the
//! solver reports its overlaps ([`PhysicsWorld::intersecting_body_pairs`]); this
//! turns those body-handle pairs into an entity map the overlay and the
//! Inspector read. `impl PhysicsBridge` here, so it reaches the private fields
//! the way the other `bridge::*` submodules do.

use std::collections::BTreeMap;

use ph2d_ecs::Entity;

use super::PhysicsBridge;

impl PhysicsBridge {
    /// Rebuild [`triggers`](Self::triggers) from the world's current sensor
    /// overlaps. Each sensor entity gets the entities inside it. Returns early —
    /// before touching the reverse map — when nothing overlaps a sensor, which
    /// is every frame of a scene with no triggers.
    pub(super) fn rebuild_triggers(&mut self) {
        self.triggers.clear();
        let pairs = self.world.intersecting_body_pairs();
        if pairs.is_empty() {
            return;
        }
        // handle → entity, built once here (only when a sensor actually
        // overlaps something) rather than maintained every frame.
        let mut by_handle: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
        for (e, b) in &self.bodies {
            by_handle.insert(b.handle.into_raw_parts(), *e);
        }
        for (h1, h2) in pairs {
            let (Some(&e1), Some(&e2)) = (
                by_handle.get(&h1.into_raw_parts()),
                by_handle.get(&h2.into_raw_parts()),
            ) else {
                continue;
            };
            // At least one side is a sensor (a solid pair never intersects), but
            // both can be — each sensor lists the OTHER body as inside it.
            if self.bodies.get(&e1).is_some_and(|b| b.rest.is_sensor) {
                self.triggers.entry(e1).or_default().push(e2);
            }
            if self.bodies.get(&e2).is_some_and(|b| b.rest.is_sensor) {
                self.triggers.entry(e2).or_default().push(e1);
            }
        }
        for inside in self.triggers.values_mut() {
            inside.sort_unstable_by_key(|e| e.to_bits());
            inside.dedup();
        }
    }

    /// Is `entity` a **sensor** with at least one body inside it right now? The
    /// overlay reads this to light a triggered sensor up.
    ///
    /// ⚠️ This used to claim the Inspector read it for an "N inside" readout. It never
    /// did — §11 has no readout row, and grepping for a consumer found none (caught
    /// while building the contact channel next door, which faced the same question and
    /// gave the same answer: the visible half is the OVERLAY). A comment that names a
    /// consumer which does not exist is worse than none, because it reads as coverage
    /// ([[feedback_stale_comment_and_dead_code_lie]]).
    pub fn is_triggered(&self, entity: Entity) -> bool {
        self.triggers.get(&entity).is_some_and(|v| !v.is_empty())
    }

    /// The entities currently inside sensor `entity` (empty slice if it is not a
    /// triggered sensor). Sorted for a stable readout.
    ///
    /// Queryable state with no consumer in this build — deliberate, and the same
    /// shape the contact list has: *what a game DOES with an overlap* is the next
    /// layer, and this is the door it will come through.
    pub fn bodies_inside(&self, entity: Entity) -> &[Entity] {
        self.triggers.get(&entity).map_or(&[], Vec::as_slice)
    }

    /// The sensor entities that have at least one body inside them right now —
    /// what the overlay lights up. Sorted (the map is). Empty without sensors.
    pub fn triggered_sensors(&self) -> Vec<Entity> {
        self.triggers
            .iter()
            .filter(|(_, inside)| !inside.is_empty())
            .map(|(e, _)| *e)
            .collect()
    }
}
