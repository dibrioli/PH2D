//! **O despacho da secção GATILHO.**
//!
//! ⚠️ **Clicar numa linha NÃO vai ao barramento** — qual gatilho se edita é um facto da UI e fica
//! no painel, a lei que o `Timers` escreveu: as N linhas ouvem todas ao mesmo tempo, logo a cena
//! não tem onde guardar *«o gatilho actual»* e publicá-lo seria um passo de undo por clique.
//!
//! ⚠️ **O valor que uma edição afirma vem do SNAPSHOT, nunca do store** — ler o store faz o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto anterior.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::action_trigger_edits::ActionTriggerFieldEdit as E;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

use crate::state::InspectorState;

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_action_trigger_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_action_trigger() else {
        return false;
    };
    let bits = info.entity_bits;
    let sel = panel
        .trigger_selected
        .min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        if let Some(i) = crate::ids::INSP_TRIGGER_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.trigger_selected = i;
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_TRIGGER_ADD {
            push(host, bits, E::Add);
            // ⚠️ **Abre o que acabou de nascer** — senão o `+` parece não ter feito nada.
            panel.trigger_selected = info.rows.len();
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_TRIGGER_REMOVE && !info.rows.is_empty() {
            push(host, bits, E::Remove(sel_u8));
            // ⚠️ **Recua um** — senão o índice aberto aponta para além do fim e o editor some.
            panel.trigger_selected = sel.saturating_sub(1);
            demote(host, id);
            return true;
        }
        // ⚠️ **A posição na tabela de ids É o valor do enum** — a lei que o array declara, com
        // gate na shell a prendê-la à ordem do `ph2d_ecs::ActionEdge`.
        if let Some(i) = crate::ids::INSP_TRIGGER_EDGE_OPT
            .iter()
            .position(|&o| o == id)
            && !info.rows.is_empty()
        {
            push(host, bits, E::Edge(sel_u8, u8::try_from(i).unwrap_or(0)));
            fecha(host);
            demote(host, id);
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev
        && !info.rows.is_empty()
    {
        let texto = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Os dois campos por uma porta só** — um `if` por campo é como o segundo acaba a
        // escrever no primeiro, e trocar o nome da acção por um nome de sinal é o defeito que não
        // se vê até alguém carregar na tecla.
        let edit = if id == crate::ids::INSP_TRIGGER_ACTION {
            E::Action(sel_u8, texto)
        } else if id == crate::ids::INSP_TRIGGER_SIGNAL {
            E::Signal(sel_u8, texto)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut()
        .push(EditorAction::InspectorActionTriggerEdit { entity_bits, edit });
}

/// Fecha o popover depois de uma escolha.
fn fecha(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(crate::ids::INSP_TRIGGER_EDGE_PICK)
    {
        *open = false;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
