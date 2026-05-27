//! Background-Removal panel `apply_event` — thin forwarder.
//!
//! ADR-0040 TG-B: the panel no longer holds the semantic mapping
//! (slider id → `BgRemovalUiEdit::Tolerance(v)`) — that lives in
//! `BgRemovalTool::handle_panel_event` over in `ph2d-tool-bgremoval`.
//! Here we just classify each `WidgetEvent` into a tool-agnostic
//! [`PanelEvent`], push it through `EditorAction::ToolPanelEvent`, and
//! let the action-bus drain in the shell call `Tool::handle_panel_event`
//! on the active tool. Cancel maps to `EditorAction::CancelActiveTool`.
//!
//! This is the same pattern future tool panels will follow — the panel
//! crate stays a pure widget bag; semantics live with the tool.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;

use crate::state::BgRemovalPanelState;

pub(crate) fn apply_event(
    _state: &mut BgRemovalPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider drag — read the freshly-dispatched value and forward it
        // normalized. The tool maps 0..1 back to full scale.
        WidgetEvent::ValueChanged(id) if is_bgr_slider(id) => {
            let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    value as f64,
                )));
            true
        }
        // Numeric chip edited (keyboard commit or drag-scrub) — the chip
        // is normalized 0..1 already (canonical NumberInput dispatch,
        // identical to Inspector / color-picker chips).
        WidgetEvent::ValueChanged(id) if is_bgr_number(id) => {
            let value = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, value,
                )));
            true
        }
        // Cancel — abandon the preview and deactivate the tool. Shell
        // switches back to the default tool via `CancelActiveTool`,
        // hiding this panel and restoring the Inspector.
        WidgetEvent::Click(id) if id == ids::BGR_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // Every other button (falloff / apply / eyedropper / protect /
        // protect-clear / show-mask) is a plain click — reset the
        // pressed visual then forward the click. The tool's
        // `handle_panel_event` maps the id back to the semantic edit.
        WidgetEvent::Click(id) if is_bgr_click(id) => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        _ => false,
    }
}

fn is_bgr_slider(id: NodeId) -> bool {
    id == ids::BGR_TOLERANCE
        || id == ids::BGR_FEATHER
        || id == ids::BGR_REFINE
        || id == ids::BGR_GROW
        || id == ids::BGR_BRUSH_SIZE
        || id == ids::BGR_MIN_ISLAND_PX
}

fn is_bgr_number(id: NodeId) -> bool {
    id == ids::BGR_TOLERANCE_NUM
        || id == ids::BGR_FEATHER_NUM
        || id == ids::BGR_REFINE_NUM
        || id == ids::BGR_GROW_NUM
        || id == ids::BGR_BRUSH_SIZE_NUM
        || id == ids::BGR_MIN_ISLAND_PX_NUM
}

fn is_bgr_click(id: NodeId) -> bool {
    id == ids::BGR_FALLOFF_SMOOTH
        || id == ids::BGR_FALLOFF_SPHERE
        || id == ids::BGR_FALLOFF_SHARP
        || id == ids::BGR_FALLOFF_CONSTANT
        || id == ids::BGR_APPLY
        || id == ids::BGR_RESET
        || id == ids::BGR_EYEDROPPER
        || id == ids::BGR_PROTECT
        || id == ids::BGR_PROTECT_CLEAR
        || id == ids::BGR_SHOW_MASK
        || id == ids::BGR_SEPARATE_ISLANDS
        || id == ids::BGR_AUTO_PROTECT_SUBJECT
        || id == ids::BGR_ADD_AREA
        || id == ids::BGR_ADD_AREA_CLEAR
}

fn reset_button(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
