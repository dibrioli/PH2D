//! O registo dos widgets do **TEXTURE PATTERN** (plano 33, W5) — par natural do
//! [`super::super::paint_texture_pattern`], que os PINTA.
//!
//! Registar é o que os torna clicáveis: pintar + hit-rect não basta. O gate
//! `architecture_panel_wiring_parity` cobra a correspondência entre os dois ficheiros.
//!
//! ⚠️ **O `scale`/`offset` de cada chip numérico é o MESMO mapa que o `event` aplica ao track do
//! slider** — a fronteira única. Escritos em dois sítios, o número e a barra divergiriam.

use super::{button, slider_chip, slider_chip_int};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// Passo do campo do **Size**, em unidades de mundo. Acompanha o piso da faixa (`0,02`): um passo
/// maior tornaria a ponta fina inalcançável pelo teclado, e *um piso que só o arrasto alcança não é
/// um piso*.
const TEXPAT_SIZE_STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo do **Gap**, em unidades de mundo.
const TEXPAT_GAP_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo do **Angle**, em GRAUS — 1° por tecla é o passo que se autora.
const TEXPAT_ANGLE_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo do **Offset**: o denominador é INTEIRO, então o passo é `1`.
const TEXPAT_DENOM_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento
/// Passo dos campos do **Shift X/Y**, em PERCENTAGEM de uma repetição — 1 % por tecla.
const TEXPAT_SHIFT_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento

/// Os botões + os quatro sliders da secção Pattern. Registados INCONDICIONALMENTE como todos os
/// irmãos — quem decide se o clique é possível é a PINTURA (sem hit-rect não há Click).
pub(super) fn populate_texture_pattern(store: &mut WidgetStore) {
    button(store, ids::VECTOR_TEXPAT_SOURCE);
    button(store, ids::VECTOR_TEXPAT_PICK_SHAPE);
    for i in 0..4 {
        button(store, crate::paint_sections::texture_pattern::tile_id(i));
    }
    for i in 0..3 {
        button(store, crate::paint_sections::texture_pattern::mode_id(i));
    }

    // Size: track `0..1` → `TEXPAT_SIZE_MIN..TEXPAT_SIZE_MAX`.
    let size_span = (crate::TEXPAT_SIZE_MAX - crate::TEXPAT_SIZE_MIN) as f32;
    slider_chip(
        store,
        ids::VECTOR_TEXPAT_SIZE,
        ids::VECTOR_TEXPAT_SIZE_NUM,
        crate::paint_sections::texture_pattern::size_track(1.0),
        1.0,
        size_span,
        crate::TEXPAT_SIZE_MIN as f32,
    );
    store.set_number_range(
        ids::VECTOR_TEXPAT_SIZE_NUM,
        crate::TEXPAT_SIZE_MIN,
        crate::TEXPAT_SIZE_MAX,
        TEXPAT_SIZE_STEP,
    );

    // Gap: BIPOLAR `−TEXPAT_GAP_MAX..+` (o mesmo mapa do Offset do Pattern on Path), `0.5` = zero.
    slider_chip(
        store,
        ids::VECTOR_TEXPAT_GAP,
        ids::VECTOR_TEXPAT_GAP_NUM,
        crate::paint_sections::texture_pattern::gap_track(0.0),
        0.0,
        (2.0 * crate::TEXPAT_GAP_MAX) as f32,
        -crate::TEXPAT_GAP_MAX as f32,
    );
    store.set_number_range(
        ids::VECTOR_TEXPAT_GAP_NUM,
        -crate::TEXPAT_GAP_MAX,
        crate::TEXPAT_GAP_MAX,
        TEXPAT_GAP_STEP,
    );

    // Angle: UNIPOLAR `0..360`.
    slider_chip(
        store,
        ids::VECTOR_TEXPAT_ANGLE,
        ids::VECTOR_TEXPAT_ANGLE_NUM,
        0.0,
        0.0,
        crate::TEXPAT_ANGLE_MAX as f32,
        0.0,
    );
    store.set_number_range(
        ids::VECTOR_TEXPAT_ANGLE_NUM,
        0.0,
        crate::TEXPAT_ANGLE_MAX,
        TEXPAT_ANGLE_STEP,
    );

    // Shift X/Y: UNIPOLAR `0..100 %` de uma repetição. ⚠️ `100` é o mesmo que `0` — a faixa é a
    // periodicidade do reticulado, não um limite de conforto.
    for (sid, nid) in [
        (ids::VECTOR_TEXPAT_SHIFT_X, ids::VECTOR_TEXPAT_SHIFT_X_NUM),
        (ids::VECTOR_TEXPAT_SHIFT_Y, ids::VECTOR_TEXPAT_SHIFT_Y_NUM),
    ] {
        slider_chip(
            store,
            sid,
            nid,
            crate::paint_sections::texture_pattern::shift_track(0.0),
            0.0,
            crate::TEXPAT_SHIFT_MAX as f32,
            0.0,
        );
        store.set_number_range(nid, 0.0, crate::TEXPAT_SHIFT_MAX, TEXPAT_SHIFT_STEP);
    }

    // Offset: o denominador é INTEIRO — `slider_chip_int`, senão o campo aceitaria `1/2,7`.
    let denom_span = (crate::TEXPAT_DENOM_MAX - crate::TEXPAT_DENOM_MIN) as f32;
    slider_chip_int(
        store,
        ids::VECTOR_TEXPAT_OFFSET,
        ids::VECTOR_TEXPAT_OFFSET_NUM,
        crate::paint_sections::texture_pattern::denom_track(2.0),
        2.0,
        denom_span,
        crate::TEXPAT_DENOM_MIN as f32,
    );
    store.set_number_range(
        ids::VECTOR_TEXPAT_OFFSET_NUM,
        crate::TEXPAT_DENOM_MIN,
        crate::TEXPAT_DENOM_MAX,
        TEXPAT_DENOM_STEP,
    );
}
