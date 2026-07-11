//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the M3 morph opener: a
//! `motion.fibonacci` spiral crossfaded by `motion.morph` into a `motion.scatter`
//! blue-noise cloud (**a sunflower dissolving into a cloud and back**) — through
//! the REAL registry, exactly as the bridge does.

use super::*;

fn radius(p: [f32; 2]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

/// The grid's positions at playhead `t`.
fn positions_at(state: &MotionState, t: f64) -> Vec<[f32; 2]> {
    use ph2d_nodegraph::attr::Column;
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, t)
        .unwrap();
    match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // One focused scene → one Output node → one render sink.
    assert_eq!(state.sinks.len(), 1, "the morph scene is the sole scene");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 9 nodes: fibonacci, scatter, morph, tint, drive_size, output, instance_field,
    // size_range, lfo. The two newest nodes (doc 19) — a `motion.scatter` blue-noise
    // cloud crossfaded with the `motion.fibonacci` spiral by `motion.morph`.
    assert_eq!(state.doc.graph.nodes().len(), 9);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.morph` + `motion.scatter` are alive end to end (doc 19): the `blend`
/// lfo eases the crossfade, so at the trough (t=3 s, blend≈0) the grid IS the
/// ordered `motion.fibonacci` spiral, and at the peak (t=1 s, blend≈1) it IS the
/// `motion.scatter` cloud — the two shapes really trade.
///
/// Falsifiable: a dead morph (stuck at `a`) leaves the peak frame IDENTICAL to the
/// trough (no crossfade); a dead scatter would make the peak frame the spiral too.
/// We prove the trough frame is the spiral (radii climb by index) AND the peak
/// frame is a very different shape (large total displacement) that fills the
/// scatter field (so it is the cloud, not the spiral).
#[test]
fn the_morph_dissolves_the_spiral_into_the_scatter() {
    let state = MotionState::new();
    let spiral = positions_at(&state, 3.0); // blend ≈ 0 → all fibonacci
    let cloud = positions_at(&state, 1.0); // blend ≈ 1 → all scatter
    let n = spiral.len();
    assert_eq!(n, 180, "the 180 seeds");
    assert_eq!(cloud.len(), 180, "the morph keeps the paired count");

    // TROUGH is the ordered spiral: radii climb by index (the Vogel √i).
    let sample = [1, n / 4, n / 2, 3 * n / 4, n - 1];
    for w in sample.windows(2) {
        assert!(
            radius(spiral[w[1]]) > radius(spiral[w[0]]),
            "blend≈0 is the spiral: radius grows from {} to {}",
            w[0],
            w[1]
        );
    }
    // PEAK is a very different shape — the morph really crossfaded. A dead morph
    // would leave the peak identical to the trough (displacement 0).
    let displacement: f32 = spiral
        .iter()
        .zip(cloud.iter())
        .map(|(a, b)| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
        .sum();
    assert!(
        displacement > 50.0,
        "the shapes crossfade (total displacement {displacement}); a dead morph = 0"
    );
    // PEAK fills the scatter field (±2 in a 4×4), so it is the cloud reaching the
    // output — and NOT the spiral (whose index-radius order the cloud lacks).
    for p in &cloud {
        assert!(
            p[0].abs() <= 2.2 && p[1].abs() <= 2.2,
            "cloud point in the field: {p:?}"
        );
    }
    let cloud_ordered = sample
        .windows(2)
        .all(|w| radius(cloud[w[1]]) > radius(cloud[w[0]]));
    assert!(
        !cloud_ordered,
        "the cloud has no index→radius order (it is not the spiral)"
    );
}

/// The default document replays bit-identically. The scene holds NO `pre` state
/// (fibonacci/scatter/morph/instance_field/size_range/tint/drive are Pure; the lfo
/// is a stateless Temporal read of the playhead), so it is a pure function of the
/// tick and two runs match exactly (HR-5 — the hash and parabolic trig are
/// deterministic).
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
                    .map(|i| (i.world_pos, i.tint, i.size))
                    .collect::<Vec<_>>(),
            );
        }
        frames
    };
    assert_eq!(run(), run(), "two runs of the same document match exactly");
}
