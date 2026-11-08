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
    // A ponta que GIRA com inércia (L3): quantos diâmetros de caminho a ponta leva a seguir a curva — só à vista
    // quando a ponta segue o traço (sem isso o campo não age).
    if brush.dab_follows_stroke {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr("panel.painter_layers.shape.rotation_smoothing"),
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_ROTATION_LAG,
            brush.dab_rotation_lag,
            0.0,
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_ROTATION_LAG_MAX,
            crate::number_field::FINE_STEP,
            2,
            sec,
        );
    }
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
        ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ESBATE_MIN,
        ph2d_tool_painter::ids::PAINTER_BRUSH_PONTAS_ESBATE_MAX,
        crate::number_field::FINE_STEP,
        2,
        sec,
    );
    y
}

/// O ponto fixo do spray (dono 08/11) no painel de antes: a caixa «Fixed dab size» e, ligada, o «Dab size (px)».
/// Sempre à vista; esmaecida (a inércia `PontoFixoSemEspalhar`) num pincel que não espalha. Devolve o próximo `y`.
pub(crate) fn paint_dab_fixo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: &BrushSettings,
) -> f32 {
    let sec = crate::paint_shape::seccao_da_forma(ctx);
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_FIXO,
        "panel.painter_layers.shape.fixed_dab",
        brush.dab_fixo,
    );
    if brush.dab_fixo {
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr("panel.painter_layers.shape.dab_px"),
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX,
            brush.dab_px,
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX_MIN,
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX_MAX,
            crate::number_field::SIZE_STEP,
            1,
            sec,
        );
    }
    y
}

/// ⭐ **O ponto fixo** (dono 08/11): a caixa «Fixed dab size» e, ligada, o «Dab size (px)» ao lado. Em todo
/// pincel que lê a Forma; sem espalhar marcas (Count 1, Scatter 0) a inércia `PontoFixoSemEspalhar` esmaece-a.
pub(crate) fn do_dab_fixo(l: &BrushSettings, v: &BrushSettings) -> Vec<crate::cards_linhas::Item> {
    if !l.le(ph2d_tool_painter::LinhaDaFerramenta::Forma) {
        return Vec::new();
    }
    let mut n = vec![crate::cards_linhas::Item::Caixa {
        id: ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_FIXO,
        chave: "panel.painter_layers.shape.fixed_dab",
        on: l.dab_fixo,
    }];
    if l.dab_fixo {
        n.push(crate::cards_linhas::Item::Num(crate::cards_linhas::numero(
            "panel.painter_layers.shape.dab_px",
            ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX,
            v.dab_px,
            (
                ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX_MIN,
                ph2d_tool_painter::ids::PAINTER_BRUSH_DAB_PX_MAX,
            ),
            crate::number_field::SIZE_STEP,
            1,
        )));
    }
    n
}
