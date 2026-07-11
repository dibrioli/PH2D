//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now one small value-domain scene
//! that isolates the two newest nodes (doc 17) on a single grid: a `value.switch`
//! routes the Size between an `instance_field` Ramp and Random as a `value.lfo`
//! `select` cycles, and a `pulse.on_change` fires a `motion.strobe` flash on each
//! flip — through the REAL registry, exactly as the bridge does.

use super::*;

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // One focused scene → one Output node → one render sink.
    assert_eq!(state.sinks.len(), 1, "the value demo is the sole scene");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 11 nodes: grid, tint, drive_size, strobe, output, ramp, rand, lfo, switch,
    // size_range, on_change. The two newest value-domain nodes (doc 17) on one
    // grid: `value.switch` routes the Size pattern, `pulse.on_change` fires the
    // strobe on each flip.
    assert_eq!(state.doc.graph.nodes().len(), 11);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `value.switch` is alive end to end (doc 17): the `select` lfo cycles `0 ↔ 1`,
/// so the Size is routed from the ordered Ramp (`in0`) for one half of the wave
/// and the Random scatter (`in1`) for the other. We read the grid at a tick in
/// each phase and prove the routing really flips.
///
/// Falsifiable: a switch stuck on `in0` leaves the grid ORDERED (monotonic by
/// index) at BOTH ticks, so the two frames match; a switch stuck on `in1` is never
/// the monotonic Ramp. The Ramp frame being monotonic AND differing from the
/// Random frame pins that the selector actually routed both inputs.
#[test]
fn the_switch_routes_the_size_between_two_patterns() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Capture the grid's sizes at tick 30 (t=0.5 s, the wave peak → select≈1 →
    // Random) and tick 90 (t=1.5 s, the trough → select≈0 → Ramp).
    let sizes_at = |cook: &mut ph2d_nodegraph::cook::Cook, ticks: &[u64]| -> Vec<Vec<f32>> {
        let mut caught = vec![Vec::new(); ticks.len()];
        for k in 0..=*ticks.iter().max().unwrap() {
            let t = k as f64 / 60.0;
            let out = cook
                .cook(&state.doc.graph, &state.registry, sink, t)
                .unwrap();
            if let Some(pos) = ticks.iter().position(|&w| w == k) {
                if let Some(Column::Vec2(v)) = out[0].as_stream().get("size") {
                    caught[pos] = v.iter().map(|s| s[0]).collect();
                }
            }
            cook.advance_tick(&state.doc.graph, &state.registry, t)
                .unwrap();
        }
        caught
    };
    let caught = sizes_at(&mut cook, &[30, 90]);
    let (random_frame, ramp_frame) = (&caught[0], &caught[1]);
    assert!(
        ramp_frame.len() >= 4,
        "need the full grid, got {}",
        ramp_frame.len()
    );

    // The Ramp phase is a MONOTONIC gradient by index (in0 routed).
    let monotonic = ramp_frame.windows(2).all(|w| w[0] <= w[1] + 1.0e-4);
    assert!(
        monotonic,
        "the trough tick routes the ordered Ramp (monotonic by index): {ramp_frame:?}"
    );
    // And the Random phase DIFFERS from it — the switch really flipped inputs. A
    // switch stuck on in0 would give the identical (monotonic) frame both times.
    assert_ne!(
        random_frame, ramp_frame,
        "the peak tick routes the Random scatter — a stuck switch would match the Ramp"
    );
}

/// `pulse.on_change` is alive end to end (doc 17): it watches the switched value
/// and fires a pulse the tick the pattern FLIPS (Ramp↔Random). The `motion.strobe`
/// turns that into a white flash, so the grid lights up ON each flip and is dark
/// between — the discrete "something changed" trigger.
///
/// `pulse.on_change` + `motion.strobe` are SEQUENTIAL (prev value / glow on `pre`),
/// so we pump ticks in order (cook → advance_tick). We read the tint red — the blue
/// base (0.25) flashes toward white (~0.9) on a flip.
///
/// Falsifiable two ways: a dead `on_change` (never fires) leaves red at the base
/// forever (no flash); and a chattering detector (firing every tick, not only on
/// the step) would keep red pinned high — the count of flash events (~2 over the
/// two flips in 3 s) proves it fires on the CHANGE, not continuously.
#[test]
fn the_on_change_flashes_the_grid_on_each_pattern_flip() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump ~3 s (180 ticks). The 2 s lfo flips the switch at t≈1 s and t≈2 s → two
    // flashes. Track the brightest dot's red each tick.
    let mut max_red = Vec::new();
    for k in 0..=180u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        let hi = match out[0].as_stream().get("tint") {
            Some(Column::Vec4(v)) => v.iter().map(|c| c[0]).fold(0.0_f32, f32::max),
            _ => 0.0,
        };
        max_red.push(hi);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }

    let hi = max_red.iter().copied().fold(0.0_f32, f32::max);
    let lo = max_red.iter().copied().fold(f32::MAX, f32::min);
    // FLASHES: red brightens well past the 0.25 base (a flip fired the strobe) and
    // returns to base between. A dead on_change → red stuck at ~0.25.
    assert!(
        hi > 0.6,
        "on_change must flash the grid (max red {hi}); a dead detector stays at ~0.25"
    );
    assert!(lo < 0.35, "and it goes dark between flips (min red {lo})");
    // FIRES ON THE STEP, NOT CONTINUOUSLY: count the flashes (red rising past a mid
    // level). The two lfo flips in 3 s give ~2 — a chattering detector would give
    // many more (or never fall back to base).
    let mid = (hi + lo) * 0.5;
    let mut flashes = 0;
    let mut below = true;
    for &r in &max_red {
        if below && r > mid {
            flashes += 1;
            below = false;
        } else if r < mid {
            below = true;
        }
    }
    assert!(
        (1..=3).contains(&flashes),
        "on_change fires ON the flip (~2 in 3 s), not every tick: got {flashes}"
    );
}

/// The default document replays bit-identically. The two `pre` self-loops of the
/// scene — the on_change's previous value and the strobe's decaying `glow` — carry
/// only sampled/flag state, so two runs match exactly (HR-5; the lfo, switch,
/// size_range and the two instance_fields are stateless pure functions).
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
                    .map(|i| (i.world_pos, i.tint, i.size))
                    .collect::<Vec<_>>(),
            );
        }
        frames
    };
    assert_eq!(run(), run(), "two runs of the same document match exactly");
}
