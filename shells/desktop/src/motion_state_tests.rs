//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.distribute_radial`
//! array and a 180-point `motion.voronoi` mirrored by `motion.mirror`, each a `Pure`
//! layout animated through a `value.lfo` — through the REAL registry.

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

/// The largest over-time travel of any single element — 0 for a static layout,
/// positive when it animates.
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

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 2, "two scenes → two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 11 nodes: {radial, move, tint, output, lfo} + {voronoi, mirror, move, tint,
    // output, lfo}. The two newest nodes (doc 25) — a `motion.distribute_radial` and a
    // `motion.mirror`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 11);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.distribute_radial` is alive end to end (doc 25): the `spin` lfo swings the
/// ring array round, so its points travel over time; the scene sits on the left.
/// Falsifiable: a dead spin leaves the array static.
#[test]
fn the_radial_array_swings_round() {
    let state = MotionState::new();
    let radial_sink = state.sinks[0]; // the radial scene's Output (added first)
    let mut frames = Vec::new();
    let mut means = Vec::new();
    for k in 0..=150u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, radial_sink, t);
        assert_eq!(pos.len(), 48, "the 48-point radial array");
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    assert!(max_travel(&frames) > 0.3, "the spin lfo swings the array");
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(
        mean < -3.0,
        "the radial array sits on the left (mean x {mean})"
    );
}

/// `motion.mirror` + `motion.voronoi` are alive end to end (doc 25): the 180-point
/// Voronoi is doubled to 360 by the mirror and is symmetric about its centre; the
/// `relax` lfo keeps the Lloyd relaxation live (the points travel); the scene sits on
/// the right. Falsifiable: no mirror → 180 (not 360); a dead Voronoi → no travel.
#[test]
fn the_voronoi_is_mirrored_and_relaxes() {
    let state = MotionState::new();
    let mirror_sink = state.sinks[1]; // the voronoi+mirror scene's Output (added second)
    let mut frames = Vec::new();
    let mut means = Vec::new();
    // Voronoi re-runs Lloyd each frame (parallelised); a short sweep is enough.
    for k in 0..=60u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, mirror_sink, t);
        assert_eq!(pos.len(), 360, "180 Voronoi points mirrored to 360");
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    // Symmetric about its centroid: the x-offsets sum to ~0 (the mirror adds no drift).
    let last = frames.last().unwrap();
    let cx = mean_x(last);
    let skew = last.iter().map(|p| p[0] - cx).sum::<f32>();
    assert!(
        skew.abs() < 1e-2,
        "mirror-symmetric about the centre (skew {skew})"
    );
    assert!(max_travel(&frames) > 0.05, "the Voronoi relaxation is live");
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(
        mean > 3.0,
        "the mirrored honeycomb sits on the right (mean x {mean})"
    );
}

/// The default document replays bit-identically. Both scenes are deterministic
/// (parabolic trig for the radial, grid-Lloyd arithmetic for the Voronoi, arithmetic
/// mirror; the lfos are stateless playhead reads), so two runs match exactly (HR-5).
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
