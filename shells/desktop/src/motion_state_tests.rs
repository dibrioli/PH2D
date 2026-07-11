//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.distribute_curve` marquee
//! and a `motion.spline_wrap` ribbon, each a `Pure` layout animated through a
//! `value.lfo` — through the REAL registry.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The `P` column of one sink at playhead `t`. Both scenes are `Pure` (no `pre`
/// state), so cooking at a playhead is enough — no tick advance.
fn positions_at(state: &MotionState, sink: NodeId, t: f64) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn mean_x(pos: &[[f32; 2]]) -> f32 {
    pos.iter().map(|p| p[0]).sum::<f32>() / pos.len() as f32
}

/// The y-extent (max − min y) of a layout — grows as the ribbon wraps off the flat grid.
fn y_span(pos: &[[f32; 2]]) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in pos {
        lo = lo.min(p[1]);
        hi = hi.max(p[1]);
    }
    hi - lo
}

/// The largest over-time travel of any single element — 0 for a static layout, positive
/// when it animates. Both scenes keep a FIXED count, so per-element travel is well-defined.
fn max_travel(frames: &[Vec<[f32; 2]>]) -> f32 {
    let n = frames[0].len();
    let mut worst = 0.0f32;
    for i in 0..n {
        let (mut hi, mut lo) = ([f32::MIN; 2], [f32::MAX; 2]);
        for f in frames {
            for a in 0..2 {
                hi[a] = hi[a].max(f[i][a]);
                lo[a] = lo[a].min(f[i][a]);
            }
        }
        worst = worst.max((hi[0] - lo[0]) + (hi[1] - lo[1]));
    }
    worst
}

/// Sweep one sink over a full lfo period, returning each frame's `P`.
fn sweep(state: &MotionState, sink: NodeId, kmax: u64) -> Vec<Vec<[f32; 2]>> {
    (0..=kmax)
        .step_by(3)
        .map(|k| positions_at(state, sink, k as f64 / 60.0))
        .collect()
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 2, "two scenes -> two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 11 nodes: {distribute_curve, tint, move, output, lfo} + {grid, spline_wrap, tint,
    // move, output, lfo}. The two newest nodes (doc 28) — a `motion.distribute_curve` and
    // a `motion.spline_wrap`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 11);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.distribute_curve` is alive end to end (doc 28): the saw `offset` lfo slides
/// the 24 dots along the Bézier, so they travel over time; the scene sits on the left.
/// Falsifiable: a dead offset leaves the marquee static.
#[test]
fn the_curve_marquee_flows() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[0], 240); // the marquee (saw period 4 s)
    for f in &frames {
        assert_eq!(f.len(), 24, "the 24-dot marquee");
    }
    assert!(
        max_travel(&frames) > 1.0,
        "the saw offset flows the marquee along the path"
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean < -3.0, "the marquee sits on the left (mean x {mean})");
}

/// `motion.spline_wrap` is alive end to end (doc 28): the sine `amount` lfo blends the
/// 36-dot grid flat ↔ wrapped onto the S-curve, so it travels and its y-extent grows when
/// wrapped; the scene sits on the right. Falsifiable: a dead amount leaves the flat grid.
#[test]
fn the_grid_wraps_onto_the_spline() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[1], 300); // the ribbon (sine period 5 s)
    for f in &frames {
        assert_eq!(f.len(), 36, "the 3×12 grid ribbon");
    }
    assert!(
        max_travel(&frames) > 0.5,
        "the amount lfo wraps and unwraps the ribbon"
    );
    // The wrapped frame is much taller than the flat one (the S-curve lifts the rows).
    let tallest = frames.iter().map(|f| y_span(f)).fold(0.0f32, f32::max);
    let flattest = frames.iter().map(|f| y_span(f)).fold(f32::MAX, f32::min);
    assert!(
        tallest > flattest + 1.0,
        "wrapping stretches the ribbon in y (flat {flattest}, wrapped {tallest})"
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean > 3.0, "the ribbon sits on the right (mean x {mean})");
}

/// The default document replays bit-identically. Both scenes are deterministic (Bézier
/// arithmetic + `sqrt` arc lengths, the saw/sine lfos are stateless playhead reads), so
/// two runs match exactly (HR-5).
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..12u64 {
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
