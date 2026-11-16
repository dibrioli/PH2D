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
    // As saídas anexam no objecto que a fonte LÊ (o pai, ou o From) — nunca no filho «Motion Input».
    let alvo = info.alvo_bits.unwrap_or(bits);
    let linha = |tabela: &[ph2d_a11y::NodeId], id| tabela.iter().position(|&o| o == id);

    if let WidgetEvent::Click(id) = ev {
        let edit = if let Some(i) = linha(&crate::ids::INSP_MI_SOURCE, id) {
            u8::try_from(i).ok().map(E::Source)
        } else if let Some(i) = linha(&crate::ids::INSP_MI_GRAFO, id) {
            // O chip ESCREVE o nome que o grafo lê — o artista não adivinha.
            info.nomes_do_grafo.get(i).cloned().map(E::Name)
        } else if id == crate::ids::INSP_MI_NEW {
            // «New…»: o nome esvazia (a caixa aparece, com o foco) — o Undo devolve o de antes.
            focus_o_nome(host);
            Some(E::Name(String::new()))
        } else if id == crate::ids::INSP_MI_ADD_HEALTH {
            // A saída do aviso: a mesma porta da paleta (cascata, recibo, Undo), sem a abrir.
            host.bus_mut()
                .push(EditorAction::InspectorAddComponentRequested {
                    entity_bits: alvo,
                    component: Some("ph2d::physics::Health"),
                });
            None
        } else if id == crate::ids::INSP_MI_ADD_BODY {
            // A saída da queixa «sem corpo» da Speed: a mesma porta, com o Rigid Body.
            host.bus_mut()
                .push(EditorAction::InspectorAddComponentRequested {
                    entity_bits: alvo,
                    component: Some("ph2d::physics::RigidBody"),
                });
            None
        } else if id == crate::ids::INSP_MI_OPEN_INPUT_MAP {
            // A saída da queixa do Move: a aba do Input Map, pela porta do menu Settings.
            host.store_mut().request_input_map_tab();
            None
        } else if id == crate::ids::INSP_MI_FROM_PICK || id == crate::ids::INSP_MI_PICK_OBJECT {
            // O conta-gotas (o da linha arma ou desarma; o da queixa só arma).
            Some(E::PickFrom(
                id == crate::ids::INSP_MI_PICK_OBJECT || !info.from_pick_armed,
            ))
        } else if id == crate::ids::INSP_MI_REMAP {
            Some(E::Remap(!info.remap))
        } else if let Some(n) = linha(&crate::ids::INSP_MI_CURVE, id) {
            ph2d_editor_core::motion_input_edits::CURVE_PRESETS
                .get(n)
                .map(|(_, t)| E::Curve((*t).to_string()))
        } else if let Some(edit) = opcao_da_lista(host, &info, id) {
            Some(edit)
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

    if let WidgetEvent::TextChanged(id) = ev {
        let text = host.store().text(id).unwrap_or("").to_string();
        let edit = match id {
            crate::ids::INSP_MI_NAME => E::Name(text),
            // O NOME cru: quem o resolve é o dreno (um nome que ninguém tem não escreve).
            crate::ids::INSP_MI_FROM => E::From(text.trim().to_string()),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "os campos do componente são f32"
        )]
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        // ⚠️ Um `match` e não um `if` por campo (a lição dos três campos da tabela de acções).
        let edit = match id {
            crate::ids::INSP_MI_VALUE => E::Value(v),
            crate::ids::INSP_MI_REMAP_FROM => E::RemapFrom(v),
            crate::ids::INSP_MI_REMAP_TO => E::RemapTo(v),
            crate::ids::INSP_MI_SMOOTH => E::Smooth(v.max(0.0)),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

/// Uma opção de uma das duas listas (Action escreve o nome; Property, a CHAVE do campo): fecha a lista e
/// devolve a edição. ⚠️ A escolha vem do instantâneo, nunca do `selected_index` do store.
fn opcao_da_lista(
    host: &mut dyn PanelHostInternal,
    info: &ph2d_editor_core::motion_input_edits::InspectorMotionInputInfo,
    id: ph2d_a11y::NodeId,
) -> Option<E> {
    use crate::popovers::motion_input::Lista;
    let (lista, n) = [Lista::Action, Lista::Property]
        .into_iter()
        .find_map(|l| l.opcoes_ids().posicao(id, l.len(info)).map(|n| (l, n)))?;
    if let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(lista.id()) {
        *open = false;
    }
    match lista {
        Lista::Action => info.acoes_do_mapa.get(n).cloned().map(E::Action),
        Lista::Property => info
            .campos_do_alvo
            .get(n)
            .map(|(_, k)| E::Property(k.clone())),
    }
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::MotionInput(edit),
    });
}

/// O foco na caixa do nome, já a escrever (o padrão do nome da paleta do Blender).
fn focus_o_nome(host: &mut dyn PanelHostInternal) {
    let id = crate::ids::INSP_MI_NAME;
    host.store_mut().set_focus(Some(id));
    if let Some(InteractiveState::TextInput {
        state, text, caret, ..
    }) = host.store_mut().get_mut(id)
    {
        text.clear();
        *caret = 0;
        *state = ph2d_editor_core::widget::TextInputState::Focused;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
