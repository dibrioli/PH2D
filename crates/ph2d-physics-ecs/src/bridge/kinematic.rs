//! **Driving the bodies the SCENE owns** — the bridge half of
//! `BodyKind::Kinematic`.
//!
//! A dynamic body's pose is an output of the solver and flows out through
//! `readback`. A kinematic body's is an INPUT, and it flows the other way:
//! whatever authored it (a timeline curve, a gizmo drag) writes `Transform`,
//! and this aims rapier at it. That reversal is the whole kind, and it is why
//! the two stages ask [`BodyKind::solver_owns_pose`] from opposite sides — a
//! body both claimed would have its pose written twice a tick.
//!
//! Split out of `bridge.rs` under the 700-LOC cap, and a unit in its own right:
//! everything here answers *"where does the scene say this body is?"*, which is
//! the question `SceneAtTick` exists to ask and the replay had to learn to ask.

use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics::PhysicsWorld;

use crate::components::BodyKind;

use super::PhysicsBridge;

/// **Where the scene puts its scene-driven bodies at a given tick.**
///
/// A `Kinematic` body's pose is an INPUT to the simulation, supplied by
/// whatever authored it — a timeline curve, usually. That makes the world stop
/// being a function of `(tick, authored rest state)` alone, and three places
/// assumed it still was: the owed-tick loop, the rewind replay, and the bake.
/// A body frozen at its rest pose through a replay is a platform that is not
/// where it was, and every dynamic body that touched it lands somewhere else.
///
/// So the bridge asks. The implementor is the half that owns the curves (the
/// shell), and physics learns nothing about timelines: the question is only
/// *"put the scene into the state it has at tick T"*.
///
/// Returning `false` means "I have nothing to say about that tick", and the
/// bridge falls back to spreading the frame's move across the ticks it owes —
/// which is the honest reconstruction for a body being dragged by hand, whose
/// intermediate poses were never recorded anywhere.
pub trait SceneAtTick {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool;
}

/// A scene that does not change with the tick — the answer for a caller with no
/// curves to consult (every headless gate, and the C9 harness).
pub struct FrozenScene;

impl SceneAtTick for FrozenScene {
    fn put(&mut self, _sim: &mut SimWorld, _tick: u64) -> bool {
        false
    }
}

impl PhysicsBridge {
    pub(super) fn capture_kinematic_start(&mut self) {
        self.kin_start.clear();
        for (&e, b) in self.bodies.iter() {
            if b.kind != BodyKind::Kinematic {
                continue;
            }
            if let Some(p) = self.world.body_pose(b.handle) {
                self.kin_start
                    .push((e, [p.translation.x, p.translation.y, p.rotation.angle()]));
            }
        }
    }

    pub(super) fn drive_kinematic(&mut self, sim: &SimWorld, f: f32) {
        let world = sim.world();
        for (&e, b) in self.bodies.iter() {
            // Three kinds, three answers, and `Static` is in NEITHER stage:
            // the solver does not own its pose (so `readback` skips it) and
            // the scene does not push it per tick either — a wall that has
            // been moved by hand is caught by `settle`, while paused, where
            // zeroing its velocity is the correct thing rather than a bug.
            if b.kind != BodyKind::Kinematic {
                continue;
            }
            if let Some(t) = world.get::<Transform>(e) {
                let target = [t.translation.x, t.translation.y, t.rotation];
                let start = self
                    .kin_start
                    .iter()
                    .find(|(k, _)| *k == e)
                    .map_or(target, |(_, p)| *p);
                let at = PhysicsWorld::slice_pose(start, target, f);
                self.world
                    .set_next_kinematic_pose(b.handle, at[0], at[1], at[2]);
            }
        }
    }
}
