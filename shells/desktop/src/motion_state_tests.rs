//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the two rig scenes (a skeleton at
//! rest and the same limb posed + resolved into a wave) — through the REAL registry,
//! tick by tick, so the whole chain is exercised and not just the node in isolation.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The demo's arm (law of cosines) and tentacle (FABRIK) — see `motion_demo_strobe`.
const ARM_JOINTS: usize = 3;
const ARM_BONE: f32 = 1.5;
const TENTACLE_JOINTS: usize = 10;
const TENTACLE_BONE: f32 = 0.34;

/// Drive `ticks` fixed steps of the real cook (advancing the `pre` edges, exactly as the
/// shell's pump does) and return every tick's positions for `sink`. The wave scene only
/// exists over time — cooking one frame in isolation would show the rest pose and prove
/// nothing.
fn run_scene(state: &MotionState, sink: NodeId, ticks: usize) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut frames = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let ph = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, ph)
            .unwrap();
        frames.push(match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        });
        cook.advance_tick(&state.doc.graph, &state.registry, ph)
            .unwrap();
    }
    frames
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

/// Every bone of a limb, measured on the rendered positions.
fn bone_lengths(joints: &[[f32; 2]]) -> Vec<f32> {
    (1..joints.len())
        .map(|i| dist(joints[i], joints[i - 1]))
        .collect()
}

#[test]
fn new_builds_the_well_typed_rig_document() {
    let state = MotionState::new();
    assert_eq!(
        state.sinks.len(),
        4,
        "arm, tentacle, and a goal dot beside each"
    );
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 19 nodes: the shared goal {grid, move, oscillator, orbit} + the arm {skeleton,
    // ik_2bone, scale, move, output} + the tentacle {skeleton, fabrik, scale, move, output}
    // + the goal dots {scale, move, output, move, output}. The newest nodes (doc 41); the
    // oscillator is what makes the goal breathe in and out, so the elbow actually flexes.
    assert_eq!(state.doc.graph.nodes().len(), 19);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **Both solvers, through the whole chain** (doc 41): each limb's TIP lands on the goal
/// dot that is drawn beside it — measured on what actually reaches `motion.output`, not on
/// the solver's internal arithmetic. The goal orbits, so this is a moving target: it holds
/// at every tick, from every direction of approach.
///
/// The oracle is the OTHER SINK. The goal dot is its own little scene (the same orbiting
/// point, moved by the same offset as the limb that chases it), so "the hand is on the
/// goal" is literally "these two rendered dots coincide" — no recomputation of the
/// solver's own maths to check the solver's own maths.
///
/// FALSIFIED five ways: the `target` port never wired (the limb sits at its rest pose and
/// never tracks) · the solver writing positions instead of a pose (the bones would drift
/// off their lengths through the FK downstream) · the root not nailed (the whole limb
/// would walk toward the goal) · a solver that stretches to reach · the goal dot moved by
/// a different offset than its limb (they would never coincide).
#[test]
fn both_limbs_land_their_tip_on_the_orbiting_goal() {
    let state = MotionState::new();
    let ticks = 90;
    let arm = run_scene(&state, state.sinks[0], ticks);
    let tentacle = run_scene(&state, state.sinks[1], ticks);
    let arm_goal = run_scene(&state, state.sinks[2], ticks);
    let tentacle_goal = run_scene(&state, state.sinks[3], ticks);

    assert_eq!(arm[0].len(), ARM_JOINTS, "the arm reaches the sink");
    assert_eq!(tentacle[0].len(), TENTACLE_JOINTS);
    assert_eq!(arm_goal[0].len(), 1, "the goal is one dot");

    // The goal really moves (otherwise "it tracks" is a vacuous claim).
    let travel = (0..ticks)
        .map(|k| dist(arm_goal[k][0], arm_goal[0][0]))
        .fold(0.0, f32::max);
    assert!(travel > 2.0, "the goal orbits (travelled {travel})");

    for k in 0..ticks {
        // The tolerance is the pose round trip (approximate atan2 + cos/sin), not slack in
        // the solvers: the law of cosines is exact and FABRIK converges to 1e-4.
        let arm_miss = dist(arm[k][ARM_JOINTS - 1], arm_goal[k][0]);
        assert!(arm_miss < 0.03, "tick {k}: the hand missed by {arm_miss}");
        let tip_miss = dist(tentacle[k][TENTACLE_JOINTS - 1], tentacle_goal[k][0]);
        assert!(
            tip_miss < 0.03,
            "tick {k}: the tentacle missed by {tip_miss}"
        );

        // Nailed roots, rigid bones — at every tick of the chase.
        assert_eq!(arm[k][0], arm[0][0], "tick {k}: the arm's root walked");
        assert_eq!(
            tentacle[k][0], tentacle[0][0],
            "tick {k}: the tentacle's root walked"
        );
        for (i, len) in bone_lengths(&arm[k]).iter().enumerate() {
            assert!(
                (len - ARM_BONE).abs() < 1e-3,
                "tick {k}: arm bone {i} = {len}"
            );
        }
        for (i, len) in bone_lengths(&tentacle[k]).iter().enumerate() {
            assert!(
                (len - TENTACLE_BONE).abs() < 1e-3,
                "tick {k}: tentacle bone {i} = {len}"
            );
        }
    }

    // **The elbow FLEXES** — the assertion Enio's smoke caught me not making. Asserting
    // only that the elbow sits OFF the root→hand line proves it is bent; it does not prove
    // it ever BENDS. A goal on a fixed-radius orbit around the arm's own root gives the
    // triangle (root, elbow, hand) three constant sides — and a triangle with fixed sides
    // has fixed angles, so the arm just rotates rigidly with the elbow locked, and passes
    // an "is bent" test all day. The goal's distance must therefore vary (the demo
    // modulates the orbit's radius), and the guard must measure the RANGE of the bend.
    let elbow_offsets: Vec<f32> = (0..ticks)
        .map(|k| {
            let (a, e, h) = (arm[k][0], arm[k][1], arm[k][2]);
            let (dx, dy) = (h[0] - a[0], h[1] - a[1]);
            let reach = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
            // The elbow's distance from the root-to-hand line: 0 = a straight arm.
            ((e[0] - a[0]) * dy - (e[1] - a[1]) * dx).abs() / reach
        })
        .collect();
    let lo = elbow_offsets.iter().copied().fold(f32::MAX, f32::min);
    let hi = elbow_offsets.iter().copied().fold(f32::MIN, f32::max);
    assert!(lo > 0.2, "the elbow is never straight ({lo})");
    assert!(
        hi - lo > 0.5,
        "the elbow FLEXES: it folds and opens across the run ({lo} .. {hi})"
    );

    // …because the reach itself changes. If this is flat, the guard above is vacuous.
    let reaches: Vec<f32> = (0..ticks).map(|k| dist(arm[k][0], arm[k][2])).collect();
    let spread = reaches.iter().copied().fold(f32::MIN, f32::max)
        - reaches.iter().copied().fold(f32::MAX, f32::min);
    assert!(spread > 1.0, "the goal moves nearer and farther ({spread})");
}

/// **A node dropped at its DEFAULT params may not change a single instance** — the
/// invariant the whole `size` column rests on (doc 39).
///
/// Every node that materializes `size` on a stream that has none starts from unit scale
/// (`SIZE_IDENTITY`). If the LOWERING falls back to anything else, then dropping such a
/// node at its own identity silently resizes the scene: the shell used to lower with
/// `0.4`, so a `motion.scale` at `amount = 1` — a no-op by definition — scaled every quad
/// by 2.5×. Nothing caught it, because both halves were self-consistently wrong.
///
/// The guard is behavioural and lowers through the REAL path: cook `grid -> output`, then
/// cook `grid -> scale(default) -> output`, and demand the RenderInstances be identical.
/// FALSIFIED the moment the two ends disagree again. The oscillator's Size channel at zero
/// amplitude is the same claim for the whole channel family (wiggle/noise/step/…, which
/// share the leaf).
#[test]
fn a_node_at_its_default_params_does_not_resize_the_scene() {
    use ph2d_eval_motion::evaluate_motion_into;
    use ph2d_nodegraph::attr::SIZE_IDENTITY;

    let state = MotionState::new();
    assert_eq!(
        state.default_size, SIZE_IDENTITY,
        "the lowering's fallback IS the identity the nodes assume"
    );

    // `grid -> [maybe a node] -> output`, lowered exactly as the shell lowers it.
    let render = |insert: Option<(&str, &[(&str, f32)])>| {
        let mut g = ph2d_nodegraph::graph::Graph::new();
        let grid = g.add_node("motion.grid");
        let out = g.add_node("motion.output");
        let mut tail = grid;
        if let Some((ty, params)) = insert {
            let n = g.add_node(ty.to_string());
            for (p, v) in params {
                g.set_param(n, *p, *v);
            }
            g.connect(ph2d_nodegraph::graph::Edge {
                from: (tail, 0),
                to: (n, 0),
                delayed: false,
            })
            .expect("wire");
            tail = n;
        }
        g.connect(ph2d_nodegraph::graph::Edge {
            from: (tail, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("wire");
        // Lower with the SHELL's own fallbacks — `evaluate_motion`'s 5-arg form would
        // quietly substitute the headless `[1,1]` and the guard would prove nothing
        // about the shell (it passed against the real bug until this line was fixed).
        let mut cook = Cook::new();
        let mut instances = Vec::new();
        evaluate_motion_into(
            &mut cook,
            &g,
            &state.registry,
            out,
            0.0,
            state.default_uv_rect,
            state.default_size,
            &mut instances,
        )
        .expect("cooks");
        instances.iter().map(|i| i.size).collect::<Vec<_>>()
    };

    let bare = render(None);
    assert!(!bare.is_empty(), "the grid renders");
    assert_eq!(
        render(Some(("motion.scale", &[]))),
        bare,
        "a Scale at amount = 1 is a no-op on the render"
    );
    // The channel family: an oscillator aimed at Size with zero amplitude, likewise.
    assert_eq!(
        render(Some((
            "motion.oscillator",
            &[("channel", 3.0), ("amplitude", 0.0)]
        ))),
        bare,
        "a Size oscillator at zero amplitude is a no-op on the render"
    );
    // And `rig.fk` on a stream with no bones — every element is a root (doc 40).
    assert_eq!(
        render(Some(("rig.fk", &[]))),
        bare,
        "FK on a plain point cloud is a no-op"
    );
}

/// The default document replays bit-identically: the whole boot doc is HR-5 arithmetic
/// (no expression / libm on this path), which is what makes a scrub reproducible.
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

/// The text-param channel (doc 32) still round-trips through a save/load. The boot
/// document no longer carries a formula (the demo moved on), so the guard authors its
/// own: a text param bumps the doc to `v2` and comes back byte-for-byte. FALSIFIED by the
/// old data-loss bug (the formula dropped on save).
#[test]
fn a_text_param_survives_a_text_round_trip() {
    let mut doc = ph2d_motion_doc::MotionDoc::default();
    let expr = doc.graph.add_node("motion.expression");
    doc.graph
        .set_text_param(expr, "expr", "cos(f * a + t) * f * 4");

    let text = doc.to_text();
    assert!(text.starts_with("v2"), "a text param bumps the doc to v2");
    assert!(
        text.contains("cos(f * a + t) * f * 4"),
        "the formula is serialized (interior spaces intact)"
    );
    let back = ph2d_motion_doc::MotionDoc::from_text(&text).expect("the v2 doc reloads");
    assert_eq!(
        back.graph.node_text_params(),
        doc.graph.node_text_params(),
        "every formula survives the round trip"
    );
}
