//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two M3 deformer scenes: a
//! `motion.four_point_warp` perspective keystone and a `motion.spherize` lens, each a
//! `Pure` deformer on a grid animated through a `value.lfo` — through the REAL registry.

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

/// The largest over-time travel of any single element across `frames` — 0 when the
/// deformer is dead (a static grid), positive when it animates the layout.
fn max_travel(frames: &[Vec<[f32; 2]>]) -> f32 {
    let n = frames[0].len();
    let mut worst = 0.0f32;
    for i in 0..n {
        let (mut xhi, mut xlo, mut yhi, mut ylo) = (f32::MIN, f32::MAX, f32::MIN, f32::MAX);
        for f in frames {
            xhi = xhi.max(f[i][0]);
            xlo = xlo.min(f[i][0]);
            yhi = yhi.max(f[i][1]);
            ylo = ylo.min(f[i][1]);
        }
        worst = worst.max((xhi - xlo) + (yhi - ylo));
    }
    worst
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // Two independent scenes → two Output sinks (the perspective grid and the lens).
    assert_eq!(state.sinks.len(), 2, "two deformer scenes → two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 12 nodes: {grid, deformer, move, tint, output, lfo} × 2 scenes. The two newest
    // nodes (doc 24) — a `motion.four_point_warp` and a `motion.spherize`, each driven
    // by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 12);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.four_point_warp` is alive end to end (doc 24): the `warp` lfo billows the
/// grid into perspective and flat again, so its elements travel over time; the scene
/// sits on the left. Falsifiable: a dead warp leaves the grid static (no travel).
#[test]
fn the_four_point_warp_billows_the_grid() {
    let state = MotionState::new();
    let warp_sink = state.sinks[0]; // the perspective scene's Output (added first)
    let mut frames = Vec::new();
    let mut means = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0; // ~4 s = the warp lfo's full period
        let pos = positions_at(&state, warp_sink, t);
        assert_eq!(pos.len(), 25, "the 5×5 grid");
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    assert!(
        max_travel(&frames) > 0.3,
        "the warp billows the grid over time"
    );
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(
        mean < -3.0,
        "the perspective grid sits on the left (mean x {mean})"
    );
}

/// `motion.spherize` is alive end to end (doc 24): the `amount` lfo swings pinch↔bulge,
/// so the grid's spread from its centre grows and shrinks; the scene sits on the right.
/// Falsifiable: a dead lens leaves the grid static (spread constant, no travel).
#[test]
fn the_spherize_bulges_and_pinches_the_grid() {
    let state = MotionState::new();
    let lens_sink = state.sinks[1]; // the lens scene's Output (added second)
    let mut spreads: Vec<f32> = Vec::new();
    let mut frames = Vec::new();
    let mut means = Vec::new();
    for k in 0..=180u64 {
        let t = k as f64 / 60.0; // ~3 s = the amount lfo's full period
        let pos = positions_at(&state, lens_sink, t);
        assert_eq!(pos.len(), 25, "the 5×5 grid");
        // Spread = mean distance from the centroid (grows on bulge, shrinks on pinch).
        let c = [mean_x(&pos), pos.iter().map(|p| p[1]).sum::<f32>() / 25.0];
        let spread = pos
            .iter()
            .map(|p| {
                let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
                (dx * dx + dy * dy).sqrt()
            })
            .sum::<f32>()
            / 25.0;
        spreads.push(spread);
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    let hi = spreads.iter().copied().fold(f32::MIN, f32::max);
    let lo = spreads.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        hi - lo > 0.1,
        "the lens swells and shrinks the grid (spread {lo}..{hi})"
    );
    assert!(
        max_travel(&frames) > 0.1,
        "the elements move as the lens breathes"
    );
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(
        mean > 3.0,
        "the lens grid sits on the right (mean x {mean})"
    );
}

/// The default document replays bit-identically. Both deformers are deterministic
/// arithmetic (a homography + a radial polynomial; the lfos are stateless playhead
/// reads), so two runs of the same document match exactly (HR-5).
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..20u64 {
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
