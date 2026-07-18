//! O registro dos widgets da seção **EFFECTS** — irmão do [`super`] pelo teto de 600 LOC do
//! painel, e par natural do `paint_effects` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta — sem `InteractiveState`
//! o Down nunca ativa o widget e o Up nunca emite `Click`, então o botão fica **pintado e
//! morto**. É a classe de bug que já matou os pills do vetor uma vez.
//!
//! Os quatro são registrados **incondicionalmente**, mesmo os três sliders, que só são
//! PINTADOS quando o caminho selecionado tem um Trim: o store é agnóstico de estado, e quem
//! decide se o clique é possível é a PINTURA (sem hit-rect não há Click). Mesma regra do
//! `populate_envelope`.

use super::{button, slider_chip};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// O passo dos três campos numéricos, no domínio do documento (fração do comprimento).
const STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Os três parâmetros do Trim são frações `0..=1`, então o track do slider **é** o valor do
/// documento: escala 1, deslocamento 0. (O Bend do Envelope é o contra-exemplo — lá o track
/// `0..1` mapeia para `-1..1`, e é por isso que a conversão mora no `event.rs`.)
const IDENTITY_SCALE: f32 = 1.0;
const IDENTITY_OFFSET: f32 = 0.0;

/// Os widgets da seção Effects: o toggle do Trim e os três parâmetros dele.
pub(super) fn populate_effects(store: &mut WidgetStore) {
    button(store, ids::VECTOR_FX_TRIM);
    for (slider, num, default) in [
        (
            ids::VECTOR_FX_TRIM_START,
            ids::VECTOR_FX_TRIM_START_NUM,
            0.0,
        ),
        (ids::VECTOR_FX_TRIM_END, ids::VECTOR_FX_TRIM_END_NUM, 1.0),
        (
            ids::VECTOR_FX_TRIM_OFFSET,
            ids::VECTOR_FX_TRIM_OFFSET_NUM,
            0.0,
        ),
    ] {
        // O default do track e o do display coincidem — é o que "identidade" quer dizer.
        #[allow(clippy::cast_possible_truncation)]
        slider_chip(
            store,
            slider,
            num,
            default as f32,
            default,
            IDENTITY_SCALE,
            IDENTITY_OFFSET,
        );
        // Sem o range o arrasto do campo numérico escala errado — não é opcional.
        store.set_number_range(num, 0.0, 1.0, STEP);
    }
}
