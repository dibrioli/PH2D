//! **O registo dos widgets do cartão CONSTRAINTS.** Um id que não passa por aqui é pintado,
//! hit-registado e MORTO sob o rato (o despacho decide pelo `is_focusable`).
//!
//! A barra da força é `0..1` — a fração da `Item::forca`: é uma fração, não tem outro recurso.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::widget::{
    CheckboxState, CheckboxValue, DropdownState, SliderOrientation, SliderState, TextInputState,
};

/// A força é uma fração `0..1`, mostrada em %.
const PCT: f32 = 100.0; // LITERAL-PX-OK: fração mostrada em %, não medida de desenho

pub(crate) fn populate_constraint(store: &mut WidgetStore) {
    super::populate::register_button_ids(
        store,
        &[
            ids::INSP_CONSTRAINT_ADD,
            ids::INSP_CONSTRAINT_BAKE,
            ids::INSP_CONSTRAINT_BEHAVIOURS,
        ],
    );
    super::populate::register_button_ids(store, &ids::INSP_CONSTRAINT_TIPO);
    for tabela in [
        &ids::INSP_CONSTRAINT_SUBIR,
        &ids::INSP_CONSTRAINT_DESCER,
        &ids::INSP_CONSTRAINT_APAGAR,
        &ids::INSP_CONSTRAINT_ALVO,
        &ids::INSP_CONSTRAINT_ZERAR,
        &ids::INSP_CONSTRAINT_PESO,
        &ids::INSP_CONSTRAINT_SEM_PESO,
        &ids::INSP_COLA_DESCOLAR,
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
    for tabela in ids::INSP_CONSTRAINT_OPCAO.iter().flatten() {
        super::populate::register_button_ids(store, tabela);
    }
    // Os ▾ do Follow Value: as opções são BOTÕES, o chip um `Dropdown` (só o `open` mora aqui).
    super::populate::register_button_ids(store, &ids::INSP_CONSTRAINT_PROP_OPCAO);
    for &id in ids::INSP_CONSTRAINT_PROP.iter().flatten() {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
    }
    // A partida é 0; o `sync_constraint` semeia o do documento.
    for &id in ids::INSP_CONSTRAINT_NUMERO.iter().flatten() {
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
    // A partida é a força de um item novo (`1`); o `sync_constraint` semeia a do documento.
    for (id, chip) in ids::INSP_CONSTRAINT_FORCA
        .into_iter()
        .zip(ids::INSP_CONSTRAINT_FORCA_CHIP)
    {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 1.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format_number(0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(id, chip, PCT, 0.0);
        store.set_number_range(chip, 0.0, f64::from(PCT), 1.0);
        store.mark_card_field(
            chip,
            CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: Some(f64::from(PCT)),
            },
        );
    }
}

/// A fração `0..1` da força no campo do cartão, sem o resto do `f32` («65.000»).
pub(crate) fn em_pct(forca: f32) -> f64 {
    f64::from((forca * PCT).round())
}
