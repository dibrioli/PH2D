//! Motion-graph editor paint (Motion Nodes M1.E1–E7, Phase 1b).
//!
//! Reads the published [`GraphViewSnapshot`] + the ephemeral `Panel::State`
//! (pan/zoom/selection/drag/menu) and draws the graph into the docked
//! `motion_graph` region: background + grid, wires (under), then node cards with
//! category-tinted headers, domain-colored sockets, an Accent ring on the
//! selection, the in-progress wire ghost, and the add-node popup. It registers
//! `GraphSurface` hit rects — background, then wires, then node bodies, then
//! sockets (last wins, so a socket beats the node body and a node beats a wire) —
//! and publishes the canvas rect so the M0 dispatch routes gestures back here.
//! All colors are tokens (HR-15); the add-menu is hit-tested panel-side against
//! `Background` gestures (no menu-specific `GraphHitKind`). Sizes are logical.

use crate::geom::{self, View, card_h, socket_center};
use crate::snapshot::{
    GraphEdgeView, GraphNodeView, GraphViewSnapshot, current_catalog, current_snapshot,
};
use crate::state::{AddMenu, Interaction, MotionGraphPanelState, ViewState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{GraphHitKind, InteractiveState};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_title, resolve, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::NodeUiCategory;
use ph2d_nodegraph::port::Domain;
use ph2d_tokens::{ColorToken, Theme};

// Draw-only metrics (the shared graph metrics live in `geom`). These are
// logical units in the graph's OWN coordinate space (scaled by `zoom` at paint),
// not chrome design tokens — the token system (Spacing/Radius/TypeToken/…) covers
// panel chrome, and a node-graph canvas has its own geometry (like the sprite /
// vector canvases). Hence `LITERAL-PX-OK` per line.
const CARD_RADIUS: f32 = 7.0; // LITERAL-PX-OK: node card corner radius
const SOCKET_R: f32 = 5.0; // LITERAL-PX-OK: socket dot radius
const FIT_PAD: f32 = 44.0; // LITERAL-PX-OK: fit-view margin
const TARGET_RING_PAD: f32 = 3.0; // LITERAL-PX-OK: compatible-target ring beyond the socket
const TITLE_PAD_X: f32 = 8.0; // LITERAL-PX-OK: card title left inset
const TITLE_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: card title top inset
const TITLE_SIZE: f32 = 13.0; // LITERAL-PX-OK: card title font size
const TITLE_INSET_R: f32 = 12.0; // LITERAL-PX-OK: card title right inset
const GRID_STEP: f32 = 32.0; // LITERAL-PX-OK: background grid spacing
const WIRE_W: f32 = 2.4; // LITERAL-PX-OK: wire stroke width
const WIRE_W_DELAYED: f32 = 1.6; // LITERAL-PX-OK: delayed (pre) wire stroke width
const GHOST_W: f32 = 2.2; // LITERAL-PX-OK: in-progress ghost wire stroke width
const WIRE_TANGENT: f32 = 20.0; // LITERAL-PX-OK: horizontal bezier handle length
const ZOOM_FIT_MIN: f32 = 0.35; // LITERAL-PX-OK: auto-fit zoom floor
const ZOOM_FIT_MAX: f32 = 1.2; // LITERAL-PX-OK: auto-fit zoom ceiling
// Add-node popup draw metrics.
const MENU_RADIUS: f32 = 6.0; // LITERAL-PX-OK: popup corner radius
const MENU_HEADER_PAD_X: f32 = 4.0; // LITERAL-PX-OK: header text x-pad beyond MENU_PAD
const MENU_HEADER_PAD_Y: f32 = 4.0; // LITERAL-PX-OK: header text y-pad
const MENU_HEADER_SIZE: f32 = 12.0; // LITERAL-PX-OK: header font size
const MENU_DOT_X: f32 = 7.0; // LITERAL-PX-OK: row category-dot x-inset
const MENU_DOT_R: f32 = 4.0; // LITERAL-PX-OK: row category-dot radius
const MENU_ROW_TEXT_X: f32 = 18.0; // LITERAL-PX-OK: row label x-inset
const MENU_ROW_TEXT_Y: f32 = 3.0; // LITERAL-PX-OK: row label y-inset
const MENU_ROW_SIZE: f32 = 13.0; // LITERAL-PX-OK: row label font size
const MENU_ROW_TEXT_INSET_R: f32 = 20.0; // LITERAL-PX-OK: row label right inset
/// How many polyline points a wire's hit path is sampled at (segments = N-1);
/// each segment becomes one padded hit rect sharing the wire's id.
const WIRE_HIT_SAMPLES: usize = 9;
/// Half-thickness (screen px) of a wire's hit rects — a forgiving alt-click zone.
const WIRE_HIT_R: f32 = 5.0; // LITERAL-PX-OK: wire hit half-thickness

pub(crate) fn paint(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
    let rect = ctx.layout.motion_graph;
    let theme = ctx.host.theme();
    let snap = current_snapshot();

    // Fit the graph on first sight (then the user owns pan/zoom; F re-fits).
    if !state.fitted && !snap.nodes.is_empty() {
        state.view = fit(&snap, rect);
        state.fitted = true;
    }

    // Fold this frame's gestures/zoom/keys into the state before drawing.
    crate::interact::process(state, ctx, rect, &snap);

    // Publish the selection so the shell bridge can build the params snapshot for
    // the selected node (M1.P1). Cheap: a small Vec, only while the tool is up.
    crate::snapshot::set_graph_selection(state.selected.iter().copied().collect());

    let view = View::new(rect, state.view);

    // Background + grid (Fase A opaque cover + orientation grid).
    fill_rounded_rect(ctx.scene, rect, 0.0, resolve(ColorToken::GraphBg, theme));
    draw_grid(ctx, rect, theme);

    // Wires under the cards.
    for e in &snap.edges {
        draw_wire(ctx, &snap, e, &view, theme);
    }

    // Hit rects, lowest-priority first (last registered wins): background →
    // wires → node bodies → sockets. So a socket beats the node body it sits on,
    // a node body beats a wire behind it, and a wire beats empty canvas.
    let mut hits: Vec<(NodeId, GraphHitKind, Rect)> =
        vec![(bg_hit_id(), GraphHitKind::Background, rect)];
    for e in &snap.edges {
        push_wire_hits(&mut hits, &snap, e, &view, rect);
    }
    // Cards, collecting body hits as we draw them.
    for n in &snap.nodes {
        let body = draw_card(ctx, state, n, &view, theme);
        hits.push((
            node_hit_id(n.id),
            GraphHitKind::Node { node: n.id as u64 },
            body,
        ));
    }
    // Sockets last so they win the overlap with the node edge they sit on.
    for n in &snap.nodes {
        push_socket_hits(&mut hits, n, &view);
    }
    // While the add-menu is open, a full-canvas Background shield registered LAST
    // makes every click resolve as Background — so a menu row drawn over a card /
    // socket still reaches the menu, and a click off the menu dismisses it
    // (`interact::apply_background`).
    if state.add_menu.is_some() {
        hits.push((bg_hit_id(), GraphHitKind::Background, rect));
    }

    // Overlays above the cards: the in-progress wire ghost, then the add-menu.
    if let Interaction::DrawWire { .. } = &state.interaction {
        draw_wire_ghost(ctx, &snap, state, &view, theme);
    }
    if let Some(menu) = state.add_menu {
        draw_add_menu(ctx, &menu, rect, theme);
    }

    register_hits(ctx, rect, &hits);
}

/// Auto-fit: scale + center the node bounding box into `rect`.
fn fit(snap: &GraphViewSnapshot, rect: Rect) -> ViewState {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for n in &snap.nodes {
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + geom::CARD_W);
        max_y = max_y.max(n.y + card_h(n));
    }
    let bw = (max_x - min_x).max(1.0);
    let bh = (max_y - min_y).max(1.0);
    let zoom = ((rect.w - 2.0 * FIT_PAD) / bw)
        .min((rect.h - 2.0 * FIT_PAD) / bh)
        .clamp(ZOOM_FIT_MIN, ZOOM_FIT_MAX); // CLAMP-OK: const bounds, min<max, non-NaN
    ViewState {
        pan_x: (rect.w - bw * zoom) * 0.5 - min_x * zoom,
        pan_y: (rect.h - bh * zoom) * 0.5 - min_y * zoom,
        zoom,
    }
}

fn draw_grid(ctx: &mut PaintCtx, rect: Rect, theme: Theme) {
    let grid = resolve(ColorToken::GraphGrid, theme);
    let step = GRID_STEP;
    let mut x = rect.x;
    while x < rect.x + rect.w {
        stroke_polyline(ctx.scene, &[(x, rect.y), (x, rect.y + rect.h)], 1.0, grid);
        x += step;
    }
    let mut y = rect.y;
    while y < rect.y + rect.h {
        stroke_polyline(ctx.scene, &[(rect.x, y), (rect.x + rect.w, y)], 1.0, grid);
        y += step;
    }
}

/// Draw one node card; returns its screen-space body rect (for hit registration).
fn draw_card(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
) -> Rect {
    let (sx, sy) = view.pt(n.x, n.y);
    let w = geom::CARD_W * view.zoom;
    let h = card_h(n) * view.zoom;
    let r = CARD_RADIUS * view.zoom;
    let body = Rect::new(sx, sy, w, h);

    fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::Bg2, theme));
    let header = Rect::new(sx, sy, w, geom::HEADER_H * view.zoom);
    fill_rounded_rect(ctx.scene, header, r, resolve(cat_token(n.category), theme));
    stroke_rounded_rect(ctx.scene, body, r, 1.0, resolve(ColorToken::Border, theme));
    if state.selected.contains(&n.id) {
        stroke_rounded_rect(ctx.scene, body, r, 2.0, resolve(ColorToken::Accent, theme));
    }

    paint_text_title(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        sx + TITLE_PAD_X * view.zoom,
        sy + TITLE_PAD_Y * view.zoom,
        TITLE_SIZE * view.zoom,
        w - TITLE_INSET_R * view.zoom,
        resolve(ColorToken::Text1, theme),
    );

    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        fill_circle(
            ctx.scene,
            cx,
            cy,
            SOCKET_R * view.zoom,
            resolve(domain_token(p.domain), theme),
        );
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        fill_circle(
            ctx.scene,
            cx,
            cy,
            SOCKET_R * view.zoom,
            resolve(domain_token(p.domain), theme),
        );
    }
    body
}

fn draw_wire(
    ctx: &mut PaintCtx,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    theme: Theme,
) {
    let Some((p0, p3)) = wire_endpoints(snap, e, view) else {
        return;
    };
    let pts = wire_polyline(p0, p3, view.zoom, 20);
    let width = if e.delayed { WIRE_W_DELAYED } else { WIRE_W } * view.zoom;
    stroke_polyline(
        ctx.scene,
        &pts,
        width,
        resolve(domain_token(e.out_domain), theme),
    );
}

/// The in-progress wire ghost: from the source output socket to the live pointer,
/// colored by the source domain when the hovered target is compatible, muted
/// `Border` when incompatible; a compatible target socket also gets an Accent
/// ring so the drop point is unambiguous.
fn draw_wire_ghost(
    ctx: &mut PaintCtx,
    snap: &GraphViewSnapshot,
    state: &MotionGraphPanelState,
    view: &View,
    theme: Theme,
) {
    let Interaction::DrawWire {
        from_node,
        from_port,
        cur,
        target,
    } = &state.interaction
    else {
        return;
    };
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

/// The add-node popup: a `Bg2` panel with a header + one row per catalog entry,
/// each a category-tinted dot + the English display name. Hit-tested in
/// `interact` against `Background` gestures via `geom::add_menu_row`.
fn draw_add_menu(ctx: &mut PaintCtx, menu: &AddMenu, canvas: Rect, theme: Theme) {
    let catalog = current_catalog();
    let panel = geom::add_menu_panel(menu, catalog.len(), canvas);
    fill_rounded_rect(
        ctx.scene,
        panel,
        MENU_RADIUS,
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        panel,
        MENU_RADIUS,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    paint_text_title(
        ctx.text_system,
        ctx.scene,
        "Add Node",
        panel.x + geom::MENU_PAD + MENU_HEADER_PAD_X,
        panel.y + MENU_HEADER_PAD_Y,
        MENU_HEADER_SIZE,
        panel.w - 2.0 * geom::MENU_PAD,
        resolve(ColorToken::Text2, theme),
    );
    for (i, c) in catalog.iter().enumerate() {
        let row = geom::add_menu_row(panel, i);
        fill_circle(
            ctx.scene,
            row.x + MENU_DOT_X,
            row.y + row.h * 0.5,
            MENU_DOT_R,
            resolve(cat_token(c.category), theme),
        );
        paint_text_title(
            ctx.text_system,
            ctx.scene,
            c.display,
            row.x + MENU_ROW_TEXT_X,
            row.y + MENU_ROW_TEXT_Y,
            MENU_ROW_SIZE,
            row.w - MENU_ROW_TEXT_INSET_R,
            resolve(ColorToken::Text1, theme),
        );
    }
}

/// Register the background + wire + node + socket `GraphSurface` hit rects (in
/// the order given — last wins) so the M0 dispatch routes gestures here, and
/// republish the canvas rect for the wheel-zoom hit-test. Keyboard focus is
/// cursor-gated by the shell, so it is NOT set here.
fn register_hits(ctx: &mut PaintCtx, rect: Rect, hits: &[(NodeId, GraphHitKind, Rect)]) {
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

/// Screen endpoints `(output socket, input socket)` of a wire, or `None` if
/// either endpoint node isn't in the snapshot.
fn wire_endpoints(
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
fn wire_polyline(p0: (f32, f32), p3: (f32, f32), zoom: f32, n: usize) -> Vec<(f32, f32)> {
    let dx = ((p3.0 - p0.0).abs() * 0.5 + WIRE_TANGENT * zoom).max(WIRE_TANGENT * zoom);
    cubic_polyline(p0, (p0.0 + dx, p0.1), (p3.0 - dx, p3.1), p3, n)
}

/// Push a wire's hit rects (all sharing one id encoding the target input) —
/// several padded segment boxes along the flattened bezier, culled to `canvas`.
fn push_wire_hits(
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    canvas: Rect,
) {
    let Some((p0, p3)) = wire_endpoints(snap, e, view) else {
        return;
    };
    let id = wire_hit_id(e.to_node, e.to_port);
    let kind = GraphHitKind::Wire {
        edge: wire_handle(e.to_node, e.to_port),
    };
    let pts = wire_polyline(p0, p3, view.zoom, WIRE_HIT_SAMPLES);
    for seg in pts.windows(2) {
        let (ax, ay) = seg[0];
        let (bx, by) = seg[1];
        let x = ax.min(bx) - WIRE_HIT_R;
        let y = ay.min(by) - WIRE_HIT_R;
        let r = Rect::new(
            x,
            y,
            (ax - bx).abs() + 2.0 * WIRE_HIT_R,
            (ay - by).abs() + 2.0 * WIRE_HIT_R,
        );
        if rects_overlap(r, canvas) {
            hits.push((id, kind, r));
        }
    }
}

/// Push a node's input + output socket hit rects (square, fixed screen size).
fn push_socket_hits(hits: &mut Vec<(NodeId, GraphHitKind, Rect)>, n: &GraphNodeView, view: &View) {
    for (i, _) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        hits.push((
            sock_in_id(n.id, i),
            GraphHitKind::SocketIn {
                node: n.id as u64,
                port: i as u16,
            },
            socket_hit_rect(cx, cy),
        ));
    }
    for (i, _) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        hits.push((
            sock_out_id(n.id, i),
            GraphHitKind::SocketOut {
                node: n.id as u64,
                port: i as u16,
            },
            socket_hit_rect(cx, cy),
        ));
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

fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/// An Accent ring around a socket (the compatible drop-target highlight), drawn
/// as a rounded-rect stroke whose corner radius equals its half-side — i.e. a
/// circle — so it reuses `stroke_rounded_rect` (no per-frame trig, HR-5-clean).
fn highlight_socket(ctx: &mut PaintCtx, cx: f32, cy: f32, r: f32, theme: Theme) {
    let d = 2.0 * r;
    stroke_rounded_rect(
        ctx.scene,
        Rect::new(cx - r, cy - r, d, d),
        r,
        2.0,
        resolve(ColorToken::Accent, theme),
    );
}

/// The source output port's [`Domain`] (ghost color) — the snapshot carries it
/// on the port view.
fn port_out_domain(n: &GraphNodeView, port: u16) -> Domain {
    n.outputs
        .get(port as usize)
        .map(|p| p.domain)
        .unwrap_or(Domain::Instances)
}

/// FNV-1a-64 of `key` — the runtime sibling of `hash_node_id` (which is
/// `&'static str`-only) for the dynamic per-element hit ids.
fn fnv_id(key: &str) -> NodeId {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    NodeId(h)
}
fn bg_hit_id() -> NodeId {
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
fn wire_hit_id(to_node: u32, to_port: u16) -> NodeId {
    fnv_id(&format!("motion_graph/wire/{to_node}/{to_port}"))
}

/// Opaque `Wire { edge }` handle = the target input `(to_node, to_port)`, which
/// uniquely identifies the edge (one edge per input). Decoded in `interact` for
/// `Disconnect`.
pub(crate) fn wire_handle(to_node: u32, to_port: u16) -> u64 {
    ((to_node as u64) << 16) | (to_port as u64)
}
/// Inverse of [`wire_handle`].
pub(crate) fn wire_target(handle: u64) -> (u32, u16) {
    ((handle >> 16) as u32, (handle & 0xffff) as u16)
}

fn cat_token(c: NodeUiCategory) -> ColorToken {
    match c {
        NodeUiCategory::Source => ColorToken::NodeCatSource,
        NodeUiCategory::Distribute => ColorToken::NodeCatDistribute,
        NodeUiCategory::Transform => ColorToken::NodeCatTransform,
        NodeUiCategory::Focus => ColorToken::NodeCatFocus,
        NodeUiCategory::Fx => ColorToken::NodeCatFx,
        NodeUiCategory::Output => ColorToken::NodeCatOutput,
        NodeUiCategory::Utility => ColorToken::NodeCatUtility,
    }
}

fn domain_token(d: Domain) -> ColorToken {
    match d {
        Domain::Instances => ColorToken::PortInstances,
        Domain::Vector => ColorToken::PortVector,
        Domain::Field => ColorToken::PortField,
        Domain::Signal => ColorToken::PortSignal,
        Domain::Control => ColorToken::PortControl,
    }
}

/// Flatten a cubic Bézier into `n` polyline points.
fn cubic_polyline(
    p0: (f32, f32),
    c1: (f32, f32),
    c2: (f32, f32),
    p3: (f32, f32),
    n: usize,
) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|k| {
            let t = k as f32 / n as f32;
            let u = 1.0 - t;
            // Cubic Bézier Bernstein basis: B(t) = u³·p0 + 3u²t·c1 + 3ut²·c2 + t³·p3.
            let three = 3.0; // LITERAL-PX-OK: Bernstein-basis coefficient (math, not design)
            let (a, b, c, d) = (u * u * u, three * u * u * t, three * u * t * t, t * t * t);
            (
                a * p0.0 + b * c1.0 + c * c2.0 + d * p3.0,
                a * p0.1 + b * c1.1 + c * c2.1 + d * p3.1,
            )
        })
        .collect()
}
