//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now one small value-domain scene
//! that isolates the two newest nodes (doc 16) on a single grid: a
//! `value.instance_field` × `value.lfo` → `value.math` → `value.map_range` drives
//! a breathing Size gradient, and the SAME `value.lfo` → `pulse.compare` fires a
//! `motion.strobe` flash — through the REAL registry, exactly as the bridge does.

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
    // 10 nodes: grid, tint, drive_size, strobe, output, instance_field, lfo, math,
    // size_range, compare. The two newest value-domain nodes (doc 16) on one grid:
    // `value.math` drives a breathing Size gradient, `pulse.compare` drives the
    // strobe flash — both off the same travelling `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 10);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `value.math` is alive end to end (doc 16): it MULTIPLIES the per-dot
/// `instance_field` Ramp by the travelling `value.lfo`, so each dot's Size both
/// VARIES over time (the wave) and SPREADS across the grid (the Ramp gradient) —
/// a spatial gradient modulated in time. The strobe flashes COLOUR only
/// (`size_boost = 0`), so Size is the pure `math` signal.
///
/// Falsifiable three ways: a dead/absent `math` (or a broadcast collapse) leaves
/// every dot the SAME size forever (no spread, no motion); a frozen chain never
/// moves a dot's size (no time variation); and a bypassed `size_range` would let
/// the raw `[-1,1]` field through and blow past the `[0.25, 0.6]` span.
#[test]
fn the_math_node_modulates_the_size_gradient() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump ~3 s (180 ticks ≈ 1.5 wave periods). Record every dot's size each tick.
    let mut frames: Vec<Vec<f32>> = Vec::new();
    for k in 0..=180u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        let sizes: Vec<f32> = match out[0].as_stream().get("size") {
            Some(Column::Vec2(v)) => v.iter().map(|s| s[0]).collect(),
            _ => Vec::new(),
        };
        frames.push(sizes);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    let n = frames[0].len();
    assert!(n >= 4, "need the full grid, got {n}");

    // SPREADS across the grid: at its widest instant the size gradient spans a
    // visible range (the Ramp × wave envelope). A dead/broadcast field → 0 spread.
    let max_spread = frames
        .iter()
        .map(|row| {
            let hi = row.iter().copied().fold(f32::MIN, f32::max);
            let lo = row.iter().copied().fold(f32::MAX, f32::min);
            hi - lo
        })
        .fold(0.0_f32, f32::max);
    assert!(
        max_spread > 0.1,
        "value.math must spread the sizes (max spread {max_spread}); a dead/broadcast field = 0"
    );
    // MODULATES in time: some dot's size sweeps a real range over the pump — the
    // wave moving through `math`. A frozen chain never moves a size.
    let best_time_range = (0..n)
        .map(|dot| {
            let hi = frames.iter().map(|f| f[dot]).fold(f32::MIN, f32::max);
            let lo = frames.iter().map(|f| f[dot]).fold(f32::MAX, f32::min);
            hi - lo
        })
        .fold(0.0_f32, f32::max);
    assert!(
        best_time_range > 0.1,
        "value.math must modulate a size in time (best range {best_time_range}); frozen = 0"
    );
    // BOUNDED by size_range (0.25..0.6): a bypassed remap would leak the raw
    // [-1,1] field and overshoot.
    let (all_hi, all_lo) = frames
        .iter()
        .flatten()
        .copied()
        .fold((f32::MIN, f32::MAX), |(hi, lo), s| (hi.max(s), lo.min(s)));
    assert!(
        (0.2..=0.65).contains(&all_hi) && (0.2..=0.65).contains(&all_lo),
        "the sizes stay within size_range ([{all_lo}, {all_hi}]); a raw field would overshoot"
    );
}

/// `pulse.compare` is alive end to end (doc 16): the travelling `value.lfo` feeds
/// it, and each dot fires a pulse the moment its wave rises past the threshold
/// (Schmitt) — the `motion.strobe` turns each into a white flash. Because the wave
/// TRAVELS (a per-instance stagger), the dots cross at different times, so the
/// flashes RIPPLE across the grid instead of firing in lock-step.
///
/// `pulse.compare` + `motion.strobe` are SEQUENTIAL (armed / glow on `pre`), so we
/// pump ticks in order (cook → advance_tick), exactly as playback does. We read
/// the tint's red channel — the blue base (0.25) flashes toward white (~0.9).
///
/// Falsifiable two ways: a dead `compare` (never fires) leaves every dot at the
/// blue base forever (red never brightens); and a broadcast collapse (or a compare
/// that fired all dots together) would light the whole grid at once — no spread of
/// red across the dots at a flashing instant.
#[test]
fn the_compare_bridge_flashes_the_grid_on_the_wave_crossing() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump ~4 s (240 ticks ≈ 2 wave periods) so every dot crosses and flashes.
    let mut reds: Vec<Vec<f32>> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        let r: Vec<f32> = match out[0].as_stream().get("tint") {
            Some(Column::Vec4(v)) => v.iter().map(|c| c[0]).collect(),
            _ => Vec::new(),
        };
        reds.push(r);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(
        reds[0].len() >= 4,
        "need the full grid, got {}",
        reds[0].len()
    );

    // FLASHES: red brightens well past the 0.25 blue base at some point (a flash
    // fired) and also sits at the base at others. A dead compare → red stuck at
    // ~0.25 forever.
    let (hi, lo) = reds
        .iter()
        .flatten()
        .copied()
        .fold((f32::MIN, f32::MAX), |(hi, lo), r| (hi.max(r), lo.min(r)));
    assert!(
        hi > 0.6,
        "the strobe must flash (max red {hi}); a dead compare leaves red at the ~0.25 base"
    );
    assert!(
        lo < 0.35,
        "and it returns to the blue base between flashes (min red {lo})"
    );

    // RIPPLES (element-wise): at the instant the red is most UNEVEN across the
    // grid, the spread is real — some dots mid-flash, others still at base. A
    // broadcast collapse / all-together flash would keep every dot's red equal.
    let max_spread = reds
        .iter()
        .map(|row| {
            let hi = row.iter().copied().fold(f32::MIN, f32::max);
            let lo = row.iter().copied().fold(f32::MAX, f32::min);
            hi - lo
        })
        .fold(0.0_f32, f32::max);
    assert!(
        max_spread > 0.1,
        "the flashes ripple element-wise (max red spread {max_spread}); a broadcast flash = 0"
    );
}

/// The default document replays bit-identically. The two `pre` self-loops of the
/// scene — the compare's armed flag and the strobe's decaying `glow` — carry only
/// flag/sampled state, so two runs match exactly (HR-5; the lfo, math, size_range
/// and instance_field are stateless pure functions).
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
