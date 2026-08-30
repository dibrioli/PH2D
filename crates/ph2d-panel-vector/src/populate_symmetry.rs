//! **Os controles da seção SYMMETRY** — irmão do [`super::populate`] pelo teto de 600 LOC do
//! painel, e o corte é por responsabilidade: aqui mora o registro dos controles do modo simétrico
//! (plano 25 W6.3).
//!
//! ⚠️ Sem este registro os chips ficariam pintados, com hit-rect, e **MORTOS sob o mouse** — a
//! checagem de focabilidade mora no store, e é ela que o seam prova. Foi exactamente assim que os
//! dez chips da lista de ferramentas do impasto nasceram inertes.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_symmetry::{SymmetryKind, SymmetryStyle};
use ph2d_tool_vector::params::symmetry_kind_id;

use crate::ids;
use crate::paint_symmetry::{SEGMENTS_OFFSET, SEGMENTS_SCALE, segments_to_track};

/// Os botões e o slider do modo simétrico.
pub(super) fn symmetry_controls(store: &mut WidgetStore) {
    let d = SymmetryStyle::default();
    // Os quatro tipos saem de `SymmetryKind::ALL` pela MESMA porta que os pinta e que resolve o
    // clique — um tipo novo nasce registado, pintado e vivo, sem passar por três listas.
    let kinds = SymmetryKind::ALL.iter().copied().map(symmetry_kind_id);
    for id in kinds.chain([
        ids::VECTOR_SYM_OFF,
        ids::VECTOR_SYM_ON,
        ids::VECTOR_SYM_FUSE_OFF,
        ids::VECTOR_SYM_FUSE_ON,
        ids::VECTOR_SYM_APPLY,
    ]) {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    let val = f64::from(d.segments);
    store.register(
        ids::VECTOR_SYM_SEGMENTS,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: segments_to_track(d.segments),
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_SYM_SEGMENTS_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: val,
            buffer: format!("{val:.0}"),
            caret: 0,
            last_committed: val,
            selection_anchor: None,
        },
    );
    // ⚠️ **`_integer` e não o contínuo**: a unidade pintada é uma CONTAGEM de cópias, e sem o
    // snap um `6,5` digitado ficava preso no chip enquanto o rótulo (`{n:.0}`) mostrava `6` — o
    // achado #3 da auditoria de 2026-05-28, que o chip de Segments do Painter já honra. A faixa é
    // a mesma que o `track_to_segments` inverte, que é o que faz a ida-e-volta fechar.
    store.link_slider_number_mapped_integer(
        ids::VECTOR_SYM_SEGMENTS,
        ids::VECTOR_SYM_SEGMENTS_NUM,
        SEGMENTS_SCALE,
        SEGMENTS_OFFSET,
    );
}
