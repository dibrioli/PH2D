//! **The simulation is disarmed** — the transport's Physics toggle is off.
//!
//! Its own module because "off" is not "skip the dispatch", and the list of
//! things that must still happen is long enough to be worth reading in one
//! place. See [`PhysicsBridge::hold`].

use super::PhysicsBridge;
use ph2d_ecs::SimWorld;

impl PhysicsBridge {
    /// **The clock is running and the simulation is not** — the transport's
    /// Physics toggle is off (`TimelineFlags::simulate_physics`).
    ///
    /// Not a synonym for "skip the dispatch", and the difference is the reason
    /// this is a method instead of an `if` at the call site:
    ///
    /// - The world still **reconciles**, so a body added while disarmed exists,
    ///   keeps the rest pose it was authored at, and draws its collider outline.
    ///   Arming is then instant rather than building a world mid-play.
    /// - The world still **settles** onto the authored `Transform`, so the rapier
    ///   bodies track whatever the scene does — an object posed by hand, or a
    ///   baked one following its curves. Arming therefore resumes from what is
    ///   on screen, which is the only resumption an artist can predict.
    /// - `last_stepped` **follows the target**, and skipping that is the trap:
    ///   play ten seconds disarmed, then arm, and the bridge would owe 600 ticks
    ///   and simulate every one of them inside a single frame — a freeze that
    ///   ends with the scene somewhere nobody asked for.
    /// - The ring is **dropped**, because each checkpoint in it describes a run
    ///   that is now over. Seeding a later scrub from one would answer with a
    ///   state from before the artist disarmed and moved things by hand.
    ///
    /// ⚠️ **Nothing here writes `Transform`.** That is exactly what the toggle
    /// being off promises — physics contributes no motion — and it is why the
    /// paused fixed-point rule keeps holding: `settle` reads the scene and writes
    /// the rapier world, while [`readback`](Self::readback), which goes the other
    /// way, is reachable only from the stepping paths.
    ///
    /// Scrubbing BACKWARDS across a stretch that was never simulated replays it
    /// as though it had been. No better answer exists to give: the trajectory for
    /// those ticks was never computed, because the ticks never ran.
    pub fn hold(&mut self, sim: &mut SimWorld, target: u64) {
        self.prepare(sim);
        self.settle(sim);
        self.ring.clear();
        self.last_stepped = target;
    }
}
