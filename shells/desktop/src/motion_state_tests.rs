//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the M3 deformer scene: a grid
//! curled by `motion.bend` whose squares are aimed by `motion.look_at` at a moving
//! target — through the REAL registry, exactly as the bridge does.

use super::*;
use ph2d_nodegraph::attr::Column;

/// The grid's per-element rotations (degrees) at playhead `t`.
fn rotations_at(state: &MotionState, t: f64) -> Vec<f32> {
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get("rot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 1, "the deformer scene is the sole scene");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 7 nodes: grid, bend, look_at, tint, output, lfo_bend, lfo_target. The two
    // newest nodes (doc 20) — a `motion.bend` arc and a `motion.look_at` aim, each
    // animated by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 7);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.bend` is alive end to end (doc 20): the `amount` lfo (±1) wraps the
/// grid's X extent onto an arc that curls up and uncurls, so a rim corner sweeps a
/// wide path over time. Falsifiable: a dead bend leaves the flat grid STILL — the
/// corner never moves.
#[test]
fn the_bend_curls_the_grid_over_time() {
    let state = MotionState::new();
    // Pump ~5 s (the bend lfo's full period). Track the top-left rim corner (index
    // 0, at x=-2 — a max-|x| point, so the bend moves it the most).
    let mut corner: Vec<[f32; 2]> = Vec::new();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let sink = *state.sinks.last().unwrap();
    for k in 0..=300u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        if let Some(Column::Vec2(v)) = out[0].as_stream().get("P") {
            corner.push(v[0]);
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(corner.len() > 250, "pumped the bend period");
    let (mut xhi, mut xlo, mut yhi, mut ylo) = (f32::MIN, f32::MAX, f32::MIN, f32::MAX);
    for &[x, y] in &corner {
        xhi = xhi.max(x);
        xlo = xlo.min(x);
        yhi = yhi.max(y);
        ylo = ylo.min(y);
    }
    assert!(
        (xhi - xlo) + (yhi - ylo) > 1.0,
        "the bend sweeps the rim corner (Δx {} + Δy {}); a dead bend pins it flat",
        xhi - xlo,
        yhi - ylo
    );
}

/// `motion.look_at` is alive end to end (doc 20): each square aims its `rot` at the
/// target, and the `target_x` lfo slides the target, so the squares SWIVEL to
/// follow. At any instant the squares face it from DIFFERENT angles (a spread of
/// rotations), and a given square's aim TURNS as the target moves.
///
/// Falsifiable: a dead look_at leaves every `rot` at 0 (no spread, no motion); a
/// look_at that ignored position would give every square the SAME rotation (no
/// spread across the grid).
#[test]
fn the_look_at_aims_each_square_at_the_moving_target() {
    let state = MotionState::new();

    // A SPREAD across the grid: at t=0 (target at 0) the 20 squares sit at different
    // places, so they face the centre from different angles.
    let rot0 = rotations_at(&state, 0.0);
    assert_eq!(rot0.len(), 20, "the 4×5 grid");
    let hi = rot0.iter().copied().fold(f32::MIN, f32::max);
    let lo = rot0.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        hi - lo > 30.0,
        "the squares face the target from different angles (spread {}°); a position-blind aim would be uniform",
        hi - lo
    );

    // It TURNS over time: as the target slides, a corner square's aim changes. Track
    // index 0's rotation across ~1.5 s (half the target lfo's 3 s period).
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let sink = *state.sinks.last().unwrap();
    let mut aim0: Vec<f32> = Vec::new();
    for k in 0..=90u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        if let Some(Column::Scalar(v)) = out[0].as_stream().get("rot") {
            aim0.push(v[0]);
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    let aim_hi = aim0.iter().copied().fold(f32::MIN, f32::max);
    let aim_lo = aim0.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        aim_hi - aim_lo > 15.0,
        "a square's aim turns as the target slides (range {}°); a dead look_at is fixed",
        aim_hi - aim_lo
    );
}

/// The default document replays bit-identically. The scene holds NO `pre` state
/// (grid/bend/look_at/tint are Pure; the lfos are stateless Temporal reads of the
/// playhead), so it is a pure function of the tick and two runs match exactly
/// (HR-5 — the parabolic trig and the atan2 approximation are deterministic).
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..30u64 {
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
