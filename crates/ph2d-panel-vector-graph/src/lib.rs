//! `ph2d-panel-vector-graph` — typed `Panel<State>` (ADR-0029) for the
//! Vector module's Geometry-Graph (W3.T3.1).
//!
//! A docked panel titled "Geometry Graph" hosting 8 labeled horizontal
//! sliders, one per `vector.source` node param (kind / width / height /
//! sides / inner_ratio / turns / samples_per_turn / rotation). Each slider
//! is the canonical normalized `0..1` [`SliderState`]; on `ValueChanged`
//! the `0..1` track maps to the param's real range and the real value
//! lands in a thread-local [`VectorGraphParams`] "param bus".
//!
//! The shell render bridge (built by the Coordinator) reads
//! [`current_graph_params`] each frame to cook + render the
//! `vector.source` node. The bus is seeded to [`VectorGraphParams::default`]
//! so a cook before any slider move still produces the default Rect.
//!
//! Sliders only — no node-placement / edge-drag UI (that's all the Day-8
//! smoke needs). Mirrors `ph2d-panel-vector-inspector` 1:1 in structure.

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{
    VectorGraphPanelState, VectorGraphParams, current_graph_params, last_content_h, last_visible_h,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Geometry-Graph panel contract.
pub struct VectorGraphPanel;

impl Panel for VectorGraphPanel {
    type State = VectorGraphPanelState;

    const ID: &'static str = "vector_graph";
    const NODE_ID: NodeId = ph2d_editor_core::ids::VGRAPH_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut VectorGraphPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut VectorGraphPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
