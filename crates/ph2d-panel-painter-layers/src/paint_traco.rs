//! **A sub-secção RENDERING do cartão Brush** (recolhida, o nome do Procreate Brush Studio): o modo sobre o vazio
//! (L7, onde o Blend se oferece) e o traço como uma camada (L8 — o Glaze, as orlas molhada e queimada e a largura
//! delas, só no Digital: o plano do traço não existe nos outros meios). Ids `PAINTER_TRACO_FIELDS`, pela ordem; o
//! tool encaminha (`traco_settings`). Cada linha passa pela porta `card_row` (rótulo · caixa de número, a11y pela
//! `number_field`).

use crate::card::card_row;
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids::{PAINTER_BRUSH_RENDERING_SECTION, PAINTER_TRACO_FIELDS};
use ph2d_tool_painter::{BrushSettings, PaintMedia, TRACO_LARGURA_MAX_PX};

/// A sub-secção Rendering: nada se nenhuma das linhas se oferece neste meio. Devolve o `y` seguinte.
pub(crate) fn paint_rendering(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    // O modo sobre o vazio vive onde o Blend vive (`paint_top_basics`); o plano do traço, só no Digital.
    let vazio = !brush.paints_no_color() && !brush.is_mask && !brush.watercolor;
    let plano = PaintMedia::from_u8(brush.media) == PaintMedia::Digital;
    if !vazio && !plano {
        return y;
    }
    let (mut y, fold) = crate::paint_brush_top::paint_sub_section(
        ctx,
        theme,
        x,
        w,
        y,
        tr("panel.painter_layers.brush.rendering"),
        PAINTER_BRUSH_RENDERING_SECTION,
    );
    let Some(fold) = fold else {
        return y;
    };
    if vazio {
        y = paint_modo_sobre_vazio(ctx, theme, x, w, y, brush);
    }
    if plano {
        y = paint_traco(ctx, theme, x, w, y, brush);
    }
    crate::paint_brush_top::end_fold(ctx, fold, y)
}

/// O modo sobre o VAZIO (L7): `0` o de sempre · `1` um modo de cor pousa a cor na camada transparente (a fórmula
/// geral do W3C). Devolve o `y` seguinte.
fn paint_modo_sobre_vazio(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    card_row(
        ctx,
        theme,
        x,
        w,
        y,
        "panel.painter_layers.brush.blend_on_empty",
        PAINTER_TRACO_FIELDS[4],
        if brush.blend_sobre_vazio { 1.0 } else { 0.0 },
        0.0,
        1.0,
        1.0,
        0,
    )
}

/// O traço como uma CAMADA (L8): Glaze (o tecto), Wet edges, Burnt edges e Edge width.
fn paint_traco(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    brush: &BrushSettings,
) -> f32 {
    let t = brush.traco;
    let linhas = [
        ("panel.painter_layers.brush.glaze", t.tecto, 0.0, 1.0),
        ("panel.painter_layers.brush.wet_edges", t.molhada, 0.0, 1.0),
        (
            "panel.painter_layers.brush.burnt_edges",
            t.queimada,
            0.0,
            1.0,
        ),
        (
            "panel.painter_layers.brush.edge_width",
            t.largura,
            1.0,
            TRACO_LARGURA_MAX_PX,
        ),
    ];
    for (i, (chave, v, lo, hi)) in linhas.into_iter().enumerate() {
        // O Glaze vai de 0 (sem tecto) a 1; a largura, de 1 a 48 px.
        let passo = if i == 3 {
            0.5
        } else {
            crate::number_field::FINE_STEP
        };
        y = card_row(
            ctx,
            theme,
            x,
            w,
            y,
            chave,
            PAINTER_TRACO_FIELDS[i],
            safe_clamp(v, lo, hi),
            lo,
            hi,
            passo,
            if i == 3 { 1 } else { 2 },
        );
    }
    y
}
