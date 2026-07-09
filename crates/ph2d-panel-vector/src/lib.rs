//! `ph2d-panel-vector` — typed `Panel<State>` for the Vector tool's Style
//! (ADR-0029 / ADR-0108 cutover).
//!
//! Right-docked in the Inspector slot; visible only while the `vector` tool is
//! active (the shell drives `panel_visible("vector")` from the active-tool id).
//! Holds a **Width** slider+chip (1..20 px) + **Stroke** and **Fill** colour
//! swatches (each opens the shared OKLCH picker) + a Fill **None** button.
//!
//! Tool `FloatingPanel`s are unpainted in this app (input-dispatch only), so the
//! authoritative `VectorTool` lives in the shell's `ToolRegistry`, unreachable
//! from `HeroScreen`. Therefore:
//! - the host publishes a [`VectorStyleSnapshot`] each frame via
//!   [`set_current_vector_style`] → [`paint`] reads it;
//! - Width / Fill-None edits flow out over `EditorAction::ToolPanelEvent` → the
//!   shell calls `VectorTool::handle_panel_event`;
//! - the Stroke / Fill swatches open the OKLCH picker (generic dispatch) and the
//!   shell's `vector_bridge` reads the pick back into
//!   `VectorTool::set_stroke_rgba` / `set_fill_rgba`.
//!
//! Mirrors `ph2d-panel-padding` (+ the retired vector inspector's picker
//! swatch).
//!
//! [`VectorStyleSnapshot`]: ph2d_tool_vector::VectorStyleSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod paint_arrange;
mod paint_sections;
mod paint_transform;
pub mod populate;
pub mod state;

pub use state::{
    FillKind, VectorPanelState, last_content_h, last_visible_h, set_current_fill,
    set_current_grad_influence, set_current_grad_jitter, set_current_path_closed,
    set_current_transform, set_current_vector_style, set_selected_vertex_type,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Vector Style panel contract.
pub struct VectorPanel;

impl Panel for VectorPanel {
    type State = VectorPanelState;

    const ID: &'static str = "vector";
    const NODE_ID: NodeId = ph2d_editor_core::ids::VECTOR_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut VectorPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut VectorPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
