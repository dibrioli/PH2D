//! `ph2d-panel-color-equalization` — typed `Panel<State>` stub for the
//! stateful Color Equalization tool (ADR-0029). Coord pre-work for the
//! Implementador opening this tool in parallel with the other 3 image
//! tools (Equalize Sizes / Rasterize / Upscale).
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `color_equalization` tool is active (the shell drives
//! `panel_visible("color_equalization")` from the active-tool id).
//!
//! Implementador preenche `state.rs` (UI snapshot mirror), `paint.rs`
//! (sliders + Apply/Cancel), `event.rs` (modifier → `PanelEvent` →
//! `ToolPanelEvent` bus push), and `populate.rs` (NodeId registration).
//! Vocab `ColorEqualizationUiEdit` / `ColorEqualizationUiSnapshot` /
//! `ColorEqualizationParams` lives in `crates/ph2d-tool-color-equalization/src/params.rs`
//! (TG-B/TG-C single-source-of-truth pattern — vide bgremoval/padding).

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

/// Zero-size marker implementing the typed Color Equalization panel
/// contract. Implementador can keep the marker name; the panel body
/// lives across `paint.rs` / `event.rs` / `state.rs` / `populate.rs`.
pub struct ColorEqualizationPanel;

/// Panel retained state — Implementador adds the UI mirror fields
/// (e.g. live snapshot of the tool's params, drag-in-progress flags,
/// preview cache handle). Default = empty.
#[derive(Default)]
pub struct ColorEqualizationPanelState;

impl Panel for ColorEqualizationPanel {
    type State = ColorEqualizationPanelState;
    const ID: &'static str = "color_equalization";
    const NODE_ID: NodeId = hash_node_id("panel.color_equalization");
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut ColorEqualizationPanelState, _ctx: &mut PaintCtx) {
        // Implementador preenche.
    }

    fn apply_event(
        _state: &mut ColorEqualizationPanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // Implementador registra ids (sliders, Apply, Cancel).
    }
}
