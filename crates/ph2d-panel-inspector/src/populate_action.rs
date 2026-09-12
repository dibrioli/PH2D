//! **O registo dos widgets da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe o que cada linha diz é o snapshot. E ⚠️⚠️ **um id
//! que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante decide pelo
//! `is_focusable`, e o ramo `None => false` engole o clique em silêncio.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{DropdownState, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_action(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_ACTION_ROW);
    register_button_ids(
        store,
        &[crate::ids::INSP_ACTION_ADD, crate::ids::INSP_ACTION_REMOVE],
    );
    // ⚠️ **As entradas do seletor do verbo continuam BOTÕES** — elas são as linhas do popover, e
    // o despachante decide pelo `is_focusable`: sem registo, o clique numa opção é engolido em
    // silêncio. Só o CHIP é um `Dropdown`, e o que ele guarda é o `open`, nunca a escolha.
    register_button_ids(store, &ids::INSP_ACTION_VERB);
    store.register(
        crate::ids::INSP_ACTION_VERB_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    for id in [
        crate::ids::INSP_ACTION_ON,
        crate::ids::INSP_ACTION_TARGET,
        crate::ids::INSP_ACTION_ARG,
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
