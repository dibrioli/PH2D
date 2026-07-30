//! Shared graph geometry (Motion Nodes M1) — the graph⟷screen affine, card /
//! socket metrics, socket hit-testing, and the add-node menu layout.
//!
//! One source of truth used by both `paint` (draw) and `interact` (hit-test), so
//! the socket a gesture lands on and the socket that was drawn can never drift
//! apart. All sizes are logical; the transform scales them by `zoom`.

use crate::snapshot::{GraphNodeView, GraphViewSnapshot};
use crate::state::{Menu, ViewState};
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

/// Screen-px radius within which a wire's loose end SNAPS to a socket — larger than the exact
/// hit zone so a near-miss still lands on its target (magnetic drop). Fixed in screen space,
/// like [`SOCKET_HIT_R`]: the reach must not shrink when the graph is zoomed out. Small enough
/// that a drop in genuinely empty canvas still falls through to smart-connect (doc 45).
pub(crate) const SNAP_R: f32 = 22.0; // LITERAL-PX-OK: wire-drop magnetic snap radius

// Add-node popup metrics (logical == screen; the menu never scales with zoom).
pub(crate) const MENU_W: f32 = 200.0; // LITERAL-PX-OK: add-menu popup width
pub(crate) const MENU_ROW_H: f32 = 22.0; // LITERAL-PX-OK: add-menu row height
pub(crate) const MENU_HEADER_H: f32 = 22.0; // LITERAL-PX-OK: add-menu header height
pub(crate) const MENU_PAD: f32 = 5.0; // LITERAL-PX-OK: add-menu inner padding
/// The search field's band, between the header and the list.
pub(crate) const MENU_SEARCH_H: f32 = 26.0; // LITERAL-PX-OK: add-menu search field height
/// Breathing room between the popup and the canvas edge (it is capped to the canvas height, so
/// without this it would sit flush against the top and bottom).
const MENU_MARGIN: f32 = 8.0; // LITERAL-PX-OK: add-menu margin from the canvas edge
pub(crate) const MENU_BAR_W: f32 = 6.0; // LITERAL-PX-OK: add-menu scrollbar width
const MENU_THUMB_MIN_H: f32 = 24.0; // LITERAL-PX-OK: smallest grabbable scrollbar thumb

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

// Breadcrumb metrics (doc 57) — logical == screen (chrome, never scaled by zoom).
pub(crate) const CRUMB_H: f32 = 20.0; // LITERAL-PX-OK: breadcrumb chip height
pub(crate) const CRUMB_PAD_X: f32 = 8.0; // LITERAL-PX-OK: breadcrumb chip x-padding
pub(crate) const CRUMB_GAP: f32 = 4.0; // LITERAL-PX-OK: gap between crumbs (the separator sits here)
pub(crate) const CRUMB_INSET: f32 = 8.0; // LITERAL-PX-OK: breadcrumb inset from the graph corner
pub(crate) const CRUMB_TEXT_SIZE: f32 = 12.0; // LITERAL-PX-OK: breadcrumb font size
const CRUMB_CHAR_W: f32 = 6.6; // LITERAL-PX-OK: mean advance at CRUMB_TEXT_SIZE (no measure API)
const CRUMB_MAX_W: f32 = 140.0; // LITERAL-PX-OK: a long group name is truncated, not a wall

/// The width of one crumb. **The draw and the hit rect both come from here** — a
/// second estimate is how a clickable thing comes to sit somewhere other than where
/// it was drawn. (There is no text-measure API in this stack, so the width is a
/// mean-advance estimate; the label is clipped to the same box by `paint_text_title`.)
pub(crate) fn crumb_w(title: &str) -> f32 {
    (2.0 * CRUMB_PAD_X + title.chars().count() as f32 * CRUMB_CHAR_W).min(CRUMB_MAX_W)
}

/// The screen rect of crumb `i`, laid out left to right from the graph's top-left.
pub(crate) fn crumb_rect(canvas: Rect, titles: &[String], i: usize) -> Rect {
    let x = canvas.x
        + CRUMB_INSET
        + titles[..i]
            .iter()
            .map(|t| crumb_w(t) + CRUMB_GAP)
            .sum::<f32>();
    Rect::new(x, canvas.y + CRUMB_INSET, crumb_w(&titles[i]), CRUMB_H)
}

/// The postage stamp's strip (F3): the little scatter of what the node emits, under its
/// readout. Wide as the card, tall enough to tell a spiral from a grid.
pub(crate) const PREVIEW_H: f32 = 52.0; // LITERAL-PX-OK: postage-stamp strip height
const PREVIEW_PAD: f32 = 6.0; // LITERAL-PX-OK: inset of the stamp inside its strip

/// The card's height. A node with a live **readout** is one row taller (the number sits under
/// the sockets), and a node with a **postage stamp** carries the strip under that.
///
/// This lives in `geom` (not in `paint`) because the hit-test and the paint MUST agree: a
/// card that is drawn taller than it is clickable has a dead strip along its bottom edge,
/// and a card clickable past its own border steals from the canvas behind it.
pub(crate) fn card_h(n: &GraphNodeView) -> f32 {
    let readout = if n.readout.is_some() { ROW_H } else { 0.0 };
    let preview = if n.preview.is_some() { PREVIEW_H } else { 0.0 };
    HEADER_H + card_rows(n) * ROW_H + readout + preview + PAD_BOTTOM
}

/// The postage stamp's rect in SCREEN space, inset inside the card — or `None` when the node
/// has no stamp. Shared by the paint (there is no hit path: the stamp is part of the card body,
/// and clicking a card must select it wherever you click it).
pub(crate) fn preview_rect(n: &GraphNodeView, view: &View) -> Option<Rect> {
    n.preview.as_ref()?;
    let readout = if n.readout.is_some() { ROW_H } else { 0.0 };
    let top = n.y + HEADER_H + card_rows(n) * ROW_H + readout;
    let (sx, sy) = view.pt(n.x + PREVIEW_PAD, top);
    Some(Rect::new(
        sx,
        sy,
        (CARD_W - 2.0 * PREVIEW_PAD) * view.zoom,
        (PREVIEW_H - PREVIEW_PAD) * view.zoom,
    ))
}

/// Screen center of a socket (`output` picks the right vs left edge; `i` is the
/// row index).
pub(crate) fn socket_center(n: &GraphNodeView, view: &View, output: bool, i: usize) -> (f32, f32) {
    let edge_x = if output { n.x + CARD_W } else { n.x };
    let y = n.y + HEADER_H + i as f32 * ROW_H + ROW_H * 0.5;
    view.pt(edge_x, y)
}

/// The input socket NEAREST to `(x, y)` within [`SNAP_R`], if any — the magnetic drop target
/// of a wire drag (the End gesture only carries the pointer position, so the panel resolves the
/// target socket itself). Returns `(node id, port)`. Nearest by SCREEN distance, so a drop
/// between two rows lands on the closer socket; `None` when nothing is in reach, and the drop
/// then falls through to the collapsed-card menu or smart-connect (doc 45/57).
pub(crate) fn nearest_input_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    x: f32,
    y: f32,
) -> Option<(u32, u16)> {
    nearest_socket(snap, view, x, y, false)
}

/// The OUTPUT socket nearest to `(x, y)` within [`SNAP_R`] — the mirror of
/// [`nearest_input_socket`], for a wire dragged backwards out of an input (doc 45).
pub(crate) fn nearest_output_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    x: f32,
    y: f32,
) -> Option<(u32, u16)> {
    nearest_socket(snap, view, x, y, true)
}

/// The socket of the requested side nearest to `(x, y)` within [`SNAP_R`] — the one door both
/// snap resolvers share, so the input and output magnets can never drift apart.
fn nearest_socket(
    snap: &GraphViewSnapshot,
    view: &View,
    x: f32,
    y: f32,
    output: bool,
) -> Option<(u32, u16)> {
    let r2 = SNAP_R * SNAP_R;
    let mut best: Option<(u32, u16, f32)> = None;
    for n in &snap.nodes {
        // A collapsed card's exposed INPUT sockets ARE the wires crossing it — occupied, so a
        // NEW forward wire has nothing free to land on and goes through the card's port menu
        // (doc 57), never the magnet. Its OUTPUT sockets fan out, so those stay grabbable.
        if !output && n.kind == crate::snapshot::NodeViewKind::Subgraph {
            continue;
        }
        let ports = if output { n.outputs.len() } else { n.inputs.len() };
        for i in 0..ports {
            let (cx, cy) = socket_center(n, view, output, i);
            let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d2 <= r2 && best.is_none_or(|(_, _, bd2)| d2 < bd2) {
                best = Some((n.id, i as u16, d2));
            }
        }
    }
    best.map(|(id, p, _)| (id, p))
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
///
/// A **ghost** is never selected: it does not live at this level (doc 57), and a band
/// that swept one up would let a Delete reach across the boundary and remove a node
/// from a canvas the artist is not even looking at.
pub(crate) fn nodes_in_box(snap: &GraphViewSnapshot, view: &View, band: Rect) -> Vec<u32> {
    snap.nodes
        .iter()
        .filter(|n| n.kind != crate::snapshot::NodeViewKind::Ghost)
        .filter(|n| intersects(card_rect(n, view), band))
        .map(|n| n.id)
        .collect()
}

fn intersects(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The add-node popup's panel rect, clamped so a menu opened near the right/bottom edge stays
/// fully on `canvas` — and **never taller than the canvas**.
///
/// The library has 86 node types. A menu as tall as its list runs off the bottom of the screen and
/// the last forty are unreachable (Enio's screenshot). So the panel is CAPPED, and what does not
/// fit SCROLLS.
/// The popup's chrome above the list: the header, plus the search field **when there is one**
/// (only the library has one — doc 62). Every rect below is measured from this, so the list of a
/// fieldless popup starts where the field would have been and nothing is left floating.
pub(crate) fn menu_chrome_h(menu: &Menu) -> f32 {
    MENU_HEADER_H
        + if menu.has_search() {
            MENU_SEARCH_H
        } else {
            0.0
        }
}

pub(crate) fn menu_panel(menu: &Menu, count: usize, canvas: Rect) -> Rect {
    let chrome = menu_chrome_h(menu);
    let full = chrome + count.max(1) as f32 * MENU_ROW_H + 2.0 * MENU_PAD;
    let room = (canvas.h - 2.0 * MENU_MARGIN).max(chrome + MENU_ROW_H + 2.0 * MENU_PAD);
    let h = full.min(room);
    let x = menu
        .screen
        .0
        .min(canvas.x + canvas.w - MENU_W)
        .max(canvas.x);
    let y = (menu.screen.1)
        .min(canvas.y + canvas.h - h - MENU_MARGIN)
        .max(canvas.y + MENU_MARGIN);
    Rect::new(x, y, MENU_W, h)
}

/// The scrolling VIEWPORT: the band of the panel the rows live in (everything below the header).
/// Rows are drawn clipped to it and hit-tested against it, so a row scrolled half-way out is
/// half-clickable and never spills over the header.
pub(crate) fn menu_list(menu: &Menu, panel: Rect) -> Rect {
    let top = menu_chrome_h(menu);
    Rect::new(
        panel.x + MENU_PAD,
        panel.y + top + MENU_PAD,
        panel.w - 2.0 * MENU_PAD,
        (panel.h - top - 2.0 * MENU_PAD).max(0.0),
    )
}

/// The search field's rect — under the header, above the list.
pub(crate) fn menu_search_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + MENU_PAD,
        panel.y + MENU_HEADER_H,
        panel.w - 2.0 * MENU_PAD,
        MENU_SEARCH_H - MENU_PAD,
    )
}

/// How far the list can scroll before its last row sits on the bottom edge. `0` when everything
/// fits — and then there is no scrollbar and the wheel is not stolen.
pub(crate) fn menu_max_scroll(menu: &Menu, panel: Rect, count: usize) -> f32 {
    (count as f32 * MENU_ROW_H - menu_list(menu, panel).h).max(0.0)
}

/// The clickable row rect for the `i`-th catalog entry, offset by the list's `scroll`.
pub(crate) fn menu_row(menu: &Menu, panel: Rect, i: usize, scroll: f32) -> Rect {
    let list = menu_list(menu, panel);
    Rect::new(
        list.x,
        list.y + i as f32 * MENU_ROW_H - scroll,
        list.w - scrollbar_reserve(panel, i, scroll),
        MENU_ROW_H,
    )
}

/// A row's text stops short of the scrollbar when there is one (a label running under the thumb is
/// a label you cannot read and a thumb you cannot grab).
fn scrollbar_reserve(_panel: Rect, _i: usize, _scroll: f32) -> f32 {
    MENU_BAR_W + MENU_PAD
}

/// The scrollbar's TRACK — a thin column down the right of the list. `None` when everything fits:
/// a scrollbar for a list that cannot scroll is a control that lies.
pub(crate) fn menu_track(menu: &Menu, panel: Rect, count: usize) -> Option<Rect> {
    if menu_max_scroll(menu, panel, count) <= 0.0 {
        return None;
    }
    let list = menu_list(menu, panel);
    Some(Rect::new(
        list.x + list.w - MENU_BAR_W,
        list.y,
        MENU_BAR_W,
        list.h,
    ))
}

/// The draggable THUMB: as tall a fraction of the track as the viewport is of the list, and never
/// shorter than a thing you can actually grab.
pub(crate) fn menu_thumb(menu: &Menu, panel: Rect, count: usize, scroll: f32) -> Option<Rect> {
    let track = menu_track(menu, panel, count)?;
    let list_h = count as f32 * MENU_ROW_H;
    let frac = (track.h / list_h).clamp(0.0, 1.0); // CLAMP-OK: a fraction
    let h = (track.h * frac).max(MENU_THUMB_MIN_H);
    let max_scroll = menu_max_scroll(menu, panel, count);
    let t = if max_scroll > 0.0 {
        (scroll / max_scroll).clamp(0.0, 1.0) // CLAMP-OK: a fraction
    } else {
        0.0
    };
    Some(Rect::new(track.x, track.y + (track.h - h) * t, track.w, h))
}

/// The scroll a thumb dragged to screen `y` means — the inverse of [`menu_thumb`], so the
/// thumb lands under the cursor and stays there. `grab` is where inside the thumb it was seized:
/// grabbing a thumb by its middle and having it jump to put its TOP under the cursor is the
/// scrollbar bug everyone has met.
pub(crate) fn menu_scroll_at(menu: &Menu, panel: Rect, count: usize, y: f32, grab: f32) -> f32 {
    let (Some(track), Some(thumb)) = (
        menu_track(menu, panel, count),
        menu_thumb(menu, panel, count, 0.0),
    ) else {
        return 0.0;
    };
    let span = track.h - thumb.h;
    if span <= 0.0 {
        return 0.0;
    }
    let t = ((y - grab - track.y) / span).clamp(0.0, 1.0); // CLAMP-OK: a fraction
    t * menu_max_scroll(menu, panel, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{GraphNodeView, GraphViewSnapshot, PortView};
    use crate::state::Menu;
    use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
    use ph2d_nodegraph::port::{Clock, Dim, Domain};

    fn node_with_inputs(id: u32, x: f32, n_in: usize) -> GraphNodeView {
        GraphNodeView {
            kind: crate::snapshot::NodeViewKind::Node,
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
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
        }
    }

    fn one_node(node: GraphNodeView) -> GraphViewSnapshot {
        GraphViewSnapshot {
            level: None,
            breadcrumb: Vec::new(),
            nodes: vec![node],
            edges: vec![],
            backdrops: vec![],
            probe: None,
            now: 0.0,
        }
    }

    fn node_with_outputs(id: u32, x: f32, n_out: usize) -> GraphNodeView {
        let mut n = node_with_inputs(id, x, 0);
        n.outputs = (0..n_out)
            .map(|_| PortView {
                name: "o",
                domain: Domain::Instances,
                dim: Dim::Scalar,
                clock: Clock::Frame,
            })
            .collect();
        n
    }

    #[test]
    fn nearest_input_socket_resolves_the_right_port() {
        let snap = one_node(node_with_inputs(7, 100.0, 2));
        let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
        // Input 0 center: (100, HEADER_H + ROW_H*0.5) = (100, 37).
        let (n0, p0) = socket_center(&snap.nodes[0], &view, false, 0);
        assert_eq!(nearest_input_socket(&snap, &view, n0, p0), Some((7, 0)));
        // Input 1 center: (100, HEADER_H + ROW_H*1.5) = (100, 59).
        let (n1, p1) = socket_center(&snap.nodes[0], &view, false, 1);
        assert_eq!(nearest_input_socket(&snap, &view, n1, p1), Some((7, 1)));
        // Far from any socket → no snap.
        assert_eq!(nearest_input_socket(&snap, &view, 400.0, 400.0), None);
    }

    /// **A near-miss SNAPS to the socket** — a drop 15 px above input 0 (past the 9 px exact
    /// hit box, inside the 22 px magnet) still lands on it. FALSIFIED by shrinking `SNAP_R` to
    /// `SOCKET_HIT_R`: 15 > 9, so the near-miss resolves to nothing and the wire is dropped.
    #[test]
    fn a_near_miss_snaps_to_the_socket() {
        let snap = one_node(node_with_inputs(7, 100.0, 2));
        let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
        // Input 0 center (100, 37); 15 px above it — only input 0 is within reach.
        assert_eq!(nearest_input_socket(&snap, &view, 100.0, 22.0), Some((7, 0)));
    }

    /// Between two in-range sockets the magnet takes the **nearer** one — a drop at y=54 is
    /// 17 px from input 0 and 5 px from input 1. FALSIFIED by returning the first socket found
    /// rather than the closest: input 0 (found first, still in range) would win.
    #[test]
    fn snap_takes_the_nearest_of_two_sockets() {
        let snap = one_node(node_with_inputs(7, 100.0, 2));
        let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
        assert_eq!(nearest_input_socket(&snap, &view, 100.0, 54.0), Some((7, 1)));
    }

    /// The output magnet reads **output** sockets (right edge), not inputs — the mirror used by
    /// the backward wire drag. FALSIFIED by resolving against the input side, which this node
    /// does not have.
    #[test]
    fn the_output_magnet_reads_output_sockets() {
        let snap = one_node(node_with_outputs(9, 100.0, 1));
        let view = View::new(Rect::new(0.0, 0.0, 800.0, 600.0), ViewState::default());
        let (ox, oy) = socket_center(&snap.nodes[0], &view, true, 0);
        assert_eq!(nearest_output_socket(&snap, &view, ox + 12.0, oy), Some((9, 0)));
        assert_eq!(nearest_input_socket(&snap, &view, ox + 12.0, oy), None);
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
            level: None,
            breadcrumb: Vec::new(),
            nodes: vec![bare, with_readout],
            edges: vec![],
            backdrops: vec![],
            probe: None,
            now: 0.0,
        };
        assert_eq!(
            nodes_in_box(&snap, &view, band),
            vec![2],
            "only the taller card reaches down into that row"
        );
    }

    #[test]
    fn menu_panel_clamps_into_the_canvas() {
        let canvas = Rect::new(0.0, 0.0, 300.0, 200.0);
        // Opened past the right/bottom edge → clamped fully inside.
        let menu = Menu {
            scroll: 0.0,
            screen: (290.0, 190.0),
            spawn: (0.0, 0.0),
            query: String::new(),
            opened: false,
            body: crate::state::MenuBody::Library {
                connect_from: None,
                splice: None,
            },
        };
        let p = menu_panel(&menu, 3, canvas);
        assert!(p.x + p.w <= canvas.x + canvas.w + 0.01);
        assert!(p.y + p.h <= canvas.y + canvas.h + 0.01);
        assert!(p.x >= canvas.x && p.y >= canvas.y);
    }
}
