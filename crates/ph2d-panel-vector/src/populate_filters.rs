//! O registro dos widgets do **FILTERS** (FX raster, plano 24) — irmão do [`super`] pelo teto de
//! 600 LOC, e par natural do `paint_filters` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: pintar + hit-rect não basta. O gate
//! `architecture_panel_wiring_parity` cobra a correspondência entre os dois arquivos.

use super::{button, slider_chip};
use crate::ids;
use crate::state::filters::{FILTER_OFFSET_MAX, FILTER_RADIUS_MAX};
use ph2d_editor_core::interaction::WidgetStore;

/// Passo dos campos numéricos, no domínio do documento.
const RADIUS_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const OFFSET_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (mundo)
const OPACITY_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento (0..1)

/// Os quatro chips de tipo + os quatro pares slider/campo. Registrados INCONDICIONALMENTE como
/// todos os irmãos — quem decide se o clique é possível é a PINTURA (sem hit-rect não há Click).
pub(super) fn populate_filters(store: &mut WidgetStore) {
    for id in [
        ids::VECTOR_FILTER_KIND_NONE,
        ids::VECTOR_FILTER_KIND_BLUR,
        ids::VECTOR_FILTER_KIND_GLOW,
        ids::VECTOR_FILTER_KIND_SHADOW,
    ] {
        button(store, id);
    }
    // Radius: track `0..1` → `0..FILTER_RADIUS_MAX`. O `scale`/`offset` do chip é o MESMO mapa que
    // o `event` desfaz na fronteira (`t * MAX`), senão slider e campo divergiriam.
    slider_chip(
        store,
        ids::VECTOR_FILTER_RADIUS,
        ids::VECTOR_FILTER_RADIUS_NUM,
        0.0,
        0.0,
        FILTER_RADIUS_MAX as f32,
        0.0,
    );
    store.set_number_range(ids::VECTOR_FILTER_RADIUS_NUM, 0.0, FILTER_RADIUS_MAX, RADIUS_STEP);
    // Offset X/Y: BIPOLAR `−MAX..MAX`, `0.5` = zero.
    for (slider, chip) in [
        (ids::VECTOR_FILTER_OFFX, ids::VECTOR_FILTER_OFFX_NUM),
        (ids::VECTOR_FILTER_OFFY, ids::VECTOR_FILTER_OFFY_NUM),
    ] {
        slider_chip(
            store,
            slider,
            chip,
            0.5,
            0.0,
            (2.0 * FILTER_OFFSET_MAX) as f32,
            -FILTER_OFFSET_MAX as f32,
        );
        store.set_number_range(chip, -FILTER_OFFSET_MAX, FILTER_OFFSET_MAX, OFFSET_STEP);
    }
    // Opacity: track == valor (`0..1`).
    slider_chip(
        store,
        ids::VECTOR_FILTER_OPACITY,
        ids::VECTOR_FILTER_OPACITY_NUM,
        1.0,
        1.0,
        1.0,
        0.0,
    );
    store.set_number_range(ids::VECTOR_FILTER_OPACITY_NUM, 0.0, 1.0, OPACITY_STEP);
}
