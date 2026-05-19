//! `ph2d-panel-grid-snap` — ADR-0029 Phase C.4 typed panel crate.
//!
//! Owns the Grid Snap floating panel's chrome (the "Grid Settings"
//! window painted by the TopBar Settings cluster):
//! - typed contract via [`GridSnapPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   widget registration ([`populate`]).
//! - panel-side state slot ([`GridSnapPanelState`]) — currently empty
//!   (the per-kind config + canvas color/opacity + `panel_rect` stay
//!   on [`ph2d_editor_core::grid_snap::GridSnapState`], which the
//!   canvas renderer also reads).
//!
//! Phase C.4 closes ADR-0029 Phase C: every in-tree panel
//! (Inspector / Hierarchy / Widget Gallery / Grid Snap) is now a
//! typed `Panel<State>` impl living in its own `ph2d-panel-*` crate.

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod layout;
mod paint;
mod paint_helpers;
mod paint_kinds;
mod paint_rows;
mod populate;
pub mod state;

pub use paint::{
    default_rect, sync_meter_inputs_to_display_unit_pub as sync_meter_inputs_to_display_unit,
};
pub use state::{GridSnapPanelState, last_content_h, last_visible_h, set_current_display_unit};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Grid Snap panel contract.
pub struct GridSnapPanel;

impl Panel for GridSnapPanel {
    type State = GridSnapPanelState;

    const ID: &'static str = "grid_snap";
    const NODE_ID: NodeId = ph2d_editor_core::ids::GS_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut GridSnapPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut GridSnapPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
