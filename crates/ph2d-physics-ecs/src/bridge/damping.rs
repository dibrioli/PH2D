//! The per-body damping re-stamp pass — [`PhysicsBridge::restamp_damping`].
//!
//! Split out of `bridge.rs` for the 700-LOC cap (W-Damping). It runs each dispatch,
//! after reconcile, so a change to the GLOBAL drag mid-play cannot leave an override
//! body wearing the world value.

use super::PhysicsBridge;

impl PhysicsBridge {
    /// Re-apply every override body's damping from its REST descriptor, resolving
    /// against the world's CURRENT global drag — the fix for the `apply_to_all`
    /// clobber (`PhysicsWorld::set_body_defaults` re-stamps the global drag onto every
    /// body, this one included, when the artist changes the world drag mid-play).
    ///
    /// It reads the REST override (`BodyRef::rest.damping`), not the live component,
    /// on purpose: the simulation is a function of `(tick, rest state)`, so a mid-play
    /// EDIT of the override is transient (takes effect on the next re-describe at rest),
    /// exactly like every other per-body config — while a global-drag change IS picked
    /// up live for a `Combine` override, which is what `apply_to_all` intends. Reading
    /// the live component would make the two disagree.
    ///
    /// Idempotent and zero-alloc: `apply_damping_override` re-stamps the same value
    /// (nothing wakes or resets), so a scene with no overrides pays only the `is_none`
    /// check per body. `self.bodies` and `self.world` are disjoint fields, so the
    /// immutable iterate + mutable stamp borrow-check.
    pub(super) fn restamp_damping(&mut self) {
        for b in self.bodies.values() {
            if let Some(d) = b.rest.damping {
                self.world.apply_damping_override(b.handle, d);
            }
        }
    }
}
