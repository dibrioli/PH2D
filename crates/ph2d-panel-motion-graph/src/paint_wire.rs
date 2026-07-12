//! **Everything that is a wire** (Motion Nodes) — drawing it, flattening it, testing it
//! against the knife — split out of `paint` for the panel LOC cap.
//!
//! They belong together because they must AGREE: the polyline this module flattens is the
//! one it draws, the one `hits` registers, and the one the knife crosses. A wire drawn along
//! one curve and cut along another is the bug the waypoints slice had to prove was not there
//! (doc 44), and keeping the three in one module is what keeps them honest.

use crate::geom::{View, socket_center};
use crate::snapshot::{GraphEdgeView, GraphViewSnapshot};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::paint::{fill_circle, resolve, stroke_polyline, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

use super::{
    GHOST_W, PRE_DOT_R, PRE_RING_W, SOCKET_R, TARGET_RING_PAD, WIRE_TANGENT, WIRE_W,
    WIRE_W_DELAYED, WIRE_W_HOVER, cubic_polyline, domain_token, highlight_socket, port_out_domain,
};

pub(crate) fn draw_wire(
    ctx: &mut PaintCtx,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    theme: Theme,
    hovered: bool,
) {
    let Some((p0, p3)) = wire_endpoints(snap, e, view) else {
        return;
    };
    // A `pre` edge draws as PORTAL BADGES at its sockets, never as a spline
    // (docs/Motion Nodes/03): the state loop is machine plumbing, so the canvas
    // keeps reading left→right instead of lassoing back across the graph.
    // Hovering a badge reveals its partner with a thin ghost of the pair.
    if e.delayed {
        draw_pre_badges(ctx, e, p0, p3, view, theme, hovered);
        return;
    }
    let pts = wire_polyline(p0, p3, view.zoom, 20);
    // Hovered wires draw thicker and in a bright emphasis colour so the delete
    // target is unmistakable regardless of the port domain hue; others keep
    // their domain colour and normal width.
    let (base_w, token) = if hovered {
        (WIRE_W_HOVER, ColorToken::Text1)
    } else {
        (WIRE_W, domain_token(e.out_domain))
    };
    stroke_polyline(ctx.scene, &pts, base_w * view.zoom, resolve(token, theme));
}

/// The `pre` edge's visual: a ring-and-dot badge just outside each end's
/// socket (the Bifrost feedback-port reading — "state re-enters here, one tick
/// later"), or a single badge on the input for the self-loop template. Badge
/// geometry is shared with the hit path (`hits::pre_badge_centers`).
fn draw_pre_badges(
    ctx: &mut PaintCtx,
    e: &GraphEdgeView,
    p0: (f32, f32),
    p3: (f32, f32),
    view: &View,
    theme: Theme,
    hovered: bool,
) {
    let token = if hovered {
        ColorToken::Text1
    } else {
        domain_token(e.out_domain)
    };
    let color = resolve(token, theme);
    if hovered {
        // Reveal the pair: a thin ghost of the path the state actually takes.
        let pts = wire_polyline(p0, p3, view.zoom, 20);
        stroke_polyline(ctx.scene, &pts, WIRE_W_DELAYED * view.zoom, color);
    }
    let self_loop = e.from_node == e.to_node;
    for (cx, cy) in crate::hits::pre_badge_centers(p0, p3, view.zoom, self_loop)
        .into_iter()
        .flatten()
    {
        let r = crate::hits::PRE_BADGE_R * view.zoom;
        stroke_rounded_rect(
            ctx.scene,
            Rect::new(cx - r, cy - r, 2.0 * r, 2.0 * r),
            r,
            PRE_RING_W * view.zoom,
            color,
        );
        fill_circle(ctx.scene, cx, cy, PRE_DOT_R * view.zoom, color);
    }
}

/// The in-progress wire ghost: from the source output socket to the live pointer,
/// colored by the source domain when the hovered target is compatible, muted
/// `Border` when incompatible; a compatible target socket also gets an Accent
/// ring so the drop point is unambiguous.
pub(crate) fn draw_wire_ghost(
    ctx: &mut PaintCtx,
    snap: &GraphViewSnapshot,
    state: &MotionGraphPanelState,
    view: &View,
    theme: Theme,
) {
    match &state.interaction {
        // Forwards, out of an output — or the END of a wire pulled off its input (doc 45),
        // which is the same thing: it is still anchored to its source.
        Interaction::DrawWire {
            from_node,
            from_port,
            cur,
            target,
            ..
        } => {
            let Some(src) = snap.nodes.iter().find(|n| n.id == *from_node) else {
                return;
            };
            let p0 = socket_center(src, view, true, *from_port as usize);
            let color = match target {
                Some((_, _, false)) => resolve(ColorToken::Border, theme),
                _ => resolve(domain_token(port_out_domain(src, *from_port)), theme),
            };
            let pts = wire_polyline(p0, *cur, view.zoom, 18);
            stroke_polyline(ctx.scene, &pts, GHOST_W * view.zoom, color);

            if let Some((tn, tp, true)) = target
                && let Some(t) = snap.nodes.iter().find(|n| n.id == *tn)
            {
                let (cx, cy) = socket_center(t, view, false, *tp as usize);
                highlight_socket(ctx, cx, cy, (SOCKET_R + TARGET_RING_PAD) * view.zoom, theme);
            }
        }
        // BACKWARDS, out of an empty input, hunting for an output. The ghost runs from the
        // cursor INTO the input, so the wire already reads the way it will read once it is
        // real — an arrow drawn backwards would have to be re-read the moment it lands.
        Interaction::DrawWireBack {
            to_node,
            to_port,
            cur,
            target,
        } => {
            let Some(dst) = snap.nodes.iter().find(|n| n.id == *to_node) else {
                return;
            };
            let p3 = socket_center(dst, view, false, *to_port as usize);
            let color = match target {
                Some((_, _, false)) => resolve(ColorToken::Border, theme),
                Some((fnode, fport, true)) => {
                    let d = snap
                        .nodes
                        .iter()
                        .find(|n| n.id == *fnode)
                        .map(|n| port_out_domain(n, *fport));
                    match d {
                        Some(d) => resolve(domain_token(d), theme),
                        None => resolve(ColorToken::Border, theme),
                    }
                }
                None => resolve(ColorToken::Border, theme),
            };
            let pts = wire_polyline(*cur, p3, view.zoom, 18);
            stroke_polyline(ctx.scene, &pts, GHOST_W * view.zoom, color);

            if let Some((fnode, fport, true)) = target
                && let Some(f) = snap.nodes.iter().find(|n| n.id == *fnode)
            {
                let (cx, cy) = socket_center(f, view, true, *fport as usize);
                highlight_socket(ctx, cx, cy, (SOCKET_R + TARGET_RING_PAD) * view.zoom, theme);
            }
        }
        _ => {}
    }
}

/// The wire currently being PULLED OFF its input, if any — the painter skips it.
///
/// A wire still drawn into the socket you are visibly pulling it out of is a wire that has
/// not moved: the artist would be dragging a ghost while the original sat there, and would
/// have no way to tell whether the gesture had taken.
pub(crate) fn detached_edge(state: &MotionGraphPanelState) -> Option<(u32, u16)> {
    match &state.interaction {
        Interaction::DrawWire { detached, .. } => *detached,
        _ => None,
    }
}

/// Screen endpoints `(output socket, input socket)` of a wire, or `None` if
/// either endpoint node isn't in the snapshot.
pub(crate) fn wire_endpoints(
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
) -> Option<((f32, f32), (f32, f32))> {
    let from = snap.nodes.iter().find(|n| n.id == e.from_node)?;
    let to = snap.nodes.iter().find(|n| n.id == e.to_node)?;
    Some((
        socket_center(from, view, true, e.from_port as usize),
        socket_center(to, view, false, e.to_port as usize),
    ))
}

/// Sample a horizontal-tangent cubic between two socket centers into `n+1`
/// polyline points (shared by draw + hit-path so they never diverge).
pub(crate) fn wire_polyline(
    p0: (f32, f32),
    p3: (f32, f32),
    zoom: f32,
    n: usize,
) -> Vec<(f32, f32)> {
    let dx = ((p3.0 - p0.0).abs() * 0.5 + WIRE_TANGENT * zoom).max(WIRE_TANGENT * zoom);
    cubic_polyline(p0, (p0.0 + dx, p0.1), (p3.0 - dx, p3.1), p3, n)
}

/// How finely a wire is sampled when testing it against the knife. Denser than the
/// draw sampling, for the same reason the hit path is: a coarse polyline can pass a
/// stroke that visibly crosses the curve between two samples.
const KNIFE_SAMPLES: usize = 24;

/// Every wire the knife stroke `a → b` crosses, as its target input `(to_node,
/// to_port)` — the same identity `Disconnect` uses (an input takes exactly one
/// edge, so the target names the wire uniquely).
///
/// `pre` edges are **skipped**: they have no spline on screen (they draw as portal
/// badges at their sockets), so a stroke could only "cross" one by accident — and
/// the shell refuses to cut managed state wiring anyway.
pub(crate) fn wires_crossed(
    snap: &GraphViewSnapshot,
    view: &View,
    a: (f32, f32),
    b: (f32, f32),
) -> Vec<(u32, u16)> {
    snap.edges
        .iter()
        .filter(|e| !e.delayed)
        .filter(|e| {
            wire_endpoints(snap, e, view).is_some_and(|(p0, p3)| {
                wire_polyline(p0, p3, view.zoom, KNIFE_SAMPLES)
                    .windows(2)
                    .any(|s| segments_cross(a, b, s[0], s[1]))
            })
        })
        .map(|e| (e.to_node, e.to_port))
        .collect()
}

/// Whether the segments `p→q` and `r→s` properly cross. The standard orientation
/// test: they cross when each segment's endpoints straddle the other's line.
/// Collinear-overlap is treated as no crossing — a knife dragged exactly ALONG a
/// wire is not a cut, it is a miss (and the cross-product test says so).
fn segments_cross(p: (f32, f32), q: (f32, f32), r: (f32, f32), s: (f32, f32)) -> bool {
    let cross = |o: (f32, f32), a: (f32, f32), b: (f32, f32)| {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };
    let (d1, d2) = (cross(p, q, r), cross(p, q, s));
    let (d3, d4) = (cross(r, s, p), cross(r, s, q));
    ((d1 > 0.0) != (d2 > 0.0)) && ((d3 > 0.0) != (d4 > 0.0))
}
