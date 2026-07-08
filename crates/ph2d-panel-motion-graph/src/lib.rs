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

mod geom;
mod interact;
mod paint;
mod snapshot;
mod state;

pub use snapshot::{
    GraphEdgeView, GraphIntent, GraphNodeView, GraphViewSnapshot, NodeChoice, PortView,
    drain_intents, set_current_motion_graph, set_current_node_catalog, snapshot_from,
};
pub use state::MotionGraphPanelState;

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
        _state: &mut MotionGraphPanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        // No interactive widgets yet — graph gestures arrive via the
        // `GraphSurface` dispatch channel (M0.T2/T3), not per-widget events.
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // No focusable widgets in the M0 skeleton.
    }
}
