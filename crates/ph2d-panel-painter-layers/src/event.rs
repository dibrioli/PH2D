//! Painter layers `apply_event` — thin forwarder (ADR-0040 TG-B), mirror
//! do `ph2d-panel-painter-sidebar` event.
//!
//! O panel NÃO mantém mapping semântico. Aqui classificamos cada
//! `WidgetEvent` num `PanelEvent` tool-agnostic e empurramos via
//! `EditorAction::ToolPanelEvent`; o action-bus drain no shell chama
//! `PainterTool::handle_panel_event` no tool ativo.
//!
//! **SCAFFOLD:** só o close (X) está roteado (→ CancelActiveTool). O
//! Implementador adiciona aqui o roteamento dos eventos das layer rows
//! (visibility toggle, opacity slider, blend dropdown, reorder) conforme
//! preencher `paint`/`populate`. Registrar sem paint = roteamento morto
//! (audit Y-7) — só roteie o que estiver pintado.

use crate::state::PainterLayersPanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut PainterLayersPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Close (X) button → CancelActiveTool (canon BgRemoval/Padding/
        // Painter sidebar).
        WidgetEvent::Click(id) if id == ph2d_editor_core::ids::PAINTER_LAYERS_CLOSE => {
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // TODO(impl W3.T3.4): layer row events
        //   ValueChanged(opacity slider) → ToolPanelEvent(SetValue(id, v));
        //   Click(visibility toggle / blend dropdown / row select) →
        //   ToolPanelEvent(Click(id)); reorder drag → dedicated PanelEvent.
        _ => false,
    }
}
