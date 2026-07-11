//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the single pulse-loop scene
//! (a `pulse.beat` metronome → `pulse.counter` → `motion.drive` for X + a
//! `value.lfo` → `value.map_range` → `motion.drive` for Y + `motion.strobe`) —
//! through the REAL registry, exactly as the bridge does.

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
    // 11 nodes: grid, move, tint, drive_x, drive_y, strobe, output, beat,
    // counter, lfo, map_range. TWO value chains — the discrete beat→counter→
    // drive_x (X, broadcast) and the continuous lfo→map_range→drive_y (Y,
    // element-wise) — instead of the bundled motion.step (the value domain, doc 12).
    assert_eq!(state.doc.graph.nodes().len(), 11);
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

/// The SECOND value chain (the continuous one) is alive and ELEMENT-WISE: a
/// `value.lfo` emits a length-N field, `value.map_range` reshapes it, and a
/// `motion.drive` routes it onto Y — so every dot bobs, and (unlike the X
/// broadcast) the per-instance `phase_stagger` makes the dots bob *out of phase*
/// with each other: a travelling wave. This is the doc-12 win the discrete chain
/// can't show — the length-N element-wise path next to X's length-1 broadcast.
///
/// The LFO is stateless (a pure function of the playhead), so we cook at a sweep
/// of times without advancing tick state and read Y straight off the sink.
///
/// Falsifiable three ways: a dead LFO/map_range/drive_y chain leaves Y FLAT
/// (min per-dot range ~0); a broken `map_range` lets the raw amplitude through so
/// the bob exceeds its bounded span; and a BROADCAST value (the stagger lost, or
/// a length-1 field) moves every dot in lock-step, collapsing the at-one-instant
/// spread to zero.
#[test]
fn the_continuous_lfo_chain_ripples_the_grid_in_y_element_wise() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let strobe_sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // One full LFO period (2 s ≈ 120 ticks). Record every dot's Y each tick.
    let mut frames: Vec<Vec<f32>> = Vec::new();
    for k in 0..=120u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, strobe_sink, t)
            .unwrap();
        let ys: Vec<f32> = match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
            _ => Vec::new(),
        };
        frames.push(ys);
    }
    let n = frames[0].len();
    assert!(
        n >= 2,
        "need several dots to see a travelling wave, got {n}"
    );

    // Per-dot Y range over the period, and per-dot time-mean (≈ the constant base
    // grid Y, since a full period of the wave averages out).
    let mut ranges = Vec::with_capacity(n);
    let mut means = Vec::with_capacity(n);
    for i in 0..n {
        let col: Vec<f32> = frames.iter().map(|f| f[i]).collect();
        let hi = col.iter().copied().fold(f32::MIN, f32::max);
        let lo = col.iter().copied().fold(f32::MAX, f32::min);
        ranges.push(hi - lo);
        means.push(col.iter().sum::<f32>() / col.len() as f32);
    }

    // ALIVE: every dot bobs (the map_range [-0.5,0.5] span → ~1.0 of travel). A
    // dead value chain leaves Y flat.
    let min_range = ranges.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        min_range > 0.5,
        "every dot must bob in Y (min range {min_range}); a flat Y = a dead LFO chain"
    );
    // BOUNDED by the glue: map_range clamps the raw [-1,1] into [-0.5,0.5], so no
    // dot travels much past 1.0. A bypassed map_range would let the amplitude through.
    let max_range = ranges.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        max_range < 1.2,
        "the bob is bounded by map_range (max range {max_range}); a raw LFO would overshoot"
    );
    // ELEMENT-WISE: at one mid-swing instant, subtract each dot's own base Y —
    // the remaining displacements are NOT all equal (the travelling wave). A
    // length-1 broadcast would move every dot by the identical amount → spread 0.
    let mid = 30usize; // t = 0.5 s, a quarter period in
    let disp: Vec<f32> = (0..n).map(|i| frames[mid][i] - means[i]).collect();
    let dhi = disp.iter().copied().fold(f32::MIN, f32::max);
    let dlo = disp.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        dhi - dlo > 0.2,
        "the per-instance stagger makes dots differ at one instant (spread {}); \
         a broadcast value would move them in lock-step",
        dhi - dlo
    );
}

/// The default document replays bit-identically. The three `pre` self-loops of
/// the pulse loop — the beat's cycle index, the counter's monotonic tick, and the
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
