//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the M3 opener: a
//! `motion.fibonacci` phyllotaxis spiral reshaped by an animated `motion.twist`
//! (a **twisting sunflower**) — through the REAL registry, exactly as the bridge
//! does.

use super::*;

fn radius(p: [f32; 2]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

#[test]
fn new_builds_the_well_typed_value_document() {
    let state = MotionState::new();
    // One focused scene → one Output node → one render sink.
    assert_eq!(state.sinks.len(), 1, "the sunflower is the sole scene");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 8 nodes: fibonacci, twist, tint, drive_size, output, instance_field,
    // size_range, lfo. The two newest nodes (doc 18) — a `motion.fibonacci`
    // spiral reshaped by an animated `motion.twist`.
    assert_eq!(state.doc.graph.nodes().len(), 8);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.fibonacci` is alive end to end (doc 18): the seeds sit on a Vogel
/// spiral, so their radius grows as `spacing·√i` — the centre seed at 0, the rim
/// at `spacing·√(N−1)`. The `twist` downstream only ROTATES (radius-preserving),
/// so the √i growth survives to the sink.
///
/// Falsifiable: a grid / uniform / dead generator has no centre-to-rim radius
/// growth — sampled seeds at 1/4/1/2/3/4 through the index would not climb.
#[test]
fn the_fibonacci_lays_out_a_phyllotaxis_spiral() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook
        .cook(&state.doc.graph, &state.registry, sink, 0.0)
        .unwrap();
    let p: Vec<[f32; 2]> = match out[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let n = p.len();
    assert_eq!(n, 180, "the 180 sunflower seeds");

    // The centre seed sits at the origin; the rim at spacing·√(N−1) ≈ 0.15·√179 ≈ 2.
    assert!(radius(p[0]) < 0.05, "seed 0 at the centre: {:?}", p[0]);
    let rim = 0.15 * ((n - 1) as f32).sqrt();
    assert!(
        (radius(p[n - 1]) - rim).abs() < 0.02,
        "rim at spacing·√(N-1): {}",
        radius(p[n - 1])
    );
    // The radius CLIMBS with the index (√i) — sampled quartiles strictly increase.
    let sample = [1, n / 4, n / 2, 3 * n / 4, n - 1];
    for w in sample.windows(2) {
        assert!(
            radius(p[w[1]]) > radius(p[w[0]]),
            "radius grows from seed {} ({}) to {} ({})",
            w[0],
            radius(p[w[0]]),
            w[1],
            radius(p[w[1]])
        );
    }
}

/// `motion.twist` is alive end to end (doc 18): a `value.lfo` drives its `amount`,
/// so the spiral COILS and uncoils over time. We track the rim seed: its position
/// sweeps a wide arc (the twist animates) while its RADIUS stays constant (the
/// twist is a rotation about the centre, not a scale).
///
/// The scene is a pure function of the playhead (the lfo is Temporal), so we cook
/// each tick directly. Falsifiable two ways: a dead twist leaves the rim seed
/// STILL (no arc); a non-rotation deform would change its radius as it moves.
#[test]
fn the_twist_coils_the_spiral_over_time() {
    use ph2d_nodegraph::attr::Column;

    let state = MotionState::new();
    let sink = *state.sinks.last().unwrap();
    let mut cook = ph2d_nodegraph::cook::Cook::new();

    // Pump one full lfo period (~4 s = 240 ticks). Track the rim seed (last index).
    let mut rim_pos: Vec<[f32; 2]> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .unwrap();
        if let Some(Column::Vec2(v)) = out[0].as_stream().get("P") {
            rim_pos.push(*v.last().unwrap());
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }
    assert!(rim_pos.len() > 200, "pumped the full period");

    // COILS: the rim seed sweeps a wide arc as `amount` animates the twist. A dead
    // twist would pin it in place.
    let (mut xhi, mut xlo, mut yhi, mut ylo) = (f32::MIN, f32::MAX, f32::MIN, f32::MAX);
    for &[x, y] in &rim_pos {
        xhi = xhi.max(x);
        xlo = xlo.min(x);
        yhi = yhi.max(y);
        ylo = ylo.min(y);
    }
    assert!(
        (xhi - xlo) + (yhi - ylo) > 1.0,
        "the rim seed sweeps a wide arc (Δx {} + Δy {}); a dead twist would pin it",
        xhi - xlo,
        yhi - ylo
    );
    // ROTATION, NOT SCALE: the rim seed's radius is preserved as it moves — the
    // twist rotates about the centre. A deform that scaled would break this.
    let rhi = rim_pos.iter().map(|&p| radius(p)).fold(f32::MIN, f32::max);
    let rlo = rim_pos.iter().map(|&p| radius(p)).fold(f32::MAX, f32::min);
    assert!(
        rhi - rlo < 0.05,
        "the twist preserves the rim seed's radius ([{rlo}, {rhi}]); a scale would not"
    );
}

/// The default document replays bit-identically. The scene holds NO `pre` state
/// (fibonacci/twist/instance_field/size_range/tint/drive are Pure; the lfo is a
/// stateless Temporal read of the playhead), so it is a pure function of the tick
/// and two runs match exactly (HR-5 — the parabolic trig is deterministic).
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
