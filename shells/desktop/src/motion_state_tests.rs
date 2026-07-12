//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the rig-closing scenes (an elbowless
//! rubber hose, and flesh SKINNED to a FABRIK tentacle) — through the REAL registry, tick
//! by tick, so the whole chain is exercised and not just the node in isolation.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The demo's rubber hose and its skinned tentacle — see `motion_demo_strobe`.
const HOSE_JOINTS: usize = 9;
const HOSE_BONE: f32 = 0.36;
const HOSE_TIP: usize = HOSE_JOINTS - 1;
/// The flesh: a 5 x 24 strip of points, skinned to the tentacle (whose bones are never
/// drawn — the skin IS the right-hand scene).
const FLESH_POINTS: usize = 5 * 24;

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
        5,
        "the hose, the flesh, a goal dot beside each - and the rain (the simulation zone)"
    );
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 23 nodes: the goal {grid, move, oscillator, orbit} + the hose {skeleton, rubber_hose,
    // scale, move, output} + the skinned tentacle {skeleton, fabrik, grid, move,
    // skin_deformer, scale, move, output} + the goal dots {scale, move, output, move,
    // output} + the ORPHAN that demos the inline readouts (doc 43: no sink consumes it, so
    // it never cooks, so it has no reading and the editor veils it) + the SIMULATION ZONE
    // (docs 48/49/50/51/52 — the snow: {sim.zone, combine, wind, sim.step, sim.collide,
    // sim.lifetime, color_ramp, drive(size), drive(opacity), falloff, cull} interior +
    // {value.attribute, value.map_range} the fade + {grid, move, sim.spawn} birth +
    // {scale, move, output} render = 19).
    assert_eq!(state.doc.graph.nodes().len(), 42);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The rubber hose, through the whole chain** (doc 42): the tip lands on the goal that is
/// drawn beside it (the other sink is the oracle — "these two rendered dots coincide"), the
/// bones stay rigid, the root stays nailed, and — the whole point of the node — there is
/// **NO ELBOW**: every joint turns by the same angle, so the limb is one arc.
///
/// FALSIFIED four ways: the `target` wire missing (the hose never tracks) · a solver that
/// concentrates the bend in one joint (an elbow: the turns would not all be equal) · a hose
/// that stretches to reach · a curl that never changes (which is what a goal at a FIXED
/// distance would give — so the guard also demands that the curl varies over the run).
#[test]
fn the_hose_curls_onto_the_goal_with_no_elbow_anywhere() {
    let state = MotionState::new();
    let ticks = 90;
    let hose = run_scene(&state, state.sinks[0], ticks);
    let goal = run_scene(&state, state.sinks[2], ticks);
    assert_eq!(
        hose[0].len(),
        HOSE_JOINTS,
        "the whole chain reaches the sink"
    );

    let mut curls = Vec::new();
    for k in 0..ticks {
        let miss = dist(hose[k][HOSE_TIP], goal[k][0]);
        assert!(miss < 0.05, "tick {k}: the hose missed the goal by {miss}");
        assert_eq!(hose[k][0], hose[0][0], "tick {k}: the root walked");
        for (i, len) in bone_lengths(&hose[k]).iter().enumerate() {
            assert!(
                (len - HOSE_BONE).abs() < 1e-3,
                "tick {k}: bone {i} stretched to {len}"
            );
        }

        // Every turn along the limb, wrapped into (-180, 180]. Constant curvature = no
        // elbow; the spread across the limb is what a corner would blow up.
        let t: Vec<f32> = (2..HOSE_JOINTS)
            .map(|i| {
                let a = heading(hose[k][i - 1], hose[k][i - 2]);
                let b = heading(hose[k][i], hose[k][i - 1]);
                wrap(b - a)
            })
            .collect();
        let lo = t.iter().copied().fold(f32::MAX, f32::min);
        let hi = t.iter().copied().fold(f32::MIN, f32::max);
        assert!(
            hi - lo < 1.0,
            "tick {k}: the turns are not all equal ({lo} .. {hi}) — that is an elbow"
        );
        curls.push(hi.abs());
    }

    // …and the curl CHANGES: the goal breathes in and out, so the hose coils and uncoils.
    let (lo, hi) = (
        curls.iter().copied().fold(f32::MAX, f32::min),
        curls.iter().copied().fold(f32::MIN, f32::max),
    );
    assert!(hi - lo > 5.0, "the hose coils and uncoils ({lo}° .. {hi}°)");
}

/// **The skin, through the whole chain** (doc 42): the right-hand scene draws NO bones —
/// every dot on screen is a point of flesh, moved by Linear Blend Skinning from the
/// skeleton's rest pose to its solved one.
///
/// FALSIFIED four ways: `rig.skin_deformer` not wired (the flesh would sit in its rest strip
/// forever) · the `rest` and `posed` wires swapped or one of them cut (the deformation is
/// their DIFFERENCE — with both the same, nothing moves) · a skin that tears (the flesh's
/// own shape would fly apart) · a skin bound to nothing (the points would collapse).
#[test]
fn the_flesh_follows_the_bones_it_is_skinned_to() {
    let state = MotionState::new();
    let ticks = 90;
    let flesh = run_scene(&state, state.sinks[1], ticks);
    assert_eq!(
        flesh[0].len(),
        FLESH_POINTS,
        "the whole strip reaches the sink"
    );

    // The flesh MOVES — a lot, and not as one rigid block (it is being deformed, not
    // translated): the far end travels much farther than the end by the root.
    let travel = |i: usize| {
        (0..ticks)
            .map(|k| dist(flesh[k][i], flesh[0][i]))
            .fold(0.0, f32::max)
    };
    let near_root = travel(2); // the bottom row of the strip
    let far_out = travel(FLESH_POINTS - 3); // the top row
    assert!(far_out > 1.0, "the far end of the flesh swings ({far_out})");
    assert!(
        far_out > 3.0 * near_root,
        "the flesh DEFORMS rather than moving rigidly (near {near_root}, far {far_out})"
    );

    // And it does not tear: neighbouring points of the strip stay neighbours. (A skin whose
    // weights blow up would scatter the cloud; this is the cheap, decisive check.)
    for k in [0, 30, 60, 89] {
        for i in 1..FLESH_POINTS {
            let d = dist(flesh[k][i], flesh[k][i - 1]);
            assert!(
                d < 1.0,
                "tick {k}: the flesh tore between {} and {i}",
                i - 1
            );
        }
    }
}

/// The heading of `a` seen from `b`, in degrees, and the wrap that keeps a difference of two
/// headings honest across ±180° (without it, a turn of +42° reads as -318° and every guard
/// about constant curvature reports an elbow that is not there).
fn heading(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[1] - b[1]).atan2(a[0] - b[0]).to_degrees()
}

fn wrap(mut t: f32) -> f32 {
    while t <= -180.0 {
        t += 360.0;
    }
    while t > 180.0 {
        t -= 360.0;
    }
    t
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

/// **The snow: the Simulation Zone, in the boot document, end to end** (docs 48 + 49).
///
/// The whole triangle of a particle system, on the real graph, through the real cook:
///
/// 1. **BIRTH** — the zone's `init` is unwired, so the population starts at NOTHING and is
///    entirely born. If the spawn were dead, the scene would stay empty forever.
/// 2. **LIFE** — a flake ACCELERATES: its velocity lives in the state, so each tick's fall is
///    longer than the last. Measured on one identified flake (`id = 0`), because the population
///    changes underneath any average — the survivorship trap this test walked into once already,
///    when it tracked the LOWEST drop and found it *rising* (the lowest drop is the one the disc
///    is about to kill).
/// 3. **DEATH + STEADY STATE** — birth balances death, so the population climbs and then SETTLES
///    instead of growing without bound. A zone that re-seeded when it emptied, or a cull that was
///    only a filter, would both break the plateau.
#[test]
fn the_snow_is_born_accelerates_and_settles_into_a_steady_state() {
    let state = MotionState::new();
    let snow = *state.sinks.last().expect("the snow is the last sink");
    let mut cook = Cook::new();

    let mut counts: Vec<usize> = Vec::new();
    // The fall of flake `id = 0` — the first one ever born — tick by tick, while it lives.
    let mut flake: Vec<f32> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, snow, t)
            .unwrap();
        let s = out[0].as_stream();
        counts.push(s.count());
        if let (Some(Column::Vec2(p)), Some(Column::Scalar(ids))) = (s.get("P"), s.get("id"))
            && let Some(i) = ids.iter().position(|id| *id == 0.0)
        {
            flake.push(p[i][1]);
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }

    // 1. BIRTH: nothing exists on the first tick (dt = 0), and then the sky fills.
    assert_eq!(counts[0], 0, "the zone starts EMPTY - every flake is born");
    assert!(
        counts[60] > 10,
        "a second in, it is snowing: {}",
        counts[60]
    );

    // 2. LIFE: flake 0 accelerates — each fall is longer than the one before.
    assert!(flake.len() > 30, "flake 0 lived long enough to measure");
    let falls: Vec<f32> = flake.windows(2).map(|w| w[0] - w[1]).collect();
    assert!(
        falls.windows(2).all(|w| w[1] >= w[0] - 1e-4),
        "gravity accumulates in the STATE: {falls:?}"
    );
    assert!(
        falls[falls.len() - 1] > falls[0] * 2.0,
        "…and it really is accelerating, not drifting"
    );

    // 3. AGE: the flakes grow old and are COLOURED by how old they are. Both readings come off
    //    the same live state, so a broken `sim.step` age or a mis-wired ramp shows here.
    let out = cook
        .cook(&state.doc.graph, &state.registry, snow, 4.0)
        .unwrap();
    let s = out[0].as_stream();
    let (ages, lifes) = (
        match s.get("age") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("the flakes have no age"),
        },
        match s.get("life") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("the flakes do not know their life fraction"),
        },
    );
    assert!(
        ages.iter().any(|a| *a > 1.0) && ages.iter().any(|a| *a < 0.3),
        "old flakes and newborns coexist: {:?}..{:?}",
        ages.iter().cloned().fold(f32::MAX, f32::min),
        ages.iter().cloned().fold(f32::MIN, f32::max)
    );
    assert!(
        lifes.iter().all(|t| (0.0..=1.0).contains(t)),
        "the life fraction stays in [0,1]"
    );
    // The colour is DRIVEN by that fraction: a young flake and an old one are not the same
    // colour. (FALSIFIED by a `value.attribute` that reads a missing column: every t would be 0
    // and the whole snowfall would be one flat colour.)
    let tints = match s.get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("the ramp never coloured them"),
    };
    let spread = tints.iter().map(|c| c[2]).fold(f32::MIN, f32::max)
        - tints.iter().map(|c| c[2]).fold(f32::MAX, f32::min);
    assert!(
        spread > 0.05,
        "the flakes are coloured by their AGE, not all alike: blue spread {spread}"
    );

    // …and it does not merely get LIGHTER: an old flake is SMALLER and more TRANSPARENT (doc 51 —
    // Enio's smoke: *"ficou claro mas não menor nem transparente"*). Size and alpha are driven by
    // the same life fraction, so the oldest flake on screen is the smallest AND the faintest.
    let sizes = match s.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("the shrink never wrote a size"),
    };
    let pick = |cmp: fn(&f32, &f32) -> std::cmp::Ordering| {
        lifes
            .iter()
            .enumerate()
            .max_by(|a, b| cmp(a.1, b.1))
            .map(|(i, _)| i)
            .expect("the snow is not empty")
    };
    let (oldest, youngest) = (pick(|a, b| a.total_cmp(b)), pick(|a, b| b.total_cmp(a)));
    assert!(
        tints[oldest][3] < tints[youngest][3] * 0.5,
        "the old flake is FADED: alpha {} vs {}",
        tints[oldest][3],
        tints[youngest][3]
    );
    assert!(
        sizes[oldest][0] < sizes[youngest][0] * 0.5,
        "…and SMALLER: {:?} vs {:?}",
        sizes[oldest],
        sizes[youngest]
    );
    // **`Set`, not `Multiply`.** `size` and `tint` RIDE THE STATE, so a multiplying drive would
    // compound every tick: after four seconds the flakes would be 1e-30 across — a fade that is a
    // function of FRAME COUNT rather than of age, and invisible in any test that only checks
    // "does the old one look different". `Set` is idempotent, so nothing ever falls below the
    // floor its own life maps to.
    assert!(
        sizes.iter().all(|q| q[0] > 0.01),
        "the drive is Set, not Multiply: nothing compounded away ({:?})",
        sizes.iter().map(|q| q[0]).fold(f32::MAX, f32::min)
    );

    // 4. THE GROUND: the flakes LAND on it. Nothing is below the floor (a collider that only
    //    clamped the position would let them ooze through, because their velocity would still
    //    point down), and some are resting ON it — so the floor is being hit, not merely drawn.
    //    The positions here are the RENDERED ones, so the scene's own `motion.move` is added in.
    let ys: Vec<f32> = match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[1]).collect(),
        _ => panic!("no positions"),
    };
    let floor = -2.0 + 2.4; // the collider's height, in the rendered frame (`move` dy = 2.4)
    let lowest = ys.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        lowest > floor - 0.05,
        "nothing fell through the floor: lowest {lowest} vs floor {floor}"
    );
    assert!(
        ys.iter().any(|y| *y < floor + 0.15),
        "…and the snow is settling ON it, not hovering above it: lowest {lowest}"
    );

    // 5. DEATH + STEADY STATE: the population settles instead of growing without bound.
    let late = &counts[180..];
    let (lo, hi) = (
        *late.iter().min().unwrap() as f32,
        *late.iter().max().unwrap() as f32,
    );
    assert!(lo > 0.0, "the snow does not die out: birth keeps up");
    assert!(
        hi < lo * 1.35,
        "birth balances death - the population is a plateau, not a ramp: {lo}..{hi}"
    );
}
