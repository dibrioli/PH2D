//! Motion-graph editor paint (Motion Nodes M1.E1–E5, Phase 1b).
//!
//! Reads the published [`GraphViewSnapshot`] + the ephemeral `Panel::State`
//! (pan/zoom/selection/drag) and draws the graph into the docked `motion_graph`
//! region: background + grid, wires (under), then node cards with category-tinted
//! headers, domain-colored sockets, and an Accent ring on the selection. It also
//! registers a `GraphSurface` hit rect per node + one for the background, and
//! publishes the canvas rect + keyboard focus so the M0 dispatch routes gestures
//! back here. All colors are tokens (HR-15); all sizes logical.

use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot, current_snapshot};
use crate::state::{Interaction, MotionGraphPanelState, ViewState};
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

// Card metrics in graph (logical) space.
const CARD_W: f32 = 190.0;
const HEADER_H: f32 = 26.0;
const ROW_H: f32 = 22.0;
const PAD_BOTTOM: f32 = 10.0;
const CARD_RADIUS: f32 = 7.0;
const SOCKET_R: f32 = 5.0;
const FIT_PAD: f32 = 44.0;

/// graph-space → screen-space affine: `screen = base + pan + graph * zoom`.
struct View {
    base_x: f32,
    base_y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
}
impl View {
    fn new(rect: Rect, v: ViewState) -> Self {
        Self {
            base_x: rect.x,
            base_y: rect.y,
            pan_x: v.pan_x,
            pan_y: v.pan_y,
            zoom: v.zoom,
        }
    }
    fn pt(&self, gx: f32, gy: f32) -> (f32, f32) {
        (
            self.base_x + self.pan_x + gx * self.zoom,
            self.base_y + self.pan_y + gy * self.zoom,
        )
    }
}

fn card_rows(n: &GraphNodeView) -> f32 {
    (n.inputs.len().max(n.outputs.len()).max(1)) as f32
}
fn card_h(n: &GraphNodeView) -> f32 {
    HEADER_H + card_rows(n) * ROW_H + PAD_BOTTOM
}

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
    crate::interact::process(state, ctx, rect);

    let view = View::new(rect, state.view);

    // Background + grid (Fase A opaque cover + orientation grid).
    fill_rounded_rect(ctx.scene, rect, 0.0, resolve(ColorToken::GraphBg, theme));
    draw_grid(ctx, rect, theme);

    // Wires under the cards.
    for e in &snap.edges {
        draw_wire(ctx, state, &snap, e, &view, theme);
    }
    // Cards, collecting hit rects as we go (background first so nodes win).
    let mut hits: Vec<(NodeId, GraphHitKind, Rect)> =
        vec![(bg_hit_id(), GraphHitKind::Background, rect)];
    for n in &snap.nodes {
        let body = draw_card(ctx, state, n, &view, theme);
        hits.push((
            node_hit_id(n.id),
            GraphHitKind::Node { node: n.id as u64 },
            body,
        ));
    }

    register_hits(ctx, rect, &hits);
}

/// Auto-fit: scale + center the node bounding box into `rect`.
fn fit(snap: &GraphViewSnapshot, rect: Rect) -> ViewState {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for n in &snap.nodes {
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + CARD_W);
        max_y = max_y.max(n.y + card_h(n));
    }
    let bw = (max_x - min_x).max(1.0);
    let bh = (max_y - min_y).max(1.0);
    let zoom = ((rect.w - 2.0 * FIT_PAD) / bw)
        .min((rect.h - 2.0 * FIT_PAD) / bh)
        .clamp(0.35, 1.2);
    ViewState {
        pan_x: (rect.w - bw * zoom) * 0.5 - min_x * zoom,
        pan_y: (rect.h - bh * zoom) * 0.5 - min_y * zoom,
        zoom,
    }
}

fn draw_grid(ctx: &mut PaintCtx, rect: Rect, theme: Theme) {
    let grid = resolve(ColorToken::GraphGrid, theme);
    let step = 32.0;
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
    let (ox, oy) = node_offset(state, n.id);
    let (sx, sy) = view.pt(n.x + ox, n.y + oy);
    let w = CARD_W * view.zoom;
    let h = card_h(n) * view.zoom;
    let r = CARD_RADIUS * view.zoom;
    let body = Rect::new(sx, sy, w, h);

    fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::Bg2, theme));
    let header = Rect::new(sx, sy, w, HEADER_H * view.zoom);
    fill_rounded_rect(ctx.scene, header, r, resolve(cat_token(n.category), theme));
    stroke_rounded_rect(ctx.scene, body, r, 1.0, resolve(ColorToken::Border, theme));
    if state.selected.contains(&n.id) {
        stroke_rounded_rect(ctx.scene, body, r, 2.0, resolve(ColorToken::Accent, theme));
    }

    paint_text_title(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        sx + 8.0 * view.zoom,
        sy + 5.0 * view.zoom,
        13.0 * view.zoom,
        w - 12.0 * view.zoom,
        resolve(ColorToken::Text1, theme),
    );

    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, ox, oy, false, i);
        fill_circle(ctx.scene, cx, cy, SOCKET_R * view.zoom, resolve(domain_token(p.domain), theme));
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, ox, oy, true, i);
        fill_circle(ctx.scene, cx, cy, SOCKET_R * view.zoom, resolve(domain_token(p.domain), theme));
    }
    body
}

/// Screen center of a socket (`output` picks the right vs left edge; `i` is the
/// row). `ox/oy` is the node's live drag offset in graph space.
fn socket_center(n: &GraphNodeView, view: &View, ox: f32, oy: f32, output: bool, i: usize) -> (f32, f32) {
    let edge_x = if output { n.x + CARD_W } else { n.x } + ox;
    let y = n.y + oy + HEADER_H + i as f32 * ROW_H + ROW_H * 0.5;
    view.pt(edge_x, y)
}

fn draw_wire(
    ctx: &mut PaintCtx,
    state: &MotionGraphPanelState,
    snap: &GraphViewSnapshot,
    e: &GraphEdgeView,
    view: &View,
    theme: Theme,
) {
    let (Some(from), Some(to)) = (
        snap.nodes.iter().find(|n| n.id == e.from_node),
        snap.nodes.iter().find(|n| n.id == e.to_node),
    ) else {
        return;
    };
    let (fox, foy) = node_offset(state, from.id);
    let (tox, toy) = node_offset(state, to.id);
    let (x0, y0) = socket_center(from, view, fox, foy, true, e.from_port as usize);
    let (x1, y1) = socket_center(to, view, tox, toy, false, e.to_port as usize);
    let dx = ((x1 - x0).abs() * 0.5 + 20.0 * view.zoom).max(20.0 * view.zoom);
    let pts = cubic_polyline((x0, y0), (x0 + dx, y0), (x1 - dx, y1), (x1, y1), 20);
    let width = if e.delayed { 1.6 } else { 2.4 } * view.zoom;
    stroke_polyline(ctx.scene, &pts, width, resolve(domain_token(e.out_domain), theme));
}

/// Live graph-space drag offset for `node` (0 unless it is in the active drag).
fn node_offset(state: &MotionGraphPanelState, node: u32) -> (f32, f32) {
    if let Interaction::DragNodes {
        nodes,
        graph_dx,
        graph_dy,
        ..
    } = &state.interaction
        && nodes.contains(&node)
    {
        (*graph_dx, *graph_dy)
    } else {
        (0.0, 0.0)
    }
}

/// Register the background + per-node `GraphSurface` hit rects so the M0 dispatch
/// routes gestures here, and publish the canvas + keyboard focus. States are
/// written first, then rects (background first so nodes win the overlap).
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
        // Wheel-over-canvas → zoom, and route graph keys while docked.
        store.set_graph_canvas(parent, rect);
        store.set_graph_focused(Some(parent));
    }
    let hit_index = ctx.host.hit_index_mut();
    for (id, _, r) in hits {
        hit_index.register(*id, *r);
    }
}

/// FNV-1a-64 of `key` — the runtime sibling of `hash_node_id` (which is
/// `&'static str`-only) for the dynamic per-node hit ids.
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
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            (
                a * p0.0 + b * c1.0 + c * c2.0 + d * p3.0,
                a * p0.1 + b * c1.1 + c * c2.1 + d * p3.1,
            )
        })
        .collect()
}
