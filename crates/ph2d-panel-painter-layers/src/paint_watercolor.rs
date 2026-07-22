//! The Painter dock's **Watercolor** section — the wet-media look, grouped into technique cards
//! (redesign 2026-07-07, `docs/Painter/12_…`): an **Enable** master toggle gates the section; when on
//! it paints three bordered cards named for what the painter controls —
//! **Wash** (how the stroke dries: Body · Concentration · Edge Darkening · Bleed · Ragged Edge),
//! **Brush** (what's on the brush: Charge · Dilution · Pull), and
//! **Water** (interaction with paint already down: Rewet · Smudge · Pigment). Names mirror the
//! industry vocabulary (Rebelle "Edge Darkening" / "Re-wet", Corel "Concentration", Procreate
//! Charge/Dilution/Pull). The **Pigment** slider is the merged old Pigment-toggle + Mix pair (`0` = off).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module only
//! paints them off the published [`BrushSettings`] snapshot. The number fields forward the real value as
//! `SetValue` (routed via [`ph2d_editor_core::ids::PAINTER_WATERCOLOR_FIELDS`] + `is_param_field`); the
//! Enable toggle + section reset forward as `PanelEvent::Click`.

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

/// Slider range bounds — the parameter domains (matching the tool's `set_brush_*` clamps), not design
/// tokens. The `0..1` params (Charge / Dilution / …) use the allowlisted `0.0`/`1.0` inline.
const EDGE_MAX: f32 = 8.0; // LITERAL-PX-OK: watercolor Edge-darkening gain range bound (parameter domain)
const SPREAD_MIN: f32 = 1.0; // LITERAL-PX-OK: watercolor Bleed blur-radius min (px)
const SPREAD_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Bleed blur-radius max (px; 48 so big brushes aren't capped dry)
const DEPTH_MIN: f32 = 0.1; // LITERAL-PX-OK: Beer–Lambert optical-depth min (parameter domain)
const DEPTH_MAX: f32 = 8.0; // LITERAL-PX-OK: Beer–Lambert optical-depth max (parameter domain)
const WARP_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Ragged-Edge displacement max (px; range pair of SPREAD_MAX)
const DRY_TIME_MIN: f32 = 2.0; // LITERAL-PX-OK: Drying-Time slider min (seconds; matches DRY_TIME_MIN_S clamp)
const DRY_TIME_MAX: f32 = 60.0; // LITERAL-PX-OK: Drying-Time slider max (seconds; matches DRY_TIME_MAX_S clamp)

/// Paint the Watercolor section starting at `y`, returning the next `y`.
pub(crate) fn paint_watercolor_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, collapsed) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Watercolor",
        core_ids::PAINTER_WATERCOLOR_SECTION,
        core_ids::PAINTER_WATERCOLOR_SECTION_COLOR,
        core_ids::PAINTER_WATERCOLOR_RESET,
    );
    if collapsed {
        return y;
    }

    // ⚠️ The master enable is the **Paint Mode** dropdown at the head of the appearance half
    // (2026-07-22): this section is painted only while Watercolor is the selected medium, so a checkbox
    // here would be a second door to the same fact. The guard stays as the belt to that braces —
    // `paint_media` derives the medium from this flag, so they cannot disagree.
    if !brush.watercolor {
        return y;
    }

    // Three technique cards — each paints its rows off the snapshot and returns
    // the next `y`. Split out to keep this fn under the panel LOC cap.
    y = paint_wash_card(ctx, theme, x, content_w, y, &brush);
    y = paint_brush_card(ctx, theme, x, content_w, y, &brush);
    y = paint_water_card(ctx, theme, x, content_w, y, &brush);
    y = paint_wetness_card(ctx, theme, x, content_w, y, &brush);

    y
}

/// Card 4: WETNESS — canvas-level moisture controls (doc 13 #9-#11), NOT brush params: the
/// **Drying Time** slider (how long the paper stays mergeable) + **Dry** (end the wet session now,
/// the bake becomes permanent) / **Wet** (re-moisten the canvas so strokes made now fuse). The
/// slider reads `brush.dry_time_s` (carried in the display snapshot; it maps to the canvas rate).
fn paint_wetness_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Wetness", 3);
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Drying Time",
        core_ids::PAINTER_WATERCOLOR_DRY_TIME,
        brush.dry_time_s,
        DRY_TIME_MIN,
        DRY_TIME_MAX,
        1.0, // whole-second scrub step
        0,   // whole seconds
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Preview",
        core_ids::PAINTER_WATERCOLOR_WET_PREVIEW,
        brush.wet_preview,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let _ = wetness_button_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        &[
            (core_ids::PAINTER_WATERCOLOR_DRY_NOW, "Dry"),
            (core_ids::PAINTER_WATERCOLOR_WET_NOW, "Wet"),
        ],
    );
    next_y
}

/// A row of momentary action buttons (none selected) — the Dry/Wet canvas actions. Mirrors
/// `paint_deform`'s `seg_group` (a `SegmentedAdaptive` reused as a button row); each option forwards
/// a plain `Click` via the `PAINTER_WATERCOLOR_CLICKS` membership in the panel's `event.rs`.
fn wetness_button_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    options: &[(ph2d_a11y::NodeId, &str)],
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, *label))
        .collect();
    let seg = SegmentedAdaptive::new(ph2d_a11y::NodeId(0), "Canvas wetness actions", opts)
        .selected(usize::MAX);
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

/// Card 1: WASH — how the stroke dries (the flat glaze + its dried character).
fn paint_wash_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Wash", 7);
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Body",
        core_ids::PAINTER_WATERCOLOR_FILL,
        brush.fill,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Concentration",
        core_ids::PAINTER_WATERCOLOR_DEPTH,
        brush.depth,
        DEPTH_MIN,
        DEPTH_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Opacity",
        core_ids::PAINTER_WATERCOLOR_OPACITY,
        brush.opacity,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Edge Darkening",
        core_ids::PAINTER_WATERCOLOR_EDGE,
        brush.edge_gain,
        0.0,
        EDGE_MAX,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Bleed",
        core_ids::PAINTER_WATERCOLOR_SPREAD,
        brush.edge_spread,
        SPREAD_MIN,
        SPREAD_MAX,
        number_field::SIZE_STEP,
        1,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Ragged Edge",
        core_ids::PAINTER_WATERCOLOR_WARP,
        brush.warp,
        0.0,
        WARP_MAX,
        number_field::SIZE_STEP,
        1,
    );
    // Smooth Edges (BUGS #16): screen-space AA of the silhouette — the default look; off restores
    // the pre-AA hard/serrated edge as a deliberate style.
    let _ = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        core_ids::PAINTER_WATERCOLOR_SMOOTH_EDGES,
        "Smooth Edges",
        brush.smooth_edges,
    );
    next_y
}

/// Card 2: BRUSH — what's on the brush (the Wet Mix reservoir: pickup, water, carry).
fn paint_brush_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Brush", 3);
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Charge",
        core_ids::PAINTER_WATERCOLOR_CHARGE,
        brush.wet_charge,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Dilution",
        core_ids::PAINTER_WATERCOLOR_DILUTION,
        brush.wet_dilution,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let _ = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Pull",
        core_ids::PAINTER_WATERCOLOR_PULL,
        brush.wet_pull,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    next_y
}

/// Card 3: WATER — interaction with paint already on the canvas (rewet / smear / subtractive mix).
fn paint_water_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    // The **Pigment** slider is the merged old Pigment-toggle + Mix pair: it shows the mixing amount when
    // the gate is on, else `0` (off); `set_brush_pigment_mixing` flips the gate + remembers the amount.
    let pigment_amt = if brush.pigment {
        brush.pigment_mix
    } else {
        0.0
    };
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Water", 3);
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Rewet",
        core_ids::PAINTER_WATERCOLOR_WET,
        brush.wet_rewet,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Smudge",
        core_ids::PAINTER_WATERCOLOR_SMUDGE,
        brush.wet_smudge,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    let _ = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Pigment",
        core_ids::PAINTER_WATERCOLOR_MIX,
        pigment_amt,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    next_y
}
