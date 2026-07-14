//! `ph2d-panel-motion-graph` — the Motion Nodes graph-editor panel (M0.T9).
//!
//! Docked in the `motion_graph` region of [`HeroLayout`] (the graph half of the
//! center split), visible only while the `motion` tool is active — the shell's
//! `motion_bridge` drives `panel_visible("motion_graph")`.
//!
//! **M0 skeleton:** paints the opaque graph-canvas background (`graph-bg`, Fase A
//! of plan §2.1 — the fill covers the sprite render beneath the graph half) and
//! publishes its rect so pointer dispatch can route to it. Node cards, wiring,
//! pan/zoom and gestures land in M1 (the `GraphSurface` dispatch + snapshot come
//! from M0.T2/T3 + the bridge).

#![forbid(unsafe_code)]

mod backdrop;
mod flow;
mod geom;
mod hits;
mod interact;
mod paint;
mod paint_chrome;
mod probe;
mod rename;
mod snapshot;
mod state;

/// The smallest a backdrop may be resized to (graph space). Published so the
/// shell — which owns the document and therefore the clamp — enforces exactly the
/// minimum the panel drew and hit-tested, instead of a second number that could
/// drift from it.
pub use backdrop::{MIN_H as BACKDROP_MIN_H, MIN_W as BACKDROP_MIN_W};
pub use snapshot::{
    ChoiceTarget, Crumb, GraphBackdropView, GraphEdgeView, GraphIntent, GraphNodeView,
    GraphViewSnapshot, HiddenPorts, NodeChoice, NodeViewKind, PROBE_SAMPLES, PortChoice, PortView,
    ProbeView, RenameTarget, SUBGRAPH_VIEW_TAG, card_hidden_ports,
    current_graph_backdrop_selection, current_graph_selection, drain_intents, is_subgraph_view,
    pending_graph_selection, push_intent, request_graph_selection, set_card_hidden_ports,
    set_current_motion_graph, set_current_node_catalog, set_graph_backdrop_selection,
    set_graph_selection, snapshot_from,
};
pub use state::MotionGraphPanelState;

/// **Is the add-menu open?** (seam gates only — the panel's state is private, and a test that
/// cannot see the menu cannot tell "it never opened" from "it opened and ate the click".)
pub fn menu_is_open(state: &MotionGraphPanelState) -> bool {
    state.menu_for_test().is_some()
}

/// The first row's rect, as the PAINT laid it out — the row the artist clicks. `None` when no
/// menu is open (or nothing matched the search).
/// The add-menu's search field id (seam gates: *does the field hold the keyboard, and does it
/// give it back?*).
pub fn menu_search_widget() -> NodeId {
    hits::menu_search_id()
}

/// Screen → graph, through the panel's OWN view (seam gates: a test that places a node "under
/// the cursor" has to use the same map the panel draws with, or it places it somewhere else).
pub fn graph_point(
    state: &MotionGraphPanelState,
    rect: zones::Rect,
    sx: f32,
    sy: f32,
) -> (f32, f32) {
    geom::View::new(rect, state.view_for_test()).graph(sx, sy)
}

pub fn first_menu_row(state: &MotionGraphPanelState, rect: zones::Rect) -> Option<zones::Rect> {
    let menu = state.menu_for_test()?;
    let snap = snapshot::current_snapshot();
    let rows = snapshot::menu_rows(&snap, menu);
    if rows.is_empty() {
        return None;
    }
    let panel = geom::menu_panel(menu, rows.len(), rect);
    Some(geom::menu_row(menu, panel, 0, menu.scroll))
}
use ph2d_editor_core::zones;

#[path = "menu_search.rs"]
mod menu_search;

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed graph-editor panel contract.
pub struct MotionGraphPanel;

impl Panel for MotionGraphPanel {
    type State = MotionGraphPanelState;

    const ID: &'static str = "motion_graph";
    const NODE_ID: NodeId = ids::MOTION_GRAPH_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionGraphPanel::ID) {
            // Stale-rect cleanup so `panel_at` stops returning this panel once the
            // tool deactivates; also drop the graph keyboard focus.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            ctx.host.store_mut().set_graph_focused(None);
            return;
        }
        let rect = ctx.layout.motion_graph;
        if rect.w <= 0.0 || rect.h <= 0.0 {
            // No split (or a degenerate one) → nothing to draw.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            return;
        }
        // Publish the rect so wheel/click dispatch can route to this panel.
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_GRAPH_PANEL, rect);
        // Fase A background + the M1 graph (cards / sockets / wires) + gesture
        // handling — reads the snapshot the shell bridge published this frame.
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut MotionGraphPanelState,
        _host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        // The graph's own gestures arrive on the `GraphSurface` dispatch channel, not here.
        // The ONE widget in this panel is the add-menu's search field (doc 59), and it owns
        // the keyboard while it is open — so Enter and Esc are ITS events, not the graph's.
        match ev {
            // **Enter takes the top match.** A search box where the obvious key does nothing
            // makes you reach for the mouse to click the row you are already looking at.
            WidgetEvent::Submit(id) if id == hits::menu_search_id() => {
                if let Some(menu) = state.menu.take() {
                    menu_search::pick_first(&menu);
                }
                EventOutcome::Consumed
            }
            // **Esc closes the popup**, rather than merely blurring the field and leaving a
            // menu on screen that no longer answers the keyboard.
            WidgetEvent::Cancel(id) if id == hits::menu_search_id() => {
                state.menu = None;
                EventOutcome::Consumed
            }
            // **Enter names the thing** (doc 61). The box is the other widget in this panel, and
            // it owns the keyboard while it is open, so Enter and Esc are ITS events too.
            WidgetEvent::Submit(id) if id == hits::rename_id() => {
                rename::commit(state, _host.store());
                EventOutcome::Consumed
            }
            // **Esc keeps the old name.** (`mark_cancel_on_escape` is what routes Esc here rather
            // than merely blurring the field.)
            WidgetEvent::Cancel(id) if id == hits::rename_id() => {
                state.rename = None;
                EventOutcome::Consumed
            }
            _ => EventOutcome::Ignored,
        }
    }

    fn populate(_store: &mut WidgetStore) {
        // No focusable widgets in the M0 skeleton.
    }
}
