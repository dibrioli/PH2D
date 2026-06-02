//! `ph2d-panel-vector-inspector` — typed `Panel<State>` (ADR-0029) for the
//! Vector module inspector (W2.T2.4).
//!
//! Right-docked (Inspector geometry slot); visible only while a vector
//! selection tool (`vector_select` / `vector_direct`) is active — the shell
//! drives `panel_visible("vector_inspector")` from the active-tool id.
//!
//! Minimal v1 hosts the **fill-color swatch**: a picker swatch registered via
//! [`ph2d_editor_core::interaction::WidgetStore::register_picker_swatch`], so a
//! Down opens the shared Blender picker (generic dispatch, pointer.rs). The
//! shell read-back (`vector_inspector_bridge`) reflects the picked color into
//! the swatch + `App.vector_fill_color` and applies it to the selected regions.
//! Future: vertex / node param editors land here (the deferred T2.3 inspector).

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{
    VectorInspectorPanelState, last_content_h, last_visible_h, set_current_fill, set_current_shape,
    take_pending_shape_selection,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Vector Inspector panel contract.
pub struct VectorInspectorPanel;

impl Panel for VectorInspectorPanel {
    type State = VectorInspectorPanelState;

    const ID: &'static str = "vector_inspector";
    const NODE_ID: NodeId = ph2d_editor_core::ids::VECTOR_INSPECTOR_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut VectorInspectorPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut VectorInspectorPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
