//! The **Sculpt** card (`docs/Painter/18_plano_sculpt_relevo.md`, Wave 1).
//!
//! Unlike [`crate::paint_deform`], this section is **additive**: it is painted at the top of the ordinary
//! brush body, not instead of it. The sculpt rides the same dab list the colour does, so the brush's own
//! Size / Spacing / Falloff / Shape / Grain / Symmetry / Tiling / stroke method are what shape the
//! spatula — hiding them would take the tool away and leave the settings for it. What the mode DOES hide
//! is the colour half (`BrushSettings::paints_no_color`), which a brush that lays no pigment cannot use.
//!
//! So the card carries only what the brush cannot already say: **which verb** (Smooth / Sharpen) and
//! **over what scale** (Radius). Strength is deliberately absent — the brush already has one, and a second
//! one competing with it would be a design bug wearing the costume of a feature.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Card, SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_card,
    paint_segmented_adaptive, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::widget::{DEFAULT_CHIP_W, DEFAULT_LABEL_W};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

/// Paint the Sculpt card, returning the next `y`. Only called in Sculpt mode.
///
/// The knob row **swaps with the family**: Radius for Smooth / Sharpen (the blur's own scale), Offset for
/// Flatten / Scrape / Fill (where the fitted plane sits). Never both. Radius has no meaning to a plane verb
/// — the plane is fitted to the brush's own footprint, so the brush's Size already IS its scale — and Offset
/// has none to a blur, which has no plane to offset. Showing the inert one would be a knob that lies, and
/// this card has already paid once for exactly that class of mistake (see `SculptState`).
pub(crate) fn paint_sculpt_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let mut y = mode_card(ctx, theme, x, content_w, y, brush.sculpt_mode as usize);
    y = if brush.sculpt_is_plane {
        offset_row(ctx, theme, x, content_w, y, brush)
    } else {
        radius_row(ctx, theme, x, content_w, y, brush)
    };
    y
}

/// The **Offset** row (plane family). The chip shows **paint-loads** — signed, because the sign is the whole
/// meaning: a negative Offset sinks the plane INTO the paint so Scrape digs, a positive one lifts it so Fill
/// mounds. A bare `0..1` track would hide the one number the artist is steering by.
fn offset_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let loads = brush.sculpt_offset_loads;
    let display = format!("{loads:+.2}");
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, content_w, ROW_H_PX),
        "Offset",
        brush.sculpt_offset,
        f64::from(loads),
        Some(&display),
        core_ids::PAINTER_SCULPT_OFFSET_SLIDER,
        core_ids::PAINTER_SCULPT_OFFSET_CHIP,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// The **Radius** row. The slider rides the usual `0..1` track; the chip shows the number the artist
/// actually cares about — **px** — because "0.50" is not a length and the kernel does not take one.
/// (`link_slider_number_mapped_integer` in `populate_sculpt` maps an edit back onto the track, so typing
/// `12` into the chip lands the slider where 12 px is. Mirrors the Symmetry segment-count row.)
fn radius_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let px = brush.sculpt_radius_px;
    let display = format!("{} px", px as u32);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, content_w, ROW_H_PX),
        "Radius",
        brush.sculpt_radius,
        f64::from(px),
        Some(&display),
        core_ids::PAINTER_SCULPT_RADIUS_SLIDER,
        core_ids::PAINTER_SCULPT_RADIUS_CHIP,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// The **Mode** card (Smooth / Sharpen) — a titled surface whose segmented group reflows onto extra rows
/// on a narrow panel; the card grows to fit. Mirrors `paint_deform::mode_card`. Returns next `y`.
fn mode_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    let header_h = Spacing::Xl3.px();
    let pad = Spacing::Lg.px();
    let card = Card::new(core_ids::PAINTER_SCULPT_CARD).title("SCULPT");
    let seg = SegmentedAdaptive::new(
        core_ids::PAINTER_SCULPT_MODE,
        "Sculpt mode",
        vec![
            SegmentedOption::new(core_ids::PAINTER_SCULPT_MODE_SMOOTH, "Smooth"),
            SegmentedOption::new(core_ids::PAINTER_SCULPT_MODE_SHARPEN, "Sharpen"),
            SegmentedOption::new(core_ids::PAINTER_SCULPT_MODE_FLATTEN, "Flatten"),
            SegmentedOption::new(core_ids::PAINTER_SCULPT_MODE_SCRAPE, "Scrape"),
            SegmentedOption::new(core_ids::PAINTER_SCULPT_MODE_FILL, "Fill"),
        ],
    )
    .selected(selected);
    // Body width is height-independent — probe it, then measure the REFLOWED segmented height so the card
    // grows to fit when the segments wrap on a narrow panel. A card sized by a fixed row count under
    // content that reflows is how the next section quietly paints over these rows and kills them
    // (`tests/seam_impasto_rig.rs::no_impasto_widget_loses_its_hit_to_the_section_below`).
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
