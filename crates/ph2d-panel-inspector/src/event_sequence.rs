//! **O clique da secção SEQUENCE** (TOP-20 #19, W3) — escolher uma cutscene, ou largar a que está.
//!
//! ⚠️ **O que vai ao barramento é o NOME**, nunca o índice da opção: apagar um container renumera
//! os de baixo, e um índice guardado passaria a tocar a cutscene do vizinho **em silêncio**. O
//! selector escolhe por posição porque é assim que se pinta uma lista; o que viaja é o nome.
//!
//! ⚠️ **E o `selected_index` do store NÃO é escrito aqui.** Quem é dono da escolha é o snapshot,
//! que o quadro seguinte relê da cena — escrever aqui abriria a segunda porta para o mesmo estado,
//! e ela mentiria exactamente no caso em que a shell recusasse a edição.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::sequence_edits::SequenceFieldEdit as E;
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn apply_sequence_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_sequence() else {
        return false;
    };
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let bits = info.entity_bits;

    if id == crate::ids::INSP_SEQ_CLEAR {
        push(host, bits, E::Container(String::new()));
        demote(host, id);
        return true;
    }

    // ⚠️ A posição na tabela de ids É o índice do container — a lei que o array declara.
    if let Some(i) = crate::ids::INSP_SEQ_OPT.iter().position(|&o| o == id) {
        // ⛔ Uma opção fora da lista do snapshot não escreve nada: o popover pode ter ficado um
        // quadro atrás de um container apagado, e inventar um nome ali seria escrever o que
        // ninguém escolheu.
        if let Some(nome) = info.nomes.get(i) {
            push(host, bits, E::Container(nome.clone()));
        }
        fecha(host);
        demote(host, id);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut()
        .push(EditorAction::InspectorSequenceEdit { entity_bits, edit });
}

/// Fecha o popover depois de uma escolha.
fn fecha(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(crate::ids::INSP_SEQ_PICK)
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
