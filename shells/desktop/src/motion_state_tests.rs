//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.kaleidoscope` mandala
//! and a `motion.collide` circle-packing, each a `Pure` layout animated through a
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
    assert_eq!(state.sinks.len(), 2, "two scenes -> two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 12 nodes: {fibonacci, kaleidoscope, move, tint, output, lfo} + {grid, collide,
    // move, tint, output, lfo}. The two newest nodes (doc 26) — a `motion.kaleidoscope`
    // and a `motion.collide`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 12);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.kaleidoscope` is alive end to end (doc 26): the 6-seed Fibonacci spiral is
/// folded into 8 mirrored slices (48 dots) and the `spin` lfo turns the mandala, so its
/// points travel over time; the scene sits on the left. Falsifiable: a dead spin leaves
/// the mandala static; no fold → 6 (not 48).
#[test]
fn the_mandala_spins() {
    let state = MotionState::new();
    let mandala_sink = state.sinks[0]; // the mandala scene's Output (added first)
    let mut frames = Vec::new();
    let mut means = Vec::new();
    for k in 0..=150u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, mandala_sink, t);
        assert_eq!(pos.len(), 48, "6 spiral seeds × 8 kaleidoscope slices");
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    assert!(max_travel(&frames) > 0.3, "the spin lfo turns the mandala");
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(mean < -3.0, "the mandala sits on the left (mean x {mean})");
}

/// `motion.collide` is alive end to end (doc 26): the 8×8 overlapping grid is pushed
/// apart into a packing (64 dots) and the `spread` lfo breathes the radius, so the
/// points travel; the scene sits on the right. Falsifiable: a dead collide leaves the
/// tight grid (no travel, closer than the disc diameter).
#[test]
fn the_grid_packs_apart_and_breathes() {
    let state = MotionState::new();
    let packing_sink = state.sinks[1]; // the grid+collide scene's Output (added second)
    let mut frames = Vec::new();
    let mut means = Vec::new();
    for k in 0..=90u64 {
        let t = k as f64 / 60.0;
        let pos = positions_at(&state, packing_sink, t);
        assert_eq!(pos.len(), 64, "the 8×8 grid");
        means.push(mean_x(&pos));
        frames.push(pos);
    }
    // The packing separates: at the widest breath no two dots are closer than the disc
    // diameter (radius 0.3 × spread ≤ 1.5 → up to 0.9; assert the tight-grid 0.45
    // spacing is broken open — the collide fired). Check the frame with the largest span.
    let widest = frames
        .iter()
        .max_by(|a, b| span(a).partial_cmp(&span(b)).unwrap())
        .unwrap();
    assert!(
        min_pair_dist(widest) > 0.45 + 1e-3,
        "collide broke the 0.45 grid open (min pair {})",
        min_pair_dist(widest)
    );
    assert!(
        max_travel(&frames) > 0.1,
        "the spread lfo breathes the packing"
    );
    let mean = means.iter().sum::<f32>() / means.len() as f32;
    assert!(mean > 3.0, "the packing sits on the right (mean x {mean})");
}

/// The x-span of a layout (max − min x) — a cheap proxy for how far the packing inflated.
fn span(pos: &[[f32; 2]]) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in pos {
        lo = lo.min(p[0]);
        hi = hi.max(p[0]);
    }
    hi - lo
}

/// The smallest distance between any two elements (O(n²) — fine at this count).
fn min_pair_dist(pos: &[[f32; 2]]) -> f32 {
    let mut worst = f32::MAX;
    for a in 0..pos.len() {
        for b in (a + 1)..pos.len() {
            let (dx, dy) = (pos[a][0] - pos[b][0], pos[a][1] - pos[b][1]);
            worst = worst.min((dx * dx + dy * dy).sqrt());
        }
    }
    worst
}

/// The default document replays bit-identically. Both scenes are deterministic
/// (parabolic trig for the mandala fold, arithmetic push-apart for the packing; the lfos
/// are stateless playhead reads), so two runs match exactly (HR-5).
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
