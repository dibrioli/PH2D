//! **O registo dos widgets da secção SEQUENCE** (TOP-20 #19, W3).
//!
//! ⚠️⚠️ **Um id que não passa por aqui é pintado, hit-registado e MORTO sob o rato** (o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio) — a lição que
//! esta crate já pagou sete vezes, e a última delas foi as duas amostras de cor do emissor.
//!
//! ⚠️ **As ENTRADAS do selector são botões** — elas são as linhas do popover, e o chip é o único
//! `Dropdown`: o que ele guarda é o `open`, nunca a escolha.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::DropdownState;

use super::populate::register_button_ids;

pub(crate) fn populate_sequence(store: &mut WidgetStore) {
    store.register(
        crate::ids::INSP_SEQ_PICK,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    register_button_ids(store, &crate::ids::INSP_SEQ_OPT);
    register_button_ids(store, &[crate::ids::INSP_SEQ_CLEAR]);
}
