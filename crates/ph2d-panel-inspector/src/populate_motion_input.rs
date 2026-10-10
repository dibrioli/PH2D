//! **O registo dos widgets da secção MOTION INPUT** (ciclo 19 do Motion).
//!
//! ⚠️ Um id que não passa por aqui é pintado, hit-registado e MORTO sob o rato (o despachante decide
//! pelo `is_focusable`).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

use super::populate::register_button_ids;

pub(crate) fn populate_motion_input(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_MI_SOURCE);
    register_button_ids(store, &crate::ids::INSP_MI_GRAFO);
    // A amostra não carrega valor (a cor vive no `widget_colors`): `Plain`, como as das partículas.
    store.register(crate::ids::INSP_MI_COLOR, InteractiveState::Plain);
    store.register(
        crate::ids::INSP_MI_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.register(
        crate::ids::INSP_MI_VALUE,
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
