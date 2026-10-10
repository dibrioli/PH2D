//! **O despacho da secção MOTION INPUT** (ciclo 19 do Motion).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::motion_input_edits::MotionInputFieldEdit as E;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_motion_input_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_motion_input() else {
        return false;
    };
    let bits = info.entity_bits;
    let linha = |tabela: &[ph2d_a11y::NodeId], id| tabela.iter().position(|&o| o == id);

    if let WidgetEvent::Click(id) = ev {
        let edit = if let Some(i) = linha(&crate::ids::INSP_MI_SOURCE, id) {
            u8::try_from(i).ok().map(E::Source)
        } else if let Some(i) = linha(&crate::ids::INSP_MI_GRAFO, id) {
            // O chip ESCREVE o nome que o grafo lê — o artista não adivinha.
            info.nomes_do_grafo.get(i).cloned().map(E::Name)
        } else if id == crate::ids::INSP_MI_COLOR {
            // A amostra ABRE o selector partilhado, semeado com a cor do documento; a escolha volta
            // pela semente (`sync_motion_input`).
            let seed = crate::state_tint::tint_f32_to_u8(info.color);
            host.store_mut().set_widget_color(id, seed);
            host.store_mut().set_picker_target(Some(id));
            host.store_mut().set_blender_value(
                ph2d_editor_core::ids::INSP_BLENDER_PICKER,
                ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
            );
            demote(host, id);
            return true;
        } else {
            return false;
        };
        if let Some(edit) = edit {
            push(host, bits, edit);
        }
        demote(host, id);
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == crate::ids::INSP_MI_NAME
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, E::Name(text));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && id == crate::ids::INSP_MI_VALUE
    {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o campo do componente é f32"
        )]
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        push(host, bits, E::Value(v));
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::MotionInput(edit),
    });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
