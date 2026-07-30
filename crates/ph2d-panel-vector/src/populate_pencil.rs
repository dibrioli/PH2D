//! **Os dois knobs do LÁPIS** — irmão do [`super`] pelo teto de 600 LOC do painel, e o corte é por
//! responsabilidade: aqui mora o registro dos controles da mão livre (plano 25 W1).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_tool_vector::params::{
    PENCIL_FIDELITY_DEFAULT_PX, PENCIL_FIDELITY_SLIDER_OFFSET, PENCIL_FIDELITY_SLIDER_SCALE,
    PENCIL_STABILIZER_DEFAULT, PENCIL_STABILIZER_SLIDER_OFFSET, PENCIL_STABILIZER_SLIDER_SCALE,
    fidelity_px_to_slider,
};

use crate::ids;

/// **Fidelity + Stabilizer**, os dois controles da mão livre. Ambos são slider + chip ligados: o
/// chip é onde se digita um número exato, e o slider é onde se sente a faixa.
pub(super) fn pencil_knobs(store: &mut WidgetStore) {
    for (slider, chip, track, val, scale, offset) in [
        (
            ids::VECTOR_PENCIL_FIDELITY,
            ids::VECTOR_PENCIL_FIDELITY_NUM,
            fidelity_px_to_slider(PENCIL_FIDELITY_DEFAULT_PX),
            PENCIL_FIDELITY_DEFAULT_PX,
            PENCIL_FIDELITY_SLIDER_SCALE,
            PENCIL_FIDELITY_SLIDER_OFFSET,
        ),
        (
            // O track do estabilizador é `0..=1`; o CHIP mostra por cento (o idioma dos chips
            // de opacidade), então a conversão de unidade mora no mapeamento e não num readout.
            ids::VECTOR_PENCIL_STABILIZER,
            ids::VECTOR_PENCIL_STABILIZER_NUM,
            PENCIL_STABILIZER_DEFAULT,
            f64::from(PENCIL_STABILIZER_DEFAULT * PENCIL_STABILIZER_SLIDER_SCALE),
            PENCIL_STABILIZER_SLIDER_SCALE,
            PENCIL_STABILIZER_SLIDER_OFFSET,
        ),
    ] {
        store.register(
            slider,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: track,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: val,
                buffer: format!("{val:.1}"),
                caret: 0,
                last_committed: val,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(slider, chip, scale, offset);
    }
}
