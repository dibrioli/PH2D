//! **As linhas do TRAÇO no cartão Brush**: o modo sobre o vazio (L7, debaixo do Blend) e o traço como uma camada
//! (L8 — o Glaze, as orlas molhada e queimada e a largura delas, debaixo do Accumulate, só no Digital: o plano do
//! traço não existe nos outros meios). Ids `PAINTER_TRACO_FIELDS`, pela ordem; o tool encaminha (`traco_settings`).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids::PAINTER_TRACO_FIELDS;
use ph2d_tool_painter::{BrushSettings, PaintMedia, TRACO_LARGURA_MAX_PX};

/// A coluna do cartão Brush.
fn seccao(ctx: &mut PaintCtx) -> ph2d_editor_core::property_row::Seccao {
    crate::seccoes::seccao_da_chave(ctx.text_system, "panel.painter_layers.brush.glaze")
}

/// O modo sobre o VAZIO (L7): `0` o de sempre · `1` um modo de cor pousa a cor na camada transparente (a fórmula
/// geral do W3C). Debaixo do Blend, onde o Blend se oferece. Devolve o `y` seguinte.
pub(crate) fn paint_modo_sobre_vazio(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let sec = seccao(ctx);
    crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        w,
        y,
        tr("panel.painter_layers.brush.blend_on_empty"),
        PAINTER_TRACO_FIELDS[4],
        if brush.blend_sobre_vazio { 1.0 } else { 0.0 },
        0.0,
        1.0,
        1.0,
        0,
        sec,
    )
}

/// O traço como uma CAMADA (L8): Glaze (o tecto), Wet edges, Burnt edges e Edge width — só no Digital.
pub(crate) fn paint_traco(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    mut y: f32,
    brush: &BrushSettings,
) -> f32 {
    if PaintMedia::from_u8(brush.media) != PaintMedia::Digital {
        return y;
    }
    let sec = seccao(ctx);
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
    for (i, (rotulo, v, lo, hi)) in linhas.into_iter().enumerate() {
        // O Glaze vai de 0 (sem tecto) a 1; a largura, de 1 a 48 px.
        let passo = if i == 3 {
            0.5
        } else {
            crate::number_field::FINE_STEP
        };
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            w,
            y,
            tr(rotulo),
            PAINTER_TRACO_FIELDS[i],
            v.clamp(lo, hi),
            lo,
            hi,
            passo,
            if i == 3 { 1 } else { 2 },
            sec,
        );
    }
    y
}
