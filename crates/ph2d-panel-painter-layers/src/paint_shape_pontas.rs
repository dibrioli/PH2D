//! As PONTAS do traço do Wet Paint na secção **Shape**: a ponta reta do carimbo (Square tip), as pontas RECTAS do
//! traço (Flat ends · Ends beyond) e as que ESBATEM ao comprido (Ends fade). Só o pincel da água as pousa
//! ([`ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado`]). Saíram de `paint_shape.rs` pelo tecto de LOC
//! (só mover, 29/10).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;

/// As linhas das pontas do Wet Paint, abaixo do gizmo que desenha a pegada. Devolve o próximo `y`.
pub(crate) fn paint_pontas_do_molhado(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: &BrushSettings,
) -> f32 {
    if !(brush.wetpaint && brush.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado)) {
        return y;
    }
    let sec = crate::paint_shape::seccao_da_forma(ctx);
    y = crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.shape.square_tip"),
        ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_QUINA,
        brush.dab_quina,
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
        sec,
    );
    // As pontas RECTAS do traço (J6): o início e o fim cortados a direito — a fita do pastel, não a meia-lua.
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_RECTAS,
        "panel.painter_layers.shape.flat_ends",
        brush.pontas_rectas,
    );
    // Quanto o corte passa do 1.º e do último centro (em raios): só com as pontas rectas.
    if brush.pontas_rectas {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr("panel.painter_layers.shape.flat_ends_beyond"),
            ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ALEM,
            brush.pontas_alem,
            0.0,
            ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ALEM_MAX,
            crate::number_field::FINE_STEP,
            2,
            sec,
        );
    }
    // As pontas que ESBATEM ao comprido (J6t): o depósito cai para as duas pontas ao longo de uma pegada.
    y = crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.shape.ends_fade"),
        ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ESBATE,
        brush.pontas_esbate,
        0.0,
        ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ESBATE_MAX,
        crate::number_field::FINE_STEP,
        2,
        sec,
    );
    y
}
