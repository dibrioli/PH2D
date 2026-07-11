//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now a `motion.color_ramp` rainbow
//! sunburst and a `motion.color_array` palette grid, each a `Pure` layout animated
//! through a `value.lfo` — through the REAL registry.

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

/// The `tint` column (linear RGBA) of one sink at playhead `t` — what the colour nodes
/// write.
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

/// The largest over-time travel of any single element (fixed count in both scenes).
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

/// The number of distinct colours in a tint set (exact equality — palette colours are
/// exact param values).
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
    // 10 nodes: {distribute_radial, color_ramp, move, output, lfo} + {grid, color_array,
    // move, output, lfo}. The two newest nodes (doc 29) — a `motion.color_ramp` and a
    // `motion.color_array`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 10);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.color_ramp` is alive end to end (doc 29): the 60-point sunburst is coloured by
/// a rainbow (many distinct hues) and spins via the `spin` lfo, so it travels; the scene
/// sits on the left. Falsifiable: a solid colour → one hue; a dead spin → static.
#[test]
fn the_rainbow_sunburst_spins_and_is_colourful() {
    let state = MotionState::new();
    let sink = state.sinks[0]; // the rainbow scene (added first)
    let mut frames = Vec::new();
    for k in 0..=150u64 {
        let pos = positions_at(&state, sink, k as f64 / 60.0);
        assert_eq!(pos.len(), 60, "the 60-point sunburst");
        frames.push(pos);
    }
    assert!(max_travel(&frames) > 0.3, "the spin lfo turns the sunburst");
    // The rainbow spans many colours (well beyond a solid tint's one).
    let tints = tints_at(&state, sink, 0.0);
    assert!(
        distinct_colours(&tints) > 20,
        "the rainbow is colourful ({} distinct)",
        distinct_colours(&tints)
    );
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean < -3.0, "the sunburst sits on the left (mean x {mean})");
}

/// `motion.color_array` is alive end to end (doc 29): the 100-point grid takes exactly a
/// 4-colour palette, and the `offset` saw lfo marches it (the colour at a fixed index
/// changes over time); the scene sits on the right. Falsifiable: >4 colours → not a
/// palette; a dead offset → the index-0 colour never changes.
#[test]
fn the_palette_grid_marches() {
    let state = MotionState::new();
    let sink = state.sinks[1]; // the palette scene (added second)
    let pos = positions_at(&state, sink, 0.0);
    assert_eq!(pos.len(), 100, "the 10×10 grid");
    // Exactly a 4-colour palette.
    let tints0 = tints_at(&state, sink, 0.0);
    assert_eq!(distinct_colours(&tints0), 4, "a 4-colour palette");
    // The palette marches: index-0's colour differs at some later frame (offset shifted).
    let marched = (0..=240u64).any(|k| {
        let t = tints_at(&state, sink, k as f64 / 60.0);
        !t.is_empty() && t[0] != tints0[0]
    });
    assert!(marched, "the offset lfo marches the palette");
    let mean = mean_x(&pos);
    assert!(
        mean > 3.0,
        "the palette grid sits on the right (mean x {mean})"
    );
}

/// The default document replays bit-identically. Both scenes are deterministic (radial
/// parabolic trig, ramp/palette arithmetic; the lfos are stateless playhead reads), so
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
