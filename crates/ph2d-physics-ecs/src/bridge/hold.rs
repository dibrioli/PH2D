//! **The simulation is disarmed** — the transport's Physics toggle is off.
//!
//! Its own module because "off" is not "skip the dispatch", and the list of
//! things that must still happen is long enough to be worth reading in one
//! place. See [`PhysicsBridge::hold`].

use super::PhysicsBridge;
use super::space;
use crate::components::BodyKind;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics::RigidBodyHandle;

impl PhysicsBridge {
    /// **Make one body track its authored `Transform`** — the comparison and the
    /// teleport, in one place, because two callers need exactly this and the
    /// guard below is the load-bearing half of it.
    ///
    /// Returns whether it moved. The `moved_by_hand` test is not an
    /// optimisation: [`set_body_pose`](ph2d_physics::PhysicsWorld::set_body_pose)
    /// zeroes the velocity, so teleporting unconditionally would make Pause →
    /// Play restart every fall from a standstill and would re-wake every
    /// sleeping stack on every frame. The readback writes the body's pose into
    /// `Transform` exactly, so an untouched pair compares equal; a gizmo drag
    /// makes them differ.
    ///
    /// ⚠️ The authored pose is LOCAL and the body lives in WORLD. Comparing the
    /// two raw would report every child body as "moved by hand" on every frame.
    fn follow_authored_pose(&mut self, sim: &SimWorld, e: Entity, handle: RigidBodyHandle) -> bool {
        let Some(t) = space::world_transform(sim.world(), e, &mut self.chain) else {
            return false;
        };
        let (ax, ay, ar) = (t.translation.x, t.translation.y, t.rotation);
        let moved_by_hand = match self.world.body_pose(handle) {
            Some(pose) => {
                pose.translation.x != ax || pose.translation.y != ay || pose.rotation.angle() != ar
            }
            None => false,
        };
        if moved_by_hand {
            self.world.set_body_pose(handle, ax, ay, ar, true);
        }
        moved_by_hand
    }

    /// **A STATIC body is where the artist put it, clock running or not.**
    ///
    /// Runs on every dispatch (from [`prepare`](super::PhysicsBridge::prepare)),
    /// which is the difference between this and [`settle`](Self::settle) — and
    /// the reason it exists. A static body's pose has exactly **one** author:
    /// the solver never writes it (`readback` asks `solver_owns_pose`) and the
    /// scene does not push it per tick (`drive_kinematic` skips it), so there is
    /// no second writer for this to disagree with, and honouring the authored
    /// `Transform` on every dispatch cannot be a double write.
    ///
    /// ⚠️ Before this, `drive_kinematic`'s own comment declared the coverage
    /// complete — *"a wall that has been moved by hand is caught by `settle`,
    /// while paused"* — and nothing caught it with the clock running: dragging a
    /// wall during play moved the DRAWING and left the collider behind. A
    /// phantom collider, which is what the artist reported, and the class of bug
    /// this module is otherwise full of warnings about (two halves of one fact
    /// disagreeing, silently).
    ///
    /// Dynamic and kinematic are deliberately NOT here: they have owners, and a
    /// second author writing their pose mid-play is the frame-order bug W4
    /// documented (the write that runs last wins, in silence).
    pub(super) fn settle_static(&mut self, sim: &SimWorld) {
        // Collected first: `follow_authored_pose` borrows `self` mutably (the
        // `chain` scratch), and the map is being iterated. `to_settle` is a
        // retained buffer for the same zero-alloc reason every other scratch
        // here is one — empty in a scene with no static bodies moved by hand.
        self.to_settle.clear();
        for (&e, b) in self.bodies.iter() {
            if b.kind == BodyKind::Static {
                self.to_settle.push((e, b.handle));
            }
        }
        for i in 0..self.to_settle.len() {
            let (e, handle) = self.to_settle[i];
            self.follow_authored_pose(sim, e, handle);
        }
    }

    /// While paused: make every body track the authored `Transform` (and
    /// zero its velocity), so play starts from exactly where the artist
    /// left it. No stepping. Lives here beside [`hold`](Self::hold), its main
    /// caller, and moved out of `bridge.rs` for the 700-LOC cap (W-Damping).
    ///
    /// **A drag past tick 0 is transient, and that is a decision.** The
    /// simulation is a function of `(tick, authored rest state)`, so any
    /// rewind — through the ring or through the rest rebuild — reproduces the
    /// timeline as authored and drops a mid-sim nudge. Both paths discard it
    /// identically, which is why there is no cache invalidation here: a
    /// `ring.clear()` on a hand-move would be a defensive line protecting
    /// nothing, and its comment would be claiming otherwise. (Unity and Godot
    /// discard play-mode edits on stop for the same reason. Making a
    /// mid-timeline pose *stick* is authoring a keyframe, which is what W4's
    /// bake is for.)
    pub(super) fn settle(&mut self, sim: &SimWorld) {
        // EVERY body, through the same door `settle_static` uses — one answer to
        // "make this body track its authored pose", so the paused branch and the
        // always-on static branch cannot come to mean different things.
        self.to_settle.clear();
        for (&e, b) in self.bodies.iter() {
            self.to_settle.push((e, b.handle));
        }
        for i in 0..self.to_settle.len() {
            let (e, handle) = self.to_settle[i];
            self.follow_authored_pose(sim, e, handle);
        }
    }

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
        // No sim ran, so there is no live overlap to report. A stale trigger
        // state would light a sensor that nothing is inside anymore.
        self.triggers.clear();
        // ⚠️ And the SOLID half of exactly that sentence, which was missing until
        // the contact-events wave went looking for a baseline here. `contacts` is
        // read from the narrow phase, and only `step` updates that — so disarming
        // the transport's Physics toggle used to LEAVE THE CROSSES ON SCREEN,
        // describing touches in a world the artist can now pull apart by hand
        // while the marks sit there. Clearing also drops the events' baseline, so
        // re-arming re-adopts the set in silence instead of reporting the pause as
        // a collision (`bridge::contacts::discard_contact_history`).
        self.discard_contact_history();
    }
}
