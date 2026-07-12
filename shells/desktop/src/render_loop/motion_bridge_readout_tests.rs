//! Guards for the inline readouts (F2). `super` is `motion_bridge::readout`.

use super::*;
use crate::motion_state::MotionState;
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
    stamp(&motion, &mut snap);
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
    stamp(&motion, &mut snap);
    let r = readout(&snap, 0).expect("the lfo cooked");
    assert!(
        r.parse::<f32>().is_ok(),
        "a value stream reads out a NUMBER, got {r:?}"
    );
    assert!(!r.contains("inst"), "…not an instance count");
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
    stamp(&motion, &mut snap);
    assert_eq!(before.as_deref(), Some("12 inst"));
    assert_eq!(readout(&snap, 0), Some("20 inst"), "5x4 after the edit");
}
