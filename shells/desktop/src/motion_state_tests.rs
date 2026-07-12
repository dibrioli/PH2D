//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.make_point` Lissajous and
//! a `motion.luminance` recolour, each a `Pure` layout through the REAL registry.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The `P` column of one sink at playhead `t`.
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

/// The `tint` column (linear RGBA) of one sink at playhead `t`.
fn tints_at(state: &MotionState, sink: NodeId, t: f64) -> Vec<[f32; 4]> {
    let mut cook = Cook::new();
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn mean_x(pos: &[[f32; 2]]) -> f32 {
    pos.iter().map(|p| p[0]).sum::<f32>() / pos.len() as f32
}

/// The largest over-time travel of any single element (both scenes keep a fixed count).
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

fn distinct_colours(tints: &[[f32; 4]]) -> usize {
    let mut seen: Vec<[f32; 4]> = Vec::new();
    for c in tints {
        if !seen.iter().any(|s| s == c) {
            seen.push(*c);
        }
    }
    seen.len()
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
    // 13 nodes: {grid, lfoX, lfoY, make_point, tint, move, output} + {grid, color_ramp,
    // luminance, color_ramp, move, output}. The two newest nodes (doc 31) — a
    // `motion.make_point` and a `motion.luminance`.
    assert_eq!(state.doc.graph.nodes().len(), 13);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.make_point` is alive end to end (doc 31): the two staggered LFOs plot a
/// 64-point Lissajous that stays within the amplitude box and the playhead animates; the
/// scene sits on the left. Falsifiable: no value fields → all points at one spot; a dead
/// playhead → static.
#[test]
fn the_lissajous_is_plotted_and_animates() {
    let state = MotionState::new();
    let sink = state.sinks[0]; // the make_point scene (added first)
    let mut frames = Vec::new();
    for k in 0..=150u64 {
        let pos = positions_at(&state, sink, k as f64 / 60.0);
        assert_eq!(pos.len(), 64, "the 64-point Lissajous");
        frames.push(pos);
    }
    // The plotted points spread over the curve (not collapsed to a point).
    let last = frames.last().unwrap();
    let (mut xlo, mut xhi) = (f32::MAX, f32::MIN);
    for p in last {
        xlo = xlo.min(p[0]);
        xhi = xhi.max(p[0]);
    }
    assert!(xhi - xlo > 3.0, "the curve spans (x extent {})", xhi - xlo);
    assert!(
        max_travel(&frames) > 0.3,
        "the playhead animates the Lissajous"
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(
        mean < -3.0,
        "the Lissajous sits on the left (mean x {mean})"
    );
}

/// `motion.luminance` is alive end to end (doc 31): the rainbow's brightness drives a
/// Heat ramp, so the 100 dots take a spread of Heat colours (not one). The scene sits on
/// the right. Falsifiable: a dead luminance (all v=0) → every dot is the Heat ramp's
/// start colour (one distinct tint).
#[test]
fn the_grid_is_recoloured_by_luminance() {
    let state = MotionState::new();
    let sink = state.sinks[1]; // the luminance scene (added second)
    let pos = positions_at(&state, sink, 0.0);
    assert_eq!(pos.len(), 100, "the 10×10 grid");
    let tints = tints_at(&state, sink, 0.0);
    assert!(
        distinct_colours(&tints) > 5,
        "luminance drives a spread of Heat colours ({} distinct)",
        distinct_colours(&tints)
    );
    let mean = mean_x(&pos);
    assert!(
        mean > 3.0,
        "the recoloured grid sits on the right (mean x {mean})"
    );
}

/// The default document replays bit-identically. Both scenes are deterministic (grid /
/// lfo / make_point / luminance arithmetic; the lfos are stateless playhead reads), so
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
