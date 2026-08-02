//! **Os controles da seção LAYOUT** — irmão do [`super::populate_frame`], e pela mesma razão.
//!
//! ⚠️ Sem este registro os chips e os campos ficariam pintados, com hit-rect, e **MORTOS sob o
//! mouse** — a checagem de focabilidade mora no store. É o defeito que este painel já pagou três
//! vezes (os pills de modo, o Cut, a simetria), e o seam é o que o prova.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

use crate::ids;

/// Os chips do rádio de direção, do modo de recuo, do alinhamento e da distribuição.
///
/// ⚠️ **A lista é uma só, e é a mesma que o `paint_layout` desenha.** Uma variante nova entra aqui
/// e ali; o gate de seam clica **todos** eles, que é o que impede a lista de registro de ficar
/// para trás da lista pintada.
pub(crate) const LAYOUT_CHIPS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_LAYOUT_DIR_OFF,
    ids::VECTOR_LAYOUT_DIR_ROW,
    ids::VECTOR_LAYOUT_DIR_COL,
    ids::VECTOR_LAYOUT_DIR_WRAP,
    ids::VECTOR_LAYOUT_PAD_ALL_MODE,
    ids::VECTOR_LAYOUT_PAD_EACH_MODE,
    ids::VECTOR_LAYOUT_ALIGN_START,
    ids::VECTOR_LAYOUT_ALIGN_CENTER,
    ids::VECTOR_LAYOUT_ALIGN_END,
    ids::VECTOR_LAYOUT_ALIGN_STRETCH,
    ids::VECTOR_LAYOUT_JUSTIFY_START,
    ids::VECTOR_LAYOUT_JUSTIFY_CENTER,
    ids::VECTOR_LAYOUT_JUSTIFY_END,
    ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN,
    ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
];

/// Os nove campos numéricos: vão (×2), recuo (×5 contando o *All*), Grow e Shrink.
pub(crate) const LAYOUT_FIELDS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_LAYOUT_GAP_MAIN,
    ids::VECTOR_LAYOUT_GAP_CROSS,
    ids::VECTOR_LAYOUT_PAD_ALL,
    ids::VECTOR_LAYOUT_PAD_T,
    ids::VECTOR_LAYOUT_PAD_R,
    ids::VECTOR_LAYOUT_PAD_B,
    ids::VECTOR_LAYOUT_PAD_L,
    ids::VECTOR_LAYOUT_ITEM_GROW,
    ids::VECTOR_LAYOUT_ITEM_SHRINK,
];

pub(super) fn layout_controls(store: &mut WidgetStore) {
    for &id in LAYOUT_CHIPS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for &id in LAYOUT_FIELDS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::from("0"),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // ⚠️ **Sem `set_number_range`**, e é a mesma decisão dos campos do Transform: vão e recuo
        // são DISTÂNCIAS DE MUNDO, que não têm magnitude máxima, e a shell calibra o arrasto pela
        // câmara (`px_to_world`) para o gesto ser 1:1 com a tela em qualquer zoom. Grow e Shrink
        // são RAZÕES adimensionais — `grow = 1000` e `grow = 1` fazem a MESMA coisa quando só um
        // filho cresce, então um teto ali seria um número inventado, não um recurso.
        //
        // O que o modelo de facto recusa — vão e recuo NEGATIVOS, que o flexbox não exprime — é
        // recusado na PORTA de edição da shell, onde a regra é do documento e não do widget.
    }
}
