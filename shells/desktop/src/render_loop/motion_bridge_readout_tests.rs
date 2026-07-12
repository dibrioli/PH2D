//! Guards for the inline readouts (F2). `super` is `motion_bridge::readout`.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::{Edge, Graph, Pos};

/// Build `grid -> output` plus an ORPHAN node wired to nothing, cook the sink exactly as
/// the shell does, and read the snapshot the panel would receive.
fn cooked(pump_ticks: usize) -> (MotionState, ph2d_panel_motion_graph::GraphViewSnapshot) {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    let orphan = g.add_node("motion.grid"); // wired to nothing at all
    let value = g.add_node("value.lfo"); // a VALUE stream, also orphaned
    for (n, x) in [(grid, 0.0), (out, 200.0), (orphan, 0.0), (value, 0.0)] {
        g.set_pos(n, Pos { x, y: 0.0 });
    }
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("wire");
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    motion.doc.graph = g;
    motion.sinks = vec![out];

    for k in 0..pump_ticks {
        motion.pump.pump(
            &motion.doc.graph,
            &motion.registry,
            &motion.sinks,
            k as u64,
            k as f64 / 60.0,
            motion.default_uv_rect,
            motion.default_size,
        );
    }
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    stamp(&mut motion, &mut snap);
    (motion, snap)
}

fn readout(snap: &ph2d_panel_motion_graph::GraphViewSnapshot, i: usize) -> Option<&str> {
    snap.nodes[i].readout.as_deref()
}

/// **A cooked node reads out; an ORPHAN reads out NOTHING** — and the blank is the point.
///
/// The grid feeding the Output shows its instance count. The identical grid wired to
/// nothing shows nothing at all, because the cook never pulled it — which is exactly the
/// fact an artist needs (a card with no reading is a card nobody consumes).
///
/// FALSIFIED by a readout built with `cook()` instead of `peek()`: the orphan would happily
/// evaluate and report `12 inst`, the diagnosis would vanish, and the editor would be
/// paying for a second full evaluation of the graph every frame to hide it.
#[test]
fn a_cooked_node_reads_out_and_an_orphan_stays_blank() {
    let (_, snap) = cooked(1);
    assert_eq!(readout(&snap, 0), Some("12 inst"), "3x4 = 12 instances");
    assert_eq!(
        readout(&snap, 1),
        Some("12 inst"),
        "the Output passes them on"
    );
    assert_eq!(
        readout(&snap, 2),
        None,
        "the orphaned grid was never cooked — and says so by saying nothing"
    );
    assert_eq!(readout(&snap, 3), None, "…so is the orphaned value node");
}

/// A VALUE stream reads out its SCALAR, not a count — the same two questions the probe
/// answers ("what is it worth?" / "how many are there?"), answered the same way, so the two
/// readings can never contradict each other.
#[test]
fn a_value_stream_reads_out_its_number_not_its_count() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let lfo = g.add_node("value.lfo");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (lfo, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("wire");
    motion.doc.graph = g;
    motion.sinks = vec![out];
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        0,
        0.0,
        motion.default_uv_rect,
        motion.default_size,
    );

    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    stamp(&mut motion, &mut snap);
    let r = readout(&snap, 0).expect("the lfo cooked");
    assert!(
        r.parse::<f32>().is_ok(),
        "a value stream reads out a NUMBER, got {r:?}"
    );
    assert!(!r.contains("inst"), "…not an instance count");
}

// ── F3: the mass, the sink flag, and what is FLOWING (doc 46) ────────────────

/// Cook `grid -> drive <- lfo -> output` at time `t` and stamp — the same two calls the shell
/// makes, in the same order, so the digest sees exactly what the panel will draw.
fn flow_scene() -> (MotionState, [ph2d_nodegraph::graph::NodeId; 4]) {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid"); // 12 instances, the SAME 12 every frame
    let lfo = g.add_node("value.lfo"); // one number, and it MOVES with the clock
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");
    let value_port = motion
        .registry
        .resolve(ph2d_nodegraph::node::NodeTypeId::of("motion.drive"))
        .expect("drive is registered")
        .manifest()
        .inputs
        .iter()
        .position(|p| p.ty.dim == ph2d_nodegraph::port::Dim::Scalar)
        .expect("drive takes a value") as u16;
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 4.0);
    for (from, fp, to, tp) in [
        (grid, 0u16, drive, 0u16),
        (lfo, 0, drive, value_port),
        (drive, 0, out, 0),
    ] {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed: false,
        })
        .expect("wire");
    }
    motion.doc.graph = g;
    motion.sinks = vec![out];
    (motion, [grid, lfo, drive, out])
}

fn frame(
    motion: &mut MotionState,
    tick: u64,
    t: f64,
) -> ph2d_panel_motion_graph::GraphViewSnapshot {
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        tick,
        t,
        motion.default_uv_rect,
        motion.default_size,
    );
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    stamp(motion, &mut snap);
    snap
}

fn hot(
    snap: &ph2d_panel_motion_graph::GraphViewSnapshot,
    id: ph2d_nodegraph::graph::NodeId,
) -> bool {
    snap.nodes
        .iter()
        .find(|n| n.id == id.0)
        .expect("in view")
        .hot
}

/// **A wire runs hot when its VALUE moved — not when its size did.**
///
/// The `lfo` emits exactly one number every frame, forever; what changes is *the number*. The
/// grid emits the same twelve instances, forever. So the lfo's wire is the one with data
/// running through it, and the grid's is wired, alive, and perfectly still.
///
/// FALSIFIED by a digest over the element COUNT (or over the column NAMES, or the stream's
/// shape): a 400-instance grid being swung around by an oscillator is 400 instances every
/// frame, and it is the most alive thing on the canvas. That digest would call it cold and the
/// marching dashes would appear on nothing that matters.
#[test]
fn a_wire_runs_hot_when_its_value_moved_not_when_its_size_did() {
    let (mut motion, [grid, lfo, drive, out]) = flow_scene();

    // FIRST frame: there is no "last frame" to differ from, so NOTHING marches. A graph that
    // flashed its whole canvas on load would be crying wolf on frame one.
    let first = frame(&mut motion, 0, 0.0);
    assert!(
        !first.nodes.iter().any(|n| n.hot),
        "a graph does not flash on its first frame"
    );

    // SECOND frame, a quarter second on: the clock moved, so the lfo moved, so everything
    // downstream of it moved — and the grid did not.
    let s = frame(&mut motion, 15, 0.25);
    assert!(hot(&s, lfo), "the oscillator's number changed");
    assert!(hot(&s, drive), "…so what it drives changed");
    assert!(hot(&s, out), "…so the scene changed");
    assert!(
        !hot(&s, grid),
        "the grid emits the same 12 instances every frame: wired, alive, and STILL"
    );
}

/// A node the cook never pulled is **never hot** — no data flows through a wire nothing
/// consumes, and a dead branch flickering with dashes would be the loudest lie on the canvas.
#[test]
fn an_inert_node_never_marches() {
    let (mut motion, _) = flow_scene();
    let orphan = motion.doc.graph.add_node("value.lfo".to_string()); // wired to nothing
    frame(&mut motion, 0, 0.0);
    let s = frame(&mut motion, 15, 0.25);
    assert!(
        !hot(&s, orphan),
        "it reads the clock, but nothing reads IT: no flow"
    );
    assert!(
        !motion.flow_digest.contains_key(&orphan.0),
        "and it holds no digest at all"
    );
}

/// The wire's WIDTH comes from the mass of the stream, and the panel's reachability walk needs
/// to know where the sinks are — both stamped from the same memo lookup as the readout.
#[test]
fn the_mass_and_the_sink_flag_ride_along() {
    let (mut motion, [grid, lfo, _, out]) = flow_scene();
    let s = frame(&mut motion, 0, 0.0);
    let node =
        |id: ph2d_nodegraph::graph::NodeId| s.nodes.iter().find(|n| n.id == id.0).expect("in view");
    assert_eq!(node(grid).count, Some(12), "3x4 instances of mass");
    assert_eq!(node(lfo).count, Some(1), "a value is a thread");
    assert!(node(out).is_sink, "the Output is where the cook pulls from");
    assert!(!node(grid).is_sink);
}

/// The readout follows the live cook: it changes as the graph animates, and it never costs
/// a cook of its own (a stale reading would mean `peek` is looking at the wrong lane).
#[test]
fn the_readout_tracks_the_frame_it_was_taken_on() {
    let (motion, snap) = cooked(1);
    let before = readout(&snap, 0).map(str::to_owned);

    // Re-cook with a bigger grid: the reading must follow the DOCUMENT, not a cached
    // string from the last frame.
    let mut motion = motion;
    let grid = motion.doc.graph.nodes()[0].id;
    motion.doc.graph.set_param(grid, "rows", 5.0);
    motion.pump.mark_dirty();
    motion.pump.pump(
        &motion.doc.graph,
        &motion.registry,
        &motion.sinks,
        1,
        1.0 / 60.0,
        motion.default_uv_rect,
        motion.default_size,
    );
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    stamp(&mut motion, &mut snap);
    assert_eq!(before.as_deref(), Some("12 inst"));
    assert_eq!(readout(&snap, 0), Some("20 inst"), "5x4 after the edit");
}
