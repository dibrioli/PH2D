//! Painel → shell. Cada braço sai da MESMA lista que o `paint` percorre.
//!
//! ⚠️ **O evento traz só o `NodeId`; o valor é RELIDO do store.** É o contrato do `WidgetEvent`
//! (*"value-bearing variants carry only the `NodeId` — the caller re-reads from the store"*), e é
//! o que garante que o número que sai daqui é o mesmo que o `paint` desenha no frame seguinte.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::CheckboxValue;

use crate::AuthoredPanel;
use crate::rows::{Row, row_key_for};
use crate::state::{AuthoredIntent, AuthoredPanelState, push_intent};

pub(crate) fn apply_event(
    _state: &mut AuthoredPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let WidgetEvent::Click(id) = ev
        && id == crate::ids::AUTHORED_CLOSE
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

    // ⚠️ **A opção de uma lista aberta é resolvida ANTES da row**, e a ordem é o que a torna
    // correta: uma opção não tem `key` própria (o `row_key_for` devolveria `None`), e cair no
    // caminho da row daria `Ignored` — a lista pintaria, o clique chegaria, e nada aconteceria.
    if let Some((key, index)) = crate::rows::option_for(id) {
        // ⚠️ **Nada mais no repo escreve esta seleção.** O despacho genérico não tem braço para
        // `InteractiveState::Tabs` nem para `::Radio` — medido —, então sem esta linha a faixa
        // desenha as N opções, fica viva sob o rato e **não muda a marcada** ao ser clicada.
        if let Some(live) = host.store_mut().get_mut(crate::ids::authored_row_id(&key)) {
            crate::rows::select_in(live, index);
        }
        push_intent(AuthoredIntent::Choice { key, index });
        return EventOutcome::Consumed;
    }

    let Some(key) = row_key_for(id) else {
        return EventOutcome::Ignored;
    };
    push_intent(intent_de(host.store(), id, key));
    EventOutcome::Consumed
}

/// **O que um gesto na row `id` publica** — a decisão única, que o clique e o [`intents_que_emite`] perguntam.
fn intent_de(store: &WidgetStore, id: NodeId, key: String) -> AuthoredIntent {
    // A leitura é do store, e o braço escolhido é o do que o store DE FACTO guarda — não o do
    // tipo declarado. Registo trocado dá `Ignored`, nunca um valor inventado.
    if let Some((_, value)) = store.slider(id) {
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
    } else if let Some(index) = crate::rows::selected_of(store.get(id)) {
        // ⚠️ A família de LISTA inteira, pela MESMA porta do `paint`. Um `matches!` sobre os quatro
        // tipos escrito aqui seria a segunda resposta a *"onde mora a seleção deste widget?"*, e o
        // quinto membro da família nasceria a emitir `Fired`.
        AuthoredIntent::Choice { key, index }
    } else {
        AuthoredIntent::Fired { key }
    }
}

/// ⭐ **Tudo o que `row` pode publicar** (GEF02 5.1, o censo de sinais): a row registada como o
/// `populate` a regista, a decisão do [`intent_de`]; uma escolha dá uma por opção. Vazio = não é controle.
#[must_use]
pub fn intents_que_emite(row: &Row) -> Vec<AuthoredIntent> {
    let mut store = WidgetStore::default();
    crate::populate::adopt(&mut store, row);
    if store.get(row.id).is_none() {
        return Vec::new();
    }
    match intent_de(&store, row.id, row.key.clone()) {
        AuthoredIntent::Choice { key, .. } => (0..row.options.len())
            .map(|index| AuthoredIntent::Choice {
                key: key.clone(),
                index,
            })
            .collect(),
        outro => vec![outro],
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
