//! **O roteamento dos SLIDERS do Texture Pattern** (plano 33) — irmão de [`super`] pelo teto de
//! 600 LOC do painel, com o mesmo corte do `event_contour`: um assunto, uma porta.
//!
//! ⚠️⚠️ **Cada mapa aqui é o MESMO que o `populate_texture_pattern` dá ao chip numérico e que o
//! `paint_texture_pattern` usa para o track** — a fronteira única. Três cópias divergiriam no dia
//! em que uma faixa mudasse, e o sintoma seria a barra e o número a discordarem sob o dedo.

use super::forward_track;
use ph2d_editor_core::panel::PanelHostInternal;

/// `Some(consumido)` se `id` é um slider de UMA das duas secções *Pattern*; `None` se não é delas.
///
/// ⭐⭐ **Resolvido pela MESMA porta que os pinta** (`texpat_knob_of`), e não por uma escada de
/// constantes: com duas secções (plano 35, wave F) uma escada por id seria a lista de controlos
/// escrita **quatro** vezes. O slot não entra no mapa de propósito — *a faixa de um knob é a mesma
/// nas duas tintas; o que muda é em QUEM ela escreve*, e isso é da shell.
pub(super) fn texpat_slider_event(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    use crate::ids::TexPatKnob as K;
    let (_slot, knob) = crate::paint_sections::texture_pattern::texpat_knob_of(id)?;
    // ⚠️ Os mapas são os MESMOS que o `populate` dá ao chip numérico e que o `paint` usa para o
    // track — a fronteira única. Três cópias divergiriam no dia em que uma faixa mudasse.
    match knob {
        K::Width | K::Height => Some(forward_track(host, id, 0.5, |t| {
            t.mul_add(
                crate::TEXPAT_SIZE_MAX - crate::TEXPAT_SIZE_MIN,
                crate::TEXPAT_SIZE_MIN,
            )
        })),
        // Gap: BIPOLAR, `0.5` = encostado. Negativo é a SOBREPOSIÇÃO.
        K::Gap => Some(forward_track(host, id, 0.5, |t| {
            t.mul_add(2.0 * crate::TEXPAT_GAP_MAX, -crate::TEXPAT_GAP_MAX)
        })),
        // Angle: UNIPOLAR `0..360` — o repouso é `0`, na PONTA do curso (ao contrário da Rotation
        // do Pattern on Path, que tem um neutro no meio).
        K::Angle => Some(forward_track(host, id, 0.0, |t| {
            t * crate::TEXPAT_ANGLE_MAX
        })),
        // Shift X/Y: UNIPOLAR `0..100 %` de UMA repetição, com o repouso em `0`.
        K::ShiftX | K::ShiftY => Some(forward_track(host, id, 0.0, |t| {
            t * crate::TEXPAT_SHIFT_MAX
        })),
        // Offset: o denominador do desfasamento, INTEIRO — o `round` aqui é o que impede um `1/2,7`.
        K::Offset => Some(forward_track(host, id, 0.0, |t| {
            t.mul_add(
                crate::TEXPAT_DENOM_MAX - crate::TEXPAT_DENOM_MIN,
                crate::TEXPAT_DENOM_MIN,
            )
            .round()
        })),
        _ => None,
    }
}
