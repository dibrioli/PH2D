//! `ph2d-panel-brush-studio` — typed `Panel<State>` for the Painter Brush
//! Studio (W5, ADR-0043..0053 / ADR-0044 brush surface).
//!
//! The Brush Studio is the full brush-parameter editor. The brush sidebar
//! ([`ph2d-panel-painter-sidebar`]) carries only the live painting essentials
//! (size / opacity / color + the three W5 toggles); the Studio exposes the rest
//! of the [`ph2d_painter_brush::Brush`] surface across three sections — Stroke
//! Path, Shape, Rendering — mirroring the Inspector's section + scroll layout.
//!
//! Right-docked (shares the Inspector geometry slot, same as the sidebar /
//! layers panels); visible only when the `painter` tool is active AND
//! `PainterTool::show_brush_studio()` is set (the shell `painter_bridge` drives
//! `panel_visible("painter_brush_studio")`). Opened from the sidebar header
//! "Brush Studio" button; closed from this panel's X.
//!
//! ## Param canon location
//!
//! `PainterTool` (shell-side ToolRegistry) owns the canonical `Brush`. Each
//! frame the shell publishes a [`ph2d_tool_painter::BrushStudioSnapshot`] via
//! [`set_current_brush_studio_snapshot`] → `paint` reads it. Edits go out via
//! `EditorAction::ToolPanelEvent` (ADR-0040 TG-B generic channel) → the shell
//! calls `PainterTool::handle_panel_event` which dispatches via
//! `PainterUiEdit::SetBrushParam` (one capped variant for the whole studio).

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod populate;
mod section_rows;
mod sections;
pub mod state;

pub use state::{
    BrushStudioPanelState, last_content_h, last_visible_h, set_current_brush_studio_snapshot,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Brush Studio panel contract.
pub struct BrushStudioPanel;

impl Panel for BrushStudioPanel {
    type State = BrushStudioPanelState;

    const ID: &'static str = "painter_brush_studio";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PAINTER_BRUSH_STUDIO_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut BrushStudioPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut BrushStudioPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
