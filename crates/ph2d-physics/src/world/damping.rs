//! Per-body damping override resolution — [`super::PhysicsWorld::apply_damping_override`].
//!
//! Split out of `world.rs` for the 700-LOC cap (W-Damping). It is the single door
//! both `spawn_body` and the ECS bridge's re-stamp pass go through, so the two cannot
//! resolve one override differently.

use super::PhysicsWorld;
use rapier2d::dynamics::RigidBodyHandle;

use super::desc::DampingDesc;

impl PhysicsWorld {
    /// Resolve a per-body [`DampingDesc`] against the world defaults and stamp it on
    /// the body — **the single door** both `spawn_body` and the ECS bridge's re-stamp
    /// pass use, so the two cannot resolve one override differently.
    ///
    /// `Replace` uses the override outright; `Combine` ADDS it to the global
    /// [`BodyDefaults`](super::BodyDefaults) drag. It is stamped AFTER
    /// [`stamp_defaults`](PhysicsWorld) (which writes the global drag), so the override
    /// always wins; and re-applying it each dispatch is what keeps a change to the
    /// GLOBAL drag mid-play from leaving an override body wearing the world value
    /// ([`set_body_defaults`](PhysicsWorld::set_body_defaults) re-stamps the global
    /// onto *every* body, this one included).
    ///
    /// No-op if the handle is gone. Does not wake the body or touch its velocity, so
    /// re-stamping an unchanged value every dispatch is inert (and deterministic).
    pub fn apply_damping_override(&mut self, handle: RigidBodyHandle, d: DampingDesc) {
        let (lin, ang) = if d.replace {
            (d.linear, d.angular)
        } else {
            (
                self.body_defaults.linear_damping + d.linear,
                self.body_defaults.angular_damping + d.angular,
            )
        };
        if let Some(b) = self.bodies.get_mut(handle) {
            b.set_linear_damping(lin);
            b.set_angular_damping(ang);
        }
    }
}
