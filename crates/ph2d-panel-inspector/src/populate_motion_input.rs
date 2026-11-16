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
    register_button_ids(
        store,
        &[crate::ids::INSP_MI_NEW, crate::ids::INSP_MI_ADD_HEALTH],
    );
    // A amostra não carrega valor (a cor vive no `widget_colors`): `Plain`, como as das partículas.
    store.register(crate::ids::INSP_MI_COLOR, InteractiveState::Plain);
    for id in [crate::ids::INSP_MI_NAME, crate::ids::INSP_MI_FROM] {
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
    // Os de fábrica do `MotionInput` (faixa `0..1`, Smooth desligado). Sem faixa no store: o passo vem da
    // linha, e o Smooth negativo prende-se a `0` no dreno (a mesma porta de um script).
    for (id, v) in [
        (crate::ids::INSP_MI_VALUE, 0.0),
        (crate::ids::INSP_MI_REMAP_FROM, 0.0),
        (crate::ids::INSP_MI_REMAP_TO, 1.0),
        (crate::ids::INSP_MI_SMOOTH, 0.0),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: v,
                buffer: format_number(v),
                caret: 0,
                last_committed: v,
                selection_anchor: None,
            },
        );
    }
    // GEF 08 onda 2: o conta-gotas, as caixas, as predefinições da curva e as saídas das queixas.
    register_button_ids(
        store,
        &[
            crate::ids::INSP_MI_FROM_PICK,
            crate::ids::INSP_MI_REMAP,
            crate::ids::INSP_MI_ADD_BODY,
            crate::ids::INSP_MI_PICK_OBJECT,
            crate::ids::INSP_MI_OPEN_INPUT_MAP,
        ],
    );
    register_button_ids(store, &crate::ids::INSP_MI_CURVE);
    for id in [crate::ids::INSP_MI_ACTION, crate::ids::INSP_MI_PROPERTY] {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: ph2d_editor_core::widget::DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
    }
    store.mark_collapsible_section(crate::ids::INSP_MI_FOLD_SHAPE);
    store.set_collapsed_if_unchosen(crate::ids::INSP_MI_FOLD_SHAPE, true);
    use crate::ids as d;
    for (id, dica) in [
        (d::INSP_MI_FROM, "panel.inspector.motion_input.from_tip"),
        (
            d::INSP_MI_FROM_PICK,
            "panel.inspector.motion_input.from_pick_tip",
        ),
        (
            d::INSP_MI_PICK_OBJECT,
            "panel.inspector.motion_input.from_pick_tip",
        ),
        (d::INSP_MI_ACTION, "panel.inspector.motion_input.action_tip"),
        (
            d::INSP_MI_PROPERTY,
            "panel.inspector.motion_input.property_tip",
        ),
        (
            d::INSP_MI_FOLD_SHAPE,
            "panel.inspector.motion_input.shape_tip",
        ),
        (d::INSP_MI_REMAP, "panel.inspector.motion_input.remap_tip"),
        (
            d::INSP_MI_REMAP_FROM,
            "panel.inspector.motion_input.remap_range_tip",
        ),
        (
            d::INSP_MI_REMAP_TO,
            "panel.inspector.motion_input.remap_range_tip",
        ),
        (d::INSP_MI_SMOOTH, "panel.inspector.motion_input.smooth_tip"),
    ]
    .into_iter()
    .chain(d::INSP_MI_CURVE.map(|id| (id, "panel.inspector.motion_input.curve_tip")))
    {
        store.set_tooltip(id, ph2d_i18n::tr(dica));
    }
}
