//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the two rig scenes (a skeleton at
//! rest and the same limb posed + resolved into a wave) — through the REAL registry,
//! tick by tick, so the whole chain is exercised and not just the node in isolation.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Both demo limbs: 14 joints, bones of 0.55 (see `motion_demo_strobe`).
const JOINTS: usize = 14;
const BONE: f32 = 0.55;
const TIP: usize = JOINTS - 1;

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
    assert_eq!(state.sinks.len(), 2, "two scenes -> two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 10 nodes: {skeleton, scale, move, output}
    // + {skeleton, oscillator, rig.fk, scale, move, output}. The newest nodes (doc 40).
    assert_eq!(state.doc.graph.nodes().len(), 10);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The skeleton, through the whole chain** (doc 40): the rest limb reaches
/// `motion.output` as 14 joints, strung end-to-end by bones of exactly `BONE` — and it
/// CURLS (each joint's bend is relative to the last one, which is what makes a chain a
/// chain rather than a fan out of the root).
///
/// FALSIFIED three ways: the seam never wired (no joints arrive) · the chain fanned from
/// the root instead of stacking (the bones would not measure `BONE` end-to-end) · a limb
/// that moves when nothing poses it.
#[test]
fn the_rest_skeleton_is_a_curling_chain_of_rigid_bones() {
    let state = MotionState::new();
    let frames = run_scene(&state, state.sinks[0], 10);
    let limb = &frames[0];
    assert_eq!(limb.len(), JOINTS, "the whole chain reaches the sink");

    for (i, len) in bone_lengths(limb).iter().enumerate() {
        assert!(
            (len - BONE).abs() < 1e-3,
            "bone {i} measures {len}, not {BONE}"
        );
    }
    // It curls: the direction of the last bone differs from the first (a straight rod
    // would keep the same heading, a fan would not keep its bone lengths at all).
    let first = [limb[1][0] - limb[0][0], limb[1][1] - limb[0][1]];
    let last = [
        limb[TIP][0] - limb[TIP - 1][0],
        limb[TIP][1] - limb[TIP - 1][1],
    ];
    let turned = dist(first, last);
    assert!(turned > 0.5, "the chain curls (heading change {turned})");

    // Nothing poses this limb, so it must not move.
    assert_eq!(frames[9], frames[0], "the rest pose is at rest");
}

/// **FK, through the whole chain** (doc 40) — the node's whole reason to exist.
///
/// The oscillator is a GENERIC node: it writes the joints' `rot` column and knows nothing
/// about bones, so it leaves every joint exactly where it was. `rig.fk` is what turns
/// those posed angles into a pose. So the limb waves — and it waves *rigidly*: the bones
/// keep their length at every tick, and the root, which nothing can move, stays nailed.
///
/// FALSIFIED four ways: **`rig.fk` cut out of the chain** (the joints stay bent-but-still
/// — the tip never travels: the seam bug this whole module keeps re-learning) · a resolve
/// that rebuilds from the origin (the root would drift) · a resolve that moves `P`
/// directly instead of through the bones (they would stretch) · the oscillator not wired
/// (the wave never starts).
#[test]
fn the_posed_limb_waves_rigidly_from_a_nailed_root() {
    let state = MotionState::new();
    let frames = run_scene(&state, state.sinks[1], 90);
    assert_eq!(frames[0].len(), JOINTS);

    // The root is nailed: nothing in the chain can move it.
    for (k, f) in frames.iter().enumerate() {
        assert_eq!(f[0], frames[0][0], "the root drifted at tick {k}");
    }

    // The tip travels — this is the assertion that dies if FK is not there.
    let swing = frames
        .iter()
        .map(|f| dist(f[TIP], frames[0][TIP]))
        .fold(0.0, f32::max);
    assert!(swing > 0.5, "the limb waves (tip travelled {swing})");

    // And it waves RIGIDLY: no bone stretches, at any tick of the wave.
    for (k, f) in frames.iter().enumerate() {
        for (i, len) in bone_lengths(f).iter().enumerate() {
            assert!(
                (len - BONE).abs() < 1e-3,
                "tick {k}: bone {i} stretched to {len}"
            );
        }
    }
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
