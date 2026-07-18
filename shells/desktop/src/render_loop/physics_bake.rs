//! **Bake — the simulated pose becomes editable keys** (ADR-0131 D11, W4).
//!
//! `ph2d_physics_ecs::bake` reads the trajectory out of the simulation. This is
//! the other half: turning those numbers into tracks, in ONE undo step, through
//! the same fit the timeline's record already uses ([`super::record_fit`]).
//!
//! # Baking flips the body to Kinematic, and that IS the bake
//!
//! Frame order decides this, not preference. The timeline apply writes
//! `Transform` and the physics readback writes it *after*, so a dynamic body
//! that has just been baked is overwritten by the solver every frame: the
//! artist clicks Bake, watches nothing change, and concludes the button is
//! broken. Two authors of one fact, and the later one wins in silence.
//!
//! So the bake hands the pose over. `BodyKind::Kinematic` is precisely the
//! state "the scene drives this, and the solver is told" — the body stays in
//! the world, still shoves what it passes, but its motion now comes from the
//! curve. That is what *runtime-truth becomes animation* means, and it is why
//! the kind landed in this wave.
//!
//! It is flipped through `apply_physics_edit`, the same door the §11 chip uses,
//! so "what does becoming Kinematic do" has one answer.
//!
//! ⚠️ **Two undo queues, and the artist sees two steps.** The keys are the
//! timeline's (one bracket, gated below); the kind is an object edit and lands
//! in the global object queue. That is the editor's existing shape — Ctrl+Z in
//! the audio editor does not undo a sprite move either — but it does mean a
//! single Ctrl+Z after a bake returns the body to Dynamic while the curves
//! remain. The toast says what happened, and the §11 chip shows Kinematic
//! selected, so the state is visible rather than inferred.

use ph2d_anim::{AnimValue, RationalTime};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PoseChannel, bake_trajectories};
use ph2d_timeline::{PropKind, TimelineState};
use std::collections::BTreeMap;

use super::record_fit::{RecSpan, simplify_recorded};
use super::timeline_bridge::default_interp;

/// The `PhysicsFieldEdit::Kind` tag for `BodyKind::Kinematic`. Read from the
/// enum rather than written as `2`, so the two cannot drift.
fn kinematic_tag() -> u8 {
    ph2d_physics_ecs::BodyKind::Kinematic.tag()
}

/// How long a bake covers when the timeline has no opinion — no keys, no clip
/// duration, no armed loop.
///
/// **Measured, not chosen.** A body dropped from the top of the default view
/// (~4 m above the floor) touches down at ~0.9 s and has stopped bouncing by
/// ~2.5 s; a six-link chain settles by ~3.5 s. 5 s covers those with room, and
/// is the number a fresh scene bakes when the artist has not said otherwise.
/// It is a DEFAULT, not a ceiling: arming a loop in the transport overrides it,
/// and a document with keys uses its own extent.
pub(crate) const DEFAULT_BAKE_SECONDS: f64 = 5.0;

/// Low-pass passes applied to the samples before the fit — **none**, and that
/// is the whole point.
///
/// `record_fit`'s own default is 8, because a gesture recorded off a mouse
/// carries hand tremor and an unsmoothed fit over-subdivides on every noise
/// bump. A simulation carries none: it is a deterministic solver, sampled at
/// exactly the tick it advanced, and every wiggle in the signal is a wiggle the
/// body really had.
///
/// Smoothing it is therefore not conservative, it is damage — and it lands
/// precisely on the moments that matter, because a physics trajectory is made
/// of IMPACTS. A bounce is a cusp; a binomial kernel over a cusp rounds its
/// apex, and the apex is the bounce.
///
/// Measured on the gate's own scene (a ball dropped onto a floor, worst error
/// against the simulated pose as a fraction of the motion's range):
///
/// | passes | worst error |
/// |---|---|
/// | **0** | **2.13%** |
/// | 1 | 2.02% |
/// | 2 | 2.48% |
/// | 4 | 3.31% |
/// | 8 (the record's) | 5.70% |
///
/// Monotonic past one pass: the number the smoothing exists to improve is the
/// one it was making worse. `1` measures a hair better than `0` and is not
/// chosen — the difference is inside the noise of a single fixture, and "the
/// solver has no tremor to remove" is a reason, while "one pass scored 0.1%
/// better on one scene" is a coincidence.
///
/// The passes are a property of the INPUT (a hand, or a solver), never of the
/// fit — which is why they are an argument to `simplify_recorded` and not a
/// constant inside it.
const BAKE_SMOOTH_PASSES: usize = 0;

/// How many seconds a Bake covers, given the document and the transport.
///
/// A bake always starts at tick 0 — the simulation is a function of the tick,
/// so there is no such thing as starting it in the middle — which is why this
/// answers a DURATION rather than a range.
///
/// In order: the armed loop's end, then the document's own extent, then
/// [`DEFAULT_BAKE_SECONDS`]. The loop first because it is the control the
/// artist already has for "this much of the timeline", and re-inventing a range
/// field beside the button would be a second way to say the same thing. The
/// document's extent next, so a scene that has been animated bakes to the
/// length it already is. The default last, for the fresh scene that has said
/// nothing — and it is shown ON the button, so it is never a surprise.
#[must_use]
pub(crate) fn bake_seconds(
    doc: &ph2d_timeline::TimelineDoc,
    playhead: &ph2d_core::Playhead,
) -> f64 {
    if let Some((_, end)) = playhead.loop_range()
        && end > 0.0
    {
        return end;
    }
    let extent = doc.end_seconds();
    if extent > 0.0 {
        return extent;
    }
    DEFAULT_BAKE_SECONDS
}

/// One track's worth of work: whose, which property, and the dense samples that
/// become its keys.
type BakedTrack = (u64, PropKind, Vec<(f64, f64)>);

/// What a bake did, for the toast and for the gates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BakeOutcome {
    /// Bodies whose motion became curves.
    pub bodies: usize,
    /// Tracks written across all of them. A body that never moved contributes
    /// none — see `BakedTrajectory::channel`.
    pub tracks: usize,
}

impl BakeOutcome {
    pub(crate) fn is_empty(&self) -> bool {
        self.tracks == 0
    }
}

/// The channel-to-track mapping. Physics has three things to say about a rigid
/// body and the timeline has a property for each, in the same unit — radians on
/// both sides, so nothing converts.
fn prop_for(channel: PoseChannel) -> PropKind {
    match channel {
        PoseChannel::X => PropKind::TranslationX,
        PoseChannel::Y => PropKind::TranslationY,
        PoseChannel::Rotation => PropKind::Rotation,
    }
}

/// How many ticks a bake of `seconds` covers, at the caller's tick length.
///
/// At least one: a range that rounds to zero ticks would write a single key and
/// call it an animation.
#[must_use]
pub(crate) fn ticks_for(seconds: f64, fixed_dt: f64) -> u64 {
    if fixed_dt <= 0.0 {
        return 1;
    }
    ((seconds / fixed_dt).round() as i64).max(1) as u64
}

/// Simulate `entities` over `seconds` and write the result as keys.
///
/// One timeline undo step for the whole thing, however many bodies, channels
/// and frames it covers — opened before the first key and committed after the
/// fit, so the dense keys and their cleanup revert together.
///
/// The bodies are flipped to `Kinematic` (module docs) via `queue`, which the
/// caller applies with the rest of the frame's ECS edits.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bake_selection(
    timeline: &mut TimelineState,
    physics: &mut PhysicsBridge,
    sim: &mut SimWorld,
    entities: &[Entity],
    seconds: f64,
    fixed_dt: f64,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> BakeOutcome {
    let ticks = ticks_for(seconds, fixed_dt);
    let trajectories = bake_trajectories(physics, sim, entities, ticks, fixed_dt);

    // Collect first, write second: a bake that opened an undo bracket and then
    // found it had nothing to say would leave an empty step on the stack.
    let mut work: Vec<BakedTrack> = Vec::new();
    for traj in &trajectories {
        for channel in PoseChannel::ALL {
            if let Some(samples) = traj.channel(channel) {
                work.push((traj.entity.to_bits(), prop_for(channel), samples));
            }
        }
    }
    if work.is_empty() {
        return BakeOutcome::default();
    }

    // The stack has to be primed before anything asks where a key lands — the
    // same first line `apply_samples` runs.
    timeline.doc.prime_stack(0.0);
    let opened = !timeline.history.is_open();
    if opened {
        timeline.history.begin(&timeline.doc);
    }

    let interp = default_interp();
    let mut record: BTreeMap<(u64, PropKind), RecSpan> = BTreeMap::new();
    for (entity_bits, prop, samples) in &work {
        for &(t, v) in samples {
            // Real seconds, NOT frame-snapped: the fit reads the shape of the
            // motion, and snapping 300 samples onto frame boundaries before
            // fitting throws away the sub-frame timing the curve is made of.
            // (This is also why the bake writes keys directly rather than
            // emitting `AddKey` intents, which snap — and which would each open
            // an undo step of their own.)
            timeline.doc.upsert_key(
                *entity_bits,
                *prop,
                RationalTime::from_seconds(t),
                AnimValue::Float(v as f32),
                interp,
            );
            record
                .entry((*entity_bits, *prop))
                .and_modify(|s| s.extend(t, v))
                .or_insert_with(|| RecSpan::seed(t, v));
        }
    }

    // Inside the bracket, exactly as the record does it: the dense keys and the
    // curve that replaces them are one step, not two.
    simplify_recorded(timeline, &record, BAKE_SMOOTH_PASSES);
    if opened {
        let doc = timeline.doc.clone();
        timeline.history.commit_if_changed(&doc);
    }

    // Hand the pose over. Same door as the §11 chip.
    for traj in &trajectories {
        super::inspector_physics::apply_physics_edit(
            sim,
            traj.entity.to_bits(),
            ph2d_editor::PhysicsFieldEdit::Kind(kinematic_tag()),
            queue,
            registry,
        );
    }

    BakeOutcome {
        bodies: trajectories.len(),
        tracks: work.len(),
    }
}

#[cfg(test)]
#[path = "physics_bake_tests.rs"]
mod tests;
