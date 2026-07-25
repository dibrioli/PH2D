//! **Bake — reading the simulation out as a trajectory** (ADR-0131 D11, W4).
//!
//! The sim is already a pure function of `(tick, authored rest state)`; W1.5
//! built the machinery that makes any tick reachable and proved it bit-exact.
//! So baking is not a new simulation — it is the *same* one, run over a range,
//! with the pose written down at every tick instead of only at the one the
//! playhead happens to be on.
//!
//! What this module does NOT do is decide anything about curves, keys, undo or
//! the timeline. It hands back numbers. The fit and the writing live shell-side
//! with the machinery the timeline's record already owns, because that is where
//! "what is a key" is answered — and a second answer to it, living here, is how
//! a baked curve would come to differ from a recorded one.
//!
//! # It puts the world back
//!
//! A bake runs the clock from 0 to the end of the range and then returns it to
//! wherever the artist left it. That restore is not best-effort: the sim is a
//! function of the tick, so re-dispatching the original tick reproduces it
//! exactly — the same guarantee `scrub.rs` gates. Without it, clicking Bake
//! would silently teleport the scene to the end of the range.

use ph2d_ecs::{Entity, SimWorld, Transform};

use crate::{FrozenScene, PhysicsBridge, SceneAtTick};

/// The channels a rigid body's pose actually has. Physics' own vocabulary, not
/// the timeline's: a body has a position and a heading, and knows nothing about
/// scale, opacity or what a `PropKind` is. The shell maps these onto tracks.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PoseChannel {
    X,
    Y,
    /// Radians, CCW — the same unit `Transform.rotation` and the timeline's
    /// `PropKind::Rotation` both use, so nothing converts anywhere.
    Rotation,
}

impl PoseChannel {
    /// All three, in a fixed order — a bake's output must not depend on how a
    /// map happened to iterate.
    pub const ALL: [PoseChannel; 3] = [PoseChannel::X, PoseChannel::Y, PoseChannel::Rotation];
}

/// One entity's simulated pose, tick by tick, starting at tick 0.
#[derive(Clone, Debug, PartialEq)]
pub struct BakedTrajectory {
    pub entity: Entity,
    /// `(seconds, x, y, rotation)`, one entry per tick including the rest pose
    /// at `t = 0`. The first entry matters: a curve that started at tick 1
    /// would leave the object un-driven for its first frame and it would snap.
    pub samples: Vec<(f64, f32, f32, f32)>,
}

impl BakedTrajectory {
    /// This trajectory's `channel` as `(time, value)` pairs — or `None` when
    /// the value **never changes**.
    ///
    /// ⚠️ The `None` is the load-bearing part, and it is about not destroying
    /// work rather than about tidiness. Writing a track means owning that
    /// channel: a box that falls straight down has a constant X, and baking a
    /// flat two-key X track over an entity the artist had already animated
    /// sideways would delete that animation and look like the bake was
    /// "wrong". The sim has nothing to say about a channel it never moved, so
    /// it says nothing.
    ///
    /// Constancy is **bit-exact**, deliberately, and not a tolerance: rapier
    /// writes the identical `f32` for a channel it does not touch, so exact
    /// equality is the test that lives where the domain is empty. A body that
    /// jitters by one ulp really did move, and the fit collapses that to two
    /// keys anyway.
    ///
    /// ⚠️ **This guards the UNTOUCHED channel only, and the difference matters.**
    /// A channel the solver moved at all is a channel the bake takes ownership
    /// of across the whole baked span — hand-authored keys inside that span are
    /// replaced, not merged. Measured: an X track keyed by hand `0 → 5 → 10`
    /// over 1.5 s comes back as the simulated trajectory. That is what baking
    /// MEANS (the artist asked for the simulation's answer), but the reasoning
    /// above is about protecting a channel the sim has nothing to say about,
    /// and it should not be read as a promise that hand work survives a bake of
    /// a channel the sim DOES move. Undo is one press.
    #[must_use]
    pub fn channel(&self, channel: PoseChannel) -> Option<Vec<(f64, f64)>> {
        self.channel_in(channel, f64::NEG_INFINITY, f64::INFINITY)
    }

    /// [`channel`](Self::channel), restricted to the samples in the closed
    /// window `[start, end]` seconds — the keys a partial-range bake writes.
    ///
    /// ⚠️ **The window is applied BEFORE the constancy check, and that order is
    /// load-bearing.** A body that fell during `[0, start)` and is at rest
    /// across `[start, end]` moved over the *whole* trajectory but is CONSTANT
    /// in the window; writing its motion would mean a flat two-key track laid
    /// over whatever the artist animated inside the window — the exact
    /// destruction the `None` of [`channel`](Self::channel) exists to prevent.
    /// Constancy is a property of the samples that BECOME keys, not of the whole
    /// simulation, so it is measured after the slice. (A front-to-end bake keeps
    /// the old behaviour exactly: the infinite window selects every sample.)
    ///
    /// The front `[0, start)` is still SIMULATED by the caller — the sim is a
    /// function of the tick, so there is no starting it in the middle, and the
    /// scene must be advanced through the front for any body riding a baked
    /// platform. This only decides which of those simulated samples become keys.
    #[must_use]
    pub fn channel_in(
        &self,
        channel: PoseChannel,
        start: f64,
        end: f64,
    ) -> Option<Vec<(f64, f64)>> {
        let pick = |s: &(f64, f32, f32, f32)| match channel {
            PoseChannel::X => s.1,
            PoseChannel::Y => s.2,
            PoseChannel::Rotation => s.3,
        };
        let windowed: Vec<(f64, f64)> = self
            .samples
            .iter()
            .filter(|s| s.0 >= start && s.0 <= end)
            .map(|s| (s.0, f64::from(pick(s))))
            .collect();
        let first = windowed.first()?.1;
        if windowed.iter().all(|&(_, v)| v == first) {
            return None;
        }
        Some(windowed)
    }
}

/// Run the simulation from the rest state through `ticks`, writing down the
/// pose of each entity in `entities` at every tick, then put the clock back
/// where it was.
///
/// `fixed_dt` is the tick length the caller's clock uses — the same number the
/// shell derives the physics tick from, so the times handed back are timeline
/// seconds and need no conversion downstream.
///
/// The result is ordered by entity bits, not by the order the caller listed
/// them: a bake of the same selection must produce the same thing whichever
/// way round the artist clicked.
///
/// Entities that carry no `Transform` are **skipped**, not reported empty:
/// there is no pose to bake, and an empty trajectory downstream is a body the
/// caller would count, flip and toast about for no reason. (This sentence used
/// to be here while the code did the opposite — `out` was built from every
/// target and only `record` skipped, so a Transform-less entity came back with
/// zero samples and was treated as a baked body.)
pub fn bake_trajectories(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    entities: &[Entity],
    ticks: u64,
    fixed_dt: f64,
) -> Vec<BakedTrajectory> {
    bake_trajectories_with_scene(bridge, sim, entities, ticks, fixed_dt, &mut FrozenScene)
}

/// [`bake_trajectories`], told where the scene's scene-driven bodies are at
/// each tick it simulates.
///
/// ⚠️ **A bake that does not advance the scene simulates a DIFFERENT scene.**
/// Nothing else moves a kinematic body, so through a whole bake it sits at
/// whatever pose the current frame left it in — and a body baked earlier is
/// exactly a kinematic body. Measured: a box riding a baked platform reaches
/// `x ≈ 1.05` when played, and a bake of it reports X **constant**, so no
/// horizontal track is written at all and the artist gets a Y-only curve for an
/// object that travels a metre.
pub fn bake_trajectories_with_scene(
    bridge: &mut PhysicsBridge,
    sim: &mut SimWorld,
    entities: &[Entity],
    ticks: u64,
    fixed_dt: f64,
    scene: &mut dyn SceneAtTick,
) -> Vec<BakedTrajectory> {
    let resume_at = bridge.last_stepped();

    let mut targets: Vec<Entity> = entities.to_vec();
    targets.sort_unstable_by_key(|e| e.to_bits());
    targets.dedup();

    let mut out: Vec<BakedTrajectory> = targets
        .iter()
        .filter(|&&e| sim.world().get::<Transform>(e).is_some())
        .map(|&entity| BakedTrajectory {
            entity,
            samples: Vec::with_capacity(ticks as usize + 1),
        })
        .collect();

    // Tick 0 is the rest state, and it is a SAMPLE, not a warm-up: it is where
    // the artist placed the object, and the first key of the baked curve.
    scene.put(sim, 0);
    bridge.dispatch_with_scene(sim, false, 0, scene);
    record(sim, &mut out, 0.0);

    for tick in 1..=ticks {
        // One dispatch per tick, and that is not a style choice: asking for the
        // last tick directly would step the world all the way there and read
        // back ONCE, giving a trajectory that is a single pose.
        bridge.dispatch_with_scene(sim, true, tick, scene);
        record(sim, &mut out, tick as f64 * fixed_dt);
    }

    // Hand the scene back. The sim is a function of the tick, so this restores
    // the exact state the artist was looking at — the property `scrub.rs`
    // already gates, borrowed rather than re-proved.
    scene.put(sim, resume_at);
    bridge.dispatch_with_scene(sim, false, resume_at, scene);
    out
}

fn record(sim: &SimWorld, out: &mut [BakedTrajectory], seconds: f64) {
    let world = sim.world();
    for traj in out.iter_mut() {
        if let Some(t) = world.get::<Transform>(traj.entity) {
            traj.samples
                .push((seconds, t.translation.x, t.translation.y, t.rotation));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_channel_that_never_moves_is_not_a_track() {
        let traj = BakedTrajectory {
            entity: Entity::from_bits(1),
            samples: vec![
                (0.0, 1.0, 5.0, 0.0),
                (0.1, 1.0, 4.0, 0.0),
                (0.2, 1.0, 3.0, 0.0),
            ],
        };
        assert!(
            traj.channel(PoseChannel::X).is_none(),
            "a constant X became a flat track, which would overwrite whatever \
             the artist had animated there"
        );
        assert!(traj.channel(PoseChannel::Rotation).is_none());
        let y = traj.channel(PoseChannel::Y).expect("Y moved");
        assert_eq!(y, vec![(0.0, 5.0), (0.1, 4.0), (0.2, 3.0)]);
    }

    #[test]
    fn a_window_hides_the_front_and_keeps_the_windowed_keys() {
        // Y falls over [0.0, 0.2] and is at rest over [0.2, 0.4].
        let traj = BakedTrajectory {
            entity: Entity::from_bits(1),
            samples: vec![
                (0.0, 0.0, 5.0, 0.0),
                (0.1, 0.0, 4.0, 0.0),
                (0.2, 0.0, 3.0, 0.0),
                (0.3, 0.0, 3.0, 0.0),
                (0.4, 0.0, 3.0, 0.0),
            ],
        };
        // The whole trajectory moved, so the un-windowed channel is written.
        assert!(traj.channel(PoseChannel::Y).is_some());
        // But over [0.2, 0.4] Y is CONSTANT: writing a flat track there would
        // overwrite whatever the artist animated inside the window.
        assert!(
            traj.channel_in(PoseChannel::Y, 0.2, 0.4).is_none(),
            "constancy must be judged on the windowed samples, not the whole sim"
        );
        // And a window over the moving part keeps exactly those keys.
        let front = traj
            .channel_in(PoseChannel::Y, 0.0, 0.2)
            .expect("front moved");
        assert_eq!(front, vec![(0.0, 5.0), (0.1, 4.0), (0.2, 3.0)]);
    }

    #[test]
    fn an_empty_trajectory_has_no_channels() {
        let traj = BakedTrajectory {
            entity: Entity::from_bits(1),
            samples: Vec::new(),
        };
        for ch in PoseChannel::ALL {
            assert!(traj.channel(ch).is_none(), "{ch:?} came out of nothing");
        }
    }
}
