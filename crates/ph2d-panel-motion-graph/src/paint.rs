//! Motion-graph editor paint (Motion Nodes M1.E1–E4, Phase 1a: read-only draw).
//!
//! Reads the published [`GraphViewSnapshot`] and draws the graph into the docked
//! `motion_graph` region: background + grid, wires (under), then node cards with
//! category-tinted headers + domain-colored sockets. The view auto-fits the
//! graph into the panel each frame (interactive pan/zoom + hit registration land
//! in Phase 1b). All colors come from tokens (HR-15); all sizes are logical.

use crate::snapshot::{GraphEdgeView, GraphNodeView, GraphViewSnapshot, current_snapshot};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, resolve, stroke_polyline, stroke_rounded_rect};
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

/// graph-space → screen-space affine (uniform scale + translate).
struct View {
    base_x: f32,
    base_y: f32,
    off_x: f32,
    off_y: f32,
    scale: f32,
}
impl View {
    fn pt(&self, gx: f32, gy: f32) -> (f32, f32) {
        (
            self.base_x + self.off_x + gx * self.scale,
            self.base_y + self.off_y + gy * self.scale,
        )
    }
}

/// Header rows a card needs = max(inputs, outputs), at least one.
fn card_rows(n: &GraphNodeView) -> f32 {
    (n.inputs.len().max(n.outputs.len()).max(1)) as f32
}
fn card_h(n: &GraphNodeView) -> f32 {
    HEADER_H + card_rows(n) * ROW_H + PAD_BOTTOM
}

pub(crate) fn paint(ctx: &mut PaintCtx) {
    let rect = ctx.layout.motion_graph;
    let theme = ctx.host.theme();

    // Background + grid always paint (Fase A opaque cover + orientation grid).
    fill_rounded_rect(ctx.scene, rect, 0.0, resolve(ColorToken::GraphBg, theme));
    draw_grid(ctx, rect, theme);

    let snap = current_snapshot();
    if snap.nodes.is_empty() {
        return;
    }
    let view = fit_view(&snap, rect);

    // Wires first (drawn under the cards).
    for e in &snap.edges {
        draw_wire(ctx, &snap, e, &view, theme);
    }
    for n in &snap.nodes {
        draw_card(ctx, n, &view, theme);
    }
}

/// Auto-fit: scale + center the graph's node bounding box into `rect`.
fn fit_view(snap: &GraphViewSnapshot, rect: Rect) -> View {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for n in &snap.nodes {
        min_x = min_x.min(n.x);
        min_y = min_y.min(n.y);
        max_x = max_x.max(n.x + CARD_W);
        max_y = max_y.max(n.y + card_h(n));
    }
    let bw = (max_x - min_x).max(1.0);
    let bh = (max_y - min_y).max(1.0);
    let scale = ((rect.w - 2.0 * FIT_PAD) / bw)
        .min((rect.h - 2.0 * FIT_PAD) / bh)
        .clamp(0.35, 1.2);
    View {
        base_x: rect.x,
        base_y: rect.y,
        off_x: (rect.w - bw * scale) * 0.5 - min_x * scale,
        off_y: (rect.h - bh * scale) * 0.5 - min_y * scale,
        scale,
    }
}

fn draw_grid(ctx: &mut PaintCtx, rect: Rect, theme: Theme) {
    let grid = resolve(ColorToken::GraphGrid, theme);
    let step = 32.0;
    // Vertical lines.
    let mut x = rect.x;
    while x < rect.x + rect.w {
        stroke_polyline(ctx.scene, &[(x, rect.y), (x, rect.y + rect.h)], 1.0, grid);
        x += step;
    }
    // Horizontal lines.
    let mut y = rect.y;
    while y < rect.y + rect.h {
        stroke_polyline(ctx.scene, &[(rect.x, y), (rect.x + rect.w, y)], 1.0, grid);
        y += step;
    }
}

fn draw_card(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    let (sx, sy) = view.pt(n.x, n.y);
    let w = CARD_W * view.scale;
    let h = card_h(n) * view.scale;
    let r = CARD_RADIUS * view.scale;
    let body = Rect::new(sx, sy, w, h);

    // Body + border.
    fill_rounded_rect(ctx.scene, body, r, resolve(ColorToken::Bg2, theme));
    // Header strip (category tint) — rounded top, sits over the body's top.
    let header = Rect::new(sx, sy, w, HEADER_H * view.scale);
    fill_rounded_rect(ctx.scene, header, r, resolve(cat_token(n.category), theme));
    stroke_rounded_rect(ctx.scene, body, r, 1.0, resolve(ColorToken::Border, theme));

    // Title on the header.
    ph2d_editor_core::paint::paint_text_title(
        ctx.text_system,
        ctx.scene,
        &n.display_name,
        sx + 8.0 * view.scale,
        sy + 5.0 * view.scale,
        13.0 * view.scale,
        w - 12.0 * view.scale,
        resolve(ColorToken::Text1, theme),
    );

    // Sockets: inputs on the left edge, outputs on the right.
    for (i, p) in n.inputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, false, i);
        fill_circle(ctx.scene, cx, cy, SOCKET_R * view.scale, resolve(domain_token(p.domain), theme));
    }
    for (i, p) in n.outputs.iter().enumerate() {
        let (cx, cy) = socket_center(n, view, true, i);
        fill_circle(ctx.scene, cx, cy, SOCKET_R * view.scale, resolve(domain_token(p.domain), theme));
    }
}

/// Screen center of a socket. `output` picks the right vs left edge; `i` is the
/// row index.
fn socket_center(n: &GraphNodeView, view: &View, output: bool, i: usize) -> (f32, f32) {
    let edge_x = if output { n.x + CARD_W } else { n.x };
    let y = n.y + HEADER_H + i as f32 * ROW_H + ROW_H * 0.5;
    view.pt(edge_x, y)
}

fn draw_wire(
    ctx: &mut PaintCtx,
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
    let (x0, y0) = socket_center(from, view, true, e.from_port as usize);
    let (x1, y1) = socket_center(to, view, false, e.to_port as usize);
    // Horizontal tangents (Cavalry-style): control handles reach toward each
    // other by |dx|·0.5 + 20 logical px.
    let dx = ((x1 - x0).abs() * 0.5 + 20.0 * view.scale).max(20.0 * view.scale);
    let pts = cubic_polyline((x0, y0), (x0 + dx, y0), (x1 - dx, y1), (x1, y1), 20);
    let width = if e.delayed { 1.6 } else { 2.4 } * view.scale;
    stroke_polyline(ctx.scene, &pts, width, resolve(domain_token(e.out_domain), theme));
}

/// Flatten a cubic Bézier into `n` polyline points (Phase 1a — dashed `delayed`
/// wires + activity-fire are Phase 1b polish).
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
