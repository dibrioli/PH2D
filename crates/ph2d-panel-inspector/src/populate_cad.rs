//! **O registo dos widgets da secção DRAWING RULES.** Um id que não passa por aqui é pintado,
//! hit-registado e MORTO sob o rato (o despacho decide pelo `is_focusable`).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

pub(crate) fn populate_cad(store: &mut WidgetStore) {
    super::populate::register_button_ids(store, &ids::INSP_CAD_APAGAR);
    // A partida é 0; o `sync_cad` semeia a medida do documento.
    for id in ids::INSP_CAD_MEDIDA {
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
