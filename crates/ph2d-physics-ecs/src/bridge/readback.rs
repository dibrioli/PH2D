//! **The solver's poses flow back into `Transform`** — the read half of the
//! per-frame seam. Its own module for the 700-LOC cap (the gold-standard joint
//! anchor added the pivot sync to `bridge.rs`); see [`PhysicsBridge::readback`].

use super::PhysicsBridge;
use super::space;
use ph2d_ecs::SimWorld;

impl PhysicsBridge {
    /// Read each dynamic body's pose back into its entity's `Transform`
    /// (meters, radians CCW, Y-up — no conversion; ADR-0131 D4). Static
    /// bodies never move and kinematic ones are DRIVEN by that same
    /// `Transform` ([`Self::drive_kinematic`]), so both are skipped — asked
    /// through [`BodyKind::solver_owns_pose`], the one door, because a body
    /// that both stages claimed would have its pose written twice a tick.
    ///
    /// [`BodyKind::solver_owns_pose`]: crate::BodyKind::solver_owns_pose
    ///
    /// ⚠️ The solver answers in WORLD space and `Transform` is LOCAL, so the
    /// pose goes back through [`space::write_world_pose`]. Assigning it raw
    /// works for a root (where the two coincide) and is wrong for every child:
    /// the renderer composes the parent onto it again, so the body simulates
    /// in one place and draws in another. A parent that cannot be inverted
    /// (scaled to zero) leaves the `Transform` untouched rather than storing
    /// a non-finite pose that would poison the whole subtree.
    pub(super) fn readback(&mut self, sim: &mut SimWorld) {
        let world = sim.world_mut();
        for (&e, b) in self.bodies.iter() {
            if !b.kind.solver_owns_pose() {
                continue;
            }
            if let Some(pose) = self.world.body_pose(b.handle) {
                space::write_world_pose(
                    world,
                    e,
                    pose.translation.x,
                    pose.translation.y,
                    pose.rotation.angle(),
                    &mut self.chain,
                );
            }
        }
    }
}
