//! The Stroke section's multi-shape **OPERATION** card (Overlay / Add / Remove). Split from
//! [`super`](../paint_stroke.rs) for the panel-file LOC cap; mirrors the Selection OPERATION card.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_card,
    paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// Paint the multi-shape **OPERATION** card (Overlay / Add / Remove) — a framed surface with a title, above
/// the Apply / Offset controls. Returns the next `y`. The segmented group inside reflows on narrow panels
/// (measured like the Selection card so nothing clips). `selected` is the op wire value (`0`/`1`/`2`).
pub(super) fn operation_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    let header_h = Spacing::Xl3.px();
    let pad = Spacing::Lg.px();
    let card = Card::new(core_ids::PAINTER_STROKE_OP_CARD).title("OPERATION");
    let seg = SegmentedAdaptive::new(
        core_ids::PAINTER_STROKE_OP,
        "Shape operation",
        vec![
            SegmentedOption::new(core_ids::PAINTER_STROKE_OP_OVERLAY, "Overlay"),
            SegmentedOption::new(core_ids::PAINTER_STROKE_OP_ADD, "Add"),
            SegmentedOption::new(core_ids::PAINTER_STROKE_OP_REMOVE, "Remove"),
        ],
    )
    .selected(selected);
    let probe_body = card.body_rect(Rect::new(x, y, content_w, header_h + pad * 2.0 + ROW_H_PX));
    let seg_h = measure_segmented_adaptive(&seg, probe_body.w, ROW_H_PX, ctx.text_system);
    let card_h = header_h + pad * 2.0 + seg_h;
    let card_rect = Rect::new(x, y, content_w, card_h);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_card(&card, card_rect, scene, text_system, theme);
    }
    let body = card.body_rect(card_rect);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_segmented_adaptive(
            &seg,
            Rect::new(body.x, body.y, body.w, ROW_H_PX),
            scene,
            text_system,
            theme,
            store,
            hit_index,
        );
    }
    y + card_h + Spacing::Sm.px()
}
