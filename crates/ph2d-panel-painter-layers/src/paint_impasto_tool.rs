//! The Impasto section's **TOOL** row and the body it dispatches to — the ten operations on the body of
//! the paint, gathered into one list (Enio, 2026-07-19).
//!
//! They used to be three lists in three files, each reachable in exactly one paint mode: the Body card in
//! `Paint`, the Plow row in `Smear`, the Sculpt card in `Sculpt`. The **Lighting** card sat with the
//! first of those, which meant the artist entered Sculpt — the mode whose whole purpose is to shape
//! relief — and lost the controls that make relief visible.
//!
//! Now: one section, the tool list directly under "Adjust Last Stroke", and below it **every
//! configuration card** — Body · Knife · Sculpt · Material · Lighting (Enio, 2026-07-22 smoke:
//! *"faça aparecer todos os cards de configuração Impasto"*; this reversed an earlier selected-tool
//! narrowing that hid the other tools' knobs). Each card edits ITS tool's authored state — alive the
//! moment that tool is picked — and Material writes the three relief slots on the tool side, so no
//! card is a knob editing a slot nothing reads. The radio still says which tool the BRUSH is.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_card,
    paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

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
    let card = Card::new(ph2d_tool_painter::ids::PAINTER_IMPASTO_TOOL).title("TOOL");
    let labels = [
        tr("panel.painter_layers.impasto.deposit"),
        tr("panel.painter_layers.impasto.knife"),
        tr("panel.painter_layers.impasto.smooth"),
        tr("panel.painter_layers.impasto.sharpen"),
        tr("panel.painter_layers.impasto.flatten"),
        tr("panel.painter_layers.impasto.scrape"),
        tr("panel.painter_layers.impasto.fill"),
        tr("panel.painter_layers.impasto.chisel"),
        tr("panel.painter_layers.impasto.layer"),
        tr("panel.painter_layers.impasto.inflate"),
    ];
    let opts: Vec<SegmentedOption> = ph2d_tool_painter::ids::PAINTER_IMPASTO_TOOL_IDS
        .iter()
        .zip(labels)
        .map(|(id, label)| SegmentedOption::new(*id, label))
        .collect();
    let seg = SegmentedAdaptive::new(
        ph2d_tool_painter::ids::PAINTER_IMPASTO_TOOL,
        tr("panel.painter_layers.impasto.tool_group"),
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
    y + card_h + ph2d_tokens::control_gap_px()
}
