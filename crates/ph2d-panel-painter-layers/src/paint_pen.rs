//! O cartão **Pen** — a MÃO do pincel nos quatro meios (L2, `docs/Painter/Pinceis/03_MOTORES.md` §4): a
//! pressão (tamanho, opacidade, as curvas, o piso e o tecto da mão, a suavização, a corda) e a inclinação
//! (rotação, ângulos, a pegada deitada, o grão alisado, o atraso, a opacidade). Uma linha por
//! [`ph2d_tool_painter::LinhaDaMao`]; o MESMO cartão em todo meio — no Wet Paint as linhas são os knobs da
//! caneta do painel Tuning, nos outros os campos do pincel (a lei única é a do tool, `mao.rs`).
//!
//! ⛔ Só se pintam as linhas que o meio LÊ (`LinhaDaMao::lida_por`): a Watercolor não lê a opacidade do
//! dab (a Strength e a pressão nela não são da lavagem, dono 12/08), o Wet Paint não tem a
//! inclinação→opacidade. Uma linha pintada que não age seria um controlo que mente.

use crate::paint_brush_top::{end_fold, paint_collapsible_section};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::{BrushSettings, LINHAS_DA_MAO, PaintMedia, ids};
use std::cell::Cell;

thread_local! {
    /// O menu da rotação aberto neste quadro (o chip e a opção de agora), pintado por cima do corpo.
    static PENDING_ROTATION_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };
}

/// O nome de cada opção do menu da rotação (`DabRotation`, pelo índice do knob).
const ROTACOES: [&str; 3] = [
    "panel.painter_layers.pen.rotation_none",
    "panel.painter_layers.pen.rotation_follow",
    "panel.painter_layers.pen.rotation_tilt",
];

/// O rótulo de cada linha, pela ordem de [`LINHAS_DA_MAO`] (as palavras do painel Tuning do Wet Paint).
pub(crate) const ROTULOS: [&str; 23] = [
    "panel.painter_layers.pen.pressure_size",
    "panel.painter_layers.pen.pressure_size_curve",
    "panel.painter_layers.pen.pressure_opacity",
    "panel.painter_layers.pen.pressure_opacity_curve",
    "panel.painter_layers.pen.pressure_floor",
    "panel.painter_layers.pen.pressure_ceiling",
    "panel.painter_layers.pen.pressure_smoothing",
    "panel.painter_layers.pen.pulled_string",
    "panel.painter_layers.pen.dab_rotation",
    "panel.painter_layers.pen.tilt_start",
    "panel.painter_layers.pen.tilt_full",
    "panel.painter_layers.pen.tilt_width",
    "panel.painter_layers.pen.tilt_length",
    "panel.painter_layers.pen.tilt_smooth",
    "panel.painter_layers.pen.tilt_lag",
    "panel.painter_layers.pen.tilt_opacity",
    "panel.painter_layers.pen.speed_size",
    "panel.painter_layers.pen.speed_opacity",
    "panel.painter_layers.pen.pressure_s_curve",
    "panel.painter_layers.pen.pressure_jitter",
    "panel.painter_layers.pen.speed_lag",
    "panel.painter_layers.pen.speed_hardness",
    "panel.painter_layers.pen.pressure_hardness",
];

/// As casas decimais de uma linha pelo passo dela (`1` → inteiros; `0,5` → uma; `0,01` → duas).
fn casas(passo: f64) -> usize {
    (-passo.log10()).ceil().max(0.0) as usize
}

/// Pinta o cartão Pen; devolve o `y` seguinte.
pub(crate) fn paint_pen_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, fold) = paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.pen.title"),
        ids::PAINTER_PEN_SECTION,
        ids::PAINTER_PEN_RESET,
    );
    let Some(fold) = fold else {
        return y;
    };
    let meio = PaintMedia::from_u8(brush.media);
    let sec = crate::seccoes::seccao_da_chave(ctx.text_system, ROTULOS[0]);
    for (i, l) in LINHAS_DA_MAO.into_iter().enumerate() {
        if !l.lida(meio, brush.watercolor_shape_auto) {
            continue;
        }
        if ids::PAINTER_PEN_FIELDS[i] == ids::PAINTER_PEN_ROTATION {
            y = paint_rotation_row(ctx, theme, x, content_w, y, brush.mao[i]);
            continue;
        }
        let (lo, hi, passo) = l.faixa();
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr(ROTULOS[i]),
            ids::PAINTER_PEN_FIELDS[i],
            brush.mao[i] as f32,
            lo as f32,
            hi as f32,
            passo,
            casas(passo),
            sec,
        );
    }
    end_fold(ctx, fold, y)
}

/// A rotação do carimbo como um menu (o índice do knob: 0 None · 1 Follow · 2 Pen tilt).
fn paint_rotation_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    valor: f64,
) -> f32 {
    let cur = (valor.round() as u8).min(ids::PAINTER_PEN_ROTATION_COUNT - 1);
    let (ny, open) = crate::paint_brush_rows::paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.pen.dab_rotation",
        ids::PAINTER_PEN_ROTATION,
        cur,
        tr(ROTACOES[usize::from(cur)]),
    );
    if let Some(r) = open {
        PENDING_ROTATION_DD.with(|c| c.set(Some((r, cur))));
    }
    ny
}

/// O menu da rotação, se aberto — pintado por cima do corpo, depois do clip (como os outros).
pub(crate) fn paint_pen_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    let Some((r, cur)) = PENDING_ROTATION_DD.with(Cell::take) else {
        return;
    };
    let opcoes = (0..ids::PAINTER_PEN_ROTATION_COUNT)
        .map(|i| {
            DropdownOption::new(
                ids::painter_pen_rotation_option_id(i),
                i,
                tr(ROTACOES[usize::from(i)]),
            )
        })
        .collect();
    crate::dropdown_popover::paint_dropdown_popover(
        ctx,
        theme,
        ids::PAINTER_PEN_ROTATION,
        opcoes,
        r,
        cur,
    );
}

/// Uma opção do menu da rotação → o índice dela.
pub(crate) fn decode_rotation_option(id: ph2d_a11y::NodeId) -> Option<u8> {
    (0..ids::PAINTER_PEN_ROTATION_COUNT).find(|&i| ids::painter_pen_rotation_option_id(i) == id)
}
