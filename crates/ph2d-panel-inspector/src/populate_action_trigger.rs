//! **O registo dos widgets da secção GATILHO.**
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato.** O despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique **em silêncio** — é o
//! defeito que a caça aos knobs mortos de 30/08 mediu em 34 controlos, e que esta crate pagou
//! **sete** vezes. O gate de costura desta secção carrega com `Down`+`Up` REAIS.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe a acção, a aresta e o sinal de cada linha é o
//! **snapshot**. Semear texto aqui faria os gatilhos de um objecto mostrar os do anterior até ao
//! primeiro sync.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{DropdownState, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_action_trigger(store: &mut WidgetStore) {
    // As linhas da lista, os dois botões E **as três opções da aresta** — as opções de um dropdown
    // são BOTÕES, e é o `is_focusable` delas que decide se a escolha chega.
    register_button_ids(store, &crate::ids::INSP_TRIGGER_ROW);
    register_button_ids(store, &crate::ids::INSP_TRIGGER_EDGE_OPT);
    register_button_ids(
        store,
        &[
            crate::ids::INSP_TRIGGER_ADD,
            crate::ids::INSP_TRIGGER_REMOVE,
            crate::ids::INSP_TRIGGER_CREATE_ACTION,
        ],
    );

    // ⚠️ **O CHIP é o único `Dropdown` no store** — ele guarda só o `open`; a escolha vem do
    // snapshot a cada quadro.
    store.register(
        crate::ids::INSP_TRIGGER_EDGE_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );

    for id in [
        crate::ids::INSP_TRIGGER_ACTION,
        crate::ids::INSP_TRIGGER_SIGNAL,
    ] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
}
