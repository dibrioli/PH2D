//! `ph2d-panel-color-equalization` — typed `Panel<State>` for the stateful
//! Color Equalization tool (ADR-0029).
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `color_equalization` tool is active (the shell drives
//! `panel_visible("color_equalization")` from the active-tool id). Holds
//! five slider+chip rows (clip limit / tile grid / brightness / contrast
//! / saturation), an Auto-WB toggle, and Cancel / Apply.
//!
//! The authoritative `ColorEqualizationTool` lives in the shell's
//! `ToolRegistry`, unreachable from `HeroScreen`. So:
//! - the host publishes a [`ColorEqualizationUiSnapshot`] each frame via
//!   [`set_current_snapshot`] → [`paint`] reads it;
//! - panel edits flow out over `EditorAction::ToolPanelEvent` → the shell
//!   calls `ColorEqualizationTool::handle_panel_event`, which routes each
//!   event through `apply_ui_edit` (ADR-0040 TG-C precedent — clamps live
//!   exactly once in the tool's params module).
//!
//! Mirrors `ph2d-panel-padding` 1:1 in chrome shape, with a couple of
//! adjustments: five rows instead of four, an extra toggle, and the
//! number chips display natural units (not pixels) — clip limit `2.0`,
//! tile grid `8`, brightness `0.50`, etc.
//!
//! [`ColorEqualizationUiSnapshot`]:
//! ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;

pub use state::{last_content_h, last_visible_h, set_current_histogram, set_current_snapshot};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Color Equalization panel
/// contract.
pub struct ColorEqualizationPanel;

/// Retained per-instance state slot for `ColorEqualizationPanel`.
/// Intentionally empty — the authoritative state lives on the
/// shell-side `ColorEqualizationTool`; the panel renders the per-frame
/// snapshot. `Default` required by the `Panel::State: Default` bound.
#[derive(Clone, Debug, Default)]
pub struct ColorEqualizationPanelState;

impl Panel for ColorEqualizationPanel {
    type State = ColorEqualizationPanelState;
    const ID: &'static str = "color_equalization";
    const NODE_ID: NodeId = ids::CEQ_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut ColorEqualizationPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut ColorEqualizationPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
