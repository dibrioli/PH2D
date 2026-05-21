//! `ph2d-panel-padding` — typed `Panel<State>` for the stateful Padding
//! tool (ADR-0029).
//!
//! Right-docked in the Inspector's geometry slot; visible only while the
//! `padding` tool is active (the shell drives `panel_visible("padding")`
//! from the active-tool id). Holds four signed per-edge `NumberInput`
//! fields (Top / Right / Bottom / Left) + Cancel / Apply. No live
//! preview in v1 — Apply bakes the resized canvas shell-side.
//!
//! The authoritative `PaddingTool` lives in the shell's `ToolRegistry`,
//! unreachable from `HeroScreen`. So:
//! - the host publishes a [`PaddingUiSnapshot`] each frame via
//!   [`set_current_padding_snapshot`] → [`paint`] reads it;
//! - panel edits flow out over `EditorAction::PaddingUiEdit` → the shell
//!   drains them into `PaddingTool::apply_ui_edit`.
//!
//! Mirrors `ph2d-panel-bgremoval` 1:1, minus the preview machinery.
//!
//! [`PaddingUiSnapshot`]: ph2d_editor_core::tools::padding::PaddingUiSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{PaddingPanelState, last_content_h, last_visible_h, set_current_padding_snapshot};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Padding panel contract.
pub struct PaddingPanel;

impl Panel for PaddingPanel {
    type State = PaddingPanelState;

    const ID: &'static str = "padding";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PAD_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut PaddingPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PaddingPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
