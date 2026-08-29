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

/// Os botões + os sliders das DUAS secções *Pattern* (plano 35, wave F). Registados
/// INCONDICIONALMENTE como todos os irmãos — quem decide se o clique é possível é a PINTURA (sem
/// hit-rect não há Click).
///
/// ⭐⭐ **Percorre a MESMA lista que a pintura** (`TexPatKnob::ALL` × os slots): um knob novo nasce
/// registado nas duas secções sozinho. ⛔ Uma lista escrita à mão aqui seria a terceira cópia dos
/// controlos, e a que deixa um chip *pintado e MORTO sob o rato* — o defeito que esta casa já
/// pagou com 36 células de física e dez chips do Painter.
/// ⭐⭐⭐ Os sliders da secção **BRUSH** (plano 36, W4).
///
/// ⚠️ **Os mapas são os MESMOS que o `paint` usa para o track e que o `event` aplica** — a fronteira
/// única. Três cópias divergiriam no dia em que uma faixa mudasse, e o sintoma seria a barra e o
/// número a discordarem sob o dedo.
/// Passo dos campos do **Size** e do **Spacing** — os dois são multiplicadores adimensionais, e
/// `0,05` é 5 % por tecla: fino o bastante para afinar, grosso o bastante para atravessar a faixa.
const BRUSH_MULT_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo do **Offset**, em unidades de mundo — acompanha o dos multiplicadores.
const BRUSH_OFFSET_STEP: f64 = 0.05; // LITERAL-PX-OK: passo no domínio do documento
/// Passo do campo da **Rotation**, em GRAUS — 1° por tecla é o passo que se autora, como no padrão.
const BRUSH_ROTATION_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento

pub(super) fn populate_brush(store: &mut WidgetStore) {
    use crate::paint_sections::brush as b;
    for (sid, nid, max, passo) in [
        (
            ids::VECTOR_BRUSH_SCALE,
            ids::VECTOR_BRUSH_SCALE_NUM,
            b::BRUSH_SCALE_MAX,
            BRUSH_MULT_STEP,
        ),
        (
            ids::VECTOR_BRUSH_SPACING,
            ids::VECTOR_BRUSH_SPACING_NUM,
            b::BRUSH_SPACING_MAX,
            BRUSH_MULT_STEP,
        ),
        (
            ids::VECTOR_BRUSH_ROTATION,
            ids::VECTOR_BRUSH_ROTATION_NUM,
            b::BRUSH_ROTATION_MAX,
            BRUSH_ROTATION_STEP,
        ),
    ] {
        slider_chip(store, sid, nid, b::unipolar(1.0, max), 1.0, max as f32, 0.0);
        store.set_number_range(nid, 0.0, max, passo);
    }
    // O **Offset** é BIPOLAR: `0.5` = zero, e o negativo põe a arte do outro lado da linha.
    slider_chip(
        store,
        ids::VECTOR_BRUSH_OFFSET,
        ids::VECTOR_BRUSH_OFFSET_NUM,
        b::bipolar(0.0, b::BRUSH_OFFSET_MAX),
        0.0,
        (2.0 * b::BRUSH_OFFSET_MAX) as f32,
        -b::BRUSH_OFFSET_MAX as f32,
    );
    store.set_number_range(
        ids::VECTOR_BRUSH_OFFSET_NUM,
        -b::BRUSH_OFFSET_MAX,
        b::BRUSH_OFFSET_MAX,
        BRUSH_OFFSET_STEP,
    );
}

pub(super) fn populate_texture_pattern(store: &mut WidgetStore) {
    use crate::ids::TexPatKnob as K;
    for slot in 0..ids::TEXPAT_SLOTS {
        let kid = |k| crate::paint_sections::texture_pattern::kid(slot, k);
        // Os botões: a arte, o picker, o cadeado, os 4 reticulados e as 3 repetições.
        for k in K::ALL {
            if matches!(
                k,
                K::Source | K::PickShape | K::Lock | K::Tile(_) | K::Mode(_)
            ) {
                button(store, kid(k));
            }
        }

        // Width/Height: os DOIS eixos, no mesmo mapa `0..1` -> `TEXPAT_SIZE_MIN..TEXPAT_SIZE_MAX`.
        // ⚠️ A MESMA faixa nos dois: um eixo com faixa própria faria o cadeado (que multiplica os
        // dois pelo mesmo factor) saturar num deles e continuar no outro — e a razão que ele promete
        // preservar quebrava sozinha na ponta do curso.
        let size_span = (crate::TEXPAT_SIZE_MAX - crate::TEXPAT_SIZE_MIN) as f32;
        for (sk, nk) in [(K::Width, K::WidthNum), (K::Height, K::HeightNum)] {
            slider_chip(
                store,
                kid(sk),
                kid(nk),
                crate::paint_sections::texture_pattern::size_track(1.0),
                1.0,
                size_span,
                crate::TEXPAT_SIZE_MIN as f32,
            );
            store.set_number_range(
                kid(nk),
                crate::TEXPAT_SIZE_MIN,
                crate::TEXPAT_SIZE_MAX,
                TEXPAT_SIZE_STEP,
            );
        }

        // Gap: BIPOLAR `-TEXPAT_GAP_MAX..+` (o mesmo mapa do Offset do Pattern on Path), `0.5` = zero.
        slider_chip(
            store,
            kid(K::Gap),
            kid(K::GapNum),
            crate::paint_sections::texture_pattern::gap_track(0.0),
            0.0,
            (2.0 * crate::TEXPAT_GAP_MAX) as f32,
            -crate::TEXPAT_GAP_MAX as f32,
        );
        store.set_number_range(
            kid(K::GapNum),
            -crate::TEXPAT_GAP_MAX,
            crate::TEXPAT_GAP_MAX,
            TEXPAT_GAP_STEP,
        );

        // Angle: UNIPOLAR `0..360`.
        slider_chip(
            store,
            kid(K::Angle),
            kid(K::AngleNum),
            0.0,
            0.0,
            crate::TEXPAT_ANGLE_MAX as f32,
            0.0,
        );
        store.set_number_range(
            kid(K::AngleNum),
            0.0,
            crate::TEXPAT_ANGLE_MAX,
            TEXPAT_ANGLE_STEP,
        );

        // Shift X/Y: UNIPOLAR `0..100 %` de uma repetição. ⚠️ `100` é o mesmo que `0` — a faixa é a
        // periodicidade do reticulado, não um limite de conforto.
        for (sk, nk) in [(K::ShiftX, K::ShiftXNum), (K::ShiftY, K::ShiftYNum)] {
            slider_chip(
                store,
                kid(sk),
                kid(nk),
                crate::paint_sections::texture_pattern::shift_track(0.0),
                0.0,
                crate::TEXPAT_SHIFT_MAX as f32,
                0.0,
            );
            store.set_number_range(kid(nk), 0.0, crate::TEXPAT_SHIFT_MAX, TEXPAT_SHIFT_STEP);
        }

        // Offset: o denominador é INTEIRO — `slider_chip_int`, senão o campo aceitaria `1/2,7`.
        let denom_span = (crate::TEXPAT_DENOM_MAX - crate::TEXPAT_DENOM_MIN) as f32;
        slider_chip_int(
            store,
            kid(K::Offset),
            kid(K::OffsetNum),
            crate::paint_sections::texture_pattern::denom_track(2.0),
            2.0,
            denom_span,
            crate::TEXPAT_DENOM_MIN as f32,
        );
        store.set_number_range(
            kid(K::OffsetNum),
            crate::TEXPAT_DENOM_MIN,
            crate::TEXPAT_DENOM_MAX,
            TEXPAT_DENOM_STEP,
        );
    }
}
