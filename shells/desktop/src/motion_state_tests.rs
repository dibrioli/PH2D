//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two grids revealed by a shared
//! `value.lfo` sweeping a `motion.cull` fraction, ordered by a `motion.sort` (radial
//! wipe left, random dissolve right) — through the REAL registry.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The `P` column of one sink at playhead `t`. Both scenes are `Pure` (no `pre`
/// state), so cooking at a playhead is enough — no tick advance. The count VARIES over
/// time (the cull reveal), so callers must not assume a fixed length.
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

/// The x-extent (max − min x) of a layout — a proxy for how spread the survivors are.
fn span(pos: &[[f32; 2]]) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in pos {
        lo = lo.min(p[0]);
        hi = hi.max(p[0]);
    }
    hi - lo
}

/// Sweep one sink over a full lfo period (5 s = 300 frames), returning each frame's `P`.
fn sweep(state: &MotionState, sink: NodeId) -> Vec<Vec<[f32; 2]>> {
    (0..=300u64)
        .step_by(3)
        .map(|k| positions_at(state, sink, k as f64 / 60.0))
        .collect()
}

/// The frames with the fewest and the most survivors (the reveal's extremes).
fn min_and_max_by_count(frames: &[Vec<[f32; 2]>]) -> (&Vec<[f32; 2]>, &Vec<[f32; 2]>) {
    let lo = frames.iter().min_by_key(|f| f.len()).unwrap();
    let hi = frames.iter().max_by_key(|f| f.len()).unwrap();
    (lo, hi)
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
    // 13 nodes: one shared {lfo} + two scenes of {grid, sort, cull, tint, move, output}.
    // The two newest nodes (doc 27) — a `motion.sort` and a `motion.cull`, the shared
    // `value.lfo` driving both culls' amount.
    assert_eq!(state.doc.graph.nodes().len(), 13);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.sort` (radial) + `motion.cull` are alive end to end (doc 27): the cull reveal
/// varies the count, and because the sort orders radially the fewest-survivor frame is a
/// TIGHT centre cluster while the most-survivor frame is the wide grid — a centre-out
/// wipe. The scene sits on the left. Falsifiable: no sort → the sparse frame would be
/// spread, not clustered; a dead cull → the count wouldn't vary.
#[test]
fn the_radial_wipe_grows_from_the_centre() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[0]); // the radial scene (added first)
    let (lo, hi) = min_and_max_by_count(&frames);
    assert!(lo.len() < hi.len(), "the cull reveal varies the count");
    // Centre-out: the sparse frame is a tight centre cluster; the full frame is wide.
    assert!(
        span(lo) < 2.0 && span(hi) > 3.5 && span(lo) < span(hi) * 0.6,
        "radial wipe grows outward (sparse span {}, full span {})",
        span(lo),
        span(hi)
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean < -3.0, "the wipe sits on the left (mean x {mean})");
}

/// `motion.sort` (random) + `motion.cull` (doc 27): the SAME cull reveal, but the random
/// order makes even the fewest-survivor frame stay spread across the grid — a dissolve,
/// not a centre wipe. The scene sits on the right. Falsifiable: a radial sort would make
/// the sparse frame cluster (small span) like the left scene.
#[test]
fn the_random_dissolve_stays_spread() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[1]); // the random scene (added second)
    let (lo, hi) = min_and_max_by_count(&frames);
    assert!(lo.len() < hi.len(), "the cull reveal varies the count");
    // A random subset spans most of the grid even when sparse (contrast the left wipe).
    assert!(
        span(lo) > 3.0,
        "random dissolve stays spread even when sparse (sparse span {})",
        span(lo)
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean > 3.0, "the dissolve sits on the right (mean x {mean})");
}

/// The default document replays bit-identically. Both scenes are deterministic (grid
/// arithmetic, a stable sort + splitmix hash for the random key, integer cull counting;
/// the lfo is a stateless playhead read), so two runs match exactly (HR-5).
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
