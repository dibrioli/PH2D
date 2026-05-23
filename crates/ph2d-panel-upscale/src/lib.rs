//! `ph2d-panel-upscale` — typed `Panel<State>` for the stateful Upscale
//! tool (ADR-0029).
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `upscale` tool is active (the shell drives `panel_visible("upscale")`
//! from the active-tool id). Holds the algorithm segmented control
//! (Lanczos3 / Nearest / xBR), a scale slider (1×–16×) paired with a
//! number chip, and Cancel / Apply.
//!
//! The authoritative [`UpscaleTool`] lives in the shell's `ToolRegistry`,
//! unreachable from `HeroScreen`. So:
//! - the host publishes an [`UpscaleUiSnapshot`] each frame via
//!   [`set_current_upscale_snapshot`] → [`paint`] reads it;
//! - panel edits flow out over `EditorAction::ToolPanelEvent` → the
//!   shell calls `UpscaleTool::handle_panel_event`, which routes each
//!   event through `apply_ui_edit` (ADR-0040 TG-C).
//!
//! Mirrors `ph2d-panel-padding` 1:1, plus a live preview thumbnail
//! that paints the tool's current `preview_rgba` (via the snapshot).
//!
//! [`UpscaleTool`]: ph2d_tool_upscale::UpscaleTool
//! [`UpscaleUiSnapshot`]: ph2d_tool_upscale::params::UpscaleUiSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{UpscalePanelState, last_content_h, last_visible_h, set_current_upscale_snapshot};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

/// Zero-size marker implementing the typed Upscale panel contract.
pub struct UpscalePanel;

impl Panel for UpscalePanel {
    type State = UpscalePanelState;
    const ID: &'static str = "upscale";
    const NODE_ID: NodeId = hash_node_id("panel.upscale");
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut UpscalePanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut UpscalePanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
