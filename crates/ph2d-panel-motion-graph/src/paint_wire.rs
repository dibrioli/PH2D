//! **Everything that is a wire** (Motion Nodes) — drawing it, flattening it, testing it
//! against the knife — split out of `paint` for the panel LOC cap.
//!
//! They belong together because they must AGREE: the polyline this module flattens is the
//! one it draws, the one `hits` registers, and the one the knife crosses. A wire drawn along
//! one curve and cut along another is the bug the waypoints slice had to prove was not there
//! (doc 44), and keeping the three in one module is what keeps them honest.

use crate::flow;
use crate::geom::{View, socket_center};
use crate::snapshot::{GraphEdgeView, GraphViewSnapshot};
use crate::state::{Interaction, MotionGraphPanelState};
use ph2d_editor_core::paint::{fill_circle, resolve, stroke_polyline, stroke_rounded_rect};
use ph2d_editor_core::paint_batch::stroke_subpaths;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

use super::{
    GHOST_W, PRE_DOT_R, PRE_RING_W, SOCKET_R, TARGET_RING_PAD, WIRE_TANGENT, WIRE_W_DELAYED,
    WIRE_W_HOVER, cubic_polyline, domain_token, highlight_socket, port_out_domain,
};

pub(crate) fn draw_wire(
    ctx: &mut PaintCtx,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    theme: Theme,
    hovered: bool,
    bright: bool,
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
    let src = snap.nodes.iter().find(|n| n.id == e.from_node);
    let pts = wire_polyline(p0, p3, view.zoom);
    // Hovered wires draw thicker and in a bright emphasis colour so the delete target is
    // unmistakable regardless of the port domain hue. Otherwise the wire keeps its domain
    // colour and takes its WIDTH from the mass of the stream it carries (F3).
    let (base_w, token) = if hovered {
        (WIRE_W_HOVER, ColorToken::Text1)
    } else {
        (
            flow::wire_width(src.and_then(|n| n.count)),
            domain_token(e.out_domain),
        )
    };
    stroke_polyline(ctx.scene, &pts, base_w * view.zoom, resolve(token, theme));

    // **Data is moving through this wire right now** — the source's output changed since last
    // frame (TouchDesigner's animated wire). Bright dashes march along it, source → target.
    // They ride the SAME polyline, so they cannot drift off the wire they belong to.
    if bright && src.is_some_and(|n| n.hot) && !hovered {
        let z = view.zoom;
        // ONE draw call for the whole marching pattern: a dozen little strokes per wire, on every
        // wire, every frame, is a dozen draw objects Vello has to bound and bin (doc 53).
        let dashes = flow::dashes(
            &pts,
            flow::DASH_PERIOD * z,
            flow::DASH_ON * z,
            snap.now * flow::DASH_SPEED * z,
        );
        stroke_subpaths(
            ctx.scene,
            &dashes,
            base_w * z,
            resolve(ColorToken::Text1, theme),
        );
    }

    // **Veiled** — either nothing runs through this wire (no sink pulls it), or a selection is
    // up and this wire is outside its influence. The SAME veil the card gets, so a region of the
    // canvas recedes as one region instead of leaving faded cards strung together by
    // full-strength wires. A veil, not a repaint: the domain hue survives, faintly.
    if !bright {
        stroke_polyline(
            ctx.scene,
            &pts,
            base_w * view.zoom,
            resolve(ColorToken::GraphInert, theme),
        );
    }
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
        let pts = wire_polyline(p0, p3, view.zoom);
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

/// A drag target's compatibility: `Some(true)` = a legal drop, `Some(false)` = an
/// illegal one (the `connects_directly` verdict the interaction already resolved onto
/// the socket under the pointer), `None` = empty space.
fn target_compat(target: &Option<(u32, u16, bool)>) -> Option<bool> {
    target.map(|(_, _, ok)| ok)
}

/// The TARGET socket's screen centre once the drag has locked onto one, else `None`. The
/// ghost wire's loose end snaps HERE (so it visibly jumps to where it will land, matching the
/// magnet in `geom` — the drop already resolves to this socket), and the highlight ring lands
/// here too. `output` picks the edge: a forward wire's target is an INPUT (left), a backward
/// wire's is an OUTPUT (right).
fn ghost_target_center(
    target: &Option<(u32, u16, bool)>,
    snap: &GraphViewSnapshot,
    view: &View,
    output: bool,
) -> Option<(f32, f32)> {
    let (tn, tp, _) = (*target)?;
    let t = snap.nodes.iter().find(|n| n.id == tn)?;
    Some(socket_center(t, view, output, tp as usize))
}

/// The ghost wire's colour. **DANGER over an incompatible target** — the affirmative
/// "no" the artist must SEE before dropping, not the muted grey that reads as "inactive".
/// Otherwise `on_target`: the source domain's hue (the "this is what I'll carry" preview)
/// over empty space or a compatible socket, or the muted `Border` when no source domain
/// is known yet (a backward drag that has found nothing).
fn ghost_wire_token(compat: Option<bool>, on_target: ColorToken) -> ColorToken {
    match compat {
        Some(false) => ColorToken::Danger,
        _ => on_target,
    }
}

/// The ring around the hovered drop-target socket: **ACCENT** to say "drop here",
/// **DANGER** to say "not here". The same `connects_directly` verdict as the wire
/// colour, two hues — the refusal lands on the exact socket the artist is pointing at.
fn target_ring_token(compatible: bool) -> ColorToken {
    if compatible {
        ColorToken::Accent
    } else {
        ColorToken::Danger
    }
}

/// The in-progress wire ghost: from the source output socket to the live pointer,
/// coloured by the source domain over empty space or a compatible target, and **DANGER
/// red over an incompatible one**. The hovered target socket gets a ring too — Accent
/// when the drop is legal, Danger when it is not.
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
            let on_target = domain_token(port_out_domain(src, *from_port));
            let color = resolve(ghost_wire_token(target_compat(target), on_target), theme);
            // Forward wire's target is an INPUT socket; the loose end snaps to it when locked on.
            let tc = ghost_target_center(target, snap, view, false);
            let pts = wire_polyline(p0, tc.unwrap_or(*cur), view.zoom);
            stroke_polyline(ctx.scene, &pts, GHOST_W * view.zoom, color);

            if let (Some((_, _, compat)), Some((cx, cy))) = (target, tc) {
                let r = (SOCKET_R + TARGET_RING_PAD) * view.zoom;
                highlight_socket(ctx, cx, cy, r, theme, target_ring_token(*compat));
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
            // The found output's hue when the pointer is over one, else muted Border (no
            // source known yet). `ghost_wire_token` overrides this to Danger when illegal.
            let on_target = match target {
                Some((fnode, fport, _)) => snap
                    .nodes
                    .iter()
                    .find(|n| n.id == *fnode)
                    .map(|n| domain_token(port_out_domain(n, *fport)))
                    .unwrap_or(ColorToken::Border),
                None => ColorToken::Border,
            };
            let color = resolve(ghost_wire_token(target_compat(target), on_target), theme);
            // Backward wire's target is an OUTPUT socket; the loose end (the START, running INTO
            // the input) snaps to it when locked on.
            let tc = ghost_target_center(target, snap, view, true);
            let pts = wire_polyline(tc.unwrap_or(*cur), p3, view.zoom);
            stroke_polyline(ctx.scene, &pts, GHOST_W * view.zoom, color);

            if let (Some((_, _, compat)), Some((cx, cy))) = (target, tc) {
                let r = (SOCKET_R + TARGET_RING_PAD) * view.zoom;
                highlight_socket(ctx, cx, cy, r, theme, target_ring_token(*compat));
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
///
/// It stays suppressed for one frame PAST the drop (`pending_detach`), because the shell only
/// answers on the next frame — see `MotionGraphPanelState::pending_detach`.
pub(crate) fn detached_edge(state: &MotionGraphPanelState) -> Option<(u32, u16)> {
    match &state.interaction {
        Interaction::DrawWire { detached, .. } => *detached,
        _ => None,
    }
    .or(state.pending_detach)
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

/// **The wire, as a polyline** — a horizontal-tangent cubic between two socket centres,
/// flattened to a SUB-PIXEL tolerance rather than to a fixed number of segments.
///
/// There is no `n`, and that is the point. It used to take one: the draw asked for 20, the
/// ghost for 18, the knife for 24, the hit path for 16 — four different curves, in a module
/// whose whole premise is that the wire you see is the wire you cut. And a fixed count is wrong
/// on both ends anyway: it facets a long or zoomed-in wire (Enio saw the straights), and it
/// over-samples a short one.
///
/// The count now comes from the CURVE, in screen space, so zooming in refines it for free.
pub(crate) fn wire_polyline(p0: (f32, f32), p3: (f32, f32), zoom: f32) -> Vec<(f32, f32)> {
    let (c1, c2) = wire_handles(p0, p3, zoom);
    cubic_polyline(p0, c1, c2, p3, flatten_steps(p0, c1, c2, p3))
}

/// The two control points: horizontal tangents, their length scaled with the horizontal span.
fn wire_handles(p0: (f32, f32), p3: (f32, f32), zoom: f32) -> ((f32, f32), (f32, f32)) {
    let dx = ((p3.0 - p0.0).abs() * 0.5 + WIRE_TANGENT * zoom).max(WIRE_TANGENT * zoom);
    ((p0.0 + dx, p0.1), (p3.0 - dx, p3.1))
}

/// How far the drawn polyline may stray from the true cubic, in screen px.
///
/// Deep sub-pixel, and deliberately tighter than "you cannot see the error": the marching
/// dashes (F3) are SHORT straight sub-paths cut out of this very polyline, so what the eye
/// reads as faceting is not the chord error — it is the vertex DENSITY. At the old fixed 20
/// segments a long wire strayed only ~0.5 px from its curve and still looked like a chain of
/// sticks, because every dash landed inside one 12-px straight. The tolerance is what buys the
/// density (Enio's screenshot).
const FLATTEN_TOL: f32 = 0.1; // LITERAL-PX-OK: max deviation of the flattened wire from the curve
const FLATTEN_MIN: usize = 6; // LITERAL-PX-OK: segments, even for a stub of a wire
const FLATTEN_MAX: usize = 192; // LITERAL-PX-OK: segments, even for a wire across a 4K canvas

/// **Wang's formula** — the segment count that holds a uniformly-sampled cubic within `tol`.
///
/// The error of an `n`-segment uniform flattening is bounded by `max|B''(t)| / (8n²)`, and for
/// a cubic `max|B''| ≤ 6·max(|p0 − 2c1 + c2|, |c1 − 2c2 + p3|)` (the second differences of the
/// control polygon). Invert it for `n`. Clamped, because a wire is not worth an unbounded
/// vertex budget — and `FLATTEN_MAX` is the one place that budget is written down.
fn flatten_steps(p0: (f32, f32), c1: (f32, f32), c2: (f32, f32), p3: (f32, f32)) -> usize {
    let second = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        let (x, y) = (a.0 - 2.0 * b.0 + c.0, a.1 - 2.0 * b.1 + c.1);
        (x * x + y * y).sqrt()
    };
    let six = 6.0; // LITERAL-PX-OK: Wang's bound max|B''| <= 6·max(second differences) — math
    let eight = 8.0; // LITERAL-PX-OK: Wang's error bound max|B''| / (8n²) — math, not design
    let m = six * second(p0, c1, c2).max(second(c1, c2, p3));
    let n = (m / (eight * FLATTEN_TOL)).sqrt().ceil();
    (n as usize).clamp(FLATTEN_MIN, FLATTEN_MAX) // CLAMP-OK: const bounds, min < max
}

/// The wire's HIT boxes ride a deliberately COARSER sampling than the drawing: each segment
/// becomes an axis-aligned box padded by `WIRE_HIT_R` (8 px), so a handful of fat boxes cover
/// the curve conservatively. Sampling the hit path at the DRAWN tolerance would multiply the
/// rect count per wire for no accuracy at all — the padding already dwarfs the chord error, and
/// a guard pins exactly that (`the_coarse_hit_boxes_still_cover_the_drawn_wire`).
pub(crate) fn wire_hit_polyline(p0: (f32, f32), p3: (f32, f32), zoom: f32) -> Vec<(f32, f32)> {
    const HIT_SAMPLES: usize = 16; // LITERAL-PX-OK: segments per wire in the hit-box list
    let (c1, c2) = wire_handles(p0, p3, zoom);
    cubic_polyline(p0, c1, c2, p3, HIT_SAMPLES)
}

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
                wire_polyline(p0, p3, view.zoom)
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

#[cfg(test)]
#[path = "paint_wire_tests.rs"]
mod tests;
