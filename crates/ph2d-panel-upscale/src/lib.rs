//! `ph2d-panel-upscale` — typed `Panel<State>` stub for the stateful
//! Upscale tool (ADR-0029). Coord pre-work stub.
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `upscale` tool is active (shell drives `panel_visible("upscale")`).
//!
//! Implementador preenche `state.rs` (UI snapshot — algorithm enum
//! + scale factor + preview cache handle), `paint.rs`, `event.rs`,
//! `populate.rs`. Vocab `UpscaleUiEdit` / `UpscaleUiSnapshot` /
//! `UpscaleParams` lives in the tool crate's `params.rs`
//! (TG-B/TG-C single-source-of-truth).

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

pub struct UpscalePanel;

#[derive(Default)]
pub struct UpscalePanelState;

impl Panel for UpscalePanel {
    type State = UpscalePanelState;
    const ID: &'static str = "upscale";
    const NODE_ID: NodeId = hash_node_id("panel.upscale");
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut UpscalePanelState, _ctx: &mut PaintCtx) {
        // Implementador preenche.
    }

    fn apply_event(
        _state: &mut UpscalePanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // Implementador registra ids (algorithm dropdown, scale slider, Apply, Cancel).
    }
}
