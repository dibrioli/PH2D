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
    assert_eq!(motion.playhead(dt), 0.0, "starts paused at t=0");
    motion.transport.play();
    motion.transport.advance(30);
    let t = motion.playhead(dt);
    assert!(
        t > 0.0,
        "playing advances the playhead -> behaviours animate"
    );
    motion.transport.toggle(); // -> paused
    motion.transport.advance(30);
    assert_eq!(motion.playhead(dt), t, "paused freezes the playhead");
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

/// The Output node IS the render sink: the bridge auto-selects it, cooking it
/// draws whatever feeds it, and deleting it stops the render (no Output -> no
/// sink -> empty). The output follows the graph, not a hidden toggle.
#[test]
fn output_node_is_the_render_sink() {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut motion = MotionState::new();
    let (uv, size) = (motion.default_uv_rect, motion.default_size);

    // Fresh graph: grid -> Output.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .unwrap();
    motion.doc.graph = g;

    // The bridge resolves the sink to the Output node…
    let sink = output_node(&motion.doc.graph);
    assert_eq!(sink, Some(out), "the Output node is the render sink");
    // …and cooking it draws the grid cells feeding it.
    motion.pump.mark_dirty();
    motion
        .pump
        .pump(&motion.doc.graph, &motion.registry, sink, 0, 0.0, uv, size);
    assert!(
        motion.pump.instances.len() >= 4,
        "Output renders whatever feeds it"
    );

    // Delete the Output node -> no sink -> nothing renders.
    assert!(motion.doc.graph.remove_node(out));
    let sink = output_node(&motion.doc.graph);
    assert_eq!(sink, None, "no Output node -> no sink");
    motion.pump.mark_dirty();
    motion
        .pump
        .pump(&motion.doc.graph, &motion.registry, sink, 0, 0.0, uv, size);
    assert_eq!(motion.pump.instances.len(), 0, "no Output -> empty render");
}
