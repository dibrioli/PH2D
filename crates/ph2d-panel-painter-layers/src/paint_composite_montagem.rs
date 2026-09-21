//! **A MONTAGEM da pilha** — a fileira do `+`, o menu ao lado dele e a lista aberta.
//!
//! Ordem do dono, 2026-09-21: *«a seção nasce sem nenhuma camada. Teremos um botão + para criar
//! camadas (as possibilidades aparecem no dropdown ao lado do +) … as opções vão sumindo do
//! dropdown à medida que vão sendo usadas. Ao usar todas inativa-se o dropdown e o botão +»*.
//!
//! ⚠️ Corte por RESPONSABILIDADE do [`super::paint_composite`] (que bateu `659` linhas contra o
//! tecto de `600`): lá mora *como se pinta uma camada*, aqui *como se monta a pilha*.

use super::paint_composite::{ARROW_W, op_name};
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

/// **A fileira do `+`** — o botão que cria uma camada e, ao lado, o menu com as operações que
/// ainda têm quota (ordem do dono, 2026-09-21).
///
/// ⚠️ Quais operações o menu oferece **não se calcula aqui**: ele lê
/// `BrushSettings::composite_add_available`, que o motor já resolveu pela porta da quota. *Uma
/// segunda aritmética da quota no painel divergiria no dia em que uma quota mudasse.*
///
/// ⛔ Com a quota toda gasta o `+` e o menu pintam **apagados e inertes** — o mesmo idioma das
/// setas de reordenar nas pontas da lista.
pub(crate) fn paint_add_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    row_w: f32,
    y: f32,
    brush: BrushSettings,
) {
    let gap = Spacing::Xs.px();
    let pode = brush.composite_add_available.iter().any(|&a| a);

    // O `+`, na coluna do número da camada para alinhar com as fileiras de cima.
    let add_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD;
    let add_rect = Rect::new(x, y, ARROW_W, ROW_H_PX);
    crate::paint_rows::paint_reorder_btn(ctx, theme, add_id, add_rect, pode, IconId::Add);

    // ⭐⭐ **O menu ao lado dele é o DROPDOWN da casa** — o mesmo pintor do Blend e do Falloff
    // ([`crate::paint_brush_rows::paint_dropdown_chip_activo`]), com a seta e a moldura do tema.
    //
    // ⛔⛔ **A 1.ª redacção pintou-o com o `paint_button` e o dono devolveu-a**: *«ao lado do +
    // deveria ser um dropdown como eu especifiquei»*. Ele estava REGISTADO como dropdown e abria
    // a lista — o que faltava era a AFORDÂNCIA. *Um controlo que se comporta como um dropdown e
    // se desenha como um botão é um dropdown que ninguém clica.*
    //
    // ⚠️ E ele leva a LARGURA que sobra na fileira, não a do nome mais largo: um dropdown estreito
    // ao lado de um `+` lê-se como um segundo botão.
    let kind_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND;
    let kind_x = x + ARROW_W + gap;
    let kind_rect = Rect::new(kind_x, y, (x + row_w - kind_x).max(0.0), ROW_H_PX);
    let aberto = crate::paint_brush_rows::paint_dropdown_chip_activo(
        ctx,
        theme,
        kind_id,
        brush.composite_add_op,
        op_name(brush.composite_add_op),
        kind_rect,
        pode,
    );

    if aberto {
        crate::state::set_pending_composite_add_menu(Some(kind_rect));
    }
}

/// **A lista aberta do menu do `+`** — uma passagem DIFERIDA, para ela flutuar por cima das
/// fileiras em vez de ficar recortada pelo cartão. Molde: o «+ Adjustment» do painel de Layers.
///
/// ⚠️ **Só as operações com quota entram na lista** — é isso que faz as opções «irem sumindo à
/// medida que vão sendo usadas» (ordem do dono). A quota vem resolvida do motor, em
/// `composite_add_available`.
pub(crate) fn paint_add_menu_popover(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, chip: Rect) {
    let Some(brush) = crate::state::current_brush() else {
        return;
    };
    let opcoes: Vec<ph2d_editor_core::widget::DropdownOption<u8>> = (0
        ..ph2d_tool_painter::N_COMPOSITE_OPS)
        .filter(|&i| brush.composite_add_available[i])
        .map(|i| {
            ph2d_editor_core::widget::DropdownOption::new(
                ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_OPTION[i],
                i as u8,
                op_name(i as u8).to_string(),
            )
        })
        .collect();
    if opcoes.is_empty() {
        return;
    }
    crate::paint_brush::paint_dropdown_popover(
        ctx,
        theme,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        opcoes,
        chip,
        brush.composite_add_op,
    );
}
