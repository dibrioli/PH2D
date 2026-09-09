//! **O despacho da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_timer`].
//!
//! ⚠️ **Clicar numa linha NÃO vai ao barramento** (a §12 é o precedente): qual acção se edita é um
//! facto da UI, e publicá-la faria um passo de undo por clique sobre um facto que a cena não tem.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::ActionFieldEdit;
use ph2d_editor_core::widget::ButtonState;

use crate::state::{self, InspectorState};

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_action_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = state::current_inspector_action() else {
        return false;
    };
    let sel = panel.action_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        if let Some(i) = ids::INSP_ACTION_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.action_selected = i;
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ACTION_ADD {
            push(host, info.entity_bits, ActionFieldEdit::Add);
            // ⚠️ **Abre o que acabou de nascer** — senão o `+` lê-se como se não fizesse nada.
            panel.action_selected = info.rows.len();
            demote(host, id);
            return true;
        }
        if id == ids::INSP_ACTION_REMOVE && !info.rows.is_empty() {
            push(host, info.entity_bits, ActionFieldEdit::Remove(sel_u8));
            panel.action_selected = sel.saturating_sub(1);
            demote(host, id);
            return true;
        }
        // ⚠️ **A POSIÇÃO no array É a tag** — reordenar aquele array faria um clique escrever
        // outro verbo, e compila. Há gate na lei pura (`the_verb_tag_is_its_position…`).
        if let Some(i) = ids::INSP_ACTION_VERB.iter().position(|&o| o == id)
            && !info.rows.is_empty()
        {
            let tag = u8::try_from(i).unwrap_or(0);
            push(host, info.entity_bits, ActionFieldEdit::Verb(sel_u8, tag));
            demote(host, id);
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev
        && !info.rows.is_empty()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Os três campos de texto por uma porta só** — um `if` por campo é como o terceiro
        // acaba a escrever no primeiro.
        let edit = if id == ids::INSP_ACTION_ON {
            ActionFieldEdit::On(sel_u8, text)
        } else if id == ids::INSP_ACTION_TARGET {
            ActionFieldEdit::Target(sel_u8, text)
        } else if id == ids::INSP_ACTION_ARG {
            ActionFieldEdit::Arg(sel_u8, text)
        } else {
            return false;
        };
        push(host, info.entity_bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: ActionFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorActionEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
