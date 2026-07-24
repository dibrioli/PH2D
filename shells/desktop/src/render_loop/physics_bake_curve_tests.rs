//! **W4 — what the baked CURVE is**, the other half of `physics_bake_tests`.
//!
//! Split under the shell's 600-LOC cap, and the seam is a real one: the sibling
//! file asks whether the bake reproduces the SIMULATION (fidelity, determinism,
//! the undo step, the hand-over), and this one asks whether what it wrote is an
//! ANIMATION an artist can work with — few keys instead of one per frame,
//! aligned columns, a range the artist chose, and keys on the entity's own
//! clock. A bake can be perfectly faithful and still useless, which is exactly
//! the gap the first of these gates was written to close.

use ph2d_ecs::scene::EditorCommandQueue;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_timeline::{PropKind, TimelineState};

use super::tests::{BAKE_SECONDS, DT, baked, registry, scene, simulated};
use super::{BakeChannels, DEFAULT_BAKE_SECONDS, bake_selection, ticks_for};

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

/// **The window comes from the loop, then the document, then the default —
/// and the loop's START is honoured, not dropped.**
///
/// Each step of the chain, because the ones that never fire are the ones that
/// rot: with no gate, ignoring the armed loop entirely left everything green
/// and the artist's chosen range silently unused. The start is the W-BakeRange
/// half — an armed `[0.5s, 2s]` loop used to bake `[0, 2s]`, throwing away the
/// start the artist had set.
#[test]
fn the_bake_window_prefers_the_armed_loop_and_honours_its_start() {
    use super::bake_range;
    use ph2d_core::Playhead;

    let mut doc = ph2d_timeline::TimelineDoc::default();
    let mut playhead = Playhead::new(DT);

    assert_eq!(
        bake_range(&doc, &playhead),
        (0.0, DEFAULT_BAKE_SECONDS),
        "a fresh scene says nothing, so a bake covers the whole measured default"
    );

    // A document with content bakes to its own length, from the top.
    doc.upsert_key(
        1,
        PropKind::TranslationX,
        ph2d_anim::RationalTime::from_seconds(3.0),
        ph2d_anim::AnimValue::Float(1.0),
        ph2d_anim::Interp::Linear,
    );
    let (s, e) = bake_range(&doc, &playhead);
    assert!(
        s == 0.0 && (e - 3.0).abs() < 1e-6,
        "an animated document baked {s}..{e} instead of 0..3 s"
    );

    // An armed loop outranks it: it is the control the artist reached for, and
    // BOTH of its ends are honoured. The old resolver returned only `2.0` here,
    // silently dropping the 0.5 s start.
    playhead.set_loop(0.5, 2.0);
    let (s, e) = bake_range(&doc, &playhead);
    assert!(
        (s - 0.5).abs() < 1e-6 && (e - 2.0).abs() < 1e-6,
        "the armed loop's window was not honoured: baked {s}..{e}, expected \
         0.5..2.0 — the artist's own range start does nothing"
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
        0.0,
        BAKE_SECONDS,
        DT,
        BakeChannels::All,
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

/// A FLAT floor + a ball dropped from height, so its motion has a clean bounce
/// feature (a small peak after the first floor contact) that a loose fit drops.
fn bouncing_ball(secs: f64) -> (Vec<(f64, f32)>, TimelineState, SimWorld, Entity) {
    use ph2d_core::Vec2;
    use ph2d_ecs::Name;
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

    let build = || -> (SimWorld, Entity) {
        let mut sim = SimWorld::new();
        sim.world_mut().spawn((
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 8.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ));
        let ball = sim
            .world_mut()
            .spawn((
                Name::new("Ball"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    restitution: 0.75,
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, 4.0)),
            ))
            .id();
        (sim, ball)
    };

    let ticks = ticks_for(secs, DT);
    // Truth: the dense per-tick Y.
    let (mut tsim, tball) = build();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut tsim, false, 0);
    let mut truth: Vec<(f64, f32)> = Vec::new();
    for tick in 0..=ticks {
        if tick > 0 {
            bridge.dispatch(&mut tsim, true, tick);
        }
        truth.push((
            tick as f64 * DT,
            tsim.world().get::<Transform>(tball).unwrap().translation.y,
        ));
    }

    // Bake through the product path.
    let (mut sim, ball) = build();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();
    bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[ball],
        0.0,
        secs,
        DT,
        BakeChannels::All,
        &queue,
        &reg,
    );
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("apply");
    (truth, timeline, sim, ball)
}

/// **The bake's tight tolerance tracks the simulation, bounces and all** — the
/// answer to Enio's "funciona mas é imperfeito" (W-BakeRange follow-up).
///
/// The fit's tolerance was inherited from the RECORD (1%), calibrated for a noisy
/// hand. A solver has no noise, so 1% is too loose: it dropped a small bounce and
/// rounded it (measured 2.53% error). `BAKE_SIMPLIFY_REL` is 10× tighter, which
/// captures the bounce — and does NOT explode the key count, because the fit
/// anchors at extrema (7 keys, not 181). This is the exact twin of
/// `BAKE_SMOOTH_PASSES = 0`: a parameter of the input, not the fit.
///
/// The oracle is the product path (`apply_from_doc`) and the bar is 1% of the
/// motion's range across the WHOLE curve. Mutation-tested: baking at
/// `REC_SIMPLIFY_REL` (the record's 1%) instead of `BAKE_SIMPLIFY_REL` rounds the
/// bounce to 2.53% and this goes red.
#[test]
fn the_bake_tracks_the_sim_closely_including_bounces() {
    let (truth, mut timeline, mut sim, ball) = bouncing_ball(3.0);

    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for &(_, y) in &truth {
        lo = lo.min(y);
        hi = hi.max(y);
    }
    let span = (hi - lo).max(1e-3);

    // The fixture must actually CONTAIN a bounce, or the gate proves nothing: a
    // local Y maximum strictly above the floor, after the first contact.
    let has_bounce = (1..truth.len() - 1).any(|i| {
        let y = truth[i].1;
        y > truth[i - 1].1 && y >= truth[i + 1].1 && y > lo + span * 0.02
    });
    assert!(
        has_bounce,
        "the fixture has no bounce to capture — the gate would be vacuous"
    );

    let mut worst = 0.0f32;
    let mut worst_t = 0.0;
    for &(t, sim_y) in &truth {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, t);
        let fit_y = sim.world().get::<Transform>(ball).unwrap().translation.y;
        let e = (fit_y - sim_y).abs();
        if e > worst {
            worst = e;
            worst_t = t;
        }
    }
    assert!(
        worst / span < 0.01,
        "the baked curve strays {:.2}% of range from the sim (worst at t={worst_t:.3}s) \
         — the fit tolerance is too loose and is rounding the motion; at the record's 1% \
         this bounce measured 2.53%",
        worst / span * 100.0
    );
}

/// Bake the fixture over the window `[start, end]` and hand back the doc.
fn baked_over(start: f64, end: f64) -> (TimelineState, SimWorld, Entity) {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();
    bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[ball],
        start,
        end,
        DT,
        BakeChannels::All,
        &queue,
        &reg,
    );
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("apply");
    (timeline, sim, ball)
}

/// **A partial-range bake writes keys only inside its window; the front is
/// discarded** (W-BakeRange). Baking `[0.5s, 1.5s]` must produce a curve that
/// agrees with the full `[0, 1.5s]` bake INSIDE the window and holds the start
/// pose BEFORE it — the front `[0, 0.5s)` was simulated (the sim is a function
/// of the tick) but its samples are not keys.
///
/// An APPEARANCE oracle, read through the frame loop's own `apply_from_doc`:
///  - inside the window (t = 1.0s) the two curves must AGREE — same simulation,
///    same samples;
///  - before the window (t = 0.0s) they must DIFFER — the full bake has the ball
///    at its rest pose (y ≈ 2.0), the partial bake holds the fallen start pose,
///    because no key describes a time before the window and the curve
///    extrapolates its first key backward.
///
/// Mutation-tested: `channel_in(ch, start, end)` → `channel(ch)` (ignore the
/// window) makes the partial bake write the front too, so at t=0 it matches the
/// full bake and the DIFFER assertion goes red.
#[test]
fn a_partial_range_bake_writes_only_inside_its_window() {
    const START: f64 = 0.5;
    let (mut full, mut fsim, fball) = baked_over(0.0, BAKE_SECONDS);
    let (mut part, mut psim, pball) = baked_over(START, BAKE_SECONDS);

    let y_at = |ts: &mut TimelineState, sim: &mut SimWorld, e: Entity, t: f64| -> f32 {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut ts.doc, t);
        sim.world().get::<Transform>(e).unwrap().translation.y
    };

    // Inside the window the two bakes read the same simulation.
    let inside = 1.0;
    let (fy_in, py_in) = (
        y_at(&mut full, &mut fsim, fball, inside),
        y_at(&mut part, &mut psim, pball, inside),
    );
    assert!(
        (fy_in - py_in).abs() < 0.02,
        "inside the window the partial bake ({py_in:.4}) disagreed with the \
         full bake ({fy_in:.4}) — same sim, same samples, they must match"
    );

    // Before the window the front is gone: the full bake sits at the rest pose,
    // the partial bake holds the fallen start pose. They must differ, and the
    // full one must still be near the authored rest of 2.0.
    let (fy0, py0) = (
        y_at(&mut full, &mut fsim, fball, 0.0),
        y_at(&mut part, &mut psim, pball, 0.0),
    );
    assert!(
        (fy0 - 2.0).abs() < 0.1,
        "the full bake should describe the rest pose at t=0 (y≈2.0), got {fy0:.4}"
    );
    assert!(
        (fy0 - py0).abs() > 0.5,
        "the partial bake wrote the front: at t=0 it reads {py0:.4}, the same as \
         the full bake ({fy0:.4}) — the window `[{START}s, {BAKE_SECONDS}s]` was \
         not honoured"
    );
}
