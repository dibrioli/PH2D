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
const BUTTONS: [ph2d_a11y::NodeId; 19] = [
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
    ids::FLIP_KEY_PIN,
    ids::FLIP_KEY_DELETE,
    ids::FLIP_KEY_LEFT,
    ids::FLIP_KEY_RIGHT,
    ids::FLIP_TWEEN_ADD,
    ids::FLIP_TWEEN_FADE,
    ids::FLIP_TWEEN_PAIRS,
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

/// A opção `n` de um dos dropdowns, se `id` for uma delas — devolve TAMBÉM de qual chip,
/// porque o dispatch precisa fechar o popover certo e mandar o `SelectOption` com o id do
/// dono (mandar o do outro chip seria despachar a escolha para o campo errado).
fn dropdown_option(id: ph2d_a11y::NodeId) -> Option<(ph2d_a11y::NodeId, u8)> {
    (0u8..4)
        .find(|&m| ids::flip_cycle_option_id(m) == id)
        .map(|m| (ids::FLIP_CYCLE_DD, m))
        .or_else(|| {
            (0u8..4)
                .find(|&m| ids::flip_tween_ease_option_id(m) == id)
                .map(|m| (ids::FLIP_TWEEN_EASE_DD, m))
        })
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
            // Uma opção de dropdown: fecha o popover DAQUELE chip e aplica.
            if let Some((chip, mode)) = dropdown_option(id) {
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(chip)
                {
                    *open = false;
                    *selected_index = Some(mode as usize);
                }
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        chip,
                        mode.to_string(),
                    )));
                return EventOutcome::from_bool(true);
            }
            // ⚠️ **Uma célula NÃO chega mais aqui**, e o braço que a tratava foi removido:
            // desde que a célula virou superfície de gesto (`strip_drag`), o `dispatch`
            // a captura no Down e o toque volta como `GesturePhase::Click` — que o
            // `strip_drag::process` traduz no MESMO `PanelEvent::Click(flip_cell_id(i))`
            // de sempre. Manter o braço aqui deixaria duas portas para a mesma pergunta,
            // e a que o produto não usa é a que os testes acham primeiro (era o caso: o
            // seam clicava por aqui e ficava verde sobre o caminho morto).
            //
            // O que sobra: o chip em si. O Click nele é só o open/close genérico, e
            // **abrir um FECHA o outro** — dois popovers abertos ao mesmo tempo é um
            // estado que ninguém pediu, e só um deles chega a ser pintado (o `pending`
            // é um).
            if id == ids::FLIP_CYCLE_DD || id == ids::FLIP_TWEEN_EASE_DD {
                let other = if id == ids::FLIP_CYCLE_DD {
                    ids::FLIP_TWEEN_EASE_DD
                } else {
                    ids::FLIP_CYCLE_DD
                };
                if let Some(InteractiveState::Dropdown { open, .. }) =
                    host.store_mut().get_mut(other)
                {
                    *open = false;
                }
                true
            } else {
                false
            }
        }
        // A régua de scrub: a mecânica de slider deu um `value` `0..1`; mapeamos ao
        // quadro pelo vão exibido (o inverso do handle — `scrub_frame`) e mandamos o
        // QUADRO ao shell, que só faz seek. O mapa mora aqui (o painel é o dono do
        // layout), não no shell: `PanelEvent` está congelado, então reusamos `SetValue`.
        WidgetEvent::ValueChanged(id) if id == ids::FLIP_SCRUB => {
            let value = host.store().slider(id).map_or(0.0, |(_, v)| v);
            if let Some(frame) = current_flip_strip().scrub_frame(value) {
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                        id,
                        f64::from(frame),
                    )));
            }
            true
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
