//! O card **Line** — o `Style` do Alchemy (*"toggle between drawing with a line or solid fill"*) e,
//! nas waves seguintes, o dropdown dos tipos de traço procedural.
//!
//! Ele mora **imediatamente acima do Composite Brush** (plano 38 §1, pedido do Enio), e desenha o
//! próprio fundo com borda, como o irmão [`crate::paint_composite`].
//!
//! ⚠️ **O dropdown `Type` ainda NÃO é pintado, e é decisão, não pendência esquecida:** ele nasce com
//! `None` e mais nada, e *um dropdown de uma opção só é um controle morto com outra roupa* (plano 38
//! §1, lei 3). Ele chega na W2, junto com o primeiro tipo.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Checkbox, CheckboxValue, paint_checkbox};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken};
use ph2d_tool_painter::BrushSettings;

/// Pinta o card **Line** e devolve o próximo `y`.
///
/// ⚠️ **O checkbox é oferecido sempre que a ferramenta pinta um traço**, e não por modo — a §5.1 do
/// plano 38 é do Enio (*"Solid é oferecido para todos que forem possíveis"*), e a resposta medida é
/// que **são todos**, porque todo tipo de linha mantém o caminho-base do gesto. Quem responde a
/// pergunta é uma porta única no motor, não este pintor.
pub(crate) fn paint_line_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let card_h = pad + ROW_H_PX + pad;
    let card = Rect::new(x, y, content_w, card_h);
    let radius = Radius::Md.px();
    fill_rounded_rect(ctx.scene, card, radius, resolve(ColorToken::Bg1, theme));
    stroke_rounded_rect(
        ctx.scene,
        card,
        radius,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );

    let ix = x + pad;
    let iw = content_w - 2.0 * pad;
    let iy = y + pad;

    let cb = Checkbox::new(core_ids::PAINTER_LINE_SOLID, "Solid").value(if brush.style_solid {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    });
    let cb_rect = Rect::new(ix, iy, iw, ROW_H_PX);
    paint_checkbox(&cb, cb_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(core_ids::PAINTER_LINE_SOLID, cb_rect);

    y + card_h + Spacing::Sm.px()
}
