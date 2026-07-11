//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.combine` (a grid + a ring
//! concatenated) and a `motion.mixer` (a grid blended into a circle), each a `Pure`
//! layout animated through a `value.lfo` — through the REAL registry.

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
    // 14 nodes: {grid, radial, combine, tint, move, output, lfo} + {grid, radial, mixer,
    // tint, move, output, lfo}. The two newest nodes (doc 30) — a `motion.combine` and a
    // `motion.mixer`, each fed by two sources (the first branch-and-merge graphs).
    assert_eq!(state.doc.graph.nodes().len(), 14);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.combine` is alive end to end (doc 30): the 100-point grid and the 40-point
/// ring concatenate into ONE 140-point stream, and the ring spins so it travels; the
/// scene sits on the left. Falsifiable: no merge → 100 (not 140); a dead ring → static.
#[test]
fn the_grid_and_ring_combine() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[0], 150); // the combine scene (added first)
    for f in &frames {
        assert_eq!(f.len(), 140, "100 grid + 40 ring concatenated");
    }
    assert!(max_travel(&frames) > 0.3, "the ring spins");
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(
        mean < -3.0,
        "the combined cloud sits on the left (mean x {mean})"
    );
}

/// `motion.mixer` is alive end to end (doc 30): the 64-point grid is blended toward the
/// 64-point circle (count = the min, 64), and the `blend` sine lfo morphs it, so the
/// points travel a lot; the scene sits on the right. Falsifiable: a dead blend → the grid
/// never moves toward the circle.
#[test]
fn the_grid_morphs_into_the_circle() {
    let state = MotionState::new();
    let frames = sweep(&state, state.sinks[1], 300); // the mixer scene (added second)
    for f in &frames {
        assert_eq!(f.len(), 64, "min(64 grid, 64 circle)");
    }
    assert!(
        max_travel(&frames) > 0.8,
        "the blend lfo morphs grid into circle"
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean > 3.0, "the morph sits on the right (mean x {mean})");
}

/// The default document replays bit-identically. Both scenes are deterministic (grid /
/// radial arithmetic, combine copying, mixer component lerp; the lfos are stateless
/// playhead reads), so two runs match exactly (HR-5).
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
