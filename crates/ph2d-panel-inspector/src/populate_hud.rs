//! **O registo dos widgets da secção HUD** (TOP-20 #20).
//!
//! ⚠️⚠️ **Um id que não passa por aqui é pintado, hit-registado e MORTO sob o rato** (o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio) — a lição que
//! esta crate já pagou sete vezes.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_hud(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_HUD_FIT);
    register_button_ids(store, &crate::ids::INSP_HUD_SOURCE);
    store.register(
        crate::ids::INSP_HUD_DISABLED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    store.register(
        crate::ids::INSP_HUD_COUNTER_KEEP,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in crate::ids::INSP_HUD_TEXT {
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
    for id in crate::ids::INSP_HUD_NUM {
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
