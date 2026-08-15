//! **Os primitivos de ROW que o card Brush empresta aos outros** — o rótulo de coluna fixa, a row
//! `rótulo + chip` de dropdown, e o chip nu.
//!
//! ⚠️ **Eles moram aqui porque têm SETE consumidores** (Stroke, Shape, Texture, Paper, Ramp, Line,
//! e o próprio Brush): enquanto viviam dentro do `paint_brush.rs` o arquivo crescia por causa de
//! quem o importava, não de quem ele é — e foi assim que ele cruzou o teto de LOC. O corte é por
//! RESPONSABILIDADE: *a seção Brush* de um lado, *as peças de linha que qualquer seção usa* do
//! outro.

use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// A coluna do rótulo de uma row — o número que faz o chip começar todos no mesmo `x`.
///
/// ⚠️ Ele mora AQUI e não no `paint_brush.rs` porque é propriedade da ROW, e a row tem sete
/// consumidores; guardá-lo no card que por acaso a inventou é como nasce a segunda cópia (o
/// `paint_watercolor_paper.rs` já carrega uma, com o comentário *"mirrors paint_brush::LABEL_W"*).
pub(crate) const LABEL_W: f32 = 60.0; // LITERAL-PX-OK: coluna do rótulo de uma row de pincel

/// A left-aligned, vertically-centred row label in a `ROW_H_PX` cell.
pub(crate) fn label(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    x: f32,
    y: f32,
    font: f32,
) {
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
}

/// Paint a "label + dropdown chip" row. Returns `(next_y, Some(chip_rect))` when
/// the chip is open (the caller stashes the rect into the matching pending slot).
/// `pub(crate)` so the Stroke section reuses it for Method + Jitter Unit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
) -> (f32, Option<Rect>) {
    let gap = Spacing::Sm.px();
    label(ctx, theme, label_txt, x, y, TypeToken::Sm.px());
    let chip_w = (content_w - LABEL_W - gap).max(0.0);
    let rect = Rect::new(x + LABEL_W + gap, y, chip_w, ROW_H_PX);
    let open = paint_dropdown_chip(ctx, theme, id, cur_value, cur_label, rect);
    (y + ROW_H_PX + Spacing::Sm.px(), open.then_some(rect))
}

/// Paint a dropdown chip (registered as a `Dropdown` for the generic open/close
/// dispatch). Returns whether it is open. Shared by the Blend + Falloff chips.
pub(crate) fn paint_dropdown_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
) -> bool {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_value as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );

    let radius = Radius::Sm.px();
    fill_rounded_rect(ctx.scene, rect, radius, resolve(ColorToken::Bg1, theme));
    // ⚠️ **Pela porta do widget, e não por uma quarta cópia da lei.** Este chip é desenhado à
    // mão (não constrói um `Dropdown`) e por isso carregava a sua própria regra de borda — que
    // não conhecia `BorderEmph` e portanto **nunca acendia sob o ponteiro**.
    let (dd_state, dd_t) = ctx.host.store().dropdown_visual(id);
    stroke_rounded_rect(
        ctx.scene,
        rect,
        radius,
        StrokeToken::Default.px(),
        ph2d_editor_core::widget::chip_border_color(dd_state, dd_t, theme),
    );

    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        cur_label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(ColorToken::Text1, theme),
    );

    ctx.host.hit_index_mut().register(id, rect);
    open
}
