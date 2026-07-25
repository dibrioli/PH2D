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

/// **The bake writes ONE KEY PER TICK — no fit.** (Enio: "sem simplificação;
/// busque o padrão ouro".)
///
/// The exact reversal of what this gate used to assert. A fit RESAMPLES the
/// motion, and a resampled bounce is a rounded one; the gold standard for
/// reproducing a discrete simulation is to not resample it at all — every tick
/// the solver advanced becomes a key, linear between. Sampled at 60 fps that is
/// byte-exact to the sim (the reproduction gates), and the cost is density,
/// which is the trade "sem simplificação" asks for: fidelity over editability.
///
/// Mutation-tested: re-introducing any fit (`simplify_recorded`) collapses a
/// moving channel from ~90 keys to single digits, and this goes red.
#[test]
fn the_bake_writes_one_key_per_tick() {
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
        // One key per simulated tick (tick 0..=ticks is `ticks + 1` samples). A
        // fit would collapse a moving channel to a handful; the un-fitted signal
        // is dense.
        assert!(
            n >= ticks,
            "{prop:?} holds only {n} keys for {ticks} ticks — a fit ran and ate \
             the per-tick fidelity the artist asked for"
        );
    }
}

/// **Every channel of one body keys at the SAME times.**
///
/// Column alignment: the animator grabs a column in the dope sheet and re-times
/// the whole object. With one key per tick this is free — every channel is
/// sampled on the one tick grid, so they land on identical times by
/// construction (the fit's three-pass column merge, which the record needs to
/// align hand-keyed extrema, is not in this path at all). It is still worth a
/// gate: a bug that offset one channel's key times would break re-timing, and
/// nothing else here would notice.
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

    // ⚠️ No crowding assertion here. A bake samples every channel on ONE tick
    // grid, so the times are identical by construction — there is nothing to
    // merge and nothing a merge could get wrong. `record_fit`'s near-coincident
    // column merge (`COLUMN_MERGE_S`) exists to align hand-keyed extrema that
    // land a few milliseconds apart; it is not in this path, and is the record's
    // to prove.
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

/// **The baked curve reproduces the sim EXACTLY at every tick, and never
/// overshoots between them** — the gold standard "sem simplificação" delivers
/// (Enio: "funciona mas é imperfeito" → "busque a perfeição").
///
/// The old fit RESAMPLED the motion, and both directions were wrong: at the
/// record's 1% tolerance it dropped a small bounce and rounded its cusp (2.53%
/// error), and tightening it traded that for a 6.6% OVERSHOOT under a Time
/// Remap (a smooth-tangent cubic packed around a sharp bounce). One key per
/// tick has neither, and this pins both halves:
///
///  - **exact at ticks** — the playhead lands on the tick times, so the curve
///    returns the simulated pose verbatim; worst error is at the noise floor,
///    orders of magnitude under the fit's 1-3%;
///  - **no overshoot between ticks** — linear interpolation cannot leave the
///    band its two endpoints define, so a mid-segment sample can never bulge
///    past the bracketing ticks (the exact failure the tight fit had).
///
/// The bounce is the point: the fixture CONTAINS one (asserted), and it is
/// precisely the feature a fit rounds. Mutation-tested: re-introducing any fit
/// (`simplify_recorded`) rounds the bounce (breaks the at-tick half) and its
/// cubic overshoots (breaks the between-tick half).
#[test]
fn the_bake_reproduces_the_sim_exactly_with_no_overshoot() {
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

    let y_at = |ts: &mut TimelineState, sim: &mut SimWorld, t: f64| -> f32 {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut ts.doc, t);
        sim.world().get::<Transform>(ball).unwrap().translation.y
    };

    // Half 1 — exact at every tick.
    let mut worst_tick = 0.0f32;
    let mut worst_t = 0.0;
    for &(t, sim_y) in &truth {
        let e = (y_at(&mut timeline, &mut sim, t) - sim_y).abs();
        if e > worst_tick {
            worst_tick = e;
            worst_t = t;
        }
    }
    assert!(
        worst_tick / span < 1e-3,
        "the baked curve is not exact at the ticks: it strays {:.4}% of range \
         (worst at t={worst_t:.3}s) — one key per tick must reproduce the \
         simulated pose verbatim",
        worst_tick / span * 100.0
    );

    // Half 2 — between ticks it never leaves the band the two bracketing ticks
    // define. Linear cannot; a fit's cubic can, and that is the overshoot.
    let mut worst_over = 0.0f32;
    let mut over_t = 0.0;
    for w in truth.windows(2) {
        let (t0, y0) = w[0];
        let (t1, y1) = w[1];
        let (band_lo, band_hi) = (y0.min(y1), y0.max(y1));
        let mid = (t0 + t1) * 0.5;
        let ym = y_at(&mut timeline, &mut sim, mid);
        let over = (ym - band_hi).max(band_lo - ym).max(0.0);
        if over > worst_over {
            worst_over = over;
            over_t = mid;
        }
    }
    assert!(
        worst_over / span < 1e-3,
        "the baked curve overshoots between ticks by {:.4}% of range (worst at \
         t={over_t:.3}s) — a mid-segment sample left the band its two bracketing \
         keys define, which linear interpolation cannot do and a fit's cubic can",
        worst_over / span * 100.0
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
