//! ⭐⭐⭐ **Os widgets do CARTÃO DE PROPRIEDADES** — o campo do valor, os chips das versões e o
//! gesto de *Salvar Variação…*.
//!
//! # ⚠️ Corte por RESPONSABILIDADE, e por LOC
//!
//! Eles saíram do [`super`] em 2026-09-01, quando o formulário de gravar levou aquele ficheiro a
//! **631 de 600**. Andam juntos por uma lei e não por vizinhança: são **um cartão**, e o cartão é
//! a única superfície do Inspector que não é uma secção.
//!
//! # ⛔⛔ O que esta lista compra, e nenhum outro gate vê
//!
//! **Focabilidade.** Um widget pintado e no hit-index continua **morto sob o ponteiro** se não
//! estiver aqui — foi assim que o botão *Salvar Variação…* nasceu, e quem o apanhou foi o
//! `seam_properties`, na primeira corrida. *Pintado e registado não é vivo.*

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;

pub(super) fn populate(store: &mut WidgetStore) {
    // ⭐⭐ **Os chips de VARIANTE** (F5, critério 2) — a tabela inteira, e não só as que a cópia
    // vigente mostra: o `populate` corre uma vez e a lista muda com a selecção. ⚠️ Registar só as
    // pintadas repõe exactamente a costura que o `hit_indexed_ids_are_registered` apanhou aqui há
    // um bloco: um chip pintado e não registado **nunca** é focável, logo o Down/Up nunca dispara.
    // ⭐⭐⭐ **O CAMPO que reescreve o valor vigente** — e o gate
    // `hit_indexed_ids_are_registered` apanhou-o em falta.
    //
    // ⚠️ **O meu gate de costura NÃO o alcançava:** ele abre por um clique no CHIP (que está
    // registado) e o `Submit` chega por teclado, então o campo funcionava de ponta a ponta sem
    // nunca ser focável **por clique próprio**. O que ficava morto era o gesto de **carregar
    // dentro do campo** para pôr o cursor — o artista escreve, quer corrigir uma letra, carrega, e
    // não acontece nada. *Um gate de seam prova o caminho que ele percorre; o censo prova os que
    // ele PODE percorrer.*
    store.register(
        ids::INSP_INSTANCE_VALUE_EDIT,
        ph2d_editor_core::interaction::InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for &id in ids::INSP_INSTANCE_AXIS_OPTION.iter().flatten() {
        store.register(
            id,
            ph2d_editor_core::interaction::InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    // ⭐⭐⭐ **O gesto de GRAVAR UMA VERSÃO** (Enio, 2026-09-01) — o botão, o selector de
    // propriedade e os dois botões do formulário.
    //
    // ⛔⛔ **O gate de costura apanhou isto na PRIMEIRA corrida**: eles eram pintados e estavam no
    // hit-index, e o clique não produzia evento nenhum — *pintado e registado não é vivo; focável
    // é outra pergunta, e é esta lista que a responde.*
    for &id in ids::INSP_INSTANCE_SAVE_PROP.iter().chain(&[
        ids::INSP_INSTANCE_SAVE_VARIATION,
        ids::INSP_INSTANCE_SAVE_CONFIRM,
        ids::INSP_INSTANCE_SAVE_CANCEL,
    ]) {
        store.register(
            id,
            ph2d_editor_core::interaction::InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    // ⚠️ Os três CAMPOS, pela razão do irmão acima: sem isto, carregar dentro do campo para pôr o
    // cursor não faz nada.
    for &id in &[
        ids::INSP_INSTANCE_SAVE_NEW_PROP,
        ids::INSP_INSTANCE_SAVE_EXISTING,
        ids::INSP_INSTANCE_SAVE_VALUE,
    ] {
        store.register(
            id,
            ph2d_editor_core::interaction::InteractiveState::TextInput {
                state: ph2d_editor_core::widget::TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
}
