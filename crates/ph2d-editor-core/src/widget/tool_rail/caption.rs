//! **A LEGENDA de um chip** — a palavra curta por cima dele (na fila) ou rodada à esquerda (na
//! coluna), e a porta da legenda solta que a barra do topo também usa.
//!
//! Irmão por tecto de LOC do `paint.rs` (2026-10-02): a tinta do chip fica lá, a da legenda aqui.

use super::*;
use ph2d_i18n::TextKey;

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

/// A legenda em caixa normal nos cartões. Só as palavras; as siglas (`FK`, `CEQ`) ficam como são.
const EM_CAIXA_NORMAL: [(&str, TextKey); 35] = [
    ("MOVE", TextKey::new("chrome.rail.caption.move")),
    ("ROT", TextKey::new("chrome.rail.caption.rot")),
    ("SCALE", TextKey::new("chrome.rail.caption.scale")),
    ("PIVOT", TextKey::new("chrome.rail.caption.pivot")),
    ("SPACE", TextKey::new("chrome.rail.caption.space")),
    ("VIEW", TextKey::new("chrome.rail.caption.view")),
    ("UNDO", TextKey::new("chrome.rail.caption.undo")),
    ("REDO", TextKey::new("chrome.rail.caption.redo")),
    ("FILL", TextKey::new("chrome.rail.caption.fill")),
    ("LINE", TextKey::new("chrome.rail.caption.line")),
    ("PROFILE", TextKey::new("chrome.rail.caption.profile")),
    ("TYPE", TextKey::new("chrome.rail.caption.type")),
    ("OPACITY", TextKey::new("chrome.rail.caption.opacity")),
    ("TRIM", TextKey::new("chrome.rail.caption.trim")),
    ("SQUAR", TextKey::new("chrome.rail.caption.squar")),
    ("BRUSH", TextKey::new("chrome.rail.caption.brush")),
    ("PICK", TextKey::new("chrome.rail.caption.pick")),
    ("BLEND", TextKey::new("chrome.rail.caption.blend")),
    ("BLUR", TextKey::new("chrome.rail.caption.blur")),
    ("CLONE", TextKey::new("chrome.rail.caption.clone")),
    ("SMEAR", TextKey::new("chrome.rail.caption.smear")),
    ("ERASE", TextKey::new("chrome.rail.caption.erase")),
    ("SIZE", TextKey::new("chrome.rail.caption.size")),
    ("MASK", TextKey::new("chrome.rail.caption.mask")),
    ("SHAPE", TextKey::new("chrome.rail.caption.shape")),
    ("TOOL", TextKey::new("chrome.rail.caption.tool")),
    ("WET", TextKey::new("chrome.rail.caption.wet")),
    ("DRY", TextKey::new("chrome.rail.caption.dry")),
    ("MODE", TextKey::new("chrome.rail.caption.mode")),
    ("MIX", TextKey::new("chrome.rail.caption.mix")),
    ("GRID", TextKey::new("chrome.rail.caption.grid")),
    ("EDIT", TextKey::new("chrome.rail.caption.edit")),
    ("CURVE", TextKey::new("chrome.rail.caption.curve")),
    ("DASH", TextKey::new("chrome.rail.caption.dash")),
    ("FREE", TextKey::new("chrome.rail.caption.free")),
];

/// A legenda de `text` em cartões.
pub(super) fn em_caixa_normal(text: &str) -> &str {
    EM_CAIXA_NORMAL
        .iter()
        .find(|(cru, _)| *cru == text)
        .map_or(text, |(_, chave)| chave.tr())
}

/// O corpo do rótulo do cartão, se ele couber em `w`; senão o de sempre, que a fila já garante.
fn fonte_da_legenda(text_system: &mut TextSystem, text: &str, w: f32) -> f32 {
    let cartao = ph2d_card_stack::metrics::LABEL_PX;
    if text_system.prefix_width(text, cartao) <= w {
        cartao
    } else {
        sub_label_font_px()
    }
}

/// A legenda de um chip em cartões: em caixa normal, centrada por cima dele na fila; na coluna, a de
/// sempre.
pub(super) fn paint_sub_label_card(
    barra: crate::widget::cromo_barra::Barra,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rail_left_x: f32,
    chip_rect: Rect,
    axis: RailAxis,
) {
    if axis == RailAxis::Vertical {
        let font = sub_label_font_px();
        let cor = barra.rotulo();
        return paint_sub_label_vertical(
            text_system,
            scene,
            text,
            font,
            rail_left_x,
            chip_rect,
            cor,
        );
    }
    let nome = em_caixa_normal(text);
    let vao = super::entry_gap_px();
    let band = Rect::new(
        chip_rect.x - vao * 0.5,
        chip_rect.y - LABEL_TO_CHIP_GAP_PX - LABEL_VISUAL_EXTENT_PX,
        chip_rect.w + vao,
        LABEL_VISUAL_EXTENT_PX,
    );
    let font = fonte_da_legenda(text_system, nome, band.w);
    paint_caption(text_system, scene, nome, band, font, barra.rotulo());
}

/// A legenda de um CAMPO em cartões: encostada à esquerda da caixa, como o rótulo de um cartão, com a
/// marca de animação à direita dela na mesma linha ([`super::number_with_key_mark`]).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_field_caption(
    barra: crate::widget::cromo_barra::Barra,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    id: NodeId,
    slot: Rect,
    axis: RailAxis,
    rail_left_x: f32,
) {
    if axis == RailAxis::Vertical || text.is_empty() {
        return paint_sub_label_card(barra, text_system, scene, text, rail_left_x, slot, axis);
    }
    let nome = em_caixa_normal(text);
    let marca = super::number_with_key_mark(id, slot).1;
    let w = slot.w - marca.map_or(0.0, |d| d.w);
    let font = fonte_da_legenda(text_system, nome, w);
    let y = slot.y - LABEL_TO_CHIP_GAP_PX - LABEL_VISUAL_EXTENT_PX;
    crate::paint::paint_text_elided(
        text_system,
        scene,
        nome,
        slot.x,
        y + (LABEL_VISUAL_EXTENT_PX - font) * 0.5,
        font,
        w,
        barra.rotulo(),
    );
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
