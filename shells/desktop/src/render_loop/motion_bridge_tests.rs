//! Headless bridge **seam** tests (split for the HR-18 LOC cap). Declared by the
//! parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.
//! Proves the bridge is really wired to the registry, the transport and the sink.
//! The param-row / widget tests live in `motion_bridge_param_tests.rs`.

use super::*;
use crate::motion_state::MotionState;

/// The Behaviours seam, cooked through the REAL registry (not a unit-test
/// stub): `grid -> stagger -> oscillator` is well-typed (validate passes — the
/// nodes are registered and their ports match) and cooks end to end, and the
/// behaviours actually displace the grid. This is the "isolamento órfão"
/// antidote — a node can be unit-green yet unregistered / mistyped in the
/// live pipeline; this proves it is wired.
#[test]
fn grid_stagger_oscillator_cook_through_the_real_registry() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let motion = MotionState::new(); // registry = register_all_nodes
    let cook_p = |g: &Graph, target| {
        let mut cook = Cook::new();
        let out = cook.cook(g, &motion.registry, target, 0.25).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    };

    // Bare grid (baseline) vs grid -> stagger(Y) -> oscillator(Y).
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let stagger = g.add_node("motion.stagger");
    let osc = g.add_node("motion.oscillator");
    g.connect(Edge {
        from: (grid, 0),
        to: (stagger, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (stagger, 0),
        to: (osc, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(stagger, "channel", 1.0);
    g.set_param(stagger, "min", 0.0);
    g.set_param(stagger, "max", 2.0);
    g.set_param(osc, "channel", 1.0);
    g.set_param(osc, "amplitude", 1.0);
    g.set_param(osc, "phase_stagger", 0.0); // uniform bob -> +amplitude at t=¼

    // The whole chain type-checks against the real registry.
    g.validate(&motion.registry)
        .expect("grid -> stagger -> oscillator is well-typed");

    let base = cook_p(&g, grid);
    let out = cook_p(&g, osc);
    assert_eq!(out.len(), base.len(), "count preserved through behaviours");
    assert!(base.len() >= 4, "grid emits its default cells");
    let n = base.len();
    for (i, (b, o)) in base.iter().zip(&out).enumerate() {
        // X untouched; Y = base + stagger ramp (i/(n-1)·2) + oscillator (+1).
        let ramp = 2.0 * i as f32 / (n as f32 - 1.0);
        assert!((o[0] - b[0]).abs() < 1e-4, "X untouched at {i}");
        assert!(
            (o[1] - (b[1] + ramp + 1.0)).abs() < 1e-4,
            "Y = base + ramp + osc at {i}"
        );
    }
}

/// The animation enabler (ask "when do we see animation?"): playing advances
/// the playhead — so any `Temporal` behaviour moves — and pausing freezes it.
/// The default transport is paused, which is why nothing moved before.
#[test]
fn transport_play_advances_time_and_pause_freezes_it() {
    let mut motion = MotionState::new();
    let dt = 1.0 / 60.0;
    assert_eq!(motion.transport.playhead(dt), 0.0, "starts paused at t=0");
    motion.transport.play();
    motion.transport.advance(30);
    let t = motion.transport.playhead(dt);
    assert!(
        t > 0.0,
        "playing advances the playhead -> behaviours animate"
    );
    motion.transport.toggle(); // -> paused
    motion.transport.advance(30);
    assert_eq!(
        motion.transport.playhead(dt),
        t,
        "paused freezes the playhead"
    );
}

/// The #1->#2 producer/consumer seam through the REAL registry: the grid's
/// `Index`/`Count` identity columns drive the tint's Gradient mode, so a grid
/// reads as a colour ramp. A 1×3 grid + gradient Start=white/End=black ->
/// tints white->grey->black across the row.
#[test]
fn grid_index_drives_the_tint_gradient() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let tint = g.add_node("motion.tint");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 3.0); // 3 cells -> Index 0,1,2 / Count 3
    g.set_param(tint, "mode", 1.0); // Gradient (white->black defaults)
    g.connect(Edge {
        from: (grid, 0),
        to: (tint, 0),
        delayed: false,
    })
    .unwrap();
    g.validate(&motion.registry)
        .expect("grid -> tint is well-typed");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &motion.registry, tint, 0.0).unwrap();
    match out[0].as_stream().get("tint").unwrap() {
        Column::Vec4(v) => {
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "index 0 -> Start (white)");
            assert_eq!(v[1], [0.5, 0.5, 0.5, 1.0], "index 1 -> mid grey");
            assert_eq!(v[2], [0.0, 0.0, 0.0, 1.0], "index 2 -> End (black)");
        }
        _ => panic!("tint"),
    }
}

/// Output nodes ARE the render sinks: the bridge auto-selects every one of them
/// (so several independent scenes compose into one draw), cooking them draws
/// whatever feeds them, and deleting them stops the render. The output follows
/// the graph, not a hidden toggle.
#[test]
fn every_output_node_is_a_render_sink() {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut motion = MotionState::new();
    let (uv, size) = (motion.default_uv_rect, motion.default_size);

    // Fresh graph: two independent grids, each into its own Output.
    let mut g = Graph::new();
    let mut outs = Vec::new();
    for _ in 0..2 {
        let grid = g.add_node("motion.grid");
        let out = g.add_node("motion.output");
        g.connect(Edge {
            from: (grid, 0),
            to: (out, 0),
            delayed: false,
        })
        .unwrap();
        outs.push(out);
    }
    motion.doc.graph = g;

    // The bridge resolves BOTH Output nodes, in id order…
    let sinks = output_nodes(&motion.doc.graph);
    assert_eq!(sinks, outs, "every Output node is a sink, id-ordered");
    // …and cooking them draws both grids into one buffer.
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &sinks,
        0,
        0.0,
        uv,
        size,
    );
    let both = motion.pump.instances.len();
    assert!(both >= 8, "both grids drew: {both}");

    // Delete one Output -> only the survivor's scene renders.
    assert!(motion.doc.graph.remove_node(outs[1]));
    let sinks = output_nodes(&motion.doc.graph);
    assert_eq!(sinks.len(), 1);
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &sinks,
        0,
        0.0,
        uv,
        size,
    );
    assert_eq!(motion.pump.instances.len(), both / 2, "half the instances");

    // Delete the last one -> no sink -> nothing renders.
    assert!(motion.doc.graph.remove_node(outs[0]));
    let sinks = output_nodes(&motion.doc.graph);
    assert!(sinks.is_empty(), "no Output node -> no sink");
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &sinks,
        0,
        0.0,
        uv,
        size,
    );
    assert_eq!(motion.pump.instances.len(), 0, "no Output -> empty render");
}

/// M2.N2 end to end through the REAL registry + transport: a `loop_range` that
/// wraps the playhead backwards **replays the simulation from the loop start**
/// instead of showing the marching-future state. This is the reachable payoff of
/// checkpoint/restore.
///
/// It builds its OWN minimal SEQUENTIAL doc — a `pulse.beat` → `motion.strobe`
/// (both hold `pre` state) — rather than the boot scene, so it stays meaningful
/// whatever the boot scene is (the current sunflower is a pure playhead function,
/// with no `pre` state to replay). The doc is cooked exactly as the bridge cooks
/// it (`advance_or_scrub_scoped`); the marching signal is the max instance size
/// (the strobe swells it on each beat, then the glow decays).
///
/// Falsifiable: the beat-then-decay makes a beat frame bright and the tail dim
/// (`hi > lo·1.5`); a naive forward pump at the wrap would carry the dim, decayed
/// glow into the new lap — so `lap2` would match the dim tail, not replay the beat.
/// The scrub path is what makes `lap2 == lap1`.
#[test]
fn a_loop_range_replays_the_simulation_from_its_start() {
    use ph2d_eval_motion::MotionCookPump;
    use ph2d_nodegraph::cook::TimeScopes;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let state = MotionState::new(); // reuse the real registry (every op registered)
    let registry = &state.registry;
    let (uv, size) = (state.default_uv_rect, state.default_size);
    let scopes = TimeScopes::new();
    const LAP: u64 = 45; // ticks per lap

    // A minimal sequential doc: a 2×2 grid → beat → strobe → output. The beat's
    // cycle index and the strobe's glow ride `pre` self-loops, so the loop wrap has
    // real state to restore.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let beat = g.add_node("pulse.beat");
    let strobe = g.add_node("motion.strobe");
    let output = g.add_node("motion.output");
    for (from, to) in [
        ((grid, 0), (beat, 0)),
        ((grid, 0), (strobe, 0)),
        ((beat, 0), (strobe, 1)),
        ((strobe, 0), (output, 0)),
    ] {
        g.connect(Edge {
            from,
            to,
            delayed: false,
        })
        .unwrap();
    }
    for (n, port) in [(beat, 1), (strobe, 2)] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, port),
            delayed: true,
        })
        .unwrap();
    }
    g.set_param(grid, "rows", 2.0);
    g.set_param(grid, "cols", 2.0);
    g.set_param(beat, "period", 0.5); // beats at ticks 0 and 30 (within the lap)
    g.set_param(strobe, "decay", 0.85);
    g.set_param(strobe, "size_boost", 2.0);
    g.validate(registry).unwrap();
    let sinks = vec![output];

    // Cook one lap, capturing each frame's strobe silhouette (max instance size).
    let lap = |pump: &mut MotionCookPump| -> Vec<f32> {
        let mut sig = Vec::new();
        let mut transport = ph2d_motion_doc::MotionTransport {
            playing: true,
            loop_range: Some((0, LAP)),
            ..Default::default()
        };
        // Drive the wrap: `advance` moves the transport (wrapping at LAP), and the
        // pump's `advance_or_scrub` restores + re-sims on the backwards jump.
        for _ in 0..LAP {
            transport.advance(1);
            let tick = transport.tick;
            pump.advance_or_scrub_scoped(
                &g,
                registry,
                &sinks,
                tick,
                |t| t as f64 / 60.0,
                uv,
                size,
                &scopes,
            );
            sig.push(
                pump.instances
                    .iter()
                    .map(|i| i.size[0])
                    .fold(0.0_f32, f32::max),
            );
        }
        sig
    };

    let mut pump = MotionCookPump::new();
    let lap1 = lap(&mut pump); // ticks 1..=LAP  (LAP wraps to 0)
    let lap2 = lap(&mut pump); // wraps back through 0 → must replay lap1

    let hi = lap1.iter().cloned().fold(0.0_f32, f32::max);
    let lo = lap1.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        hi > lo * 1.5,
        "the strobe swell marches within a lap (bright beat vs dim tail): {lo}..{hi}"
    );
    assert_eq!(
        lap2, lap1,
        "the loop wrap replays the sim identically; a marching-future frame would diverge"
    );
}
