//! **O registo dos widgets da secção SIGNAL ACTIONS** (TOP-20 #5, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — mesmo corte das irmãs.
//!
//! ⚠️ **Nenhum valor é semeado aqui** — quem sabe o que cada linha diz é o snapshot. E ⚠️⚠️ **um id
//! que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante decide pelo
//! `is_focusable`, e o ramo `None => false` engole o clique em silêncio.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;

use super::populate::register_button_ids;

pub(crate) fn populate_action(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_ACTION_ROW);
    register_button_ids(store, &[ids::INSP_ACTION_ADD, ids::INSP_ACTION_REMOVE]);
    register_button_ids(store, &ids::INSP_ACTION_VERB);
    for id in [
        ids::INSP_ACTION_ON,
        ids::INSP_ACTION_TARGET,
        ids::INSP_ACTION_ARG,
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
