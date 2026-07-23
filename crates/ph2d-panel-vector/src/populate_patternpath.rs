//! O registro dos widgets do **PATTERN ON PATH** (plano 23) — irmão do [`super`] pelo teto de 600
//! LOC do painel, e par natural do `paint_patternpath` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta. O gate
//! `architecture_panel_wiring_parity` cobra a correspondência entre os dois arquivos.

use super::{button, slider_chip};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Passo do campo numérico do Spacing (múltiplos da largura do motivo).
const PATTERNPATH_SPACING_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo numérico do Start (fração do comprimento do caminho).
const PATTERNPATH_START_STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento

/// Os quatro botões + os dois sliders do Pattern on Path. Registrados INCONDICIONALMENTE como
/// todos os irmãos — quem decide se o clique é possível é a PINTURA (sem hit-rect não há Click).
pub(super) fn populate_patternpath(store: &mut WidgetStore) {
    button(store, ids::VECTOR_PATTERNPATH_LINK);
    button(store, ids::VECTOR_PATTERNPATH_PICK);
    button(store, ids::VECTOR_PATTERNPATH_DETACH);
    button(store, ids::VECTOR_PATTERNPATH_FLIP);
    button(store, ids::VECTOR_PATTERNPATH_FLIP_OFF);
    // Spacing: track `0..1` → valor `SPACING_MIN..SPACING_MAX`. O `scale`/`offset` do chip são o
    // MESMO mapa que o `event::track_slider_event` aplica ao track do slider — a fronteira única.
    let sp_scale = (crate::SPACING_MAX - crate::SPACING_MIN) as f32;
    slider_chip(
        store,
        ids::VECTOR_PATTERNPATH_SPACING,
        ids::VECTOR_PATTERNPATH_SPACING_NUM,
        ((1.0 - crate::SPACING_MIN) / (crate::SPACING_MAX - crate::SPACING_MIN)) as f32,
        1.0,
        sp_scale,
        crate::SPACING_MIN as f32,
    );
    store.set_number_range(
        ids::VECTOR_PATTERNPATH_SPACING_NUM,
        crate::SPACING_MIN,
        crate::SPACING_MAX,
        PATTERNPATH_SPACING_STEP,
    );
    // Start / End: FRAÇÃO do comprimento (track == valor). Start nasce em 0, End em 1 (a curva
    // inteira). O trecho `[Start, End]` é onde as cópias caem.
    for (slider, chip, track) in [
        (
            ids::VECTOR_PATTERNPATH_START,
            ids::VECTOR_PATTERNPATH_START_NUM,
            0.0,
        ),
        (
            ids::VECTOR_PATTERNPATH_END,
            ids::VECTOR_PATTERNPATH_END_NUM,
            1.0,
        ),
        (
            ids::VECTOR_PATTERNPATH_SLIDE,
            ids::VECTOR_PATTERNPATH_SLIDE_NUM,
            0.5,
        ),
    ] {
        slider_chip(store, slider, chip, track, f64::from(track), 1.0, 0.0);
        store.set_number_range(chip, 0.0, 1.0, PATTERNPATH_START_STEP);
    }
    // Offset: BIPOLAR (unidades de mundo). O track `0..1` mapeia `−OFFSET_MAX..OFFSET_MAX`; `0.5`
    // (o default) é sobre a curva. O `scale`/`offset` do chip são o MESMO mapa do `event.rs`.
    slider_chip(
        store,
        ids::VECTOR_PATTERNPATH_OFFSET,
        ids::VECTOR_PATTERNPATH_OFFSET_NUM,
        0.5,
        0.0,
        (2.0 * crate::OFFSET_MAX) as f32,
        -crate::OFFSET_MAX as f32,
    );
    store.set_number_range(
        ids::VECTOR_PATTERNPATH_OFFSET_NUM,
        -crate::OFFSET_MAX,
        crate::OFFSET_MAX,
        PATTERNPATH_SPACING_STEP,
    );
}
