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

// ── W4.T7: the editor has ONE clock ────────────────────────────────────────
//
// Motion used to run a `MotionTransport` of its own, advanced by the frame's fixed
// steps, while the timeline ran `ph2d_core::Playhead`. Two clocks that each advance
// themselves are two clocks that drift — and every feature crossing Motion and the
// timeline was built on that sand. The transport is gone: the tick Motion cooks is
// now DERIVED from the playhead, so they cannot disagree by construction.

/// The animation enabler (ask "when do we see animation?"): the playhead advances
/// — so any `Temporal` behaviour moves — and pausing freezes it. Now asserted on
/// the ONE clock, because Motion no longer has one to assert on.
#[test]
fn the_cook_clock_is_the_playhead_and_pausing_freezes_it() {
    let dt = 1.0 / 60.0;
    let mut ph = ph2d_core::Playhead::new(dt);
    ph.pause();
    assert_eq!(super::motion_tick(&ph, dt), 0, "paused at t = 0 -> tick 0");

    ph.play();
    ph.advance_ticks(30);
    assert_eq!(
        super::motion_tick(&ph, dt),
        30,
        "playing advances the cook's clock -> behaviours animate"
    );

    ph.pause();
    ph.advance_ticks(30);
    assert_eq!(super::motion_tick(&ph, dt), 30, "paused freezes it");

    // …and the timeline's ruler now moves the graph, for free: there is only one
    // thing to move. This is what the second transport made impossible.
    ph.seek(1.0);
    assert_eq!(super::motion_tick(&ph, dt), 60, "a seek to 1 s is tick 60");
}

/// A playhead at double rate covers two fixed ticks per frame — the cook's clock
/// must follow it there, not run at its own pace.
#[test]
fn the_cook_clock_follows_the_playheads_rate() {
    let dt = 1.0 / 60.0;
    let mut ph = ph2d_core::Playhead::new(dt);
    ph.set_rate(2.0);
    ph.advance_ticks(10);
    assert_eq!(super::motion_tick(&ph, dt), 20, "rate 2 -> twice the ticks");
}

/// **The determinism guard.** A sequential node's trajectory (integrate / spring /
/// verlet) is the SUM of its steps, so the cook may never SKIP a tick — a slow
/// frame that produced three fixed steps owes all three, or the motion would depend
/// on the frame rate. A jump backwards is the opposite: one call, so the pump
/// restores a checkpoint and re-sims (walking it tick by tick would re-cook from
/// the ring on every step).
#[test]
fn a_slow_frame_owes_every_tick_it_skipped_and_a_jump_owes_one() {
    let owed = |last, target| super::ticks_owed(last, target).collect::<Vec<u64>>();

    assert_eq!(owed(Some(10), 11), vec![11], "the common frame: one tick");
    assert_eq!(
        owed(Some(10), 13),
        vec![11, 12, 13],
        "a slow frame sims EVERY step it owes -- a spring may not skip one"
    );
    assert_eq!(
        owed(Some(10), 5),
        vec![5],
        "backwards (a scrub, a loop wrap) is ONE call -> restore + re-sim"
    );
    assert_eq!(
        owed(Some(10), 10),
        vec![10],
        "standing still re-issues the tick (a dirty param edit re-cooks it)"
    );
    assert_eq!(owed(None, 0), vec![0], "a fresh pump starts at tick 0");
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
        // Drive the wrap directly on the tick, the way the shell now does (W4.T7:
        // there is no MotionTransport any more — the tick is derived from the ONE
        // playhead, and a looping playhead wraps it backwards). The pump's
        // `advance_or_scrub` restores + re-sims on that backwards jump.
        let mut tick = 0u64;
        for _ in 0..LAP {
            tick = (tick + 1) % LAP;
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
