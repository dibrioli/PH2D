//! Roteador de eventos da tira.
//!
//! Tudo aqui é edição de DOCUMENTO ou de TRANSPORTE — nada disso é estilo de tool.
//! Então o painel só classifica o `WidgetEvent` e empurra um `PanelEvent` no
//! barramento; quem aplica é o drain do shell (`flip_strip::apply_panel_event`),
//! que tem o `FlipDoc` e o playhead. O painel continua sem saber o que é um frame.

use crate::ids;
use crate::state::{FlipStripState, current_flip_strip};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;

/// Os botões/toggles da barra + o X, todos encaminhados como `Click`.
const BUTTONS: [ph2d_a11y::NodeId; 16] = [
    ids::FLIP_PREV_DRAWING,
    ids::FLIP_PLAY,
    ids::FLIP_NEXT_DRAWING,
    ids::FLIP_GHOST,
    ids::FLIP_AUTOKEY,
    ids::FLIP_FALLOFF,
    ids::FLIP_ADDITIVE,
    ids::FLIP_KEY_ADD,
    ids::FLIP_KEY_DUP,
    ids::FLIP_KEY_INSTANCE,
    ids::FLIP_KEY_UNLINK,
    ids::FLIP_KEY_DELETE,
    ids::FLIP_KEY_LEFT,
    ids::FLIP_KEY_RIGHT,
    ids::FLIP_TWEEN_ADD,
    ids::FLIP_STRIP_CLOSE,
];

/// As caixas numéricas — encaminhadas como `SetValue` (o valor vem da store).
const NUMBERS: [ph2d_a11y::NodeId; 5] = [
    ids::FLIP_FPS_NUM,
    ids::FLIP_GHOST_BEFORE_NUM,
    ids::FLIP_GHOST_AFTER_NUM,
    ids::FLIP_HOLD_NUM,
    ids::FLIP_TWEEN_NUM,
];

/// A célula `index` da tira, se `id` for uma delas (as células são registradas por
/// ÍNDICE, e o snapshot deste frame diz quantas existem).
fn cell_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    let n = current_flip_strip().cells.len();
    (0..n).find(|&i| ids::flip_cell_id(i) == id)
}

/// A opção de ciclo `mode`, se `id` for uma delas.
fn cycle_option(id: ph2d_a11y::NodeId) -> Option<u8> {
    (0u8..4).find(|&m| ids::flip_cycle_option_id(m) == id)
}

pub(crate) fn apply_event(
    _state: &mut FlipStripState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::Click(id) if BUTTONS.contains(&id) => {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) => {
            // Uma opção do ciclo: fecha o popover e aplica.
            if let Some(mode) = cycle_option(id) {
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::FLIP_CYCLE_DD)
                {
                    *open = false;
                    *selected_index = Some(mode as usize);
                }
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        ids::FLIP_CYCLE_DD,
                        mode.to_string(),
                    )));
                return EventOutcome::from_bool(true);
            }
            // Uma célula: o shell resolve o índice → chave (seek + seleção).
            match cell_index(id) {
                Some(_) => {
                    seam_reset_button(host, id);
                    host.bus_mut()
                        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
                    true
                }
                // O chip do ciclo em si: o Click é só o open/close genérico.
                None => id == ids::FLIP_CYCLE_DD,
            }
        }
        WidgetEvent::ValueChanged(id) if NUMBERS.contains(&id) => {
            let v = host.store().number_value(id).unwrap_or_default();
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
            true
        }
        _ => false,
    };
    EventOutcome::from_bool(consumed)
}
