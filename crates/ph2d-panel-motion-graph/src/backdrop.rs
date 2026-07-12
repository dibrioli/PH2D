//! Backdrops (Motion Nodes F2) — the translucent group regions that make a graph
//! of 70-odd nodes legible: draw a tinted, titled frame around a cluster and it
//! reads as *one thing* ("the force chain", "the colour pass").
//!
//! **The model is Nuke's BackdropNode / Houdini's network box**, not Blender's
//! frame: the backdrop owns no children in the document (`Backdrop` is
//! `{id, x, y, w, h, color, title}` and nothing else — pure decoration, never
//! part of the cook). What it frames is decided **geometrically, at grab time**:
//! dragging its header carries every node whose centre lies inside it. That is
//! why there is no parent pointer to keep in sync, no re-parenting on drop, and
//! no way for the doc to disagree with what the eye sees.
//!
//! **The body is click-through — deliberately.** Only the header strip and the two
//! bottom-corner grippers register hit rects; the body registers nothing, so
//! clicking or box-selecting *over* a backdrop still reaches the nodes and the
//! canvas beneath it. A backdrop that swallowed clicks in its body would make every
//! node it frames unselectable — the exact bug that makes a grouping tool unusable.
//!
//! **The resize affordance is the app's own**, not a shape invented here: the same
//! corner dot + hit zone every panel uses (`paint_panel_corner_dot{,_bl}` /
//! `panel_resize_handle_rect{,_bl}`), in BOTH bottom corners, and grabbing either
//! one holds the opposite edge still (Enio, smoke 2026-07-12).
//!
//! All geometry is graph-space (scaled by `zoom` at paint), like the node cards.

use crate::geom::{View, card_h};
use crate::snapshot::{GraphBackdropView, GraphNodeView};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_title, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::panel_chrome::{
    paint_panel_corner_dot, paint_panel_corner_dot_bl, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

// Graph-space metrics — the canvas has its own coordinate system (like the node
// cards in `geom`/`paint`), not chrome design tokens. Hence `LITERAL-PX-OK`.
/// The draggable / selectable title strip along the top.
pub(crate) const HEADER_H: f32 = 28.0; // LITERAL-PX-OK: backdrop header strip height
/// A backdrop can never be resized smaller than its own header + a usable body —
/// collapsing it to a sliver would take its header (the only grabbable part) with
/// it. Published from the crate root: the shell owns the document and applies the
/// clamp, so it must use the panel's number, not a copy that could drift.
pub const MIN_W: f32 = 120.0; // LITERAL-PX-OK: minimum backdrop width
pub const MIN_H: f32 = HEADER_H + 60.0; // LITERAL-PX-OK: minimum backdrop height
/// A backdrop added with nothing selected — big enough to drag nodes into.
pub(crate) const NEW_W: f32 = 340.0; // LITERAL-PX-OK: default new-backdrop width
pub(crate) const NEW_H: f32 = 220.0; // LITERAL-PX-OK: default new-backdrop height
/// Breathing room left around the nodes a new backdrop wraps.
pub(crate) const WRAP_PAD: f32 = 28.0; // LITERAL-PX-OK: wrap margin around the selection

const RADIUS: f32 = 8.0; // LITERAL-PX-OK: backdrop corner radius
const BORDER_W: f32 = 1.0; // LITERAL-PX-OK: backdrop border stroke width
const SELECTED_W: f32 = 2.0; // LITERAL-PX-OK: selected border stroke width
const TITLE_PAD_X: f32 = 10.0; // LITERAL-PX-OK: title left inset
const TITLE_PAD_Y: f32 = 6.0; // LITERAL-PX-OK: title top inset
const TITLE_SIZE: f32 = 14.0; // LITERAL-PX-OK: title font size
const TITLE_INSET_R: f32 = 14.0; // LITERAL-PX-OK: title right inset

/// The whole region, in screen space.
pub(crate) fn body_rect(b: &GraphBackdropView, view: &View) -> Rect {
    let (sx, sy) = view.pt(b.x, b.y);
    Rect::new(sx, sy, b.w * view.zoom, b.h * view.zoom)
}

/// The title strip — the ONLY part that drags and selects (see the module docs).
pub(crate) fn header_rect(b: &GraphBackdropView, view: &View) -> Rect {
    let (sx, sy) = view.pt(b.x, b.y);
    Rect::new(sx, sy, b.w * view.zoom, HEADER_H * view.zoom)
}

/// The resize grippers — **both bottom corners**, exactly the app's panels
/// (`panel_resize_handle_rect{,_bl}`): same size, same corners, same accent dot.
/// A backdrop is a region the artist resizes, so it uses the resize affordance the
/// rest of the app already taught them, not a shape of its own.
///
/// Their size is SCREEN space (a fixed grab target, like the sockets' `SOCKET_HIT_R`):
/// the corner stays grabbable when the graph is zoomed out, where a zoom-scaled
/// gripper would shrink into nothing.
pub(crate) fn grip_rects(b: &GraphBackdropView, view: &View) -> (Rect, Rect) {
    let body = body_rect(b, view);
    (
        panel_resize_handle_rect_bl(body),
        panel_resize_handle_rect(body),
    )
}

/// Pack a gripper into the opaque handle `GraphHitKind::BackdropResize` carries:
/// the backdrop id plus WHICH corner. Editor-core never looks inside these handles
/// (the panel owns them) — the same trick `paint::wire_handle` uses to pack an
/// edge's target into `GraphHitKind::Wire`.
pub(crate) fn resize_handle(id: u32, left: bool) -> u64 {
    (id as u64) | ((left as u64) << 32)
}

/// Unpack [`resize_handle`] — `(id, is the LEFT corner)`.
pub(crate) fn resize_target(handle: u64) -> (u32, bool) {
    (handle as u32, (handle >> 32) & 1 == 1)
}

/// Whether `b` frames `n` — the node's CENTRE lies inside the region. Centre, not
/// full containment: a node half-hanging off the edge still belongs to the group
/// the eye sees, and "fully inside" would silently drop it from the drag.
pub(crate) fn frames_node(b: &GraphBackdropView, n: &GraphNodeView) -> bool {
    let cx = n.x + crate::geom::CARD_W * 0.5;
    let cy = n.y + card_h(n) * 0.5;
    cx >= b.x && cx <= b.x + b.w && cy >= b.y && cy <= b.y + b.h
}

/// The graph-space rect a new backdrop takes to wrap `nodes` — their bounding box
/// plus [`WRAP_PAD`], with the header's own height added ABOVE the topmost node
/// (otherwise the title strip would sit on top of the first row of cards it is
/// meant to introduce). Empty selection → `None` (the caller places a default).
pub(crate) fn wrap_of(nodes: &[&GraphNodeView]) -> Option<(f32, f32, f32, f32)> {
    let first = nodes.first()?;
    let (mut x0, mut y0) = (first.x, first.y);
    let (mut x1, mut y1) = (first.x, first.y);
    for n in nodes {
        x0 = x0.min(n.x);
        y0 = y0.min(n.y);
        x1 = x1.max(n.x + crate::geom::CARD_W);
        y1 = y1.max(n.y + card_h(n));
    }
    let x = x0 - WRAP_PAD;
    let y = y0 - WRAP_PAD - HEADER_H;
    Some((
        x,
        y,
        (x1 + WRAP_PAD - x).max(MIN_W),
        (y1 + WRAP_PAD - y).max(MIN_H),
    ))
}

/// The tint token for a backdrop's colour index. Out-of-range clamps to the last
/// (the document may carry any `u8`; the doc's own contract says clamp at paint).
pub(crate) fn tint_token(color: u8) -> ColorToken {
    match color {
        0 => ColorToken::GraphBackdrop1,
        1 => ColorToken::GraphBackdrop2,
        2 => ColorToken::GraphBackdrop3,
        3 => ColorToken::GraphBackdrop4,
        4 => ColorToken::GraphBackdrop5,
        5 => ColorToken::GraphBackdrop6,
        6 => ColorToken::GraphBackdrop7,
        _ => ColorToken::GraphBackdrop8,
    }
}

/// Draw one backdrop — BEHIND the wires and cards (the caller's draw order).
///
/// The header reads as a header without a second token: the tints are translucent
/// (alpha 0.15), so filling the strip with the SAME tint a second time composites
/// it to roughly twice the opacity of the body. One token, two weights.
pub(crate) fn draw(
    ctx: &mut PaintCtx,
    b: &GraphBackdropView,
    view: &View,
    theme: Theme,
    selected: bool,
) {
    let body = body_rect(b, view);
    let r = RADIUS * view.zoom;
    let tint = resolve(tint_token(b.color), theme);
    fill_rounded_rect(ctx.scene, body, r, tint);
    fill_rounded_rect(ctx.scene, header_rect(b, view), r, tint);
    stroke_rounded_rect(
        ctx.scene,
        body,
        r,
        BORDER_W,
        resolve(ColorToken::Border, theme),
    );
    if selected {
        stroke_rounded_rect(
            ctx.scene,
            body,
            r,
            SELECTED_W,
            resolve(ColorToken::Accent, theme),
        );
    }
    // The resize affordance is the app's OWN: the same soft corner dot every panel
    // draws, in both bottom corners (the panels are grabbable from either).
    paint_panel_corner_dot(body, ctx.scene, theme);
    paint_panel_corner_dot_bl(body, ctx.scene, theme);
    paint_text_title(
        ctx.text_system,
        ctx.scene,
        &b.title,
        body.x + TITLE_PAD_X * view.zoom,
        body.y + TITLE_PAD_Y * view.zoom,
        TITLE_SIZE * view.zoom,
        body.w - TITLE_INSET_R * view.zoom,
        resolve(ColorToken::Text1, theme),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ViewState;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};

    fn node(id: u32, x: f32, y: f32) -> GraphNodeView {
        GraphNodeView {
            id,
            display_name: "n".into(),
            category: NodeUiCategory::Utility,
            silhouette: NodeSilhouette::Rect,
            x,
            y,
            inputs: vec![],
            outputs: vec![],
            readout: None,
        }
    }

    fn backdrop(x: f32, y: f32, w: f32, h: f32) -> GraphBackdropView {
        GraphBackdropView {
            id: 1,
            x,
            y,
            w,
            h,
            color: 0,
            title: "Group".into(),
        }
    }

    /// Framing is by the node's CENTRE: a card whose middle is inside belongs to
    /// the group even if a corner pokes out; one whose middle is outside does not.
    #[test]
    fn a_backdrop_frames_the_nodes_whose_centre_it_contains() {
        let b = backdrop(0.0, 0.0, 400.0, 300.0);
        assert!(frames_node(&b, &node(1, 100.0, 100.0)), "well inside");
        assert!(
            frames_node(&b, &node(2, 300.0, 100.0)),
            "centre inside, right edge poking out"
        );
        assert!(!frames_node(&b, &node(3, 500.0, 100.0)), "outside");
    }

    /// Wrapping a selection leaves room above the topmost card for the title strip
    /// — a header drawn ON the first row would hide the very nodes it introduces.
    #[test]
    fn wrapping_a_selection_leaves_room_for_the_header() {
        let (a, b) = (node(1, 100.0, 100.0), node(2, 400.0, 260.0));
        let (x, y, w, h) = wrap_of(&[&a, &b]).expect("a non-empty selection wraps");
        assert!(y <= 100.0 - HEADER_H, "the header clears the topmost card");
        assert!(x <= 100.0 - WRAP_PAD, "padded on the left");
        assert!(x + w >= 400.0 + crate::geom::CARD_W + WRAP_PAD, "and right");
        assert!(y + h >= 260.0 + card_h(&b) + WRAP_PAD, "and below");
        // And it frames exactly what it wrapped.
        let framed = GraphBackdropView {
            id: 1,
            x,
            y,
            w,
            h,
            color: 0,
            title: String::new(),
        };
        assert!(frames_node(&framed, &a) && frames_node(&framed, &b));
    }

    #[test]
    fn an_empty_selection_wraps_nothing() {
        assert_eq!(wrap_of(&[]), None);
    }

    /// The header sits on top and the two grippers sit in the BOTTOM CORNERS — the
    /// hit rects the pointer will look for, in the same places every panel in the
    /// app puts them (they come from `panel_resize_handle_rect{,_bl}` itself).
    #[test]
    fn the_header_and_both_grippers_sit_where_the_app_puts_them() {
        let b = backdrop(10.0, 20.0, 400.0, 300.0);
        let view = View::new(Rect::new(0.0, 0.0, 900.0, 700.0), ViewState::default());
        let (body, header) = (body_rect(&b, &view), header_rect(&b, &view));
        let (bl, br) = grip_rects(&b, &view);

        assert_eq!((header.x, header.y), (body.x, body.y));
        assert_eq!(header.w, body.w);
        assert!(header.h < body.h);
        // Bottom-left: flush with the left edge and the bottom.
        assert!((bl.x - body.x).abs() < 1e-3);
        assert!((bl.y + bl.h - (body.y + body.h)).abs() < 1e-3);
        // Bottom-right: flush with the right edge and the bottom.
        assert!((br.x + br.w - (body.x + body.w)).abs() < 1e-3);
        assert!((br.y + br.h - (body.y + body.h)).abs() < 1e-3);
        assert!(bl.x < br.x, "two distinct corners, not one");
    }

    /// The corner is packed into the opaque hit handle and comes back out — the
    /// gripper the artist grabbed is the gripper the resize honours.
    #[test]
    fn the_resize_handle_round_trips_the_corner() {
        assert_eq!(resize_target(resize_handle(7, true)), (7, true));
        assert_eq!(resize_target(resize_handle(7, false)), (7, false));
        assert_eq!(
            resize_target(resize_handle(u32::MAX, true)),
            (u32::MAX, true)
        );
    }

    /// Every colour index resolves, and an out-of-range one clamps instead of
    /// panicking (a hand-edited document may carry any `u8`).
    #[test]
    fn every_colour_index_resolves_and_out_of_range_clamps() {
        assert_eq!(tint_token(0), ColorToken::GraphBackdrop1);
        assert_eq!(tint_token(7), ColorToken::GraphBackdrop8);
        assert_eq!(tint_token(200), ColorToken::GraphBackdrop8);
    }
}
