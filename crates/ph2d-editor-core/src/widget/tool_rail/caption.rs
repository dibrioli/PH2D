//! **A LEGENDA de um chip** — a palavra curta por cima dele (na fila) ou rodada à esquerda (na
//! coluna), e a porta da legenda solta que a barra do topo também usa.
//!
//! Irmão por tecto de LOC do `paint.rs` (2026-10-02): a tinta do chip fica lá, a da legenda aqui.

use super::*;

/// **O rótulo, no eixo em que este rail corre** — rodado à esquerda do chip numa coluna, direito
/// por cima dele numa fila.
///
/// ⚠️ Uma função só, e não duas chamadas nos braços: os três braços do pintor pedem o rótulo, e a
/// escolha do eixo repetida três vezes seria três sítios onde um deles fica para trás.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_sub_label(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    rail_left_x: f32,
    chip_rect: Rect,
    axis: RailAxis,
    color: ph2d_vector::Color,
) {
    match axis {
        RailAxis::Vertical => paint_sub_label_vertical(
            text_system,
            scene,
            text,
            font_size,
            rail_left_x,
            chip_rect,
            color,
        ),
        RailAxis::Horizontal => {
            paint_sub_label_above(text_system, scene, text, font_size, chip_rect, color)
        }
    }
}

/// Helper — o rótulo direito, centrado POR CIMA do chip (a fila horizontal).
///
/// ⚠️ A banda que ele ocupa é a mesma [`LABEL_VISUAL_EXTENT_PX`] que a coluna reserva para o
/// rodado — o mesmo número, o mesmo tipo de letra, medido do mesmo sítio. É o que faz a fila e a
/// coluna terem chips do mesmo tamanho.
///
/// ⛔⛔ **A legenda é uma palavra SOLTA, não texto dentro de uma caixa com borda** (2026-10-02):
/// medida pelo `label_budget` ela perdia o respiro de `2·Md` de uma moldura que não existe —
/// `16` dos `36 px` — e saía `M…`, `S…`, `UN…` em toda fonte e tamanho (`9 208` cortes na
/// varredura). A largura dela é o PASSO do chip: ele e o vão até ao vizinho, metade de cada lado.
/// Gate: `nenhuma_legenda_da_fila_de_ferramentas_e_cortada`.
fn paint_sub_label_above(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    chip_rect: Rect,
    color: ph2d_vector::Color,
) {
    if text.is_empty() {
        return;
    }
    let vao = super::entry_gap_px();
    let band = Rect::new(
        chip_rect.x - vao * 0.5,
        chip_rect.y - LABEL_TO_CHIP_GAP_PX - LABEL_VISUAL_EXTENT_PX,
        chip_rect.w + vao,
        LABEL_VISUAL_EXTENT_PX,
    );
    paint_caption(text_system, scene, text, band, font_size, color);
}

/// O tamanho de letra das legendas dos chips — o rail e a barra do topo lêem-no daqui.
#[must_use]
pub fn sub_label_font_px() -> f32 {
    (TypeToken::Xs.px() - 2.0).max(Spacing::Md.px())
}

/// ⭐ **A legenda SOLTA de um chip, centrada na banda que é DELA** — o orçamento é a banda
/// inteira, sem o respiro de moldura do [`crate::paint::label_budget`] (ela não tem moldura). A
/// banda é o passo do chip, nunca mais larga: o vizinho tem a dele.
pub fn paint_caption(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    band: Rect,
    font_size: f32,
    color: ph2d_vector::Color,
) {
    crate::paint::paint_text_centered_com_orcamento(
        text_system,
        scene,
        text,
        band,
        band.w,
        font_size,
        color,
    );
}

/// Helper — paint a short uppercase tag vertically (CCW-rotated)
/// in the column to the LEFT of `chip_rect`.
///
/// Layout decisions (mirror user feedback):
///   - Label hugs the LEFT edge of the rail (`LABEL_LEFT_PAD`).
///   - Label is vertically centered with the chip — we measure the
///     real text width via `prefix_width` and offset the bottom
///     anchor by half that width so the rotated text's midpoint
///     lands on `chip.center_y`.
///   - Gap to the chip is fixed by `LABEL_TO_CHIP_GAP_PX`; the
///     chip's x already accounts for it via `CHIP_X_OFFSET_PX`.
fn paint_sub_label_vertical(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    font_size: f32,
    rail_left_x: f32,
    chip_rect: Rect,
    color: ph2d_vector::Color,
) {
    if text.is_empty() {
        return;
    }
    // Measure the unrotated text width — after 90° CCW this is the
    // text's VERTICAL extent on screen.
    let text_w = text_system.prefix_width(text, font_size);
    let chip_center_y = chip_rect.y + chip_rect.h * 0.5;
    // After rotation, the parley layout's (0, 0) — i.e. our anchor —
    // becomes the BOTTOM-LEFT of the rotated bbox. Center the text
    // vertically on the chip by offsetting from `chip_center_y` by
    // half the rotated extent (= text_w).
    let anchor_x = rail_left_x + LABEL_LEFT_PAD;
    let anchor_y = chip_center_y + text_w * 0.5;
    paint_text_rotated_ccw(
        text_system,
        scene,
        text,
        anchor_x,
        anchor_y,
        font_size,
        chip_rect.h,
        color,
    );
}
