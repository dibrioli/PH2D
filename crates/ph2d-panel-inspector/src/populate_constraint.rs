//! **O registo dos widgets do cartão CONSTRAINTS.** Um id que não passa por aqui é pintado,
//! hit-registado e MORTO sob o rato (o despacho decide pelo `is_focusable`).
//!
//! A barra da força é `0..1` — a fração da `Item::forca`: é uma fração, não tem outro recurso.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{CheckboxState, CheckboxValue, SliderOrientation, SliderState};

pub(crate) fn populate_constraint(store: &mut WidgetStore) {
    super::populate::register_button_ids(store, &[ids::INSP_CONSTRAINT_ADD]);
    super::populate::register_button_ids(store, &ids::INSP_CONSTRAINT_TIPO);
    for tabela in [
        &ids::INSP_CONSTRAINT_SUBIR,
        &ids::INSP_CONSTRAINT_DESCER,
        &ids::INSP_CONSTRAINT_APAGAR,
        &ids::INSP_CONSTRAINT_ALVO,
    ] {
        super::populate::register_button_ids(store, tabela);
    }
    for id in ids::INSP_CONSTRAINT_LIGADA {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Checked,
            },
        );
    }
    // A partida é a força de um item novo (`1`); o `sync_constraint` semeia a do documento.
    for id in ids::INSP_CONSTRAINT_FORCA {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 1.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
}
