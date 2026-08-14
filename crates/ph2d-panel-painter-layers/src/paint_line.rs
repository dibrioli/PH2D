//! O card **Line** — o `Style` do Alchemy (*"toggle between drawing with a line or solid fill"*) e o
//! dropdown `Type` dos tipos de traço procedural (plano 38 §1, pedido do Enio).
//!
//! Ele mora **imediatamente acima do Composite Brush**, e desenha o próprio fundo com borda, como o
//! irmão [`crate::paint_composite`].
//!
//! ⚠️ **As rows do TIPO só existem com o tipo escolhido** — `None` (o default) não tem parâmetro
//! nenhum, e uma row de `Amount` sob `None` seria um controle que não faz nada. É a mesma lei que a
//! `line/anim` aplicou ao menu de fade: *uma tabela por escopo, e um escopo sem parâmetro não pinta
//! parâmetro*.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Checkbox, CheckboxValue, DropdownOption, paint_checkbox};
use ph2d_editor_core::zones::Rect;

use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken};
use ph2d_tool_painter::{BrushSettings, LineKind};

/// Coluna fixa do rótulo de uma row de parâmetro do tipo (cabe "Amount" na fonte Base).
/// Coluna do readout "3.0" à direita do slider.
/// O nome de cada tipo, para o chip e para as opções do popover.
fn kind_name(k: LineKind) -> &'static str {
    match k {
        LineKind::None => "None",
        LineKind::Speed => "Speed",
    }
}

/// Quantos tipos existem — **derivado da lista abaixo**, nunca um literal, para que um tipo novo
/// apareça no dropdown sem ninguém lembrar de subir um número.
pub(crate) const LINE_KINDS: [LineKind; 2] = [LineKind::None, LineKind::Speed];

/// Os tipos como opções de `Dropdown` (valor = o wire `u8`, rótulo = o nome).
pub(crate) fn line_type_options() -> Vec<DropdownOption<u8>> {
    LINE_KINDS
        .iter()
        .map(|k| {
            let w = k.to_wire();
            DropdownOption::new(core_ids::painter_line_type_option_id(w), w, kind_name(*k))
        })
        .collect()
}

/// Pinta o card **Line** e devolve o próximo `y`.
///
/// ⚠️ **O checkbox `Solid` é oferecido sempre que a ferramenta pinta um traço**, e não por modo — a
/// §5.1 do plano 38 é do Enio (*"Solid é oferecido para todos que forem possíveis"*), e a resposta
/// medida é que **são todos**, porque todo tipo de linha mantém o caminho-base do gesto. Quem
/// responde a pergunta é uma porta única no motor, não este pintor.
pub(crate) fn paint_line_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Xs.px();
    let kind = LineKind::from_wire(brush.line_kind);
    // Rows do TIPO: hoje só o `Speed` tem uma (o `Amount`). A altura é exata — nenhuma row deste card
    // quebra em duas linhas.
    let param_rows = match kind {
        LineKind::None => 0.0,
        LineKind::Speed => 1.0,
    };
    let card_h = pad + ROW_H_PX + gap + ROW_H_PX + param_rows * (gap + ROW_H_PX) + pad;
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
    let mut iy = y + pad;

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
    iy += ROW_H_PX + gap;

    let (ny, open) = crate::paint_brush::paint_dropdown_row(
        ctx,
        theme,
        ix,
        iw,
        iy,
        "Type",
        core_ids::PAINTER_LINE_TYPE,
        brush.line_kind,
        kind_name(kind),
    );
    if let Some(r) = open {
        crate::state::set_pending_line_type_dd(Some((r, brush.line_kind)));
    }
    iy = ny;

    let _ = iy;
    y + card_h + Spacing::Sm.px()
}
