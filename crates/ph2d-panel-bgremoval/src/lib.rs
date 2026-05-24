//! `ph2d-panel-bgremoval` — typed `Panel<State>` for the Background
//! Removal tool (ADR-0029).
//!
//! Right-docked in the Inspector's geometry slot; visible only while the
//! `bgremoval` tool is active (the shell drives `panel_visible("bgremoval")`
//! from the active-tool id). Holds the tool's controls only — Mode
//! (Chroma / Smart Cut), Tolerance / Feather / Refine sliders, and an
//! Apply button. The **live preview renders on the canvas** (the shell
//! swaps the sprite's texture for the in-progress result), not inside
//! this panel.
//!
//! The authoritative `BgRemovalTool` lives in the shell's `ToolRegistry`,
//! unreachable from `HeroScreen`. So:
//! - the host publishes a normalized [`BgRemovalUiSnapshot`] each frame
//!   via [`set_current_bgremoval_snapshot`] → [`paint`] reads it;
//! - panel edits flow out over `EditorAction::ToolPanelEvent` → the
//!   shell calls `BgRemovalTool::handle_panel_event`, which routes
//!   each event through `apply_ui_edit` (ADR-0040 TG-B).
//!
//! [`BgRemovalUiSnapshot`]: ph2d_tool_bgremoval::params::BgRemovalUiSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod paint_sections;
mod populate;
pub mod state;

pub use state::{
    BgRemovalPanelState, last_content_h, last_visible_h, set_current_bgremoval_snapshot,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Background-Removal panel
/// contract.
pub struct BgRemovalPanel;

impl Panel for BgRemovalPanel {
    type State = BgRemovalPanelState;

    const ID: &'static str = "bgremoval";
    const NODE_ID: NodeId = ph2d_editor_core::ids::BGR_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut BgRemovalPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut BgRemovalPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
