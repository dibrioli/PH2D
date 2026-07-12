//! The **card** — a titled, bordered box of `label · number-box` rows. The shape the Painter's brush
//! panel states a technique in: Wash, Brush, Water, Impasto, Lighting.
//!
//! Extracted from `paint_watercolor.rs` (a pure move, no behaviour) when Impasto needed the same box.
//! Two sections hand-rolling their own card is how two sections quietly stop looking alike — see
//! [[feedback_ui_source_of_truth_gallery_inspector]].

use crate::number_field;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// Label column of a card row — wider than the shared `paint_num_row`, so descriptive technique names
/// fit without truncation.
pub(crate) const CARD_LABEL_W: f32 = 96.0; // LITERAL-PX-OK: card label column (descriptive names)

/// Draw a titled bordered **card** (the Composite/Clone-card idiom) sized for `n_rows` number rows, and
/// return `(inner_x, inner_w, first_row_y, y_after_card)` — the caller paints the rows into
/// `[inner_x, inner_w]` starting at `first_row_y` with [`card_row`], then continues at `y_after_card`.
pub(crate) fn card_frame(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    title: &str,
    n_rows: usize,
) -> (f32, f32, f32, f32) {
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = font + Spacing::Sm.px();
    let row_adv = ROW_H_PX + Spacing::Xs.px();
    let card_h = pad + title_h + n_rows as f32 * row_adv + pad;
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
    // Card title — a discreet caption in the card's top-left.
    paint_text(
        ctx.text_system,
        ctx.scene,
        title,
        x + pad,
        y + pad,
        font,
        content_w - 2.0 * pad,
        resolve(ColorToken::Text2, theme),
    );
    (
        x + pad,
        content_w - 2.0 * pad,
        y + pad + title_h,
        y + card_h + Spacing::Sm.px(),
    )
}

/// One `label · number-box` row inside a card — the app-standard drag-scrub [`number_field::chip`] with a
/// WIDER label column ([`CARD_LABEL_W`]) than the shared `paint_num_row`, so the descriptive technique
/// names fit. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn card_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &str,
    id: ph2d_a11y::NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) -> f32 {
    let gap = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        label,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        CARD_LABEL_W,
        resolve(ColorToken::Text2, theme),
    );
    let cx = x + CARD_LABEL_W + gap;
    let cw = (x + content_w - cx).max(0.0);
    number_field::chip(
        ctx,
        theme,
        Rect::new(cx, y, cw, ROW_H_PX),
        id,
        value,
        min,
        max,
        step,
        decimals,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}
