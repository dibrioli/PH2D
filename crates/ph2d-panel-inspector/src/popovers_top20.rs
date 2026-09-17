//! ⭐⭐⭐ **OS DOIS SELECTORES DAS SECÇÕES DO TOP-20** — a CUTSCENE (#19) e a COMPARAÇÃO da vigia do
//! contador. Irmão de [`super`] pelo tecto de 200 LOC da `paint_deferred_popovers`.
//!
//! ⚠️ **O corte é por ASSUNTO e não por tamanho** (a mesma lei do irmão dos tags): os quatro
//! popovers que ficam lá escolhem um **ENUM do objecto** e as opções deles saem do descritor;
//! estes dois escolhem dentro de uma lista que só a **fila do TOP-20** conhece.
//!
//! ⚠️⚠️ **E as duas fontes de opções NÃO são a mesma**, o que é a coisa a não trocar ao ler:
//! a lista das cutscenes rederiva-se do **snapshot** (é a fonte dela, e uma cópia guardada ficaria
//! um quadro atrás do documento), e a comparação da vigia **viaja no SLOT** — ela é da regra
//! ABERTA, cujo índice vive no `InspectorState`, que este passe não vê.

use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::Dropdown;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use super::{paint_open_popover, state_popovers};
use crate::sections;

/// Ver o cabeçalho do módulo.
pub(super) fn paint_deferred_top20_popovers(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    region: Rect,
) {
    // ⭐⭐⭐ **SEQUENCE** (TOP-20 #19) — a lista das cutscenes do documento, mesmo passe diferido.
    //
    // ⚠️ **Só o rect viaja no slot** e a lista rederiva-se do snapshot, que é a fonte dela: uma
    // cópia guardada ficaria um quadro atrás do documento no quadro em que alguém apaga um
    // container com o selector aberto.
    if let Some(chip) = state_popovers::take_pending_seq_dd()
        && let Some(info) = crate::state_components::current_inspector_sequence()
    {
        let mut dd = Dropdown::new(
            crate::ids::INSP_SEQ_PICK,
            "",
            sections::sequence::opcoes(&info.nomes),
        )
        .open(true)
        .placeholder(sections::sequence::placeholder(&info));
        if let Some(i) = info.escolhido {
            dd.select(i);
        }
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }

    // ⭐⭐⭐ **COUNTER WATCH** — as três comparações da regra aberta, mesmo passe diferido.
    //
    // ⚠️ **A comparação vem do SLOT e não do snapshot**, ao contrário do selector da cutscene: ela
    // é da regra ABERTA, cujo índice vive no `InspectorState`, que este passe não vê.
    if let Some((compare, chip)) = state_popovers::take_pending_watch_dd() {
        let mut dd = Dropdown::new(
            crate::ids::INSP_WATCH_CMP_PICK,
            "",
            sections::counter_watch::opcoes_de_comparacao(),
        )
        .open(true);
        dd.select(usize::from(compare));
        paint_open_popover(
            &dd,
            chip,
            region,
            store,
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
}
