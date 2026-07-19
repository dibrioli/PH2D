//! **W4 — what the baked CURVE is**, the other half of `physics_bake_tests`.
//!
//! Split under the shell's 600-LOC cap, and the seam is a real one: the sibling
//! file asks whether the bake reproduces the SIMULATION (fidelity, determinism,
//! the undo step, the hand-over), and this one asks whether what it wrote is an
//! ANIMATION an artist can work with — few keys instead of one per frame,
//! aligned columns, a range the artist chose, and keys on the entity's own
//! clock. A bake can be perfectly faithful and still useless, which is exactly
//! the gap the first of these gates was written to close.

use ph2d_ecs::Transform;
use ph2d_ecs::scene::EditorCommandQueue;
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_timeline::{PropKind, TimelineState};

use super::tests::{BAKE_SECONDS, DT, baked, registry, scene, simulated};
use super::{DEFAULT_BAKE_SECONDS, bake_selection, ticks_for};

/// **The bake produces a CURVE, not one key per frame.**
///
/// The wave's actual deliverable, and the one every other gate here is blind
/// to: with the fit removed, the tracks hold one key per tick, which reproduces
/// the simulation *exactly* (so the fidelity gate is greener than ever), costs
/// one undo step (so that gate passes), and is completely useless — nobody
/// edits ninety keyframes per second. "Runtime truth becomes animation" means
/// animation somebody can grab.
///
/// The bar is generous on purpose: the point is orders of magnitude, not a
/// pinned count that a tolerance tweak would have to chase. A bounce over 1.5 s
/// is a handful of keys; 90 is the un-fitted signal.
#[test]
fn the_bake_writes_a_curve_not_a_key_per_frame() {
    let (timeline, _sim, ball, outcome) = baked();
    let ticks = ticks_for(BAKE_SECONDS, DT) as usize;
    assert!(
        outcome.tracks >= 2,
        "the fixture baked {} tracks",
        outcome.tracks
    );

    for prop in PropKind::ALL {
        let Some(track) = timeline
            .doc
            .binding_for(ball.to_bits(), prop)
            .and_then(|b| timeline.doc.active_clip().track(b.target))
        else {
            continue;
        };
        let n = track.keys().len();
        assert!(
            n > 1,
            "{prop:?} collapsed to {n} key(s) — the fit ate the motion"
        );
        assert!(
            n * 4 < ticks,
            "{prop:?} holds {n} keys for {ticks} ticks — that is the dense \
             per-frame recording, not a curve. The fit did not run."
        );
    }
}

/// **Every channel of one body keys at the SAME times.**
///
/// Column alignment: the animator grabs a column in the dope sheet and re-times
/// the whole object. It is the reason `simplify_recorded` does three passes
/// instead of fitting each track independently, and it was ungated — a mutation
/// that stopped merging near-coincident times into shared columns left every
/// gate green.
#[test]
fn a_bodys_channels_key_on_shared_columns() {
    let (timeline, _sim, ball, _) = baked();
    let mut columns: Option<Vec<f64>> = None;
    let mut checked = 0;
    for prop in PropKind::ALL {
        let Some(track) = timeline
            .doc
            .binding_for(ball.to_bits(), prop)
            .and_then(|b| timeline.doc.active_clip().track(b.target))
        else {
            continue;
        };
        let times: Vec<f64> = track.keys().iter().map(|k| k.t.to_seconds()).collect();
        match &columns {
            None => columns = Some(times),
            Some(first) => {
                assert_eq!(
                    &times, first,
                    "{prop:?} keys at different times than the body's first \
                     channel — the dope sheet shows ragged keys instead of \
                     columns, and re-timing the object means dragging each \
                     channel separately"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "only one channel was baked, so this gate compared nothing"
    );

    // ⚠️ No crowding assertion here, and that is a MEASUREMENT rather than an
    // omission. `record_fit`'s near-coincident-time merge (`COLUMN_MERGE_S`) is
    // inert for a bake: a hand crosses each axis's extremum a few milliseconds
    // apart, which is what the merge exists for, but a bake samples every
    // channel on ONE tick grid, so the extrema land on exactly the same times.
    // Measured on this fixture — columns are `[0.0, 0.583, 1.5]` with the merge
    // and `[0.0, 0.583, 1.5]` without it. A gate on the merge would be a gate
    // that cannot fail here; the constant belongs to the record and is the
    // record's to prove.
    let _ = columns;
}

/// **The range comes from the loop, then the document, then the default.**
///
/// Each step of the chain, because the ones that never fire are the ones that
/// rot: with no gate, ignoring the armed loop entirely left everything green
/// and the artist's chosen range silently unused.
#[test]
fn the_bake_range_prefers_the_armed_loop_then_the_document() {
    use super::bake_seconds;
    use ph2d_core::Playhead;

    let mut doc = ph2d_timeline::TimelineDoc::default();
    let mut playhead = Playhead::new(DT);

    assert_eq!(
        bake_seconds(&doc, &playhead),
        DEFAULT_BAKE_SECONDS,
        "a fresh scene says nothing, so the measured default is what a bake \
         covers"
    );

    // A document with content bakes to its own length.
    doc.upsert_key(
        1,
        PropKind::TranslationX,
        ph2d_anim::RationalTime::from_seconds(3.0),
        ph2d_anim::AnimValue::Float(1.0),
        ph2d_anim::Interp::Linear,
    );
    assert!(
        (bake_seconds(&doc, &playhead) - 3.0).abs() < 1e-6,
        "an animated document baked {} s instead of its own 3 s extent",
        bake_seconds(&doc, &playhead)
    );

    // An armed loop outranks it: it is the control the artist reached for.
    playhead.set_loop(0.5, 2.0);
    assert!(
        (bake_seconds(&doc, &playhead) - 2.0).abs() < 1e-6,
        "the armed loop was ignored ({} s) — the artist's own range control \
         does nothing and there is no other way to ask for one",
        bake_seconds(&doc, &playhead)
    );
}

/// **The baked curve reproduces the motion under a Time Remap too.**
///
/// The entity's clock, not the playhead's. `apply` samples every track at
/// `remapped_time`, so a bake that stamped raw playhead seconds writes the
/// curve at times it will never be read at — and the further the remap is from
/// identity, the further the object is from where the simulation put it.
///
/// Born RED at **1.618 m** of error on a ~2 m drop, under a half-speed remap.
/// The fix is not arithmetic here: it is going through `key_time`, the same
/// door the record and the manual K already use
/// (`feedback_derived_coordinate_seed_must_match_sample`).
#[test]
fn a_bake_lands_on_the_clock_of_its_entity_not_the_playhead() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};

    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();

    // Half speed: playhead 2.0s -> source 1.0s.
    for (t, v) in [(0.0, 0.0), (2.0, 1.0)] {
        timeline.doc.upsert_key(
            ball.to_bits(),
            PropKind::TimeRemap,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    assert!(
        (ph2d_timeline::remapped_time(&timeline.doc, ball.to_bits(), 1.0) - 0.5).abs() < 1e-6,
        "the fixture's remap is not half speed, so it is not measuring the bug"
    );

    let ticks = ticks_for(BAKE_SECONDS, DT);
    let truth = simulated(ticks);
    let outcome = bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[ball],
        BAKE_SECONDS,
        DT,
        &queue,
        &reg,
    );
    assert!(!outcome.is_empty(), "the fixture baked nothing");

    let span = {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for s in &truth {
            lo = lo.min(s.2);
            hi = hi.max(s.2);
        }
        (hi - lo).max(1e-3)
    };
    let mut worst = 0.0f32;
    for s in &truth {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, s.0);
        let y = sim.world().get::<Transform>(ball).unwrap().translation.y;
        worst = worst.max((y - s.2).abs());
    }
    assert!(
        worst / span < 0.05,
        "under a half-speed Time Remap the baked curve misses the simulated \
         pose by {worst:.4} m ({:.1}% of the {span:.3} m range) — the keys were \
         written on a clock the apply does not read them on",
        worst / span * 100.0
    );
}

/// **A baked take PLAYS with the simulation disarmed.** (ADR-0131, the
/// transport's Physics toggle.)
///
/// This is the composition the toggle exists for, and the one smoke scene 7
/// asks the artist to perform: bake, untick Physics, press Play, and watch the
/// same motion happen. It works because a bake turns runtime truth into
/// ANIMATION — and animation is precisely what plays when the solver is off.
///
/// The loop below is the frame loop's own order: the timeline writes `Transform`
/// first, physics is dispatched second. Disarmed, that second call must leave
/// the pose alone, so the curve is the sole author of what the artist sees. A
/// `hold` that read back, or that stepped, would fight the curve here — which is
/// the shape of the bug the whole W4 hand-over exists to avoid, seen from the
/// other side.
#[test]
fn a_baked_take_plays_with_the_simulation_disarmed() {
    let (mut timeline, mut sim, ball, _outcome) = baked();
    let mut bridge = PhysicsBridge::new();
    let mut playhead = ph2d_core::Playhead::new(DT);
    playhead.play();

    let mut ys = Vec::new();
    for _ in 0..ticks_for(BAKE_SECONDS, DT) {
        playhead.advance();
        let t = playhead.time();
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, t);
        let driven = sim.world().get::<Transform>(ball).unwrap().translation.y;

        crate::render_loop::physics_bridge::dispatch(
            &mut bridge,
            &mut sim,
            &playhead,
            DT,
            &mut timeline.doc,
            false,
        );
        let after = sim.world().get::<Transform>(ball).unwrap().translation.y;

        assert_eq!(
            after, driven,
            "the disarmed dispatch overwrote the pose the baked curve had just \
             written — with physics off the curve is the only author"
        );
        ys.push(after);
    }

    assert_eq!(
        bridge.steps_taken(),
        0,
        "the solver ran while the transport's Physics toggle was off"
    );
    let (lo, hi) = ys
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), &y| (l.min(y), h.max(y)));
    assert!(
        hi - lo > 0.5,
        "the baked take did not play with physics off: the ball covered only \
         {:.4} m. A bake that only moves while the solver is running has not \
         become animation.",
        hi - lo
    );
}
