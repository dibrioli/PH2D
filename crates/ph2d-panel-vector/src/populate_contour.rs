//! O registro dos widgets do **CONTOUR** (pesquisa `20_*` #9) — irmão do [`super`] pelo teto de
//! 600 LOC do painel, e par natural do `paint_contour` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta. O gate
//! `architecture_panel_wiring_parity` cobra a correspondência entre os dois arquivos.

use super::{button, slider_chip, slider_chip_int};
use crate::contour_params::{
    CONTOUR_ACCEL_MAX, CONTOUR_D_MAX, CONTOUR_STEPS_DEFAULT, CONTOUR_STEPS_MAX,
};
use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};

/// Passo do campo numérico de **Steps**: um anel. É a única granularidade que existe.
const CONTOUR_STEPS_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo numérico do **Offset**, em PONTOS PERCENTUAIS do tamanho da forma.
const CONTOUR_OFFSET_STEP: f64 = 0.5; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo numérico da **Accel**.
const CONTOUR_ACCEL_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento
/// Quantos por cento é uma unidade de fração — o fator do readout, não uma medida de desenho.
const PERCENT: f64 = 100.0; // LITERAL-PX-OK: conversão de unidade (fração -> percentual)

/// Os três botões de comando, os dois pares exclusivos de chips e os três sliders do Contour.
/// Registrados INCONDICIONALMENTE como todos os irmãos — quem decide se o clique é possível é a
/// PINTURA (sem hit-rect não há Click).
pub(super) fn populate_contour(store: &mut WidgetStore) {
    button(store, ids::VECTOR_CONTOUR_ADD);
    button(store, ids::VECTOR_CONTOUR_REMOVE);
    button(store, ids::VECTOR_CONTOUR_EXPAND);
    for id in [
        ids::VECTOR_CONTOUR_JOIN_MITER,
        ids::VECTOR_CONTOUR_JOIN_ROUND,
        ids::VECTOR_CONTOUR_JOIN_BEVEL,
        ids::VECTOR_CONTOUR_SIDE_OUTER,
        ids::VECTOR_CONTOUR_SIDE_INNER,
        ids::VECTOR_CONTOUR_SIDE_BOTH,
    ] {
        button(store, id);
    }
    // Steps: track `0..1` → `1..=CONTOUR_STEPS_MAX`, INTEIRO. O `scale`/`offset` do chip são o
    // mesmo mapa que `contour_params::steps_from_track` aplica ao track do slider — derivados da
    // MESMA const, e é a variante `_int` que garante que o campo nunca aceite `4,5` anéis.
    slider_chip_int(
        store,
        ids::VECTOR_CONTOUR_STEPS,
        ids::VECTOR_CONTOUR_STEPS_NUM,
        crate::contour_params::steps_to_track(CONTOUR_STEPS_DEFAULT),
        CONTOUR_STEPS_DEFAULT,
        (CONTOUR_STEPS_MAX - 1.0) as f32,
        1.0,
    );
    store.set_number_range(
        ids::VECTOR_CONTOUR_STEPS_NUM,
        1.0,
        CONTOUR_STEPS_MAX,
        CONTOUR_STEPS_STEP,
    );
    // Offset: BIPOLAR, e o campo fala **percentual** do tamanho da forma enquanto o bus carrega a
    // FRAÇÃO — a mesma divisão da seção Expand, pela mesma razão: o mapa do store é estático, então
    // um rótulo em unidades de mundo mentiria a cada troca de seleção. O `scale`/`offset` do chip
    // são o mapa de `d_from_track` vezes 100.
    slider_chip(
        store,
        ids::VECTOR_CONTOUR_OFFSET,
        ids::VECTOR_CONTOUR_OFFSET_NUM,
        0.5,
        0.0,
        (2.0 * CONTOUR_D_MAX * PERCENT) as f32,
        (-CONTOUR_D_MAX * PERCENT) as f32,
    );
    store.set_number_range(
        ids::VECTOR_CONTOUR_OFFSET_NUM,
        -CONTOUR_D_MAX * PERCENT,
        CONTOUR_D_MAX * PERCENT,
        CONTOUR_OFFSET_STEP,
    );
    // Accel: o slider é registrado, o campo é registrado — mas eles **NÃO são ligados**, e é
    // deliberado. `link_slider_number_mapped` só sabe mapas AFINS (`track·scale + offset`), e a
    // faixa da aceleração é GEOMÉTRICA (é o que põe o neutro `1.0` no centro do trilho, em vez de
    // a 21% dele). Ligá-los com um mapa afim faria o campo mostrar um número que o slider não
    // representa. Quem casa os dois é o `event.rs`, com o mapa de verdade e num sítio só.
    store.register(
        ids::VECTOR_CONTOUR_ACCEL,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_CONTOUR_ACCEL_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 1.0,
            buffer: "1".to_string(),
            caret: 0,
            last_committed: 1.0,
            selection_anchor: None,
        },
    );
    store.set_number_range(
        ids::VECTOR_CONTOUR_ACCEL_NUM,
        1.0 / CONTOUR_ACCEL_MAX,
        CONTOUR_ACCEL_MAX,
        CONTOUR_ACCEL_STEP,
    );
}
