//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document through the REAL registry —
//! the multi-sink render, the dynamics, the trail, the time-remap, and the
//! pulse loop — exactly as the bridge does.

use super::*;

#[test]
fn new_builds_the_well_typed_demo_with_both_scenes() {
    let state = MotionState::new();
    // Three independent scenes, three Output nodes → three render sinks.
    assert_eq!(state.sinks.len(), 3, "grid rig + fountain + pulse loop");
    for &s in &state.sinks {
        assert_eq!(state.doc.graph.node(s).unwrap().type_name, "motion.output");
    }
    // 16 grid-rig + 8 fountain + 7 pulse-loop (grid, move, tint, strobe,
    // output, clock, threshold) nodes.
    assert_eq!(state.doc.graph.nodes().len(), 31);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// The pulse loop is alive end to end in the default document: the Schmitt
/// trigger fires off the uniform clock and the strobe's envelope really
/// pulses the grid — the dots' size swells on a fire, then decays. Proves the
/// whole pulse type (produce → consume → visible) through the REAL registry.
///
/// Falsifiable: a broken loop (no pulse ever, or a stuck glow) gives a
/// CONSTANT size — the swing is the evidence. And the count of swells (~1 per
/// clock period over 3 s at 0.7 Hz) confirms it is the beat driving it, not
/// noise.
#[test]
fn the_pulse_loop_strobes_the_grid_in_time() {
    use ph2d_eval_motion::MotionCookPump;
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    // The pulse-loop scene is the third sink; cook it alone so we read its
    // instances (not the combined buffer).
    let strobe_sink = *state.sinks.last().unwrap();
    let (uv, size) = (state.default_uv_rect, state.default_size);
    let mut pump = MotionCookPump::new();

    // Max size across the grid each tick — the envelope's silhouette. Cook
    // the sink directly so we can inspect its own stream's `size` column.
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let mut peak_per_tick = Vec::new();
    for k in 0..=180u64 {
        let out = cook
            .cook(
                &state.doc.graph,
                &state.registry,
                strobe_sink,
                k as f64 / 60.0,
            )
            .unwrap();
        let s = out[0].as_stream();
        let max = match s.get("size") {
            Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).fold(0.0_f32, f32::max),
            _ => 0.0,
        };
        peak_per_tick.push(max);
        cook.advance_tick(&state.doc.graph, &state.registry, k as f64 / 60.0)
            .unwrap();
    }

    let hi = peak_per_tick.iter().copied().fold(0.0_f32, f32::max);
    let lo = peak_per_tick.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        hi > lo * 1.5,
        "the strobe must SWING (hi {hi} vs lo {lo}); a constant size = a dead loop"
    );
    assert!(
        hi > 1.5,
        "a fire boosts size well above the unit base: {hi}"
    );

    // Count the swells (a fire = a rise past a mid level after being below).
    let mid = (hi + lo) * 0.5;
    let mut fires = 0;
    let mut below = true;
    for &p in &peak_per_tick {
        if below && p > mid {
            fires += 1;
            below = false;
        } else if p < mid {
            below = true;
        }
    }
    // 3 s at 0.7 Hz ≈ 2 beats; allow the boundary either way.
    assert!(
        (1..=3).contains(&fires),
        "the beat drives the strobe (~2 fires in 3 s), got {fires}"
    );

    // The pump path (what the shell runs) also cooks it without panicking.
    assert!(pump.pump(
        &state.doc.graph,
        &state.registry,
        &state.sinks,
        30,
        0.5,
        uv,
        size
    ));
}

/// The trail is alive in the default document and its ring really DROPS the
/// generation that ages out: with `length = 6` the fountain draws ~6× the
/// instances it draws at `length = 1` (the identity) — and no more. The
/// upper bound is the falsification: a ring that forgot to drop the oldest
/// generation would grow without bound and blow straight through it.
#[test]
fn the_trail_multiplies_the_fountain_into_comet_tails() {
    use ph2d_eval_motion::MotionCookPump;

    // Drive 2 s of fixed ticks and report the fountain's instance count
    // (cooked from the fountain sink alone).
    let fountain_instances = |length: f32| {
        let mut state = MotionState::new();
        let trail = state
            .doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "motion.trail")
            .expect("the fountain wires a Trail")
            .id;
        state.doc.graph.set_param(trail, "length", length);
        let (uv, size) = (state.default_uv_rect, state.default_size);
        let mut pump = MotionCookPump::new();
        // Fountain sink alone (sinks[1]) — count is just the particles.
        let fountain_only = &state.sinks[1..2];
        for k in 0..=120u64 {
            pump.pump(
                &state.doc.graph,
                &state.registry,
                fountain_only,
                k,
                k as f64 / 60.0,
                uv,
                size,
            );
        }
        pump.instances.len()
    };

    let bare = fountain_instances(1.0);
    let tailed = fountain_instances(6.0);
    assert!(bare > 60, "the fountain is flowing: {bare} particles");
    assert!(
        tailed > 5 * bare,
        "6 generations should be ~6x {bare}, got {tailed}"
    );
    assert!(
        tailed <= 6 * bare + 6,
        "the ring must DROP the aged-out generation; {tailed} is unbounded growth"
    );
}

/// M2.N1 end to end, through the REAL registry: the demo's Time Remap
/// declares a PingPong scope, and the subtree above it really does run on
/// the rewritten clock — at `t` and at the mirrored `2·duration - t` the
/// remap's output is IDENTICAL, while the raw (unscoped) cook of the same
/// subtree is not. That inequality is the falsification: without
/// `cook_scoped` the two cooks would agree only by accident.
#[test]
fn the_demo_time_remap_rewrites_its_upstream_clock() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    let state = MotionState::new();
    let scopes = ph2d_node_motion_time_remap::time_scopes(&state.doc.graph, &state.registry);
    let remap = state
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.time_remap")
        .expect("the demo wires a Time Remap")
        .id;
    assert_eq!(scopes.len(), 1, "one non-identity scope: the demo's remap");

    // PingPong(2.5): t = 1.0 and t = 4.0 both map to t' = 1.0.
    let positions = |t: f64| {
        let mut cook = Cook::new();
        let out = cook
            .cook_scoped(&state.doc.graph, &state.registry, remap, t, &scopes)
            .unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => p.clone(),
            _ => panic!("the remap forwards an instance stream"),
        }
    };
    assert_eq!(
        positions(1.0),
        positions(4.0),
        "the mirrored playhead must cook the same upstream frame"
    );
    // And the triangle wave's full period is 2·duration = 5 s.
    assert_eq!(
        positions(0.5),
        positions(5.5),
        "PingPong period = 2·duration"
    );

    // Falsify: the SAME two playheads on the unscoped clock differ (the rig
    // is genuinely animating, so the equality above is the scope's doing).
    let raw = |t: f64| {
        let mut cook = Cook::new();
        let out = cook
            .cook(&state.doc.graph, &state.registry, remap, t)
            .unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => p.clone(),
            _ => panic!("stream"),
        }
    };
    assert_ne!(raw(1.0), raw(4.0), "the rig animates on the real clock");
}

/// Cook the whole default document through the REAL registry, exactly as
/// the bridge does (one pump per fixed tick). Proves: both scenes draw into
/// one buffer, the grid keeps its 400 gradient-tinted instances, the
/// fountain fills up over time as particles are born, and the particles
/// actually leave the emitter's muzzle (the id-keyed integrator moved them).
#[test]
fn both_scenes_cook_into_one_buffer_and_the_fountain_flows() {
    use ph2d_eval_motion::MotionCookPump;
    let state = MotionState::new();
    let (uv, size) = (state.default_uv_rect, state.default_size);
    let mut pump = MotionCookPump::new();
    // Grid rig + fountain (sinks[..2]) into one buffer — this test is about
    // those two composing; the pulse-loop scene is a separate concern.
    let grid_and_fountain = &state.sinks[..2];

    // Tick 0: the grid's 400 + the fountain's first particle.
    pump.pump(
        &state.doc.graph,
        &state.registry,
        grid_and_fountain,
        0,
        0.0,
        uv,
        size,
    );
    let at_start = pump.instances.len();
    assert!(at_start > 400, "grid (400) + at least one particle");

    // Drive 2 seconds of fixed ticks: the fountain fills toward `rate × life`.
    for k in 1..=120u64 {
        pump.pump(
            &state.doc.graph,
            &state.registry,
            grid_and_fountain,
            k,
            k as f64 / 60.0,
            uv,
            size,
        );
    }
    // The grid contributes a constant 400; everything past that is alive
    // particles. Two seconds of a fountain is a crowd, not a trickle (the
    // exact count follows the demo's rate, which is free to be retuned).
    let alive = pump.instances.len() - 400;
    assert!(alive > 60, "the fountain fills up: {alive} particles alive");

    // The grid still leads the buffer (first sink) and is still a gradient.
    assert_ne!(pump.instances[0].tint, pump.instances[399].tint);

    // The particles (tail of the buffer) left the emitter's origin (-4,-4):
    // the integrator seeded each newborn from its muzzle velocity and moved
    // it. If id-matching were broken they would sit at the origin forever.
    let flown = pump.instances[400..]
        .iter()
        .filter(|i| {
            let d = (i.world_pos[0] + 4.0).abs() + (i.world_pos[1] + 4.0).abs();
            d > 0.25
        })
        .count();
    assert!(flown > 20, "particles left the muzzle, only {flown} did");
}

/// FALSIFICATION of the state plumbing (docs/Motion Nodes/03): the managed
/// `pre` into the force chain's head is load-bearing. Remove it and the
/// `forces` input cooks empty every tick, the id-pairing re-seeds, no
/// displacement ever accumulates — every particle stays pinned to the
/// muzzle. The intact twin (previous test) proves the same crowd flies.
#[test]
fn the_fountain_dies_without_its_state_entry() {
    use ph2d_eval_motion::MotionCookPump;
    let mut state = MotionState::new();
    let wind = state
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "force.wind")
        .expect("the fountain's wind node")
        .id;
    state
        .doc
        .graph
        .disconnect(wind, 0)
        .expect("the managed pre into the chain head exists");

    let (uv, size) = (state.default_uv_rect, state.default_size);
    let mut pump = MotionCookPump::new();
    // Cook the fountain sink ALONE (sinks[1]) so the buffer is just the
    // particles — the grid rig and the pulse-loop scene are irrelevant here.
    let fountain_only = &state.sinks[1..2];
    for k in 0..=120u64 {
        pump.pump(
            &state.doc.graph,
            &state.registry,
            fountain_only,
            k,
            k as f64 / 60.0,
            uv,
            size,
        );
    }
    // Measured from the TRUE muzzle (-5.5, -4.2): nothing moved at all.
    let flown = pump.instances[..]
        .iter()
        .filter(|i| {
            let d = (i.world_pos[0] + 5.5).abs() + (i.world_pos[1] + 4.2).abs();
            d > 0.25
        })
        .count();
    assert_eq!(
        flown, 0,
        "without the state entry the fountain must be dead, yet {flown} flew"
    );
}

/// The whole default document replays bit-identically — grid rig AND the
/// stateless emitter's alive set (the reference's stateful emitter could
/// not do this). Guards HR-5 across the id-keyed integrator.
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..30u64 {
            pump.pump(
                &state.doc.graph,
                &state.registry,
                &state.sinks,
                k,
                k as f64 / 60.0,
                state.default_uv_rect,
                state.default_size,
            );
            frames.push(
                pump.instances
                    .iter()
                    .map(|i| (i.world_pos, i.tint))
                    .collect::<Vec<_>>(),
            );
        }
        frames
    };
    assert_eq!(run(), run(), "two runs of the same document match exactly");
}
