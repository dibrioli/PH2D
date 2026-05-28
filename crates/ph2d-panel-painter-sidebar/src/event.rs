//! Painter sidebar `apply_event` — thin forwarder (ADR-0040 TG-B).
//!
//! Panel NÃO mantém mapping semântico (slider id → `PainterUiEdit::SetSize(v)`).
//! Esse mapping vive em `PainterTool::handle_panel_event` (T1.6 R7 L1-4
//! contract). Aqui classificamos cada `WidgetEvent` num `PanelEvent`
//! tool-agnostic, empurramos via `EditorAction::ToolPanelEvent`, e
//! deixamos o action-bus drain no shell chamar
//! `Tool::handle_panel_event` no tool ativo.

use crate::ids;
use crate::state::PainterSidebarPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;

pub(crate) fn apply_event(
    _state: &mut PainterSidebarPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Close (X) button → CancelActiveTool (canon BgRemoval/Padding).
        WidgetEvent::Click(id) if id == ph2d_editor_core::ids::PAINTER_SIDEBAR_CLOSE => {
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // Slider drag — read freshly-dispatched value, forward normalizado.
        // PainterTool::handle_panel_event mapeia 0..1 back to size_px /
        // opacity full scale conforme `PainterUiEdit` semântica.
        WidgetEvent::ValueChanged(id) if is_painter_sidebar_slider(id) => {
            let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    value as f64,
                )));
            true
        }
        // Undo/Redo/Modifier buttons: sem paint nesta wave (T2.2 replay
        // engine / T2.4 eyedropper-while-held). Forwarding volta junto do
        // paint correspondente — registrar sem paint = roteamento morto
        // (audit Y-7, 2026-05-28).
        _ => false,
    }
}

#[inline]
fn is_painter_sidebar_slider(id: NodeId) -> bool {
    id == ids::SIZE_SLIDER || id == ids::OPACITY_SLIDER
}
