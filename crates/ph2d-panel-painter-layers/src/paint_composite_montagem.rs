//! **A MONTAGEM da pilha** — a fileira do `+`, o menu ao lado dele e a lista aberta.
//!
//! Ordem do dono, 2026-09-21: *«a seção nasce sem nenhuma camada. Teremos um botão + para criar
//! camadas (as possibilidades aparecem no dropdown ao lado do +) … as opções vão sumindo do
//! dropdown à medida que vão sendo usadas. Ao usar todas inativa-se o dropdown e o botão +»*.
//!
//! ⚠️ Corte por RESPONSABILIDADE do [`super::paint_composite`] (que bateu `659` linhas contra o
//! tecto de `600`): lá mora *como se pinta uma camada*, aqui *como se monta a pilha*.

use super::paint_composite::{ARROW_W, largura_do_chip_da_operacao, op_name};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
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

    // O menu ao lado dele. Ele é registado como `Dropdown` (o despacho genérico abre/fecha), e a
    // lista aberta é uma passagem DIFERIDA, como o «+ Adjustment» do painel de Layers.
    let kind_id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND;
    let kind_x = x + ARROW_W + gap;
    let kind_w = largura_do_chip_da_operacao(ctx.text_system);
    let kind_rect = Rect::new(kind_x, y, kind_w, ROW_H_PX);
    let aberto = matches!(
        ctx.host.store().get(kind_id),
        Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
    );
    let mut chip = Button::new(kind_id, op_name(brush.composite_add_op));
    if aberto {
        chip.kind = ph2d_editor_core::widget::ButtonKind::Accent;
    }
    paint_button(&chip, kind_rect, ctx.scene, ctx.text_system, theme);
    if pode {
        ctx.host.hit_index_mut().register(kind_id, kind_rect);
    }

    // A frase que diz o que o `+` faz, à direita do menu — sem ela a fileira é dois glifos mudos.
    let font = TypeToken::Base.px();
    let texto_x = kind_x + kind_w + gap;
    paint_text(
        ctx.text_system,
        ctx.scene,
        tr(if pode {
            "panel.painter_layers.composite.add_layer"
        } else {
            "panel.painter_layers.composite.stack_full"
        }),
        texto_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        (x + row_w - texto_x).max(0.0),
        resolve(
            if pode {
                ColorToken::Text2
            } else {
                ColorToken::TextDisabled
            },
            theme,
        ),
    );

    if aberto && pode {
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
