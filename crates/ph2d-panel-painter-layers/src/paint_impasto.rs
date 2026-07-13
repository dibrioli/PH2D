//! The brush panel's **Impasto** section — the paint's own thickness, and the light that shows it
//! (`docs/Painter/16_impasto_plano_implementacao.md`).
//!
//! Two cards, because the two halves have different owners: **Body** is per-BRUSH (how *this* brush
//! lays paint down) and **Lighting** is per-CANVAS (one light for the whole document, like the paper
//! colour). Mirrors `paint_watercolor.rs`; the card box itself is the shared [`crate::card`].
//!
//! The section is **not painted at all** in the modes Impasto does not apply to — and a card that is
//! not painted registers no hit, so its ids are inert. The predicate is `brush.impasto_applies`,
//! published by the tool: the panel does not re-derive it (that is how a UI and its engine come to
//! disagree about when a feature is live).

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

// Domain ranges, not design values — these are the physical bounds of the controls (degrees of arc, a
// signed unit depth), so there is no token that could express them.
const DEPTH_MIN: f32 = -1.0; // LITERAL-PX-OK: signed unit depth — negative carves
const DEPTH_MAX: f32 = 1.0; // LITERAL-PX-OK: signed unit depth
pub(crate) const UNIT_MAX: f32 = 1.0; // LITERAL-PX-OK: 0..1 amount track
pub(crate) const ANGLE_MAX_DEG: f32 = 360.0; // LITERAL-PX-OK: a full turn of azimuth
pub(crate) const ELEV_MIN_DEG: f32 = 5.0; // LITERAL-PX-OK: floor above 0 — a grazing light divides by ~0
pub(crate) const ELEV_MAX_DEG: f32 = 90.0; // LITERAL-PX-OK: straight down at the canvas
pub(crate) const DEG_STEP: f64 = 1.0; // LITERAL-PX-OK: whole degrees
/// A lamp may be pushed to twice full — a rig wants headroom for a key that carries the picture while
/// the fills sit under 1. // LITERAL-PX-OK
pub(crate) const LIGHT_POWER_MAX: f32 = 2.0;

/// Paint the Impasto section. Returns the next `y`.
pub(crate) fn paint_impasto_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // The §1.2 matrix: no card, no hit targets, no knob to be confused by. Watercolor is the one that
    // matters most — it is a separate implementation and Impasto must not so much as appear there.
    //
    // The Smear is the exception the matrix always named: it deposits no paint, so it has no Body to
    // configure — but it MOVES paint, so it has a knife. One row, and nothing else.
    if brush.impasto_plow_applies {
        return paint_plow_only(ctx, theme, x, content_w, y, &brush);
    }
    if !brush.impasto_applies {
        return y;
    }
    let (mut y, collapsed) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Impasto",
        core_ids::PAINTER_IMPASTO_SECTION,
        core_ids::PAINTER_IMPASTO_SECTION_COLOR,
        core_ids::PAINTER_IMPASTO_RESET,
    );
    if collapsed {
        return y;
    }

    // Master enable — off (the default) makes a stroke byte-identical to a build with no Impasto.
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_IMPASTO_ENABLE,
        "Enable",
        brush.impasto,
    );
    if !brush.impasto {
        return y;
    }
    y = paint_body_card(ctx, theme, x, content_w, y, &brush);
    paint_lighting_card(ctx, theme, x, content_w, y, &brush)
}

/// Card 1: BODY — how this brush deposits thickness (per-brush).
fn paint_body_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Body", 6);
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Depth",
        core_ids::PAINTER_IMPASTO_DEPTH,
        brush.impasto_depth,
        DEPTH_MIN,
        DEPTH_MAX,
        number_field::FINE_STEP,
        2,
    );
    // Body = the cross-section dial: 1 = level film with a wall, 0 = the relief obeys the falloff
    // (the perfectly rounded ridge Enio asked back for, 2026-07-12).
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Body",
        core_ids::PAINTER_IMPASTO_BODY,
        brush.impasto_body,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    // Push = volume conservation: how much of the paint already on the canvas this brush shoves aside.
    // It sits under Body because it is the other half of the same question — what the paint DOES when the
    // brush arrives: pile up (Body), or get out of the way (Push).
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Push",
        core_ids::PAINTER_IMPASTO_PUSH,
        brush.impasto_push,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Smoothing",
        core_ids::PAINTER_IMPASTO_SMOOTHING,
        brush.impasto_smoothing,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = seg_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        core_ids::PAINTER_IMPASTO_SOURCE,
        "Depth source",
        &[
            (core_ids::PAINTER_IMPASTO_SOURCE_UNIFORM, "Uniform"),
            (core_ids::PAINTER_IMPASTO_SOURCE_GRAIN, "Grain"),
        ],
        brush.impasto_source as usize,
    );
    let _ = seg_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        core_ids::PAINTER_IMPASTO_DRAW_TO,
        "What the brush writes",
        &[
            (core_ids::PAINTER_IMPASTO_DRAW_BOTH, "Both"),
            (core_ids::PAINTER_IMPASTO_DRAW_COLOR, "Color"),
            (core_ids::PAINTER_IMPASTO_DRAW_DEPTH, "Depth"),
        ],
        brush.impasto_draw_to as usize,
    );
    next_y
}

/// Card 2: LIGHTING — one light for the whole canvas (per-document, not per-brush).
fn paint_lighting_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    // No "Amount" row: it was a second gain over the same percept as the brush's Depth (the pair that
    // made the section read as "hard to adjust"). The slope is geometry now — see the light pass.
    // Rows: Show · Light (1 2 3 4) · [Enable] · Angle · Elevation · Intensity+colour · Shine. The
    // selected lamp owns the middle — four lamps, but never more than six rows (`paint_impasto_rig`).
    let rows = if brush.impasto_rig.selected > 0 { 7 } else { 6 };
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Lighting", rows);
    ry = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        core_ids::PAINTER_IMPASTO_SHOW,
        "Show Impasto",
        brush.impasto_show,
    );
    ry = crate::paint_impasto_rig::paint_light_rows(ctx, theme, ix, iw, ry, brush);
    let _ = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Shine",
        core_ids::PAINTER_IMPASTO_SHINE,
        brush.impasto_shine,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    next_y
}

/// [`seg_row`] with OWNED labels — the lamp chips carry an on/off mark, so they cannot be `&'static str`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_row_owned(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    group_id: ph2d_a11y::NodeId,
    a11y: &str,
    options: &[(ph2d_a11y::NodeId, String)],
    selected: usize,
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, label.as_str()))
        .collect();
    let seg = SegmentedAdaptive::new(group_id, a11y, opts).selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + used + Spacing::Xs.px()
}

/// A segmented option group as one card row (mirrors `paint_deform::seg_group`).
#[allow(clippy::too_many_arguments)]
fn seg_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    group_id: ph2d_a11y::NodeId,
    a11y: &str,
    options: &[(ph2d_a11y::NodeId, &str)],
    selected: usize,
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, *label))
        .collect();
    let seg = SegmentedAdaptive::new(group_id, a11y, opts).selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    y + used + Spacing::Xs.px()
}

/// The Smear's whole Impasto surface: **Plow**, and nothing else.
///
/// Deliberately NOT the Body card with rows greyed out. A knife has no Depth, no Draw To and no Depth
/// Source — showing them disabled would be showing the artist four controls to explain why three of
/// them do not apply. (And a dimmed control is cosmetic: it still hit-registers. The rule of the house
/// is that a control which does not apply is not painted.)
fn paint_plow_only(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (mut y, collapsed) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Impasto",
        core_ids::PAINTER_IMPASTO_SECTION,
        core_ids::PAINTER_IMPASTO_SECTION_COLOR,
        core_ids::PAINTER_IMPASTO_RESET,
    );
    if collapsed {
        return y;
    }
    let (ix, iw, ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Knife", 1);
    let _ = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Plow",
        core_ids::PAINTER_IMPASTO_PLOW,
        brush.impasto_plow,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    y = next_y;
    y
}
