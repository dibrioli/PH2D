//! Painel → shell. Cada braço sai da MESMA lista que o `paint` percorre.
//!
//! ⚠️ **O evento traz só o `NodeId`; o valor é RELIDO do store.** É o contrato do `WidgetEvent`
//! (*"value-bearing variants carry only the `NodeId` — the caller re-reads from the store"*), e é
//! o que garante que o número que sai daqui é o mesmo que o `paint` desenha no frame seguinte.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::CheckboxValue;

use crate::AuthoredPanel;
use crate::rows::row_key_for;
use crate::state::{AuthoredIntent, AuthoredPanelState, push_intent};

pub(crate) fn apply_event(
    _state: &mut AuthoredPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let WidgetEvent::Click(id) = ev
        && id == ids::AUTHORED_CLOSE
    {
        // ⚠️ O X escreve a MESMA visibilidade que o interruptor da seção Frame lê. Um painel cujo
        // X e cujo abridor discordassem seria a falha de duas-portas na forma mais visível dela:
        // o artista fecha, o interruptor continua aceso, e clicar nele não faz nada.
        host.set_panel_visible(AuthoredPanel::ID, false);
        return EventOutcome::Consumed;
    }

    let id = match ev {
        WidgetEvent::Click(id)
        | WidgetEvent::Toggled(id)
        | WidgetEvent::ValueChanged(id)
        | WidgetEvent::TextChanged(id) => id,
        _ => return EventOutcome::Ignored,
    };
    let Some(key) = row_key_for(id) else {
        return EventOutcome::Ignored;
    };
    let store = host.store();
    // A leitura é do store, e o braço escolhido é o do que o store DE FACTO guarda — não o do
    // tipo declarado. Registo trocado dá `Ignored`, nunca um valor inventado.
    let intent = if let Some((_, value)) = store.slider(id) {
        AuthoredIntent::Value { key, value }
    } else if let Some((_, on)) = store.toggle(id) {
        AuthoredIntent::Flag { key, on }
    } else if let Some((_, value)) = store.checkbox(id) {
        AuthoredIntent::Flag {
            key,
            on: value == CheckboxValue::Checked,
        }
    } else if let Some(text) = store.text(id) {
        AuthoredIntent::Text {
            key,
            text: text.to_string(),
        }
    } else {
        AuthoredIntent::Fired { key }
    };
    push_intent(intent);
    EventOutcome::Consumed
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
