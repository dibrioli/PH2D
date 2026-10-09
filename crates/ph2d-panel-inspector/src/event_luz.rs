//! **O despacho da secção LUZ** (Luz 2D, W4). O valor que uma edição afirma vem do SNAPSHOT (a
//! caixa) ou do número que o artista comitou — nunca de um store de outro objeto.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::luz_edits::LuzFieldEdit as E;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

/// Despacha um evento da secção **LUZ**. `true` = consumido.
pub(crate) fn apply_luz_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_luz() else {
        return false;
    };
    let edit = match ev {
        // A posição na tabela de ids É a tag do tipo (gate de ida-e-volta na família).
        WidgetEvent::Click(id) => {
            let Some(i) = crate::ids::INSP_LUZ_TIPO.iter().position(|&o| o == id) else {
                return false;
            };
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            E::Tipo(u8::try_from(i).unwrap_or(0))
        }
        // Ligar o alcance devolve o número que a caixa mostra; desligar é `∞` (física pura).
        WidgetEvent::Toggled(id) if id == crate::ids::INSP_LUZ_ALCANCE_LIGADO => {
            if info.alcance_m.is_finite() {
                E::Alcance(f32::INFINITY)
            } else {
                #[allow(clippy::cast_possible_truncation)]
                let v = host
                    .store()
                    .number_value(crate::ids::INSP_LUZ_ALCANCE)
                    .unwrap_or(10.0) as f32; // LITERAL-PX-OK: a partida do `populate_luz`
                E::Alcance(v)
            }
        }
        WidgetEvent::ValueChanged(id) => {
            #[allow(clippy::cast_possible_truncation)]
            let v = host.store().number_value(id).unwrap_or(0.0) as f32;
            if id == crate::ids::INSP_LUZ_INTENSIDADE {
                E::Intensidade(v)
            } else if id == crate::ids::INSP_LUZ_TAMANHO {
                E::Tamanho(v)
            } else if id == crate::ids::INSP_LUZ_ALCANCE {
                E::Alcance(v)
            } else {
                return false;
            }
        }
        _ => return false,
    };
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits: info.entity_bits,
        edit: ComponentEdit::Luz(edit),
    });
    true
}
