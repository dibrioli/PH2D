//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two M4 simulation scenes: a
//! `motion.verlet_rope` whip and a `motion.boids` flock, each a sequential sim on
//! the `pre` self-loop — through the REAL registry, exactly as the bridge does.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The emitted `P` column of one sink at playhead `t` (no tick advance).
fn positions_at(state: &MotionState, cook: &mut Cook, sink: NodeId, t: f64) -> Vec<[f32; 2]> {
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn centroid(pos: &[[f32; 2]]) -> [f32; 2] {
    let n = pos.len() as f32;
    let s = pos
        .iter()
        .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
    [s[0] / n, s[1] / n]
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // Two independent scenes → two Output sinks (the whip and the flock).
    assert_eq!(state.sinks.len(), 2, "two sim scenes → two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 8 nodes: {rope, tint, output, lfo} × 2 scenes. The two newest nodes (doc 21)
    // — a `motion.verlet_rope` and a `motion.boids`, each herded by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 8);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.verlet_rope` is alive end to end (doc 21): gravity hangs the strand and
/// the `anchor_x` lfo whips it, so the free tail FALLS below the anchor and SWEEPS
/// sideways as the pin slides. Falsifiable: a dead rope (no gravity, no sim) leaves
/// the tail flat at the anchor's height and still.
#[test]
fn the_verlet_rope_whips_from_the_moving_anchor() {
    let state = MotionState::new();
    let rope_sink = state.sinks[0]; // the rope scene's Output (added first)
    let mut cook = Cook::new();
    let mut tail: Vec<[f32; 2]> = Vec::new();
    for k in 0..=200u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, &mut cook, rope_sink, t);
        if let Some(last) = pos.last() {
            tail.push(*last);
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(tail.len() > 180, "pumped the rope");
    let (mut xhi, mut xlo, mut ylo) = (f32::MIN, f32::MAX, f32::MAX);
    for &[x, y] in &tail {
        xhi = xhi.max(x);
        xlo = xlo.min(x);
        ylo = ylo.min(y);
    }
    assert!(
        ylo < -1.0,
        "gravity hangs the free tail below the anchor (min y {ylo})"
    );
    assert!(
        xhi - xlo > 0.5,
        "the sliding anchor whips the tail sideways (Δx {})",
        xhi - xlo
    );
}

/// `motion.boids` is alive end to end (doc 21): the flock is homed on the right by
/// its seek pull, and the `target_x` lfo slides the target, so the whole swarm
/// WHEELS to chase it — the centroid stays on the right and tracks the sliding
/// target over time. Falsifiable: a flock that ignored the target would neither
/// gather on the right nor track its motion.
#[test]
fn the_boids_flock_seeks_the_moving_target() {
    let state = MotionState::new();
    let boids_sink = state.sinks[1]; // the flock scene's Output (added second)
    let mut cook = Cook::new();
    let mut cx: Vec<f32> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, &mut cook, boids_sink, t);
        assert!(!pos.is_empty(), "the flock has agents");
        let c = centroid(&pos);
        assert!(c[0].is_finite() && c[1].is_finite(), "bounded flock");
        cx.push(c[0]);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    let mean = cx.iter().sum::<f32>() / cx.len() as f32;
    let (hi, lo) = (
        cx.iter().copied().fold(f32::MIN, f32::max),
        cx.iter().copied().fold(f32::MAX, f32::min),
    );
    assert!(
        mean > 2.0,
        "the flock is homed on the right (mean centroid x {mean})"
    );
    assert!(
        hi - lo > 1.0,
        "the centroid tracks the sliding target (x range {})",
        hi - lo
    );
}

/// The default document replays bit-identically. Both scenes are deterministic
/// (Verlet + constraint relaxation, boids arithmetic + a hashed seed; the lfos are
/// stateless playhead reads), so two runs of the same document match exactly
/// (HR-5). This drives the FULL sequential pump (state on the `pre` loop), so it
/// also proves the sims step reproducibly, not just the pure nodes.
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..40u64 {
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
                    .map(|i| (i.world_pos, i.tint, i.basis))
                    .collect::<Vec<_>>(),
            );
        }
        frames
    };
    assert_eq!(run(), run(), "two runs of the same document match exactly");
}
