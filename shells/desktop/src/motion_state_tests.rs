//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the single pulse-loop scene
//! (a `pulse.beat` metronome → `pulse.counter` → drive for X + a `value.lfo` →
//! `pulse.sample_hold` → `value.map_range` → drive for Y + a
//! `value.instance_field` → `value.map_range` → drive for Size + `motion.strobe`)
//! — through the REAL registry, exactly as the bridge does.

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
    // 15 nodes: grid, move, tint, drive_x, drive_y, drive_size, strobe, output,
    // beat, counter, lfo, sample_hold, map_range, instance_field, size_range.
    // THREE value chains — discrete beat→counter→drive_x (X, broadcast),
    // sample-and-held lfo→sample_hold→map_range→drive_y (Y), and per-element
    // instance_field→size_range→drive_size (Size) — the value domain (docs 12-14).
    assert_eq!(state.doc.graph.nodes().len(), 15);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
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
    // The biggest dot's gradient base (~0.55) times the full-glow boost (×3.2)
    // clears 1.5 — a fire boosts size well above the resting gradient.
    assert!(
        hi > 1.5,
        "a fire boosts size well above the gradient base: {hi}"
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

/// The VALUE DOMAIN is alive end to end in the default document: the same beat
/// that flashes the grid also STEPS it — `pulse.counter` reduces the beat to a
/// value and `motion.drive` routes it onto X, so the grid's X centroid sweeps in
/// discrete notches and, being a Zigzag, turns around (up then back down)
/// instead of drifting off. Proves the pulse→value→channel path (docs/Motion
/// Nodes/08, 12) through the REAL registry: a value produced by one node,
/// consumed by another, made visible.
///
/// Falsifiable: a dead counter/drive (no pulse, stuck tick, or the value never
/// reaching the channel) leaves the X centroid CONSTANT — the sweep is the
/// evidence; and a runaway (counting every tick, or a wrap that never folds)
/// would break the symmetric bound or never turn around.
#[test]
fn the_value_domain_sweeps_the_grid_in_discrete_notches() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let strobe_sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Mean X across the grid each tick. The drive shifts every dot by the same
    // `count · scale`, so the centroid = base(-1.0) + count·0.5 — a clean proxy
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
    // It SWEEPS: a dead value path would give a flat centroid.
    assert!(
        hi - lo > 1.0,
        "the drive must sweep the grid (hi {hi} vs lo {lo}); a flat X = a dead value path"
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

/// The SAMPLE-AND-HOLD chain (Y) has the `sah~` signature: a `value.lfo` feeds a
/// `pulse.sample_hold` triggered by the beat, so the smooth travelling wave is
/// FROZEN into a staircase — Y holds constant between beats and steps only on
/// them (doc 14, the doc-09 canonical combo). It stays ELEMENT-WISE: the LFO's
/// per-instance `phase_stagger` survives the sample, so dots in the same grid row
/// hold *different* levels (a broadcast/length-1 collapse would give one level
/// per row).
///
/// `pulse.sample_hold` is SEQUENTIAL (it holds state on a `pre` loop), so we pump
/// ticks in order (cook → advance_tick), exactly as playback does.
///
/// Falsifiable three ways: a smooth (un-held) chain would CHANGE Y within a beat
/// interval (no plateau); a dead chain leaves Y perfectly constant forever (no
/// step); and a broadcast collapse of the field would leave only one Y per base
/// row (3 distinct values), not the >3 the staggered hold produces.
#[test]
fn the_sample_and_hold_chain_staircases_the_grid_in_y() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let strobe_sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump ~3.3 s (200 ticks ≈ 2+ beats at 1.4 s). Record every dot's Y each tick.
    let mut frames: Vec<Vec<f32>> = Vec::new();
    for k in 0..=200u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, strobe_sink, t)
            .unwrap();
        let ys: Vec<f32> = match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
            _ => Vec::new(),
        };
        frames.push(ys);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    let n = frames[0].len();
    assert!(n >= 4, "need the full grid to see the staircase, got {n}");

    // HOLDS: the beat fires at tick 0 and ~tick 84 (period 1.4 s). Ticks 20 and 60
    // are inside the first hold interval → dot 0's Y is IDENTICAL (a plateau). A
    // smooth chain would keep moving between the beats.
    assert_eq!(
        frames[20][0], frames[60][0],
        "Y holds constant between beats (the sample-and-hold plateau)"
    );
    // STEPS: across a beat (tick 60 held-from-0 vs tick 120 held-from-~84) dot 0's
    // Y changes — the staircase steps. A frozen/dead chain never moves.
    assert_ne!(
        frames[60][0], frames[120][0],
        "Y steps to a new held level on the next beat"
    );
    // ELEMENT-WISE: at a held instant, the number of DISTINCT Y values exceeds the
    // 3 grid rows — dots that share a base row hold different sampled levels (the
    // stagger survived the hold). A broadcast/length-1 collapse → exactly 3.
    let round = |v: f32| (v * 1000.0).round() as i64;
    let mut distinct: Vec<i64> = frames[60].iter().map(|&y| round(y)).collect();
    distinct.sort_unstable();
    distinct.dedup();
    assert!(
        distinct.len() > 3,
        "the staggered hold gives >3 distinct Y (got {}); a broadcast collapse would give 3 (one per row)",
        distinct.len()
    );
}

/// The SPATIAL chain (Size) proves `value.instance_field` mints per-element
/// variation: the dots carry a size GRADIENT (small → big by index), not one
/// uniform size. This is the doc-14 capability nothing before could show — a
/// length-N field born from instance identity, not a behaviour's private stagger.
///
/// Read at a tick deep inside a hold interval, where the strobe's glow has
/// decayed to ~0, so the size is the bare gradient (not a flash). Falsifiable: a
/// dead/absent instance_field leaves every dot the same size (spread ~0); a
/// bypassed size_range would let the raw 0..1 ramp through (overshooting the span).
#[test]
fn the_instance_field_chain_gives_the_grid_a_size_gradient() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let strobe_sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump to tick 60 — inside the first hold interval, ~60 ticks after the
    // tick-0 beat, so the glow (decay 0.88) has faded to ≈0.
    let mut sizes: Vec<f32> = Vec::new();
    for k in 0..=60u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, strobe_sink, t)
            .unwrap();
        if k == 60 {
            sizes = match out[0].as_stream().get("size") {
                Some(Column::Vec2(v)) => v.iter().map(|s| s[0]).collect(),
                _ => Vec::new(),
            };
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(sizes.len() >= 4, "need the full grid, got {}", sizes.len());

    let hi = sizes.iter().copied().fold(f32::MIN, f32::max);
    let lo = sizes.iter().copied().fold(f32::MAX, f32::min);
    // A real GRADIENT: sizes span a visible range, not one uniform value.
    assert!(
        hi - lo > 0.1,
        "instance_field must spread the sizes (got [{lo}, {hi}]); a uniform size = a dead field"
    );
    // BOUNDED by size_range (0.3..0.55) with the glow ≈ 0: a bypassed range would
    // let the raw 0..1 ramp through and blow past the span.
    assert!(
        hi < 0.6,
        "the gradient is bounded by size_range at rest (max {hi}); a raw ramp would overshoot"
    );
}

/// The default document replays bit-identically. The four `pre` self-loops of
/// the scene — the beat's cycle index, the counter's monotonic tick, the
/// sample_hold's held value, and the strobe's decaying `glow` — carry only
/// integer/flag/sampled state, so two runs match exactly (HR-5; the LFO,
/// map_ranges and instance_field are stateless pure functions).
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
