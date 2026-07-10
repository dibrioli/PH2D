//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the single pulse-loop scene
//! (a `pulse.beat` metronome → `motion.step` + `motion.strobe`) — through the
//! REAL registry, exactly as the bridge does.

use super::*;

#[test]
fn new_builds_the_well_typed_pulse_document() {
    let state = MotionState::new();
    // One focused scene → one Output node → one render sink.
    assert_eq!(state.sinks.len(), 1, "the pulse loop is the sole scene");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 7 nodes: grid, move, tint, step, strobe, output, beat. No oscillator and
    // no threshold: the beat IS the source (doc 09 killed the channel clock).
    assert_eq!(state.doc.graph.nodes().len(), 7);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// The pulse loop is alive end to end in the default document: the metronome
/// fires off the playhead and the strobe's envelope really pulses the grid —
/// the dots' size swells on a beat, then decays. Proves the whole pulse type
/// (source → consume → visible) through the REAL registry, with **no transform
/// channel involved in the clocking** — the doc 09 point: there is no
/// `channel` param anywhere in the loop to retune and silently kill it.
///
/// Falsifiable: a broken loop (no pulse ever, or a stuck glow) gives a
/// CONSTANT size — the swing is the evidence. And the count of swells (the
/// start beat + one per 1.4 s period over 3 s) confirms it is the beat driving
/// it, not noise.
#[test]
fn the_pulse_loop_strobes_the_grid_in_time() {
    use ph2d_eval_motion::MotionCookPump;
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    // The pulse-loop is the sole sink; cook it directly so we can inspect its
    // own stream's `size` column.
    let strobe_sink = *state.sinks.last().unwrap();
    let (uv, size) = (state.default_uv_rect, state.default_size);
    let mut pump = MotionCookPump::new();

    // Max size across the grid each tick — the envelope's silhouette.
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
    // Beats at t = 0 (the start beat), 1.4 and 2.8 → 3 swells in 3 s; allow the
    // boundary either way.
    assert!(
        (2..=4).contains(&fires),
        "the beat drives the strobe (~3 fires in 3 s), got {fires}"
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

/// The STEP is alive end to end in the default document: the same beat that
/// flashes the grid also STEPS it — the grid's X centroid sweeps in discrete
/// notches and, being a Zigzag, turns around (up then back down) instead of
/// drifting off. Proves the pulse→persistent-value bridge (docs/Motion
/// Nodes/08, renamed by 09) through the REAL registry: an event driving a
/// PERSISTENT value, the inverse of the strobe's decay.
///
/// Falsifiable: a dead step (no pulse, or a stuck tick) leaves the X centroid
/// CONSTANT — the sweep is the evidence; and a runaway (counting every tick, or
/// a wrap that never folds) would break the symmetric bound or never turn
/// around.
#[test]
fn the_step_sweeps_the_grid_in_discrete_notches() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let strobe_sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Mean X across the grid each tick. The step shifts every dot by the same
    // `count · step`, so the centroid = base(-1.0) + count·0.5 — a clean proxy
    // for the count. Long enough (8 s ≈ 6 beats at 1.4 s) to reach the zigzag
    // top (count 4 → +1.0) and fold back down.
    let mut centroid = Vec::new();
    for k in 0..=480u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, strobe_sink, t)
            .unwrap();
        let s = out[0].as_stream();
        let mean = match s.get("P") {
            Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).sum::<f32>() / v.len().max(1) as f32,
            _ => 0.0,
        };
        centroid.push(mean);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }

    let hi = centroid.iter().copied().fold(f32::MIN, f32::max);
    let lo = centroid.iter().copied().fold(f32::MAX, f32::min);
    // It SWEEPS: a dead step would give a flat centroid.
    assert!(
        hi - lo > 1.0,
        "the step must sweep the grid (hi {hi} vs lo {lo}); a flat X = a dead step"
    );
    // It stays within the symmetric Zigzag reach about the -1.0 pre-offset centre
    // (0..4 counts · 0.5 = 0..2.0 → centroid in [-1.0, 1.0]); a runaway that counts
    // every tick, or a non-folding wrap, would blow this bound.
    assert!(
        (-1.2..=1.2).contains(&hi) && (-1.2..=1.2).contains(&lo),
        "the sweep is bounded to the zigzag reach: [{lo}, {hi}]"
    );
    // It TURNS AROUND: the peak is interior (it went up then came back down), the
    // Zigzag fold — not a monotonic drift and not a plateau.
    let peak_at = centroid
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap()
        .0;
    assert!(
        peak_at > 5 && peak_at < centroid.len() - 5,
        "the zigzag climbs to an interior peak then folds back (peak at tick {peak_at})"
    );
}

/// The default document replays bit-identically. The three `pre` self-loops of
/// the pulse loop — the beat's cycle index, the step's monotonic tick, and the
/// strobe's decaying `glow` — carry only integer/flag state, so two runs match
/// exactly (HR-5).
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
