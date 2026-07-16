//! O registro dos widgets do **BLEND / MORPH** — irmão do [`super`] pelo teto de 600
//! LOC do painel, e par natural do `paint_blend` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta — é a classe de bug que já
//! matou botões do vetor uma vez. O gate `architecture_panel_wiring_parity` cobra a
//! correspondência entre os dois arquivos.

use super::button;
use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_tool_vector::params::{
    BLEND_STEPS_DEFAULT, MAX_BLEND_STEPS, MORPH_T_DEFAULT, MORPH_T_STEP, blend_steps_to_track,
};

/// Os widgets do Blend Object vivo e do Morph.
pub(super) fn populate_blend(store: &mut WidgetStore) {
    // BLEND: o slider de passos (com a caixa ligada) + os 3 botões, sendo 2 deles o ESCAPE
    // manual da correspondência (sem eles, o dia em que o automático errar não tem saída).
    store.register(
        ids::VECTOR_BLEND_STEPS,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: blend_steps_to_track(BLEND_STEPS_DEFAULT),
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_BLEND_STEPS_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(BLEND_STEPS_DEFAULT),
            buffer: format!("{BLEND_STEPS_DEFAULT}"),
            caret: 0,
            last_committed: f64::from(BLEND_STEPS_DEFAULT),
            selection_anchor: None,
        },
    );
    store.set_number_range(
        ids::VECTOR_BLEND_STEPS_NUM,
        1.0,
        f64::from(MAX_BLEND_STEPS),
        1.0,
    );
    button(store, ids::VECTOR_BLEND_RUN);
    // **Reset Spine** — volta o spine editado (modo Node) ao automático. Registrar aqui é o que o
    // torna clicável (pintar + hit-rect não basta — a classe de bug dos botões do vetor).
    button(store, ids::VECTOR_BLEND_RESET_SPINE);
    // **Expand** / **Release** (ADR-0122 Fase D) — o mesmo motivo: sem registro, sao pintura.
    button(store, ids::VECTOR_BLEND_EXPAND);
    button(store, ids::VECTOR_BLEND_RELEASE);
    // **MORPH** — o irmão animável do Blend: uma forma só, e o `t` dela é keyável. O slider é a
    // autoria ao vivo (o artista estaciona a forma onde ela fica bem e aperta K).
    button(store, ids::VECTOR_MORPH_RUN);
    store.register(
        ids::VECTOR_MORPH_T,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: MORPH_T_DEFAULT,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_MORPH_T_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(MORPH_T_DEFAULT),
            buffer: format!("{MORPH_T_DEFAULT:.2}"),
            caret: 0,
            last_committed: f64::from(MORPH_T_DEFAULT),
            selection_anchor: None,
        },
    );
    // O `t` é uma FRAÇÃO do caminho: o range é o domínio inteiro dele, e o motor clampa lá.
    store.set_number_range(ids::VECTOR_MORPH_T_NUM, 0.0, 1.0, MORPH_T_STEP);
}
