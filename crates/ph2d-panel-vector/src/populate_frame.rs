//! **Os controles da seção FRAME** — irmão do [`super::populate`] pelo teto de 600 LOC do painel.
//!
//! ⚠️ Sem este registro os seis botões ficariam pintados, com hit-rect, e **MORTOS sob o mouse** —
//! a checagem de focabilidade mora no store, e é ela que o seam prova. É o defeito que este painel
//! já pagou com os pills de modo (duas vezes) e com o Cut.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_vector::frames::DEVICE_PRESETS;

use crate::ids;

/// Os dois chips de recorte e os presets de dispositivo.
pub(super) fn frame_controls(store: &mut WidgetStore) {
    // Os presets saem da MESMA tabela que os pinta e que resolve o clique — um aparelho novo
    // nasce registado, pintado e vivo, sem passar por três listas.
    let presets = DEVICE_PRESETS.iter().map(|p| p.id);
    for id in presets.chain([ids::VECTOR_FRAME_CLIP_OFF, ids::VECTOR_FRAME_CLIP_ON]) {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
