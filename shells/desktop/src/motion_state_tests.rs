//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two M4 continuum-media
//! simulation scenes: a `motion.soft_body` jelly and a `motion.wave` ripple field,
//! each a sequential sim on the `pre` self-loop — through the REAL registry.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// A named Vec2 column of one sink at playhead `t` (no tick advance).
fn column_at(
    state: &MotionState,
    cook: &mut Cook,
    sink: NodeId,
    t: f64,
    col: &str,
) -> Vec<[f32; 2]> {
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get(col) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // Two independent scenes → two Output sinks (the jelly and the ripple field).
    assert_eq!(state.sinks.len(), 2, "two sim scenes → two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 8 nodes: {sim, tint, output, lfo} × 2 scenes. The two newest nodes (doc 22)
    // — a `motion.soft_body` and a `motion.wave`, each driven by a `value.lfo`.
    assert_eq!(state.doc.graph.nodes().len(), 8);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.soft_body` is alive end to end (doc 22): gravity sags the jelly below its
/// pinned top row while the `anchor_x` lfo slides the pin, so the body hangs AND
/// sweeps sideways. Falsifiable: a dead body (no gravity, no sim) sits flat at the
/// anchor and still.
#[test]
fn the_soft_body_hangs_and_wobbles_from_the_moving_anchor() {
    let state = MotionState::new();
    let jelly_sink = state.sinks[0]; // the soft-body scene's Output (added first)
    let mut cook = Cook::new();
    let mut mean_x: Vec<f32> = Vec::new();
    let (mut last_top, mut last_bottom) = (0.0f32, 0.0f32);
    for k in 0..=200u64 {
        let t = k as f64 / 60.0;
        let pos = column_at(&state, &mut cook, jelly_sink, t, "P");
        assert_eq!(pos.len(), 16, "the 4×4 mesh");
        mean_x.push(pos.iter().map(|q| q[0]).sum::<f32>() / 16.0);
        // Top row = indices 0..4, bottom row = 12..16 (row-major, row 0 at top).
        last_top = pos[0..4].iter().map(|q| q[1]).sum::<f32>() / 4.0;
        last_bottom = pos[12..16].iter().map(|q| q[1]).sum::<f32>() / 4.0;
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(
        last_bottom < last_top - 1.0,
        "gravity hangs the body below the pinned top"
    );
    let mean = mean_x.iter().sum::<f32>() / mean_x.len() as f32;
    assert!(mean < -3.0, "the jelly hangs on the left (mean x {mean})");
    let (hi, lo) = (
        mean_x.iter().copied().fold(f32::MIN, f32::max),
        mean_x.iter().copied().fold(f32::MAX, f32::min),
    );
    assert!(
        hi - lo > 0.5,
        "the sliding anchor sweeps the body (Δx {})",
        hi - lo
    );
}

/// `motion.wave` is alive end to end (doc 22): the driven centre radiates ripples
/// that swell the dots, so the emitted `size` column develops a SPREAD across the
/// grid (rings of large and small dots) while staying bounded. Falsifiable: a dead
/// field (no drive, no sim) leaves every dot at the flat baseline size.
#[test]
fn the_wave_ripples_outward_from_the_driven_center() {
    let state = MotionState::new();
    let wave_sink = state.sinks[1]; // the wave scene's Output (added second)
    let mut cook = Cook::new();
    let mut seen_spread = 0.0f32;
    let mut sizes = Vec::new();
    for k in 0..=180u64 {
        let t = k as f64 / 60.0;
        sizes = column_at(&state, &mut cook, wave_sink, t, "size");
        assert_eq!(sizes.len(), 169, "the 13×13 field");
        let hi = sizes.iter().map(|s| s[0]).fold(f32::MIN, f32::max);
        let lo = sizes.iter().map(|s| s[0]).fold(f32::MAX, f32::min);
        seen_spread = seen_spread.max(hi - lo);
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(
        seen_spread > 0.1,
        "the ripples make rings of different dot sizes (max spread {seen_spread})"
    );
    assert!(
        sizes.iter().all(|s| s[0].is_finite() && s[0] > 0.0),
        "the field stays bounded and positive"
    );
}

/// The default document replays bit-identically. Both scenes are deterministic
/// (shape-matching via a closed-form 2D polar decomposition; the wave equation is
/// pure arithmetic; the lfos are stateless playhead reads), so two runs of the same
/// document match exactly (HR-5). This drives the FULL sequential pump (state on the
/// `pre` loop), so it also proves the sims step reproducibly, not just the pure ops.
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
