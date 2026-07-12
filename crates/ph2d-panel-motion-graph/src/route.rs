//! **Wire routing** (Motion Nodes F2, doc 44) — the geometry of a wire that has been
//! dragged through waypoints, and the hit-testing of the waypoint handles.
//!
//! ## One path, or the knife cuts a wire that is not there
//!
//! A wire's polyline is used by **three** things that must never disagree: what is drawn,
//! what the knife crosses, and what the pointer hovers. So routing goes into
//! [`wire_path`] — the one function all three already call — and not into the painter.
//! Route in the painter alone and the wire would *look* bent while the knife still cut it
//! along the straight line it used to take: you would slash empty canvas and watch a wire a
//! centimetre away fall.
//!
//! ## The route is a chain of the SAME cubic
//!
//! A wire with no waypoints is one horizontal-tangent cubic from socket to socket. With
//! waypoints it is that exact cubic, chained: `p0 → w0 → w1 → … → p3`. So a wire dragged
//! through a point does not change *kind* — it is the same curve, told where to go, and a
//! waypoint dragged back onto the straight line leaves the wire visually unchanged.

use crate::geom::View;
use crate::snapshot::{GraphEdgeView, GraphViewSnapshot};

/// Radius of a waypoint's drawn handle, in screen px (fixed: a routing dot you cannot grab
/// when zoomed out is a routing dot you cannot use).
pub(crate) const HANDLE_R: f32 = 4.5; // LITERAL-PX-OK: waypoint handle radius
/// Half-extent of a waypoint's square hit zone — bigger than the dot, like a socket's.
pub(crate) const HANDLE_HIT_R: f32 = 8.0; // LITERAL-PX-OK: waypoint hit half-extent

/// The full route of a wire in SCREEN space: `[source socket, waypoints…, target socket]`.
/// `None` when an endpoint node is not in the snapshot.
pub(crate) fn route(
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
) -> Option<Vec<(f32, f32)>> {
    let (p0, p3) = crate::paint::wire_endpoints(snap, e, view)?;
    let mut pts = Vec::with_capacity(e.waypoints.len() + 2);
    pts.push(p0);
    pts.extend(e.waypoints.iter().map(|&(x, y)| view.pt(x, y)));
    pts.push(p3);
    Some(pts)
}

/// The screen positions of a wire's waypoint handles (graph space → screen).
pub(crate) fn handles(e: &GraphEdgeView, view: &View) -> Vec<(f32, f32)> {
    e.waypoints.iter().map(|&(x, y)| view.pt(x, y)).collect()
}

/// Where a new waypoint, dropped at `(x, y)`, belongs in this wire's order.
///
/// The route is a chain of legs; the new point joins the leg it was dropped nearest to. Get
/// this wrong and a waypoint added between two others would jump to the end of the chain and
/// tie the wire in a knot — the insert must respect the order the artist sees.
pub(crate) fn insert_index(route: &[(f32, f32)], at: (f32, f32)) -> usize {
    let mut best = (0usize, f32::MAX);
    for (i, leg) in route.windows(2).enumerate() {
        let d = dist_to_segment(at, leg[0], leg[1]);
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

/// Distance from `p` to the segment `a → b`.
fn dist_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (p.0 - a.0, p.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 > f32::EPSILON {
        ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (apx - t * abx, apy - t * aby);
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::GraphEdgeView;
    use ph2d_nodegraph::port::Domain;

    fn edge(waypoints: Vec<(f32, f32)>) -> GraphEdgeView {
        GraphEdgeView {
            from_node: 1,
            from_port: 0,
            to_node: 2,
            to_port: 0,
            delayed: false,
            out_domain: Domain::Instances,
            waypoints,
        }
    }

    /// A new point joins the leg it was dropped ON — not the end of the chain. FALSIFIED by
    /// a naive `push`: dropping a point between two existing ones would send the wire out to
    /// the last waypoint and back, tying it in a knot.
    #[test]
    fn a_new_point_joins_the_leg_it_was_dropped_on() {
        // A route along +x: source, two waypoints, target.
        let route = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)];
        assert_eq!(insert_index(&route, (5.0, 1.0)), 0, "the first leg");
        assert_eq!(insert_index(&route, (15.0, 1.0)), 1, "between the two");
        assert_eq!(insert_index(&route, (25.0, 1.0)), 2, "the last leg");
        // A point far off to the side still lands on its NEAREST leg, not on leg 0.
        assert_eq!(insert_index(&route, (26.0, 90.0)), 2);
    }

    /// The handles are placed where the waypoints ARE — graph space through the view's own
    /// transform, so a routing dot is drawn (and hit) exactly where the document says.
    #[test]
    fn a_handle_sits_on_its_waypoint() {
        let view = View::new(
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 600.0),
            crate::state::ViewState::default(),
        );
        let e = edge(vec![(100.0, 50.0), (200.0, 80.0)]);
        assert_eq!(handles(&e, &view), vec![(100.0, 50.0), (200.0, 80.0)]);
        assert!(
            handles(&edge(vec![]), &view).is_empty(),
            "no dots on a straight wire"
        );
    }

    /// **The route threads the waypoints, in order, between the two sockets.** This is the
    /// list the drawn path, the knife path and the hover path are all built from — so a
    /// wire cannot be drawn along one curve and cut along another (doc 44).
    #[test]
    fn the_route_runs_from_socket_through_every_point_to_socket() {
        use crate::snapshot::{GraphNodeView, PortView};
        use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
        use ph2d_nodegraph::port::{Clock, Dim};

        let port = || PortView {
            name: "p",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        };
        let node = |id: u32, x: f32| GraphNodeView {
            id,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x,
            y: 0.0,
            inputs: vec![port()],
            outputs: vec![port()],
            readout: None,
        };
        let snap = GraphViewSnapshot {
            nodes: vec![node(1, 0.0), node(2, 400.0)],
            edges: vec![edge(vec![(150.0, 200.0), (250.0, 200.0)])],
            backdrops: vec![],
            probe: None,
        };
        let view = View::new(
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 600.0),
            crate::state::ViewState::default(),
        );
        let r = route(&snap, &snap.edges[0], &view).expect("both endpoints exist");
        assert_eq!(r.len(), 4, "source + two points + target");
        assert_eq!(&r[1..3], &[(150.0, 200.0), (250.0, 200.0)], "in order");
        // And the drawn path really passes through them (the same function the knife uses).
        let path = crate::paint::wire_path(&r, 1.0, 20);
        let near = |p: (f32, f32)| {
            path.iter()
                .any(|q| (q.0 - p.0).abs() < 1.0 && (q.1 - p.1).abs() < 1.0)
        };
        assert!(
            near((150.0, 200.0)) && near((250.0, 200.0)),
            "the path bends through them"
        );
    }

    /// **The knife cuts the wire WHERE IT IS DRAWN.** A stroke across the routed curve cuts
    /// it; a stroke across the straight line the wire *would* have taken with no waypoints
    /// cuts nothing.
    ///
    /// FALSIFIED by routing in the painter alone (the wire looks bent, the knife still cuts
    /// the old straight line): you would slash empty canvas and watch a wire a hand's width
    /// away fall — and slash the wire you can see and have nothing happen.
    #[test]
    fn the_knife_follows_the_routed_curve_not_the_straight_line() {
        use crate::snapshot::{GraphNodeView, PortView};
        use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
        use ph2d_nodegraph::port::{Clock, Dim};

        let port = || PortView {
            name: "p",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        };
        let node = |id: u32, x: f32| GraphNodeView {
            id,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x,
            y: 0.0,
            inputs: vec![port()],
            outputs: vec![port()],
            readout: None,
        };
        // Both sockets sit near y = 37; the wire is dragged far DOWN through (200, 300).
        let snap = GraphViewSnapshot {
            nodes: vec![node(1, 0.0), node(2, 400.0)],
            edges: vec![edge(vec![(200.0, 300.0)])],
            backdrops: vec![],
            probe: None,
        };
        let view = View::new(
            ph2d_editor_core::zones::Rect::new(0.0, 0.0, 800.0, 600.0),
            crate::state::ViewState::default(),
        );

        // A stroke across the BELLY of the routed curve cuts it.
        let cut = crate::paint::wires_crossed(&snap, &view, (200.0, 250.0), (200.0, 340.0));
        assert_eq!(
            cut,
            vec![(2, 0)],
            "the knife cut the wire where it is drawn"
        );

        // A stroke across where the wire WOULD be with no routing cuts nothing.
        let miss = crate::paint::wires_crossed(&snap, &view, (200.0, 20.0), (200.0, 55.0));
        assert!(miss.is_empty(), "nothing is there any more: {miss:?}");
    }
}
