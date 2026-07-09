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

/// M2-dynamics: a wire that would close a cycle is retried as a `pre`
/// (delayed) edge — the one legal meaning of a cycle in this substrate — so a
/// user CAN hand-wire a feedback loop (spring state, force loop). Reverting
/// the retry makes the connect refuse and this goes red.
#[test]
fn a_cycle_closing_connect_becomes_a_pre_edge() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let spring = g.add_node("motion.spring");
    g.connect(Edge {
        from: (grid, 0),
        to: (spring, 0),
        delayed: false,
    })
    .unwrap();
    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();

    // spring.out -> spring.state closes a (self-)cycle: the shell converts it.
    apply_connect(&mut motion, &mut toasts, spring.0, 0, spring.0, 1);
    let edge = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (spring, 1))
        .expect("the feedback wire was created, not refused");
    assert!(edge.delayed, "cycle-closing wire became a pre edge");

    // A plain forward wire stays plain (no over-eager delaying).
    let plain = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to == (spring, 0))
        .unwrap();
    assert!(!plain.delayed);
}

/// Sequential-node template via the plumbing reconcile (docs/Motion Nodes/03):
/// a feedback host (`forces` on integrate, `state` on spring) lands with its
/// `pre` self-loop already plumbed; a plain modifier is untouched.
#[test]
fn add_node_template_wires_the_state_self_loop() {
    use ph2d_nodegraph::graph::Graph;

    let motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let spring = g.add_node("motion.spring");
    let _plain = g.add_node("motion.move");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1) && e.delayed),
        "integrate got its pre self-loop on the forces port"
    );
    assert!(
        g.edges()
            .iter()
            .any(|e| e.from == (spring, 0) && e.to == (spring, 1) && e.delayed),
        "spring got its pre self-loop"
    );
    assert_eq!(
        g.edges().len(),
        2,
        "a node with no feedback port is untouched by the template"
    );
    // Idempotent: a second pass changes nothing.
    let snapshot = g.clone();
    plumbing::reconcile_after(&mut g, &motion.registry, &snapshot);
    assert_eq!(g.edges().len(), 2);
}

/// The O1 authoring flow (docs/Motion Nodes/03): the user wires a force chain
/// into integrate's `forces` port and NEVER draws the state loop — the bridge
/// clears the self-loop, lands the forward wire, and re-plumbs the `pre` into
/// the chain's dangling head. Pulling the chain off restores the self-loop and
/// sweeps the branch plumbing. Falsified by construction: skip `apply_connect`
/// (raw `Graph::connect`) and the forces port is occupied by the self-loop, so
/// the wire is refused — the plumbing IS what makes the gesture land.
#[test]
fn wiring_a_chain_into_forces_replumbs_the_state_entry() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, EdgeError, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    let curl = g.add_node("force.curl");
    let drag = g.add_node("force.drag");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    for (from, to) in [(wind, curl), (curl, drag)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .unwrap();
    }

    // Control: WITHOUT the bridge, the self-looped forces port refuses the wire.
    let mut raw = g.clone();
    assert_eq!(
        raw.connect(Edge {
            from: (drag, 0),
            to: (integrate, 1),
            delayed: false,
        }),
        Err(EdgeError::InputAlreadyConnected),
        "the self-loop occupies forces; only the bridge gesture can land there"
    );

    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();
    apply_connect(&mut motion, &mut toasts, drag.0, 0, integrate.0, 1);

    let edges = motion.doc.graph.edges();
    assert!(
        edges
            .iter()
            .any(|e| e.from == (drag, 0) && e.to == (integrate, 1) && !e.delayed),
        "the chain landed on forces as a plain forward wire"
    );
    assert!(
        !edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1)),
        "the self-loop is gone once a chain feeds forces"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (wind, 0) && e.delayed),
        "the state entry re-plumbed to the chain's dangling head"
    );

    // Pull the chain off `forces`: plumbing sweeps the branch and restores the
    // self-loop (the host stays alive, ballistic).
    apply_disconnect(&mut motion, &mut toasts, integrate.0, 1);
    let edges = motion.doc.graph.edges();
    assert!(
        !edges.iter().any(|e| e.to == (wind, 0)),
        "the branch's state entry was swept with the chain"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (integrate, 1) && e.delayed),
        "the forces port self-loops again"
    );
}

/// Deleting a mid-chain force re-heals the plumbing: the orphaned upstream
/// stub loses its state entry, the surviving chain's new dangling head gains
/// it. And the managed badge itself is not hand-deletable (the disconnect
/// gesture on it is refused with a toast) while an EXPERT `pre` elsewhere
/// still is.
#[test]
fn plumbing_reheals_on_delete_and_managed_pre_is_not_hand_deletable() {
    use ph2d_editor::ToastQueue;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let before = g.clone();
    let integrate = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    let curl = g.add_node("force.curl");
    let drag = g.add_node("force.drag");
    plumbing::reconcile_after(&mut g, &motion.registry, &before);
    for (from, to) in [(wind, curl), (curl, drag)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .unwrap();
    }
    motion.doc.graph = g;
    let mut toasts = ToastQueue::new();
    apply_connect(&mut motion, &mut toasts, drag.0, 0, integrate.0, 1);

    // The managed badge refuses the delete gesture...
    let before_len = motion.doc.graph.edges().len();
    apply_disconnect(&mut motion, &mut toasts, wind.0, 0);
    assert_eq!(
        motion.doc.graph.edges().len(),
        before_len,
        "the managed state entry is not hand-deletable"
    );

    // ...while deleting the mid-chain node re-heals: wind's stale entry is
    // swept, drag (the new dangling head) gains one.
    apply_delete_selection(&mut motion, vec![curl.0]);
    let edges = motion.doc.graph.edges();
    assert!(
        !edges.iter().any(|e| e.to.0 == curl || e.from.0 == curl),
        "the deleted node's edges died with it"
    );
    assert!(
        !edges.iter().any(|e| e.to == (wind, 0)),
        "the orphaned stub lost its state entry"
    );
    assert!(
        edges
            .iter()
            .any(|e| e.from == (integrate, 0) && e.to == (drag, 0) && e.delayed),
        "the surviving chain's new head gained the state entry"
    );
}

/// M2-dynamics gate: the default demo's physics loop is ALIVE through the real
/// registry (integrate accumulates displacement in the focus region) and the
/// whole trajectory replays bit-identically — cooked one pump per fixed tick,
/// exactly like the bridge. Breaking the pre wiring, the accel consumption or
/// any force's determinism turns this red.
#[test]
fn demo_dynamics_swirl_and_replay_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    use ph2d_nodegraph::attr::Column;

    let run = || {
        let motion = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..40u64 {
            let ph = k as f64 / 60.0;
            pump.pump(
                &motion.doc.graph,
                &motion.registry,
                &motion.sinks,
                k,
                ph,
                motion.default_uv_rect,
                motion.default_size,
            );
            frames.push(
                pump.instances
                    .iter()
                    .map(|i| i.world_pos)
                    .collect::<Vec<_>>(),
            );
        }
        (motion, pump, frames)
    };

    let (motion, mut pump, frames_a) = run();
    // The force loop displaced SOMETHING: integrate's sim_d is non-zero after
    // 40 ticks (the falloff gates the edges, so check the max magnitude).
    let integrate = motion
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.integrate")
        .map(|n| n.id)
        .expect("demo has an integrate node");
    let ph = 39.0 / 60.0;
    let out = pump
        .cook
        .cook(&motion.doc.graph, &motion.registry, integrate, ph)
        .unwrap();
    let max_d = match out[0].as_stream().get("sim_d").expect("sim_d column") {
        Column::Vec2(v) => v
            .iter()
            .map(|d| (d[0] * d[0] + d[1] * d[1]).sqrt())
            .fold(0.0f32, f32::max),
        _ => panic!("sim_d"),
    };
    assert!(
        max_d > 1e-3,
        "the physics loop accumulated displacement, got {max_d}"
    );

    // Bit-identical replay (HR-5): a second full run matches frame for frame.
    let (_, _, frames_b) = run();
    assert_eq!(frames_a, frames_b, "the demo trajectory replays exactly");
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
