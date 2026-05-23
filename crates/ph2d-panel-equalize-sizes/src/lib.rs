//! `ph2d-panel-equalize-sizes` — typed `Panel<State>` for the stateful
//! Equalize Sizes tool (ADR-0029). Multi-sprite obrigatório (Fase 0).
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `equalize_sizes` tool is active (the shell drives
//! `panel_visible("equalize_sizes")` from the active-tool id).
//!
//! Mirrors `ph2d-panel-padding` 1:1 in shape: the authoritative
//! `EqualizeSizesTool` lives in the shell's `ToolRegistry`, unreachable
//! from `HeroScreen`. So:
//! - the host publishes a `EqualizeSizesUiSnapshot` each frame via
//!   [`set_current_equalize_sizes_snapshot`] → [`paint`] reads it;
//! - panel edits flow out over `EditorAction::ToolPanelEvent` → the
//!   shell calls `EqualizeSizesTool::handle_panel_event`, which routes
//!   each event through `apply_ui_edit` (ADR-0040 TG-C).

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{
    EqualizeSizesPanelState, last_content_h, last_visible_h, set_current_equalize_sizes_snapshot,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

/// Zero-size marker implementing the typed Equalize Sizes panel
/// contract.
pub struct EqualizeSizesPanel;

impl Panel for EqualizeSizesPanel {
    type State = EqualizeSizesPanelState;

    const ID: &'static str = "equalize_sizes";
    const NODE_ID: NodeId = hash_node_id("panel.equalize_sizes");
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut EqualizeSizesPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut EqualizeSizesPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
