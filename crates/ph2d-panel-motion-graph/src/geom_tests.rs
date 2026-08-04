//! Unit tests for [`super`] (`geom.rs`) — extracted to a sibling (`#[path]`) so geom.rs stays under the panel LOC cap. Pure relocation of the `#[cfg(test)] mod tests` block.
use super::*;
use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView};
use crate::state::Menu;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

fn node_with_inputs(id: u32, x: f32, n_in: usize) -> GraphNodeView {
    GraphNodeView {
        kind: crate::snapshot::NodeViewKind::Node,
        id,
        display_name: "n".into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y: 0.0,
        inputs: (0..n_in)
            .map(|_| PortView {
                name: "i",
                domain: Domain::Instances,
                dim: Dim::Scalar,
                clock: Clock::Frame,
            })
            .collect(),
        outputs: vec![],
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
    }
}

fn one_node(node: GraphNodeView) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![node],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    }
}

fn node_with_outputs(id: u32, x: f32, n_out: usize) -> GraphNodeView {
    let mut n = node_with_inputs(id, x, 0);
    n.outputs = (0..n_out)
        .map(|_| PortView {
            name: "o",
            domain: Domain::Instances,
            dim: Dim::Scalar,
            clock: Clock::Frame,
        })
        .collect();
    n
}

#[test]
fn nearest_input_socket_resolves_the_right_port() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    // Input 0 center: (100, HEADER_H + ROW_H*0.5) = (100, 37).
    let (n0, p0) = socket_center(&snap.nodes[0], &view, false, 0);
    assert_eq!(nearest_input_socket(&snap, &view, n0, p0), Some((7, 0)));
    // Input 1 center: (100, HEADER_H + ROW_H*1.5) = (100, 59).
    let (n1, p1) = socket_center(&snap.nodes[0], &view, false, 1);
    assert_eq!(nearest_input_socket(&snap, &view, n1, p1), Some((7, 1)));
    // Far from any socket → no snap.
    assert_eq!(nearest_input_socket(&snap, &view, 400.0, 400.0), None);
}

/// **A near-miss SNAPS to the socket** — a drop 15 px above input 0 (past the 9 px exact
/// hit box, inside the 22 px magnet) still lands on it. FALSIFIED by shrinking `SNAP_R` to
/// `SOCKET_HIT_R`: 15 > 9, so the near-miss resolves to nothing and the wire is dropped.
#[test]
fn a_near_miss_snaps_to_the_socket() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    // Input 0 center (100, 37); 15 px above it — only input 0 is within reach.
    assert_eq!(
        nearest_input_socket(&snap, &view, 100.0, 22.0),
        Some((7, 0))
    );
}

/// Between two in-range sockets the magnet takes the **nearer** one — a drop at y=54 is
/// 17 px from input 0 and 5 px from input 1. FALSIFIED by returning the first socket found
/// rather than the closest: input 0 (found first, still in range) would win.
#[test]
fn snap_takes_the_nearest_of_two_sockets() {
    let snap = one_node(node_with_inputs(7, 100.0, 2));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert_eq!(
        nearest_input_socket(&snap, &view, 100.0, 54.0),
        Some((7, 1))
    );
}

/// The output magnet reads **output** sockets (right edge), not inputs — the mirror used by
/// the backward wire drag. FALSIFIED by resolving against the input side, which this node
/// does not have.
#[test]
fn the_output_magnet_reads_output_sockets() {
    let snap = one_node(node_with_outputs(9, 100.0, 1));
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let (ox, oy) = socket_center(&snap.nodes[0], &view, true, 0);
    assert_eq!(
        nearest_output_socket(&snap, &view, ox + 12.0, oy),
        Some((9, 0))
    );
    assert_eq!(nearest_input_socket(&snap, &view, ox + 12.0, oy), None);
}

/// **A readout makes the card taller — and the HIT geometry grows with it.**
///
/// `card_h` is the one source of truth for both the paint and the hit-test, which is
/// why the readout's row is added here and not in `paint`. FALSIFIED by adding the row
/// only where the card is drawn: the card would gain a dead strip along its bottom edge
/// — a place that looks like the node and does not answer the mouse.
#[test]
fn a_readout_grows_the_card_and_its_hit_rect_together() {
    let bare = node_with_inputs(1, 0.0, 2);
    let mut with_readout = node_with_inputs(2, 0.0, 2);
    with_readout.readout = Some("12 inst".into());

    assert_eq!(
        card_h(&with_readout) - card_h(&bare),
        ROW_H,
        "one row taller"
    );

    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let (bare_r, read_r) = (card_rect(&bare, &view), card_rect(&with_readout, &view));
    assert_eq!(read_r.h - bare_r.h, ROW_H, "…and so is what the mouse hits");

    // A point in the readout's own row is INSIDE the card that has one, and outside the
    // card that does not. That is the strip that would have gone dead.
    let y = bare_r.y + bare_r.h + ROW_H * 0.5;
    let band = Rect::new(0.0, y - 1.0, 10.0, 2.0);
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![bare, with_readout],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    };
    assert_eq!(
        nodes_in_box(&snap, &view, band),
        vec![2],
        "only the taller card reaches down into that row"
    );
}

/// A **preview-CAPABLE** node: its primary output is a positional motion stream
/// (`Instances`/`Vec2`, the `P` column the stamp scatters). This is what owns a moldura by TYPE,
/// whatever it emits this frame (doc 86 B1) — the fixture MUST carry the output, not just content,
/// or it does not contain the phenomenon.
fn positional(id: u32, x: f32) -> GraphNodeView {
    let mut n = node_with_inputs(id, x, 2);
    n.outputs = vec![PortView {
        name: "P",
        domain: Domain::Instances,
        dim: Dim::Vec2,
        clock: Clock::Frame,
    }];
    n
}

fn stamped(id: u32, x: f32) -> GraphNodeView {
    let mut n = positional(id, x);
    n.preview = Some(vec![[0.0, 0.0], [1.0, 1.0]]);
    n
}

/// **The preview left the card** (doc 86): a node WITH a stamp is no taller than one
/// without — the in-card strip is gone. FALSIFIED by `card_h` still reserving the preview
/// band (the card would keep a dead strip where the stamp used to sit).
#[test]
fn the_card_no_longer_reserves_in_card_preview_height() {
    assert_eq!(
        card_h(&stamped(2, 0.0)),
        card_h(&node_with_inputs(1, 0.0, 2)),
        "the stamp no longer lives in the card"
    );
}

/// **The moldura sits OUTSIDE the card, above or below by position** (doc 86). Below → under
/// the card bottom; Above → over the header; neither overlaps the card rect, and `pos` moves
/// it. FALSIFIED by a frame drawn inside the body, or by ignoring `pos`.
#[test]
fn the_preview_frame_is_external_above_or_below() {
    let n = stamped(1, 100.0);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let card = card_rect(&n, &view);
    let below = preview_frame_rect(&n, &view, PreviewPos::Below).unwrap();
    let above = preview_frame_rect(&n, &view, PreviewPos::Above).unwrap();
    assert!(below.y >= card.y + card.h, "below sits under the card");
    assert!(above.y + above.h <= card.y, "above sits over the card");
    assert!(
        preview_frame_rect(&node_with_inputs(2, 0.0, 2), &view, PreviewPos::Below).is_none(),
        "no positional output, no frame"
    );
}

/// **B1: the moldura survives an empty stream** (doc 86 §10). `motion.emitter` is stateless, so
/// its live count crosses zero and `preview` winks to `None` — but its output PORT stays `Vec2`,
/// so the frame AND its position toggle must stay put (the flicker was tying their existence to
/// the content). FALSIFIED by keying the frame off `n.preview` again: the empty tick returns
/// `None` and the whole moldura blinks Some↔None frame to frame.
#[test]
fn the_preview_moldura_survives_an_empty_stream() {
    let mut n = positional(1, 100.0);
    n.preview = None; // this tick the stateless emitter emitted nothing
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert!(
        preview_frame_rect(&n, &view, PreviewPos::Below).is_some(),
        "a preview-capable node keeps its frame even with no content this tick"
    );
    assert!(
        preview_toggle_rect(&n, &view).is_some(),
        "and keeps its position toggle"
    );
}

/// **The moldura does not move when the content winks** (doc 86 B1 — the measured invariant of the
/// fix). The SAME positional node, once empty and once full, yields the byte-identical frame rect:
/// the box neither vanishes nor shifts as the stateless emitter's count crosses zero. FALSIFIED by
/// any dependence of the frame's GEOMETRY on `preview`.
#[test]
fn the_preview_moldura_is_identical_whether_empty_or_full() {
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let mut empty = positional(1, 100.0);
    empty.preview = None;
    let mut full = positional(1, 100.0);
    full.preview = Some(vec![[0.0, 0.0], [1.0, 1.0]]);
    for pos in [PreviewPos::Below, PreviewPos::Above] {
        assert_eq!(
            preview_frame_rect(&empty, &view, pos),
            preview_frame_rect(&full, &view, pos),
            "the frame is the same box empty or full"
        );
    }
    assert_eq!(
        preview_toggle_rect(&empty, &view),
        preview_toggle_rect(&full, &view),
        "and so is its position toggle"
    );
}

/// **A value node grows no moldura** (doc 86). Its output is `Scalar` — its stamp is the number
/// it already shows, and an empty box under it would promise a picture that is not coming. The
/// slot is a fact of the TYPE, so even a value node carrying stray `preview` content gets none.
/// FALSIFIED by offering a frame on every card (`has_preview_slot` always true).
#[test]
fn a_value_node_has_no_preview_moldura() {
    // `node_with_outputs` gives `Scalar` outputs — a value node.
    let mut n = node_with_outputs(2, 0.0, 1);
    n.preview = Some(vec![[0.0, 0.0]]); // even with stray content, a value has no slot
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    assert!(preview_frame_rect(&n, &view, PreviewPos::Below).is_none());
    assert!(preview_toggle_rect(&n, &view).is_none());
}

/// **The header toggle exists only with a preview, and lives in the header** (doc 86). No
/// stamp → nothing to reposition → no button. FALSIFIED by a button offered on every card.
#[test]
fn the_toggle_lives_in_the_header_only_with_a_preview() {
    let n = stamped(1, 100.0);
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let btn = preview_toggle_rect(&n, &view).expect("a stamped node has the toggle");
    let (hx, hy) = view.pt(n.x, n.y);
    assert!(
        btn.y >= hy && btn.y + btn.h <= hy + HEADER_H,
        "inside the header band"
    );
    assert!(
        btn.x + btn.w <= hx + CARD_W && btn.x >= hx,
        "at the header's right, inside the card"
    );
    assert!(
        preview_toggle_rect(&node_with_inputs(2, 0.0, 2), &view).is_none(),
        "no positional output, no toggle"
    );
}

#[test]
fn menu_panel_clamps_into_the_canvas() {
    let canvas = Rect::new(0.0, 0.0, 300.0, 200.0);
    // Opened past the right/bottom edge → clamped fully inside.
    let menu = Menu {
        scroll: 0.0,
        screen: (290.0, 190.0),
        spawn: (0.0, 0.0),
        // Any body — clamping is geometry, not content. (The node library moved to the shell
        // palette; the local popups left are read by eye.)
        body: crate::state::MenuBody::NodeActions {
            multi: false,
            group: false,
        },
    };
    let p = menu_panel(&menu, 3, canvas);
    assert!(p.x + p.w <= canvas.x + canvas.w + 0.01);
    assert!(p.y + p.h <= canvas.y + canvas.h + 0.01);
    assert!(p.x >= canvas.x && p.y >= canvas.y);
}
