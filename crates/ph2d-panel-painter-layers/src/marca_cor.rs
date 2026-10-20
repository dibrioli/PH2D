//! **A cor secundária e a cor pela MÃO** na secção Randomize Color (L11, `docs/Painter/Pinceis/03_MOTORES.md`
//! §4): a amostra da cor secundária (o seletor partilhado, como as do Impasto) e as linhas da tabela da marca
//! do cartão [`CartaoDaMarca::Cor`] (`ph2d_tool_painter::marca`).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{BrushSettings, CartaoDaMarca, ids};

/// Pinta a amostra da secundária e as linhas da cor pela mão; devolve o `y` seguinte.
pub(crate) fn paint_cor_da_mao(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let sw = ids::PAINTER_MARCA_SECONDARY;
    let open = ctx.host.store().picker_target() == Some(sw);
    let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let c = brush.marca_secundaria;
    let rgb = [enc(c[0]), enc(c[1]), enc(c[2])];
    crate::paint::register_button(ctx.host.store_mut(), sw);
    let y = crate::paint_brush_rows::color_row(
        ctx,
        theme,
        x,
        w,
        y,
        "panel.painter_marca.secondary",
        sw,
        rgb,
    );
    // Read-back: o seletor escreve a escolha na cor da amostra; só se encaminha quando MUDA (senão cada quadro
    // com o seletor aberto seria um passo de desfazer).
    if open
        && let Some(p) = ctx.host.store().widget_color(sw)
        && rgb != [p[0], p[1], p[2]]
    {
        ctx.host
            .bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                sw,
                format!("{},{},{}", p[0], p[1], p[2]),
            )));
    }
    let sec = crate::seccoes::seccao_da_chave(
        ctx.text_system,
        crate::paint_stroke::jitter_card::ROTULOS_DA_MARCA[0],
    );
    crate::paint_stroke::jitter_card::paint_linhas_da_marca(
        ctx,
        theme,
        x,
        w,
        y,
        brush,
        CartaoDaMarca::Cor,
        sec,
    )
}
