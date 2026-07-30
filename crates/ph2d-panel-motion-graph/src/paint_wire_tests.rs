//! Guards for the wire's FLATTENING (doc 46.1). `super` is `paint::paint_wire`.

use super::*;

/// The true curve, densely sampled — the oracle the flattening is measured against.
fn true_curve(p0: (f32, f32), p3: (f32, f32), zoom: f32) -> Vec<(f32, f32)> {
    let (c1, c2) = wire_handles(p0, p3, zoom);
    cubic_polyline(p0, c1, c2, p3, 2000)
}

fn point_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let (wx, wy) = (p.0 - a.0, p.1 - a.1);
    let len2 = vx * vx + vy * vy;
    let t = if len2 > 0.0 {
        ((wx * vx + wy * vy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (p.0 - (a.0 + vx * t), p.1 - (a.1 + vy * t));
    (dx * dx + dy * dy).sqrt()
}

/// The worst distance from the true curve to `poly`.
fn max_deviation(poly: &[(f32, f32)], curve: &[(f32, f32)]) -> f32 {
    curve
        .iter()
        .map(|p| {
            poly.windows(2)
                .map(|s| point_to_segment(*p, s[0], s[1]))
                .fold(f32::INFINITY, f32::min)
        })
        .fold(0.0, f32::max)
}

/// **The drawn wire is deep sub-pixel from the curve it claims to be** — at any length, at any
/// zoom.
///
/// And the SECOND half of this guard is the one that matters, because measuring only the chord
/// error would have declared the bug fixed while it was still on screen: the old fixed-20
/// flattening strayed a mere ~0.5 px, and still looked like a chain of sticks. What the eye
/// reads is the vertex DENSITY (the marching dashes are short straights cut from this polyline,
/// so each facet becomes a visibly straight tick). So the guard pins the density too.
#[test]
fn a_wire_is_flattened_to_a_sub_pixel_tolerance_at_any_size() {
    // A long, steep, zoomed-in wire: the worst case, and the one in Enio's screenshot.
    for (p0, p3, zoom) in [
        ((0.0, 0.0), (60.0, 20.0), 1.0),     // a short hop
        ((0.0, 0.0), (420.0, 260.0), 1.0),   // a long diagonal
        ((0.0, 0.0), (420.0, 260.0), 2.5),   // …at full zoom
        ((300.0, 40.0), (60.0, 300.0), 1.8), // backwards (target left of source)
    ] {
        let poly = wire_polyline(p0, p3, zoom);
        let dev = max_deviation(&poly, &true_curve(p0, p3, zoom));
        assert!(
            dev < 0.3,
            "flattened wire strays {dev:.3} px from the curve ({p0:?} -> {p3:?} @ {zoom})"
        );
    }

    // The wire from Enio's screenshot: it must come out substantially DENSER than the 20
    // segments it used to get, or the dashes go on reading as straight sticks.
    let (p0, p3, zoom) = ((0.0, 0.0), (420.0, 260.0), 2.5);
    let n = wire_polyline(p0, p3, zoom).len() - 1;
    assert!(
        n >= 40,
        "a long zoomed wire gets {n} segments - it used to get 20, and it showed"
    );
    // …and the error it buys is an order of magnitude under the old one (~0.5 px, measured).
    let (c1, c2) = wire_handles(p0, p3, zoom);
    let old = max_deviation(
        &cubic_polyline(p0, c1, c2, p3, 20),
        &true_curve(p0, p3, zoom),
    );
    let new = max_deviation(&wire_polyline(p0, p3, zoom), &true_curve(p0, p3, zoom));
    assert!(new * 4.0 < old, "old {old:.3} px -> new {new:.3} px");
}

/// The count comes from the CURVE: a bigger, more curved, more zoomed wire gets more segments,
/// and a stub gets few. (Zoom refines for free — the flattening happens in screen space.)
#[test]
fn the_segment_count_follows_the_curve_not_a_constant() {
    let stub = wire_polyline((0.0, 0.0), (40.0, 0.0), 1.0).len();
    let long = wire_polyline((0.0, 0.0), (420.0, 260.0), 1.0).len();
    let zoomed = wire_polyline((0.0, 0.0), (420.0, 260.0), 2.5).len();
    assert!(stub < long, "a stub needs fewer segments: {stub} vs {long}");
    assert!(
        long < zoomed,
        "zoomed in, the same wire is refined: {long} vs {zoomed}"
    );
    assert!(zoomed <= FLATTEN_MAX + 1, "and the budget is bounded");
}

/// **The coarse HIT boxes still cover the drawn wire.** The hit path samples deliberately
/// coarsely (fat boxes, few of them), which is only legitimate while the chord error stays well
/// inside the box padding — otherwise there would be a strip of the visible wire that is not
/// clickable, and the knife (which rides the DRAWN polyline) would cut wires the hit-test
/// cannot even hover.
#[test]
fn the_coarse_hit_boxes_still_cover_the_drawn_wire() {
    for (p0, p3, zoom) in [
        ((0.0, 0.0), (420.0, 260.0), 1.0),
        ((0.0, 0.0), (420.0, 260.0), 2.5),
        ((300.0, 40.0), (60.0, 300.0), 1.8),
    ] {
        let dev = max_deviation(&wire_hit_polyline(p0, p3, zoom), &true_curve(p0, p3, zoom));
        assert!(
            dev < crate::hits::WIRE_HIT_R,
            "the hit chords stray {dev:.2} px - more than the {} px box padding",
            crate::hits::WIRE_HIT_R
        );
    }
}

/// **An incompatible drop target reads as DANGER RED, not muted grey** — the ghost wire and
/// the target-socket ring both go red, the affirmative "no" the artist sees before releasing.
/// This is the `connects_directly` verdict (resolved onto the target by the interaction) made
/// visible; a compatible or empty target keeps the domain preview and an Accent ring.
///
/// Reverting the change — Danger back to the old muted `Border`, or the ring back to Accent
/// regardless — sangra here, and `None` (empty space) must stay the neutral preview so hovering
/// nowhere never flashes red.
#[test]
fn an_incompatible_drop_target_is_danger_red() {
    let dom = ColorToken::PortInstances; // any domain hue: the "this is what I'll carry" preview

    // The ghost WIRE: incompatible ⇒ Danger; compatible or empty ⇒ the on-target preview.
    assert_eq!(ghost_wire_token(Some(false), dom), ColorToken::Danger);
    assert_eq!(ghost_wire_token(Some(true), dom), dom);
    assert_eq!(ghost_wire_token(None, dom), dom);
    // A backward drag with nothing found yet has no hue — it stays muted Border, never red.
    assert_eq!(
        ghost_wire_token(None, ColorToken::Border),
        ColorToken::Border
    );

    // The target-socket RING: compatible ⇒ Accent (drop here), incompatible ⇒ Danger (not here).
    assert_eq!(target_ring_token(true), ColorToken::Accent);
    assert_eq!(target_ring_token(false), ColorToken::Danger);

    // The verdict comes straight off the interaction's resolved target tuple.
    assert_eq!(target_compat(&Some((3, 0, false))), Some(false));
    assert_eq!(target_compat(&Some((3, 0, true))), Some(true));
    assert_eq!(target_compat(&None), None);
}

/// **The ghost wire's loose end SNAPS to the target socket** — once the drag locks onto a socket
/// (the same one the magnet in `geom` will drop onto), the wire draws to that socket's CENTRE,
/// not the cursor, so the artist SEES the lock. FALSIFIED three ways: the target is ignored (the
/// end never leaves the cursor), the wrong EDGE is used (an output target drawn on the left), or
/// empty space resolves to a point instead of `None` (the wire would jump to a phantom socket).
#[test]
fn the_ghost_end_snaps_to_the_target_socket() {
    use crate::snapshot::{GraphNodeView, NodeViewKind, PortView};
    use crate::state::ViewState;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    let port = || PortView {
        name: "p",
        domain: Domain::Instances,
        dim: Dim::Scalar,
        clock: Clock::Frame,
    };
    let snap = GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            kind: NodeViewKind::Node,
            id: 5,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x: 100.0,
            y: 0.0,
            inputs: vec![port()],
            outputs: vec![port()],
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
        }],
        edges: vec![],
        backdrops: vec![],
        probe: None,
        now: 0.0,
    };
    let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
    let node = &snap.nodes[0];

    // Locked onto the node's OUTPUT socket (right edge) — the ghost end is that socket's centre.
    assert_eq!(
        ghost_target_center(&Some((5, 0, true)), &snap, &view, true),
        Some(socket_center(node, &view, true, 0)),
    );
    // Locked onto its INPUT (left edge): proves the `output` flag picks the edge, not a constant.
    assert_eq!(
        ghost_target_center(&Some((5, 0, false)), &snap, &view, false),
        Some(socket_center(node, &view, false, 0)),
    );
    // No target, or a node that is not here → None, so the end falls back to the cursor.
    assert_eq!(ghost_target_center(&None, &snap, &view, true), None);
    assert_eq!(
        ghost_target_center(&Some((99, 0, true)), &snap, &view, true),
        None,
    );
}
