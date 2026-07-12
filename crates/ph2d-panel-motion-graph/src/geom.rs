//! Shared graph geometry (Motion Nodes M1) — the graph⟷screen affine, card /
//! socket metrics, socket hit-testing, and the add-node menu layout.
//!
//! One source of truth used by both `paint` (draw) and `interact` (hit-test), so
//! the socket a gesture lands on and the socket that was drawn can never drift
//! apart. All sizes are logical; the transform scales them by `zoom`.

use crate::snapshot::{GraphNodeView, GraphViewSnapshot};
use crate::state::{AddMenu, ViewState};
use ph2d_editor_core::zones::Rect;

// Card metrics in graph (logical) space. Shared with `paint` (draw) so the hit
// geometry matches the rendered geometry exactly. These are canvas coordinate
// units, not chrome design tokens (see the note in `paint`), hence LITERAL-PX-OK.
pub(crate) const CARD_W: f32 = 190.0; // LITERAL-PX-OK: node card width
pub(crate) const HEADER_H: f32 = 26.0; // LITERAL-PX-OK: node card header height
pub(crate) const ROW_H: f32 = 22.0; // LITERAL-PX-OK: socket-row height
pub(crate) const PAD_BOTTOM: f32 = 10.0; // LITERAL-PX-OK: node card bottom padding

/// Half-extent (screen px) of a socket's square hit zone — fixed in screen space
/// so a socket stays grabbable when the graph is zoomed out (the drawn dot
/// shrinks with zoom, but the target does not).
pub(crate) const SOCKET_HIT_R: f32 = 9.0; // LITERAL-PX-OK: socket hit half-extent

// Add-node popup metrics (logical == screen; the menu never scales with zoom).
pub(crate) const MENU_W: f32 = 200.0; // LITERAL-PX-OK: add-menu popup width
pub(crate) const MENU_ROW_H: f32 = 22.0; // LITERAL-PX-OK: add-menu row height
pub(crate) const MENU_HEADER_H: f32 = 22.0; // LITERAL-PX-OK: add-menu header height
pub(crate) const MENU_PAD: f32 = 5.0; // LITERAL-PX-OK: add-menu inner padding

/// graph-space → screen-space affine: `screen = base + pan + graph * zoom`.
pub(crate) struct View {
    base_x: f32,
    base_y: f32,
    pan_x: f32,
    pan_y: f32,
    pub(crate) zoom: f32,
}

impl View {
    pub(crate) fn new(rect: Rect, v: ViewState) -> Self {
        Self {
            base_x: rect.x,
            base_y: rect.y,
            pan_x: v.pan_x,
            pan_y: v.pan_y,
            zoom: v.zoom,
        }
    }

    /// graph → screen.
    pub(crate) fn pt(&self, gx: f32, gy: f32) -> (f32, f32) {
        (
            self.base_x + self.pan_x + gx * self.zoom,
            self.base_y + self.pan_y + gy * self.zoom,
        )
    }

    /// screen → graph (inverse of [`Self::pt`]). Used to place a new node at the
    /// R-click point and to keep the add-menu spawn stable across pan/zoom.
    pub(crate) fn graph(&self, sx: f32, sy: f32) -> (f32, f32) {
        (
            (sx - self.base_x - self.pan_x) / self.zoom,
            (sy - self.base_y - self.pan_y) / self.zoom,
        )
    }
}

pub(crate) fn card_rows(n: &GraphNodeView) -> f32 {
    (n.inputs.len().max(n.outputs.len()).max(1)) as f32
}

/// The card's height. A node with a live **readout** is one row taller — the number sits
/// under the sockets, in the space the bottom padding used to hold.
///
/// This lives in `geom` (not in `paint`) because the hit-test and the paint MUST agree: a
/// card that is drawn taller than it is clickable has a dead strip along its bottom edge,
/// and a card clickable past its own border steals from the canvas behind it.
pub(crate) fn card_h(n: &GraphNodeView) -> f32 {
    let readout = if n.readout.is_some() { ROW_H } else { 0.0 };
    HEADER_H + card_rows(n) * ROW_H + readout + PAD_BOTTOM
}

/// Screen center of a socket (`output` picks the right vs left edge; `i` is the
/// row index).
pub(crate) fn socket_center(n: &GraphNodeView, view: &View, output: bool, i: usize) -> (f32, f32) {
    let edge_x = if output { n.x + CARD_W } else { n.x };
    let y = n.y + HEADER_H + i as f32 * ROW_H + ROW_H * 0.5;
    view.pt(edge_x, y)
}

/// The input socket whose square hit zone contains `(x, y)`, if any — the drop
/// target of a wire drag (the End gesture only carries the pointer position, so
/// the panel resolves the target socket itself). Returns `(node id, port)`.
pub(crate) fn hit_input_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    x: f32,
    y: f32,
) -> Option<(u32, u16)> {
    for n in &snap.nodes {
        for (i, _) in n.inputs.iter().enumerate() {
            let (cx, cy) = socket_center(n, view, false, i);
            if (x - cx).abs() <= SOCKET_HIT_R && (y - cy).abs() <= SOCKET_HIT_R {
                return Some((n.id, i as u16));
            }
        }
    }
    None
}

/// The rubber band's screen rect, normalised — the artist may drag in any
/// direction, so the anchor is not necessarily the top-left corner.
pub(crate) fn band_rect(anchor: (f32, f32), cur: (f32, f32)) -> Rect {
    let (x0, x1) = (anchor.0.min(cur.0), anchor.0.max(cur.0));
    let (y0, y1) = (anchor.1.min(cur.1), anchor.1.max(cur.1));
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

/// A card's rect on screen — the same geometry `paint` draws and `hits` registers.
pub(crate) fn card_rect(n: &GraphNodeView, view: &View) -> Rect {
    let (sx, sy) = view.pt(n.x, n.y);
    Rect::new(sx, sy, CARD_W * view.zoom, card_h(n) * view.zoom)
}

/// Every node the rubber band **touches** — its card INTERSECTS the band. Touch,
/// not full containment: sweeping a band across a row of cards is the gesture
/// people actually make, and demanding that each card be swallowed whole would
/// silently miss the ones the band merely crossed. Blender and Nuke both select on
/// intersection.
pub(crate) fn nodes_in_box(snap: &GraphViewSnapshot, view: &View, band: Rect) -> Vec<u32> {
    snap.nodes
        .iter()
        .filter(|n| intersects(card_rect(n, view), band))
        .map(|n| n.id)
        .collect()
}

fn intersects(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The add-node popup's panel rect, clamped so a menu opened near the right/
/// bottom edge stays fully on `canvas`.
pub(crate) fn add_menu_panel(menu: &AddMenu, count: usize, canvas: Rect) -> Rect {
    let h = MENU_HEADER_H + count.max(1) as f32 * MENU_ROW_H + 2.0 * MENU_PAD;
    let x = menu
        .screen
        .0
        .min(canvas.x + canvas.w - MENU_W)
        .max(canvas.x);
    let y = menu.screen.1.min(canvas.y + canvas.h - h).max(canvas.y);
    Rect::new(x, y, MENU_W, h)
}

/// The clickable row rect for the `i`-th catalog entry inside `panel`.
pub(crate) fn add_menu_row(panel: Rect, i: usize) -> Rect {
    Rect::new(
        panel.x + MENU_PAD,
        panel.y + MENU_HEADER_H + MENU_PAD + i as f32 * MENU_ROW_H,
        panel.w - 2.0 * MENU_PAD,
        MENU_ROW_H,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView};
    use crate::state::AddMenu;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    fn node_with_inputs(id: u32, x: f32, n_in: usize) -> GraphNodeView {
        GraphNodeView {
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
        }
    }

    #[test]
    fn hit_input_socket_resolves_the_right_port() {
        let snap = GraphViewSnapshot {
            nodes: vec![node_with_inputs(7, 100.0, 2)],
            edges: vec![],
            backdrops: vec![],
            probe: None,
        };
        let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
        // Input 0 center: (100, HEADER_H + ROW_H*0.5) = (100, 37).
        let (n0, p0) = socket_center(&snap.nodes[0], &view, false, 0);
        assert!(hit_input_socket(&snap, &view, n0, p0) == Some((7, 0)));
        // Input 1 center: (100, HEADER_H + ROW_H*1.5) = (100, 59).
        let (n1, p1) = socket_center(&snap.nodes[0], &view, false, 1);
        assert_eq!(hit_input_socket(&snap, &view, n1, p1), Some((7, 1)));
        // Far from any socket → no hit.
        assert_eq!(hit_input_socket(&snap, &view, 400.0, 400.0), None);
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
            nodes: vec![bare, with_readout],
            edges: vec![],
            backdrops: vec![],
            probe: None,
        };
        assert_eq!(
            nodes_in_box(&snap, &view, band),
            vec![2],
            "only the taller card reaches down into that row"
        );
    }

    #[test]
    fn add_menu_panel_clamps_into_the_canvas() {
        let canvas = Rect::new(0.0, 0.0, 300.0, 200.0);
        // Opened past the right/bottom edge → clamped fully inside.
        let menu = AddMenu {
            screen: (290.0, 190.0),
            spawn: (0.0, 0.0),
            connect_from: None,
        };
        let p = add_menu_panel(&menu, 3, canvas);
        assert!(p.x + p.w <= canvas.x + canvas.w + 0.01);
        assert!(p.y + p.h <= canvas.y + canvas.h + 0.01);
        assert!(p.x >= canvas.x && p.y >= canvas.y);
    }
}
