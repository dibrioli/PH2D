//! Equalize Sizes panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-C shape: the panel does not own any semantic mapping
//! (NodeId → `EqualizeSizesUiEdit` lives in
//! `EqualizeSizesTool::handle_panel_event`). Here we just classify each
//! `WidgetEvent` into a tool-agnostic [`PanelEvent`], push it through
//! `EditorAction::ToolPanelEvent`, and let the action-bus drain in the
//! shell call `Tool::handle_panel_event` on the active tool. Cancel maps
//! to `EditorAction::CancelActiveTool`.
//!
//! Widget Gallery convention (DIRETRIZ §4.2): slider + chip share `0..1`
//! storage via `link_slider_number` (set up in [`crate::populate`]) — so
//! the dispatch handles drag, clamp, and chip↔slider mirror for free.
//! Both `ValueChanged(slider_id)` and `ValueChanged(chip_id)` flow
//! through here; we forward the slider's normalized track value to the
//! tool (whose `params::apply_ui_edit` owns the `0..1 → natural unit`
//! projection). NO mirror logic here, NO clamps — both are anti-patterns
//! that re-create the slot 1 bugs.

use crate::ids;
use crate::state::EqualizeSizesPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn apply_event(
    _state: &mut EqualizeSizesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Fixed-mode W/H chips — standalone, raw px (no slider pairing,
        // no track translation). The tool's `apply_ui_edit` clamps to
        // `[1, EQS_MAX_FIXED_DIM]`.
        WidgetEvent::ValueChanged(id) if id == ids::EQS_FIXED_W || id == ids::EQS_FIXED_H => {
            let px = host.store().number_value(id).unwrap_or(1.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, px)));
            true
        }
        // ── Plain click-style buttons (modes, algorithm, toggles, Apply) ──
        WidgetEvent::Click(id)
            if matches!(
                id,
                _ if id == ids::EQS_MODE_MAX
                    || id == ids::EQS_MODE_FIXED
                    || id == ids::EQS_MODE_GRID
                    || id == ids::EQS_ALIGN_TO_GRID
                    || id == ids::EQS_UPSCALE_IF_SMALLER
                    || id == ids::EQS_RASTERIZE_AFTER
                    || id == ids::EQS_ALG_LANCZOS
                    || id == ids::EQS_ALG_NEAREST
                    || id == ids::EQS_ALG_XBR
                    || id == ids::EQS_APPLY
            ) =>
        {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Cancel — abandon + deactivate the tool.
        WidgetEvent::Click(id) if id == ids::EQS_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    }
}

fn reset_button(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
