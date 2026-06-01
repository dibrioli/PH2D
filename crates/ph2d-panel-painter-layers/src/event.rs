//! Painter layers `apply_event` — thin forwarder (ADR-0040 TG-B), mirror
//! do `ph2d-panel-painter-sidebar` event.
//!
//! The panel keeps NO semantic mapping. Each `WidgetEvent` is classified into a
//! tool-agnostic [`PanelEvent`] and pushed via `EditorAction::ToolPanelEvent`;
//! the shell's action-bus drain calls `PainterTool::handle_panel_event` on the
//! active tool, which decodes the per-row id back to its `(layer, kind)` and
//! applies the edit (`set_layer_visible/opacity/blend_mode`, `select_layer`,
//! `add_raster_layer`, `toggle_dock`).
//!
//! Per-row ids are decoded here only to pick the right `PanelEvent` shape:
//! row-select / visibility eye → `Click`, opacity slider → `SetValue`, blend
//! chip → `SelectOption` carrying the *next* mode (the chip cycles). The
//! decode uses the published `current_layers()` snapshot.

use crate::state::{self, PainterLayersPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{self as core_ids, PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{LayerId, LayerStack};

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
        // Fixed chrome buttons: "+ Layer" + dock toggle → forward as a plain
        // Click; the tool routes by id.
        WidgetEvent::Click(id)
            if id == core_ids::PAINTER_LAYERS_ADD
                || id == core_ids::PAINTER_LAYERS_TOGGLE_DOCK =>
        {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Per-row click: row-select / visibility eye → Click; blend chip →
        // SelectOption(next mode).
        WidgetEvent::Click(id) => {
            let Some(stack) = state::current_layers() else {
                return false;
            };
            let Some((layer, kind)) = decode(&stack, id) else {
                return false;
            };
            match kind {
                PainterLayerWidget::Row | PainterLayerWidget::Visibility => {
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                PainterLayerWidget::Blend => {
                    let cur = stack.get(layer).map(|l| l.blend_mode.to_u8()).unwrap_or(0);
                    let next = crate::paint::next_blend_mode(cur);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                            id,
                            next.to_string(),
                        )));
                    true
                }
                // Opacity slider/chip emit ValueChanged, not Click.
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
/// Mirror of `PainterTool::decode_layer_widget` (panel-side, snapshot-driven).
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
