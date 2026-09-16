//! **O registo dos widgets da secção SCRIPT** (TOP-20 #16, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **As TRÊS tabelas de controlo registam-se inteiras**, embora cada linha pinte só uma: o tipo
//! de uma linha é o do default declarado, e um ficheiro gravado a meio da sessão pode trocá-lo. Um
//! id que não passa por aqui é **pintado, hit-registado e MORTO sob o rato** (o despachante decide
//! pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

use super::populate::register_button_ids;

fn text_input() -> InteractiveState {
    InteractiveState::TextInput {
        state: TextInputState::Normal,
        text: String::new(),
        caret: 0,
        selection_anchor: None,
    }
}

pub(crate) fn populate_script(store: &mut WidgetStore) {
    register_button_ids(store, &[crate::ids::INSP_SCRIPT_BROWSE]);
    register_button_ids(store, &crate::ids::INSP_SCRIPT_RESET);
    register_button_ids(store, &crate::ids::INSP_SCRIPT_ORPHAN_REMOVE);
    store.register(crate::ids::INSP_SCRIPT_SOURCE, text_input());
    for id in crate::ids::INSP_SCRIPT_TEXT {
        store.register(id, text_input());
    }
    for id in crate::ids::INSP_SCRIPT_BOOL {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    // ⚠️ **Sem faixa aqui**: a faixa de cada linha é a PISTA que o script declarou, e quem a
    // conhece é o snapshot — a semente escreve-a (`sync_script`).
    for id in crate::ids::INSP_SCRIPT_NUM {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
}
