//! **O registo dos widgets da secção PARTICLES** (TOP-20 #18, W3).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é pintado, hit-registado e MORTO sob o rato** (o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio) — a lição da
//! fileira de chips da booleana do vetor.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, TextInputState};

use super::populate::register_button_ids;

pub(crate) fn populate_particles(store: &mut WidgetStore) {
    register_button_ids(store, &crate::ids::INSP_PART_SHAPE);
    register_button_ids(store, &crate::ids::INSP_PART_SPACE);
    // ⚠️⚠️ **As duas AMOSTRAS DE COR são `Plain`, e a ausência delas aqui era muda:** a amostra não
    // carrega valor nenhum (a cor dela vive na tabela lateral `widget_colors`), então ela pinta-se
    // e hit-regista-se na mesma — e o clique morre no `is_focusable`, sem o selector abrir. É a
    // lei que o Color & Tint escreve ao lado dos seis dele.
    for id in [crate::ids::INSP_PART_COLOR, crate::ids::INSP_PART_COLOR_END] {
        store.register(id, InteractiveState::Plain);
    }
    for id in [
        crate::ids::INSP_PART_EMITTING,
        crate::ids::INSP_PART_ONE_SHOT,
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
    for id in crate::ids::INSP_PART_TEXT {
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
    for id in crate::ids::INSP_PART_NUM {
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
