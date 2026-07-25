//! `ph2d-panel-flip` — typed `Panel<State>` for the Flip tool (ADR-0114 W2).
//!
//! Right-docked in the Inspector slot; visible only while the `flip` tool is
//! active (the shell's `flip_bridge` drives `panel_visible("flip")` from the
//! active-tool id). Sections, top→bottom:
//! - **Mode** — Select / Draw / Erase segmented (gizmo only in Select, ADR-0112).
//! - **Brush** — Size / Hardness / Opacity / Smoothing sliders.
//! - **Color** — a Stroke swatch that opens the shared OKLCH picker.
//! - **Erase** — Soft / Hard / Stroke sub-mode (only in Erase mode).
//! - **Layers** — add / delete + per-layer visibility / lock / reorder / blend /
//!   opacity (mirror of the Painter layers panel; the idiom the plan asks for).
//!
//! Tool `FloatingPanel`s are unpainted in this app, so the authoritative
//! `FlipTool` lives in the shell's `ToolRegistry`. Therefore (mirror of the
//! Vector Style panel):
//! - the host publishes a [`FlipStyleSnapshot`] + a [`FlipLayersSnapshot`] each
//!   frame via [`set_current_flip_style`] / [`set_current_flip_layers`] → the
//!   `paint` reads them;
//! - Brush / mode / layer edits flow out over `EditorAction::ToolPanelEvent`;
//!   the tool applies the style ones, the shell drain applies the layer ones;
//! - the Stroke swatch opens the OKLCH picker (generic dispatch) and the shell's
//!   `flip_bridge` reads the pick back into `FlipTool::set_stroke_rgba`.
//!
//! [`FlipStyleSnapshot`]: ph2d_tool_flip::FlipStyleSnapshot

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod paint_colorize;
mod paint_layers;
mod paint_rows;
mod paint_sections;
mod paint_tip;
pub mod populate;
pub mod state;

pub use state::{
    FlipLayerRow, FlipLayersSnapshot, LayerRename, last_content_h, last_visible_h,
    set_current_flip_layers, set_current_flip_style,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Flip Style panel contract.
pub struct FlipPanel;

impl Panel for FlipPanel {
    type State = state::FlipPanelState;

    const ID: &'static str = "flip";
    const NODE_ID: NodeId = ph2d_editor_core::ids::FLIP_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut state::FlipPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut state::FlipPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
