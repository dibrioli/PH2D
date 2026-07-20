//! The Impasto section's **TOOL** row and the body it dispatches to — the ten operations on the body of
//! the paint, gathered into one list (Enio, 2026-07-19).
//!
//! They used to be three lists in three files, each reachable in exactly one paint mode: the Body card in
//! `Paint`, the Plow row in `Smear`, the Sculpt card in `Sculpt`. The **Lighting** card sat with the
//! first of those, which meant the artist entered Sculpt — the mode whose whole purpose is to shape
//! relief — and lost the controls that make relief visible.
//!
//! Now: one section, the tool list directly under "Adjust Last Stroke", and below it **only the selected
//! tool's properties**. Material and Lighting live at the foot of the section because they belong to
//! every tool, not to any one of them — they are the paint and the room.
//!
//! ## What each tool shows, and why the others are not painted
//!
//! | tool | its knobs |
//! |---|---|
//! | **Deposit** | Enable, then the Body card (Depth · Body · Push · Smoothing · Depth source · Draw to) |
//! | **Knife** | Plow, and nothing else — a knife has no Depth, because it lays nothing down |
//! | the eight **sculpt verbs** | whatever that verb uses (see [`crate::paint_sculpt`]) |
//!
//! A knob that does nothing to the tool in your hand is a knob that lies about what the tool can do, and
//! this section has already cost a smoke over exactly that. The rule is the house rule: a control that
//! does not apply is **not painted** (a dimmed one still hit-registers, so dimming is cosmetic).

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_card,
    paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

/// Wire index of the Deposit tool — mirrors `ph2d_tool_painter`'s `IMPASTO_TOOL_DEPOSIT`.
const TOOL_DEPOSIT: u8 = 0;
/// Wire index of the Knife — mirrors `IMPASTO_TOOL_KNIFE`.
const TOOL_KNIFE: u8 = 1;

/// The **Tool** card: ten chips, reflowing onto extra rows on a narrow panel.
///
/// The card grows to fit its reflowed content rather than being sized by a fixed row count — a card sized
/// by a guessed height is how the next section quietly paints over these chips and kills their hit
/// targets (`tests/seam_impasto_rig.rs::no_impasto_widget_loses_its_hit_to_the_section_below`).
pub(crate) fn paint_tool_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let card = Card::new(core_ids::PAINTER_IMPASTO_TOOL).title("TOOL");
    let labels = [
        "Deposit", "Knife", "Smooth", "Sharpen", "Flatten", "Scrape", "Fill", "Chisel", "Layer",
        "Inflate",
    ];
    let opts: Vec<SegmentedOption> = core_ids::PAINTER_IMPASTO_TOOL_IDS
        .iter()
        .zip(labels)
        .map(|(id, label)| SegmentedOption::new(*id, label))
        .collect();
    let seg = SegmentedAdaptive::new(
        core_ids::PAINTER_IMPASTO_TOOL,
        "Which operation acts on the paint's body",
        opts,
    )
    .selected(brush.impasto_tool as usize);

    let header_h = Spacing::Xl3.px();
    let pad = Spacing::Lg.px();
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

/// The selected tool's properties — and only those. Returns the next `y`.
///
/// Returns a second flag: whether the **Material** card should follow. Material is per-BRUSH and is baked
/// into the canvas with the deposit, so it belongs to the Deposit tool alone — the Knife lays no pigment
/// to have a material, and each sculpt verb writes `h` and only `h`. Painting it under those would be a
/// knob editing a brush slot nothing ever reads. (Lighting is the opposite case and follows in EVERY
/// tool: it is the canvas's, and being unable to light the relief you are shaping was the bug that
/// prompted the whole reorganisation.)
pub(crate) fn paint_tool_body(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> (f32, bool) {
    match brush.impasto_tool {
        // The Deposit's own card is the Body — **Enable** is not here: it is the section's master and is
        // painted at the top by [`crate::paint_impasto`], above everything it governs (Enio, 2026-07-19).
        // Nothing below is reached with it unticked.
        TOOL_DEPOSIT => (
            crate::paint_impasto::paint_body_card(ctx, theme, x, content_w, y, brush),
            true,
        ),
        TOOL_KNIFE => (
            crate::paint_impasto::paint_knife_card(ctx, theme, x, content_w, y, brush),
            false,
        ),
        _ => (
            crate::paint_sculpt::paint_sculpt_rows(ctx, theme, x, content_w, y, *brush),
            false,
        ),
    }
}
