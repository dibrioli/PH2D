//! **Os controles da seção CONSTRAINTS** — irmão do [`super::populate_layout`], e pela mesma razão.
//!
//! ⚠️ Sem este registro os oito chips ficariam pintados, com hit-rect, e **MORTOS sob o mouse** — a
//! checagem de focabilidade mora no store. É o defeito que este painel já pagou quatro vezes (os
//! pills de modo, o Cut, a simetria, o layout), e o seam é o que o prova.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

/// Os oito chips das duas fileiras.
///
/// ⚠️ **A lista é uma só, e é a mesma que o `paint_anchors` desenha.** Um chip novo entra aqui e
/// ali; o gate de seam clica **todos** eles, que é o que impede a lista de registro de ficar para
/// trás da lista pintada.
pub(crate) const ANCHOR_CHIPS: &[ph2d_a11y::NodeId] = &[
    crate::ids::VECTOR_ANCHOR_H_START,
    crate::ids::VECTOR_ANCHOR_H_CENTER,
    crate::ids::VECTOR_ANCHOR_H_END,
    crate::ids::VECTOR_ANCHOR_H_STRETCH,
    crate::ids::VECTOR_ANCHOR_V_START,
    crate::ids::VECTOR_ANCHOR_V_CENTER,
    crate::ids::VECTOR_ANCHOR_V_END,
    crate::ids::VECTOR_ANCHOR_V_STRETCH,
];

pub(super) fn anchor_controls(store: &mut WidgetStore) {
    for &id in ANCHOR_CHIPS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
