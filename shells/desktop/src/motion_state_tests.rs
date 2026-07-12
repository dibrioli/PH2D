//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two `motion.expression`-driven scenes
//! (a spiral and a colour wave) — through the REAL registry, exercising the text-param
//! channel end to end.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

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
    // 12 nodes: {grid, exprX, exprY, make_point, tint, move, output} + {grid, expr,
    // color_ramp, move, output}. The newest node (doc 32) — `motion.expression`, its
    // formula in the graph's text channel.
    assert_eq!(state.doc.graph.nodes().len(), 12);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
    assert_eq!(state.transport.playhead(1.0 / 60.0), 0.0); // paused at tick 0
}

/// `motion.expression` is alive end to end (doc 32): the cos/sin formulas plot a 144-point
/// spiral (radially spread) that rotates because they read `t`; the scene sits on the
/// left. Falsifiable: an ignored formula → all points at one spot (no spread); a formula
/// without `t` → no rotation.
#[test]
fn the_spiral_is_plotted_and_rotates() {
    let state = MotionState::new();
    let sink = state.sinks[0]; // the spiral scene (added first)
    let mut frames = Vec::new();
    for k in 0..=120u64 {
        let pos = positions_at(&state, sink, k as f64 / 60.0);
        assert_eq!(pos.len(), 144, "the 12×12 grid plotted as a spiral");
        frames.push(pos);
    }
    // The spiral spreads radially (the formula r = f·4 reaches ~4 before the −6 shift).
    let last = frames.last().unwrap();
    let (mut xlo, mut xhi) = (f32::MAX, f32::MIN);
    for p in last {
        xlo = xlo.min(p[0]);
        xhi = xhi.max(p[0]);
    }
    assert!(
        xhi - xlo > 4.0,
        "the spiral spreads (x extent {})",
        xhi - xlo
    );
    assert!(max_travel(&frames) > 0.5, "the `t` term rotates the spiral");
    let mean = frames.iter().map(|f| mean_x(f)).sum::<f32>() / frames.len() as f32;
    assert!(mean < -3.0, "the spiral sits on the left (mean x {mean})");
}

/// `motion.expression` drives colour too (doc 32): the `sin(t·2 + f·a)` wave feeds the
/// ramp's `t`, so the 100 dots take a spread of Ice colours that scroll over time. The
/// scene sits on the right. Falsifiable: a dead expression → one colour, no scroll.
#[test]
fn the_colour_wave_scrolls() {
    let state = MotionState::new();
    let sink = state.sinks[1]; // the wave scene (added second)
    let pos = positions_at(&state, sink, 0.0);
    assert_eq!(pos.len(), 100, "the 10×10 grid");
    let tints0 = tints_at(&state, sink, 0.0);
    assert!(
        distinct_colours(&tints0) > 5,
        "the wave spans many Ice colours ({} distinct)",
        distinct_colours(&tints0)
    );
    // The wave scrolls: the colour at a fixed index changes over time (the `t` term).
    let scrolled = (0..=120u64).any(|k| {
        let t = tints_at(&state, sink, k as f64 / 60.0);
        !t.is_empty() && t[0] != tints0[0]
    });
    assert!(scrolled, "the `t·2` term scrolls the colour wave");
    let mean = mean_x(&pos);
    assert!(
        mean > 3.0,
        "the wave grid sits on the right (mean x {mean})"
    );
}

/// The default document replays bit-identically **on this machine**: the expression's
/// `ph2d_expr::eval` uses `f32` transcendentals (libm), which are stable within one
/// process/libm, so two runs match. (Cross-machine, libm variance is expected — the
/// expression is presentation-side / HR-5-exempt; see the integration handoff.)
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

/// End-to-end proof that the expression formula PERSISTS through a textual save/load
/// (doc 32 §serialization, now closed): the boot doc has a text param, so its text is
/// `v2` and carries the formula; `MotionDoc::from_text` restores it byte-for-byte.
/// FALSIFIED by the old data-loss bug (the formula dropped on save).
#[test]
fn the_expression_formula_survives_a_text_round_trip() {
    let state = MotionState::new();
    let text = state.doc.to_text();
    assert!(text.starts_with("v2"), "a text param bumps the doc to v2");
    assert!(
        text.contains("cos(f * a + t) * f * 4"),
        "the spiral formula is serialized (interior spaces intact)"
    );
    let back = ph2d_motion_doc::MotionDoc::from_text(&text).expect("the v2 doc reloads");
    assert_eq!(
        back.graph.node_text_params(),
        state.doc.graph.node_text_params(),
        "every formula survives the round trip"
    );
}
