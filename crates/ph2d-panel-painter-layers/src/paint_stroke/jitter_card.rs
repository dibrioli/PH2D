//! **O card JITTER** — o que randomiza uma MARCA, e quantas marcas um ponto do caminho deixa.
//!
//! Filho de [`super`] (a seção Stroke), cortado por ASSUNTO quando o pai cruzou o teto de LOC: a
//! seção responde *como este caminho é percorrido*, este card responde *com que aparência cada
//! marca cai*. As cinco rows (Count / Position / Scale / Spacing / Rotation), a unidade do
//! espalhamento e a conversão da contagem moram aqui porque são a mesma pergunta.

use crate::paint_brush_rows::paint_dropdown_row;
use crate::paint_brush_top::paint_slider_chip_row;
use crate::state;
use ph2d_editor_core::ids::{self as core_ids, painter_brush_jitter_unit_option_id};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{BRUSH_JITTER_ABS_MAX_PX, BrushSettings, JitterUnit, SPRAY_COUNT_MAX};

/// Paint the **Jitter** group inside a decorative rounded-rect card: a titled panel holding the
/// per-dab **Position** scatter (unit-aware) + its Unit, the **Scale** scatter, and (texture only)
/// the **Rotation** scatter — so the three jitter modes read clearly. The card background is drawn
/// first (its height pre-computed from the row count), then the rows on top. Returns the next `y`.
pub(super) fn paint_jitter_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let xs = Spacing::Xs.px();
    let sm = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = ROW_H_PX;
    let inner_w = (content_w - pad * 2.0).max(0.0);
    // Pre-compute the card height. The 4 slider rows (Position / Scale / Spacing / Rotation) use the
    // ADAPTIVE slider-with-chip, which DEMOTES the label to its own row when the card's inner width is
    // narrow — so each slider row is taller then. `slider_with_chip_height` reports that exact height, so
    // the card background grows to contain them (the bug: a fixed `ROW_H` per row overflowed the card and
    // the next section overlapped — Enio 2026-06-29). Unit is a (non-adaptive) dropdown row (ROW_H + Sm).
    let slider_row_h = ph2d_editor_core::widget::slider_with_chip_height(ROW_H_PX, inner_w) + xs;
    // A altura é CONTADA a partir das rows que este card pinta, nunca escolhida; foi um número fixo
    // por row que a fez transbordar em 2026-06-29.
    const SLIDER_ROWS: f32 = 5.0; // LITERAL-PX-OK: Count/Position/Scale/Spacing/Rotation
    let rows_h = slider_row_h * SLIDER_ROWS + (ROW_H_PX + sm);
    let card_h = pad + title_h + rows_h + xs;
    let card = Rect::new(x, y, content_w, card_h);
    fill_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::Bg1, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let inner_x = x + pad;
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Jitter",
        inner_x,
        y + pad + (title_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut iy = y + pad + title_h;
    // **Count** — o SPRAY (plano 38 W5), e ele abre o card de propósito: é o número que transforma
    // as três rows abaixo dele de *tremer* em *espalhar*. Com `1` cada ponto do caminho deixa uma
    // marca (o traço de sempre); acima disso deixa `n`, cada uma sorteando o seu próprio
    // Position/Scale/Rotation — que é por que o spray não traz gêmeos daquelas três.
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Count",
        core_ids::PAINTER_BRUSH_SPRAY_COUNT,
        core_ids::PAINTER_BRUSH_SPRAY_COUNT_CHIP,
        spray_count_track(brush.spray_count),
    );
    // Position: the main per-dab position scatter. The slider track is `0..1` in BOTH units (View maps
    // the absolute px onto the `0..MAX` track); the tool maps it back per the unit.
    let jval = if brush.jitter_unit == JitterUnit::View.to_u8() {
        brush.jitter_absolute_px / BRUSH_JITTER_ABS_MAX_PX
    } else {
        brush.jitter
    };
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Position",
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_CHIP,
        jval,
    );
    // Unit (Brush / View) for the Position scatter.
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Unit",
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        brush.jitter_unit,
        jitter_unit_name(brush.jitter_unit),
    );
    iy = ny;
    if let Some(r) = open {
        state::set_pending_brush_jitter_unit_dd(Some((r, brush.jitter_unit)));
    }
    // Scale: per-dab radius scatter.
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Scale",
        core_ids::PAINTER_BRUSH_JITTER_SCALE,
        core_ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
        brush.jitter_scale,
    );
    // Spacing: per-gap scatter of the dab spacing (always relevant — placement, not appearance).
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Spacing",
        core_ids::PAINTER_BRUSH_JITTER_SPACING,
        core_ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
        brush.jitter_spacing,
    );
    // Rotation: per-dab stamp-rotation scatter (Shape + Grain) — always shown (Enio 2026-06-28). The last
    // row, so its returned `y` is unused (the card height is pre-computed above).
    paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Rotation",
        core_ids::PAINTER_BRUSH_JITTER_ROTATE,
        core_ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
        brush.jitter_rotate,
    );
    let _ = iy;
    y + card_h + Spacing::Sm.px()
}

/// A contagem de marcas do spray na pista `0..1` do slider — a INVERSA exata do mapeamento que o
/// tool aplica (`set_brush_spray_count_norm`).
///
/// ⚠️ **Ela existe porque a pista tem de mostrar o que o tool guardou**, e as duas metades da mesma
/// conversão têm de ser inversas: uma pista derivada por outra fórmula faria o polegar pousar num
/// lugar e o número dizer outro. É a lei do *seed == sample* que esta casa já pagou várias vezes.
pub(super) fn spray_count_track(count: u32) -> f32 {
    let span = (SPRAY_COUNT_MAX - 1).max(1);
    // ⚠️ O marcador tem de estar NA linha do `clamp` (o gate lê a linha, não a de cima) — a mesma
    // cicatriz do `LITERAL-PX-OK`, que já foi reflowada para fora por um `rustfmt`.
    let n = count.clamp(1, SPRAY_COUNT_MAX); // CLAMP-OK: `u32` sem NaN, bordas constantes
    #[allow(clippy::cast_precision_loss)]
    let t = (n - 1) as f32 / span as f32;
    t
}

/// Display name for a jitter-unit wire discriminant.
fn jitter_unit_name(u: u8) -> &'static str {
    match JitterUnit::from_u8(u) {
        JitterUnit::Brush => "Brush",
        JitterUnit::View => "View",
    }
}

/// The two jitter units as dropdown options.
pub(super) fn jitter_unit_options() -> Vec<DropdownOption<u8>> {
    [0u8, 1]
        .into_iter()
        .map(|u| {
            DropdownOption::new(
                painter_brush_jitter_unit_option_id(u),
                u,
                jitter_unit_name(u),
            )
        })
        .collect()
}
