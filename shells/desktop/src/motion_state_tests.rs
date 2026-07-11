//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two M3 distribution scenes: a
//! `motion.lattice` hex packing and a `motion.voronoi` Lloyd relaxation, each a
//! `Pure` distribution animated through a `value.lfo` — through the REAL registry.

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

/// The smallest pairwise distance in the set (the packing floor).
fn min_pair(pos: &[[f32; 2]]) -> f32 {
    let mut m = f32::MAX;
    for (i, a) in pos.iter().enumerate() {
        for b in &pos[i + 1..] {
            let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
            m = m.min((dx * dx + dy * dy).sqrt());
        }
    }
    m
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // Two independent scenes → two Output sinks (the lattice and the voronoi cloud).
    assert_eq!(state.sinks.len(), 2, "two distribution scenes → two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 10 nodes: {dist, move, tint, output, lfo} × 2 scenes. The two newest nodes
    // (doc 23) — a `motion.lattice` and a `motion.voronoi`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 10);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.lattice` is alive end to end (doc 23): the `jitter` lfo displaces the hex
/// points, so a given point SWEEPS as the jitter breathes; the packing sits on the
/// left half. Falsifiable: a dead jitter would freeze every point in place.
#[test]
fn the_lattice_shimmers_under_the_jitter_lfo() {
    let state = MotionState::new();
    let lattice_sink = state.sinks[0]; // the lattice scene's Output (added first)
    let mut p0: Vec<[f32; 2]> = Vec::new();
    let mut means: Vec<f32> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0; // ~4 s = the jitter lfo's full period
        let pos = positions_at(&state, lattice_sink, t);
        assert_eq!(pos.len(), 42, "the 6×7 lattice");
        p0.push(pos[0]);
        means.push(mean_x(&pos));
    }
    // Point 0 sweeps as the jitter grows and shrinks.
    let (mut xhi, mut xlo, mut yhi, mut ylo) = (f32::MIN, f32::MAX, f32::MIN, f32::MAX);
    for &[x, y] in &p0 {
        xhi = xhi.max(x);
        xlo = xlo.min(x);
        yhi = yhi.max(y);
        ylo = ylo.min(y);
    }
    assert!(
        (xhi - xlo) + (yhi - ylo) > 0.2,
        "the jitter lfo shimmers the lattice (point 0 sweep Δ {})",
        (xhi - xlo) + (yhi - ylo)
    );
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(mean < -3.0, "the lattice sits on the left (mean x {mean})");
}

/// `motion.voronoi` is alive end to end (doc 23): the `relax` lfo plays Lloyd's
/// relaxation forward, so the cloud's minimum pairwise gap GROWS when relaxed (even
/// honeycomb) and shrinks when raw (clumped white noise); the cloud sits on the right.
/// Falsifiable: a fixed cloud's min-gap would not change over time.
#[test]
fn the_voronoi_relaxes_into_an_even_honeycomb() {
    let state = MotionState::new();
    let voronoi_sink = state.sinks[1]; // the voronoi scene's Output (added second)
    let mut gaps: Vec<f32> = Vec::new();
    let mut cx: Vec<f32> = Vec::new();
    for k in 0..=300u64 {
        let t = k as f64 / 60.0; // ~5 s = the relax lfo's full period
        let pos = positions_at(&state, voronoi_sink, t);
        assert_eq!(pos.len(), 64, "the 64-point cloud");
        gaps.push(min_pair(&pos));
        cx.push(mean_x(&pos));
    }
    let hi = gaps.iter().copied().fold(f32::MIN, f32::max);
    let lo = gaps.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        hi > lo * 1.4,
        "Lloyd relaxation opens the min gap (relaxed {hi} vs raw {lo})"
    );
    let mean = cx.iter().sum::<f32>() / cx.len() as f32;
    assert!(mean > 3.0, "the cloud sits on the right (mean x {mean})");
}

/// The default document replays bit-identically. Both distributions are deterministic
/// (hashed seed, grid-Lloyd arithmetic; the lfos are stateless playhead reads), so
/// two runs of the same document match exactly (HR-5).
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
