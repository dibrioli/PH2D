//! **Os controles da seção COMPONENT** — irmão do [`super::populate_anchors`], mesma razão.
//!
//! ⚠️ Sem este registro os quatro botões ficariam pintados, com hit-rect, e **MORTOS sob o mouse**
//! — a checagem de focabilidade mora no store. É o defeito que este painel já pagou cinco vezes,
//! e o seam é o que o prova.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

use crate::ids;

/// Os quatro verbos.
///
/// ⚠️ **A lista é uma só, e é a mesma que o `paint_components` desenha.** Um verbo novo entra aqui
/// e ali; o gate de seam clica **todos** eles, que é o que impede a lista de registro de ficar
/// para trás da lista pintada.
pub(crate) const COMPONENT_BUTTONS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_COMPONENT_CREATE,
    ids::VECTOR_COMPONENT_PLACE,
    ids::VECTOR_COMPONENT_DETACH,
    ids::VECTOR_COMPONENT_RESET,
];

pub(super) fn component_controls(store: &mut WidgetStore) {
    for &id in COMPONENT_BUTTONS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
