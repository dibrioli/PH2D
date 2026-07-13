//! Graph-canvas hit rects (split from `paint` for the panel LOC cap).
//!
//! Builds the `GraphSurface` hit rects the M0 dispatch routes gestures through
//! — background, wires, node bodies, sockets — and registers them.
//!
//! **Every content hit rect is clipped to the canvas.** The graph draws inside a
//! Vello clip layer (`paint`), so content the user panned/zoomed past the panel
//! edge is invisible; an unclipped hit rect for it would stay clickable — an
//! invisible target floating over the scene, stealing the click from the
//! viewport. Clipping the geometry and the hit rects from the same `canvas`
//! keeps "what you see" and "what you can click" the same set.

use crate::geom::{self, View, socket_center};
use crate::paint::{fnv_id, wire_handle, wire_hit_polyline};
use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GraphHitKind, InteractiveState};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;

/// How many polyline points a wire's hit path is sampled at (segments = N-1);
/// each segment becomes one padded hit rect sharing the wire's id. Denser than
/// the draw sampling so the padded segment boxes hug the curve (fewer gaps /
/// off-curve false hits) now that the band is wider.
/// Half-thickness (screen px) of a wire's hit rects — a forgiving hover / alt-
/// click zone (a full band of `2 × WIRE_HIT_R`, independent of zoom).
pub(crate) const WIRE_HIT_R: f32 = 8.0; // LITERAL-PX-OK: wire hit half-thickness

// Portal-badge geometry for `pre` edges — shared by paint and the hit path so
// the clickable thing is exactly the visible thing (docs/Motion Nodes/03: a
// `pre` edge draws as badges at its sockets, never as a spline).
pub(crate) const PRE_BADGE_R: f32 = 6.0; // LITERAL-PX-OK: portal badge outer radius
pub(crate) const PRE_BADGE_OFF: f32 = 15.0; // LITERAL-PX-OK: badge centre offset out from its socket
const PRE_BADGE_DROP: f32 = 11.0; // LITERAL-PX-OK: source badge drop below the socket line
const PRE_BADGE_HIT_PAD: f32 = 3.0; // LITERAL-PX-OK: extra hit slop around a badge

/// The badge centre(s) standing in for a `pre` edge's spline: one just outside
/// the source's output socket, one just outside the target's input socket. A
/// self-loop (the sequential-node template) collapses to the input-side badge
/// alone — one glyph on the card, not a hairpin.
///
/// The source badge drops BELOW the socket line: the same output usually also
/// feeds a forward wire (integrate → tint), and a badge sitting on that
/// spline would read as a bead on the wrong edge. The target badge stays on
/// the line — a plumbed head has no other incoming wire by construction.
pub(crate) fn pre_badge_centers(
    p0: (f32, f32),
    p3: (f32, f32),
    zoom: f32,
    self_loop: bool,
) -> [Option<(f32, f32)>; 2] {
    let src = (p0.0 + PRE_BADGE_OFF * zoom, p0.1 + PRE_BADGE_DROP * zoom);
    let dst = (p3.0 - PRE_BADGE_OFF * zoom, p3.1);
    [(!self_loop).then_some(src), Some(dst)]
}

/// `r` intersected with `canvas`, or `None` when they do not overlap. The clip
/// that hides content past the panel edge must hide its hit rect too (see the
/// module docs).
pub(crate) fn clip_rect(r: Rect, canvas: Rect) -> Option<Rect> {
    let x0 = r.x.max(canvas.x);
    let y0 = r.y.max(canvas.y);
    let x1 = (r.x + r.w).min(canvas.x + canvas.w);
    let y1 = (r.y + r.h).min(canvas.y + canvas.h);
    (x1 > x0 && y1 > y0).then(|| Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// Push a node body's hit rect, clipped to the canvas (skipped when fully off).
///
/// **A ghost registers no body** (doc 57): it is a node from OUTSIDE this level,
/// drawn only because a wire crosses to it, and a click-through body is what makes
/// it read as what it is. Dragging it would move a node on a canvas the artist is
/// not looking at; deleting it would reach across the boundary. Its **sockets** are
/// registered all the same (`push_socket_hits`) — the whole point of drawing it is
/// that you can wire across the seam.
pub(crate) fn push_card_hit(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    body: Rect,
    canvas: Rect,
) {
    if n.kind == crate::snapshot::NodeViewKind::Ghost {
        return;
    }
    if let Some(r) = clip_rect(body, canvas) {
        hits.push((
            node_hit_id(n.id),
            GraphHitKind::Node { node: n.id as u64 },
            r,
        ));
    }
}

/// Push a backdrop's hit rects: its **header** (select + drag the group) and its
/// two bottom-corner **grippers** (resize — either corner, like every panel in the
/// app), all clipped to the canvas. The BODY is deliberately not registered — it
/// must stay click-through so the nodes it frames, and the empty canvas around
/// them, keep receiving clicks and box-selects (see `backdrop`'s module docs). The
/// grippers are pushed after the header so they win the corners they overlap it in
/// a short region; all three lose to the wires and cards registered after them.
pub(crate) fn push_backdrop_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    b: &crate::snapshot::GraphBackdropView,
    view: &View,
    canvas: Rect,
) {
    if let Some(r) = clip_rect(crate::backdrop::header_rect(b, view), canvas) {
        hits.push((
            backdrop_hit_id(b.id, "header"),
            GraphHitKind::Backdrop { id: b.id as u64 },
            r,
        ));
    }
    let (bl, br) = crate::backdrop::grip_rects(b, view);
    for (rect, left, part) in [(bl, true, "grip_bl"), (br, false, "grip_br")] {
        if let Some(r) = clip_rect(rect, canvas) {
            hits.push((
                backdrop_hit_id(b.id, part),
                GraphHitKind::BackdropResize {
                    id: crate::backdrop::resize_handle(b.id, left),
                },
                r,
            ));
        }
    }
}

/// The a11y/hit id of one part of a backdrop (`header` / `grip_bl` / `grip_br`).
fn backdrop_hit_id(id: u32, part: &str) -> NodeId {
    fnv_id(&format!("motion_graph/backdrop/{id}/{part}"))
}

/// Push a wire's hit rects (all sharing one id encoding the target input) —
/// several padded segment boxes along the flattened bezier, clipped to `canvas`.
pub(crate) fn push_wire_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    canvas: Rect,
) {
    let Some((p0, p3)) = crate::paint::wire_endpoints(snap, e, view) else {
        return;
    };
    let id = wire_hit_id(e.to_node, e.to_port);
    let kind = GraphHitKind::Wire {
        edge: wire_handle(e.to_node, e.to_port),
    };
    // A `pre` edge has no spline — its hit surface is its badge(s).
    if e.delayed {
        let r = (PRE_BADGE_R + PRE_BADGE_HIT_PAD) * view.zoom;
        let self_loop = e.from_node == e.to_node;
        for (cx, cy) in pre_badge_centers(p0, p3, view.zoom, self_loop)
            .into_iter()
            .flatten()
        {
            let rect = Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r);
            if let Some(rect) = clip_rect(rect, canvas) {
                hits.push((id, kind, rect));
            }
        }
        return;
    }
    let pts = wire_hit_polyline(p0, p3, view.zoom);
    for seg in pts.windows(2) {
        let (ax, ay) = seg[0];
        let (bx, by) = seg[1];
        let r = Rect::new(
            ax.min(bx) - WIRE_HIT_R,
            ay.min(by) - WIRE_HIT_R,
            (ax - bx).abs() + 2.0 * WIRE_HIT_R,
            (ay - by).abs() + 2.0 * WIRE_HIT_R,
        );
        if let Some(r) = clip_rect(r, canvas) {
            hits.push((id, kind, r));
        }
    }
}

/// Push a node's input + output socket hit rects (square, fixed screen size),
/// clipped to the canvas.
pub(crate) fn push_socket_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    n: &GraphNodeView,
    view: &View,
    canvas: Rect,
) {
    for (i, _) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        if let Some(r) = clip_rect(socket_hit_rect(cx, cy), canvas) {
            hits.push((
                sock_in_id(n.id, i),
                GraphHitKind::SocketIn {
                    node: n.id as u64,
                    port: i as u16,
                },
                r,
            ));
        }
    }
    for (i, _) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        if let Some(r) = clip_rect(socket_hit_rect(cx, cy), canvas) {
            hits.push((
                sock_out_id(n.id, i),
                GraphHitKind::SocketOut {
                    node: n.id as u64,
                    port: i as u16,
                },
                r,
            ));
        }
    }
}

fn socket_hit_rect(cx: f32, cy: f32) -> Rect {
    Rect::new(
        cx - geom::SOCKET_HIT_R,
        cy - geom::SOCKET_HIT_R,
        2.0 * geom::SOCKET_HIT_R,
        2.0 * geom::SOCKET_HIT_R,
    )
}

/// Register the background + wire + node + socket `GraphSurface` hit rects (in
/// the order given — last wins) so the M0 dispatch routes gestures here, and
/// republish the canvas rect for the wheel-zoom hit-test. Keyboard focus is
/// cursor-gated by the shell, so it is NOT set here.
pub(crate) fn register_hits(ctx: &mut PaintCtx, rect: Rect, hits: &[(NodeId, GraphHitKind, Rect)]) {
    let parent = ids::MOTION_GRAPH_PANEL;
    {
        let store = ctx.host.store_mut();
        for (id, kind, _) in hits {
            store.register(
                *id,
                InteractiveState::GraphSurface {
                    parent,
                    kind: *kind,
                    canvas: rect,
                },
            );
        }
        store.set_graph_canvas(parent, rect);
    }
    let hit_index = ctx.host.hit_index_mut();
    for (id, _, r) in hits {
        hit_index.register(*id, *r);
    }
}

pub(crate) fn bg_hit_id() -> NodeId {
    fnv_id("motion_graph/bg")
}
fn node_hit_id(node: u32) -> NodeId {
    fnv_id(&format!("motion_graph/node/{node}"))
}
fn sock_in_id(node: u32, port: usize) -> NodeId {
    fnv_id(&format!("motion_graph/sock_in/{node}/{port}"))
}
fn sock_out_id(node: u32, port: usize) -> NodeId {
    fnv_id(&format!("motion_graph/sock_out/{node}/{port}"))
}
pub(crate) fn wire_hit_id(to_node: u32, to_port: u16) -> NodeId {
    fnv_id(&format!("motion_graph/wire/{to_node}/{to_port}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::PortView;
    use crate::state::ViewState;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    const CANVAS: Rect = Rect {
        x: 0.0,
        y: 100.0,
        w: 400.0,
        h: 200.0,
    };

    fn node(id: u32, x: f32, y: f32) -> GraphNodeView {
        GraphNodeView {
            kind: crate::snapshot::NodeViewKind::Node,
            id,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x,
            y,
            inputs: vec![PortView {
                name: "i",
                domain: Domain::Instances,
                dim: Dim::Vec2,
                clock: Clock::Frame,
            }],
            outputs: vec![],
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
        }
    }

    #[test]
    fn clip_rect_intersects_and_rejects_disjoint() {
        // Fully inside → unchanged.
        let inside = Rect::new(10.0, 120.0, 50.0, 50.0);
        assert_eq!(clip_rect(inside, CANVAS), Some(inside));
        // Straddling the top edge → cropped to the canvas.
        let straddle = clip_rect(Rect::new(10.0, 80.0, 50.0, 50.0), CANVAS).unwrap();
        assert_eq!((straddle.y, straddle.h), (100.0, 30.0));
        // Fully above the canvas → gone.
        assert_eq!(clip_rect(Rect::new(10.0, 0.0, 50.0, 50.0), CANVAS), None);
        // Touching the edge exactly (zero area) → gone, not a degenerate hit.
        assert_eq!(clip_rect(Rect::new(10.0, 50.0, 50.0, 50.0), CANVAS), None);
    }

    #[test]
    fn a_card_panned_off_canvas_registers_no_hit() {
        // The defect this guards: content scrolled/zoomed past the panel edge is
        // clipped away visually, but an unclipped hit rect would still swallow
        // clicks meant for the scene behind it.
        let mut hits = Vec::new();
        push_card_hit(
            &mut hits,
            &node(7, 0.0, 0.0),
            Rect::new(10.0, -60.0, 190.0, 58.0),
            CANVAS,
        );
        assert!(hits.is_empty(), "an off-canvas card is not clickable");

        // A card straddling the edge stays clickable, but only over its visible part.
        push_card_hit(
            &mut hits,
            &node(8, 0.0, 0.0),
            Rect::new(10.0, 70.0, 190.0, 58.0),
            CANVAS,
        );
        let (_, _, r) = hits[0];
        assert_eq!((r.y, r.h), (100.0, 28.0), "only the visible band is hit");
    }

    /// **A ghost registers no body** (doc 57): it belongs to another level, and a
    /// click-through body is what makes it read as what it is. Its SOCKETS still
    /// register — wiring across the seam is the entire reason it is drawn.
    #[test]
    fn a_ghost_has_no_body_hit_but_keeps_its_sockets() {
        let ghost = GraphNodeView {
            kind: crate::snapshot::NodeViewKind::Ghost,
            ..node(9, 10.0, 10.0)
        };
        let view = View::new(CANVAS, ViewState::default());

        let mut body = Vec::new();
        push_card_hit(
            &mut body,
            &ghost,
            Rect::new(10.0, 120.0, 190.0, 58.0),
            CANVAS,
        );
        assert!(
            body.is_empty(),
            "a ghost cannot be grabbed, dragged or deleted"
        );

        let mut sockets = Vec::new();
        push_socket_hits(&mut sockets, &ghost, &view, CANVAS);
        assert_eq!(
            sockets.len(),
            1,
            "but its sockets are live: you can wire it"
        );
    }

    /// A `pre` edge's hit surface is its badge pair, not a spline band: two
    /// compact rects for a normal delayed edge, ONE for the self-loop template,
    /// and the sampled-polyline rects only for plain forward wires. This is
    /// what keeps the portal badges clickable exactly where they draw
    /// (docs/Motion Nodes/03).
    #[test]
    fn a_pre_edge_hits_as_badges_not_as_a_spline() {
        let mk_edge = |from: u32, to: u32, delayed: bool| GraphEdgeView {
            from_node: from,
            from_port: 0,
            to_node: to,
            to_port: 0,
            delayed,
            out_domain: Domain::Instances,
        };
        let snap = GraphViewSnapshot {
            level: None,
            breadcrumb: Vec::new(),
            nodes: vec![node(1, 20.0, 50.0), node(2, 240.0, 50.0)],
            edges: vec![],
            backdrops: vec![],
            probe: None,
            now: 0.0,
        };
        let view = View::new(CANVAS, ViewState::default());

        let mut plain = Vec::new();
        push_wire_hits(&mut plain, &snap, &mk_edge(1, 2, false), &view, CANVAS);
        let mut badges = Vec::new();
        push_wire_hits(&mut badges, &snap, &mk_edge(1, 2, true), &view, CANVAS);
        let mut self_loop = Vec::new();
        push_wire_hits(&mut self_loop, &snap, &mk_edge(2, 2, true), &view, CANVAS);

        assert!(
            plain.len() > badges.len(),
            "a forward wire hits along its whole band ({}), a pre edge only at its badges ({})",
            plain.len(),
            badges.len()
        );
        assert_eq!(badges.len(), 2, "one badge per end of the pre edge");
        assert_eq!(self_loop.len(), 1, "the self-loop collapses to one badge");
        // Both badge rects are compact squares, not wire-band slabs.
        for (_, _, r) in &badges {
            assert!(r.w <= 2.0 * (PRE_BADGE_R + 4.0) && r.h <= 2.0 * (PRE_BADGE_R + 4.0));
        }
    }

    #[test]
    fn sockets_off_canvas_register_no_hit() {
        // Pan the view far above the canvas: the node's socket is off-screen.
        let view = View::new(
            CANVAS,
            ViewState {
                pan_x: 0.0,
                pan_y: -500.0,
                zoom: 1.0,
            },
        );
        let mut hits = Vec::new();
        push_socket_hits(&mut hits, &node(1, 0.0, 0.0), &view, CANVAS);
        assert!(hits.is_empty(), "off-canvas sockets are not grabbable");

        // Panned back in, the socket registers.
        let view = View::new(CANVAS, ViewState::default());
        push_socket_hits(&mut hits, &node(1, 10.0, 10.0), &view, CANVAS);
        assert_eq!(hits.len(), 1);
    }
}
