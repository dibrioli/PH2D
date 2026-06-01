//! Painter layers `apply_event` — thin forwarder (ADR-0040 TG-B), mirror
//! do `ph2d-panel-painter-sidebar` event.
//!
//! The panel keeps NO semantic mapping. Each `WidgetEvent` is classified into a
//! tool-agnostic [`PanelEvent`] and pushed via `EditorAction::ToolPanelEvent`;
//! the shell's action-bus drain calls `PainterTool::handle_panel_event` on the
//! active tool, which decodes the per-row id back to its `(layer, kind)` and
//! applies the edit.
//!
//! Per-row ids are decoded here only to pick the right `PanelEvent` shape:
//! row-select / visibility eye → `Click`, opacity slider → `SetValue`, blend
//! dropdown option → `SelectOption(blend_id, mode_u8)`. The blend chip itself
//! opens/closes its popover via the generic `Dropdown` dispatch (not routed
//! here). The decode uses the published `current_layers()` snapshot.

use crate::state::{self, PainterLayersPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    self as core_ids, PainterLayerWidget, painter_layer_blend_option_id, painter_layer_widget_id,
};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{LayerId, LayerStack, MAX_BLEND_MODES};

pub(crate) fn apply_event(
    _state: &mut PainterLayersPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Close (X) → CancelActiveTool (canon BgRemoval/Painter sidebar).
        WidgetEvent::Click(id) if id == core_ids::PAINTER_LAYERS_CLOSE => {
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // Fixed chrome buttons: "+ Layer" + dock toggle → forward as Click.
        WidgetEvent::Click(id)
            if id == core_ids::PAINTER_LAYERS_ADD || id == core_ids::PAINTER_LAYERS_TOGGLE_DOCK =>
        {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) => {
            let Some(stack) = state::current_layers() else {
                return false;
            };
            // Blend dropdown option picked → close the dropdown + apply.
            if let Some((layer, mode)) = decode_blend_option(&stack, id) {
                let blend_id = painter_layer_widget_id(layer.0, PainterLayerWidget::Blend);
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(blend_id)
                {
                    *open = false;
                    *selected_index = Some(mode as usize);
                }
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        blend_id,
                        mode.to_string(),
                    )));
                return true;
            }
            // Per-row row-select / visibility eye → forward as Click. (The
            // blend chip click is the dropdown open/close — handled by the
            // generic Dropdown dispatch, not forwarded.)
            match decode(&stack, id) {
                Some((_, PainterLayerWidget::Row | PainterLayerWidget::Visibility)) => {
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                _ => false,
            }
        }
        // Per-row opacity slider drag — read the freshly-dispatched `0..1`
        // value and forward normalized (the linked chip edit propagates back
        // to the slider, so its ValueChanged arrives here too — single route).
        WidgetEvent::ValueChanged(id) => {
            let Some(stack) = state::current_layers() else {
                return false;
            };
            if let Some((_, PainterLayerWidget::Opacity)) = decode(&stack, id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                        id, v as f64,
                    )));
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Decode a per-row widget id → `(layer, kind)` via the published snapshot.
fn decode(stack: &LayerStack, id: ph2d_a11y::NodeId) -> Option<(LayerId, PainterLayerWidget)> {
    for layer in stack.all_ids() {
        for kind in PainterLayerWidget::ALL {
            if painter_layer_widget_id(layer.0, kind) == id {
                return Some((layer, kind));
            }
        }
    }
    None
}

/// Decode a blend-mode popover option id → `(layer, mode_u8)`.
fn decode_blend_option(stack: &LayerStack, id: ph2d_a11y::NodeId) -> Option<(LayerId, u8)> {
    for layer in stack.all_ids() {
        for m in 0..MAX_BLEND_MODES {
            if painter_layer_blend_option_id(layer.0, m) == id {
                return Some((layer, m));
            }
        }
    }
    None
}
