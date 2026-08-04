//! **Os controles da seção COMPONENT** — irmão do [`super::populate_anchors`], mesma razão.
//!
//! ⚠️ Sem este registro os botões ficariam pintados, com hit-rect, e **MORTOS sob o mouse** — a
//! checagem de focabilidade mora no store. É o defeito que este painel já pagou cinco vezes,
//! e o seam é o que o prova.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

use crate::ids;

/// Os verbos de seção.
///
/// ⚠️ **A lista é uma só, e é a mesma que o `paint_components` desenha.** Um verbo novo entra aqui
/// e ali; o gate de seam clica **todos** eles, que é o que impede a lista de registro de ficar
/// para trás da lista pintada.
pub(crate) const COMPONENT_BUTTONS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_COMPONENT_CREATE,
    ids::VECTOR_COMPONENT_PLACE,
    ids::VECTOR_COMPONENT_DETACH,
    ids::VECTOR_COMPONENT_RESET,
    ids::VECTOR_COMPONENT_UPDATE_MAIN,
    ids::VECTOR_COMPONENT_SWAP,
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
    // **As LINHAS de peça** (W5b). O teto é registado SEMPRE, e não a contagem viva: o `populate`
    // corre antes do corpo, então registar `pieces().len()` acoplaria o registo à ordem de duas
    // fases — e uma peça a mais num frame nasceria morta sob o mouse até ao frame seguinte. Os
    // slots a mais são widgets que nada pinta, que é o que o `paint` decide.
    for row in 0..ids::MAX_INSTANCE_PIECES {
        store.register(
            ids::vector_instance_piece_show_id(row),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        // ⚠️ A swatch é alvo de PICKER, não botão: registá-la como botão faria o clique acender o
        // widget e **nunca abrir o picker** — a cor ficaria ineditável com todos os gates verdes.
        store.register_picker_swatch(ids::vector_instance_piece_colour_id(row));
    }
    // **Os chips de VARIANT** (W5c). O teto é registado SEMPRE, pela mesma razão das peças: o
    // `populate` corre antes do corpo, e registar a contagem viva faria um chip novo nascer morto
    // sob o mouse até ao frame seguinte.
    for axis in 0..ids::MAX_VARIANT_AXES {
        for value in 0..ids::MAX_VARIANT_VALUES {
            store.register(
                ids::vector_variant_option_id(axis, value),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
}
