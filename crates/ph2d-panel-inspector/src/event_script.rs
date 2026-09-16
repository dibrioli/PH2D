//! **O despacho da secção SCRIPT** (TOP-20 #16, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função**, como os das irmãs.
//!
//! ⚠️ **A edição leva o NOME da propriedade** (ver `ph2d_editor_core::script_edits`), e o nome sai
//! do SNAPSHOT pelo índice da linha — nunca do store, que guarda o visual.
//!
//! ⚠️ **Mexer num campo PÕE o valor** (a divergência D1 do oráculo): mesmo um número igual ao
//! default fica próprio, e só o `Reset` o larga.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::script_edits::{InspectorScriptValue as V, ScriptFieldEdit as E};
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_script_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_script() else {
        return false;
    };
    let bits = info.entity_bits;
    let linha = |tabela: &[ph2d_a11y::NodeId], id| tabela.iter().position(|&o| o == id);

    if let WidgetEvent::Click(id) = ev {
        let edit = if id == crate::ids::INSP_SCRIPT_BROWSE {
            Some(E::Browse)
        } else if let Some(i) = linha(&crate::ids::INSP_SCRIPT_RESET, id) {
            info.props
                .get(i)
                .filter(|p| p.own)
                .map(|p| E::Forget(p.name.clone()))
        } else if let Some(i) = linha(&crate::ids::INSP_SCRIPT_ORPHAN_REMOVE, id) {
            info.orphans.get(i).map(|o| E::Forget(o.name.clone()))
        } else {
            return false;
        };
        if let Some(edit) = edit {
            push(host, bits, edit);
        }
        demote(host, id);
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_SCRIPT_BOOL, id)
        && let Some(p) = info.props.get(i)
        && matches!(p.value, V::Bool(_))
    {
        let on = matches!(
            host.store().get(id),
            Some(InteractiveState::Checkbox {
                value: CheckboxValue::Checked,
                ..
            })
        );
        push(host, bits, E::SetBool(p.name.clone(), on));
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev {
        let text = host.store().text(id).unwrap_or("").to_string();
        let edit = if id == crate::ids::INSP_SCRIPT_SOURCE {
            E::Source(text)
        } else if let Some(i) = linha(&crate::ids::INSP_SCRIPT_TEXT, id)
            && let Some(p) = info.props.get(i)
            && matches!(p.value, V::Text(_))
        {
            E::SetText(p.name.clone(), text)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_SCRIPT_NUM, id)
        && let Some(p) = info.props.get(i)
        && matches!(p.value, V::Number(_))
    {
        let v = host.store().number_value(id).unwrap_or(0.0);
        push(host, bits, E::SetNumber(p.name.clone(), v));
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut()
        .push(EditorAction::InspectorScriptEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
