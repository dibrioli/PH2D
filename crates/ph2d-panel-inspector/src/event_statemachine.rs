//! **O despacho da secção STATE MACHINE** (TOP-20 #15, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_action`], de
//! quem esta secção é o gémeo estrutural.
//!
//! ⚠️ **Clicar numa linha NÃO vai ao barramento** (a §12 é o precedente): qual estado se edita é um
//! facto da UI, e publicá-lo faria um passo de undo por clique sobre um facto que a cena não tem.
//!
//! ⚠️ **O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT** — nunca do
//! store, que guarda o visual. É a lei que a §11 pagou com um report.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::statemachine_edits::StateMachineFieldEdit as E;
use ph2d_editor_core::widget::ButtonState;

use crate::state::InspectorState;

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_statemachine_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_statemachine() else {
        return false;
    };
    let bits = info.entity_bits;
    let est = panel
        .sm_state_selected
        .min(info.states.len().saturating_sub(1));
    let est_u8 = u8::try_from(est).unwrap_or(0);
    let tr = panel
        .sm_trans_selected
        .min(info.transitions.len().saturating_sub(1));
    let tr_u8 = u8::try_from(tr).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        if let Some(i) = crate::ids::INSP_SM_STATE_ROW.iter().position(|&o| o == id)
            && i < info.states.len()
        {
            panel.sm_state_selected = i;
            demote(host, id);
            return true;
        }
        if let Some(i) = crate::ids::INSP_SM_TRANS_ROW.iter().position(|&o| o == id)
            && i < info.transitions.len()
        {
            panel.sm_trans_selected = i;
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_SM_STATE_ADD {
            push(host, bits, E::AddState);
            // ⚠️ **Abre o que acabou de nascer** — senão o `+` lê-se como se não fizesse nada.
            panel.sm_state_selected = info.states.len();
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_SM_STATE_REMOVE && !info.states.is_empty() {
            push(host, bits, E::RemoveState(est_u8));
            panel.sm_state_selected = est.saturating_sub(1);
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_SM_TRANS_ADD {
            push(host, bits, E::AddTransition);
            panel.sm_trans_selected = info.transitions.len();
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_SM_TRANS_REMOVE && !info.transitions.is_empty() {
            push(host, bits, E::RemoveTransition(tr_u8));
            panel.sm_trans_selected = tr.saturating_sub(1);
            demote(host, id);
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev {
        let text = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Os campos de texto por uma porta só** — um `if` por campo é como o terceiro acaba a
        // escrever no primeiro (a lição da tabela de acções).
        let edit = if id == crate::ids::INSP_SM_STATE_NAME && !info.states.is_empty() {
            E::StateName(est_u8, text)
        } else if id == crate::ids::INSP_SM_STATE_ON_ENTER && !info.states.is_empty() {
            E::StateOnEnter(est_u8, text)
        } else if id == crate::ids::INSP_SM_STATE_ON_EXIT && !info.states.is_empty() {
            E::StateOnExit(est_u8, text)
        } else if id == crate::ids::INSP_SM_TRANS_ON && !info.transitions.is_empty() {
            E::TransitionOn(tr_u8, text)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = v.max(0.0).min(f64::from(u8::MAX)) as u8; // CLAMP-OK: índice de estado
        let edit = if id == crate::ids::INSP_SM_TRANS_FROM && !info.transitions.is_empty() {
            E::TransitionFrom(tr_u8, n)
        } else if id == crate::ids::INSP_SM_TRANS_TO && !info.transitions.is_empty() {
            E::TransitionTo(tr_u8, n)
        } else if id == crate::ids::INSP_SM_INITIAL {
            E::Initial(n)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::StateMachine(edit),
    });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
