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

use crate::number_field;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Slider range bounds — the parameter domains (matching the tool's `set_brush_*` clamps), not design
/// tokens. The `0..1` params (Charge / Dilution / …) use the allowlisted `0.0`/`1.0` inline.
const EDGE_MAX: f32 = 8.0; // LITERAL-PX-OK: watercolor Edge-darkening gain range bound (parameter domain)
const SPREAD_MIN: f32 = 1.0; // LITERAL-PX-OK: watercolor Bleed blur-radius min (px)
const SPREAD_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Bleed blur-radius max (px; 48 so big brushes aren't capped dry)
const DEPTH_MIN: f32 = 0.1; // LITERAL-PX-OK: Beer–Lambert optical-depth min (parameter domain)
const DEPTH_MAX: f32 = 8.0; // LITERAL-PX-OK: Beer–Lambert optical-depth max (parameter domain)
const WARP_MAX: f32 = 48.0; // LITERAL-PX-OK: watercolor Ragged-Edge displacement max (px; range pair of SPREAD_MAX)

/// Wider label column for the Watercolor cards' descriptive technique names ("Edge Darkening",
/// "Concentration", "Pigment") — the shared `number_field` column (70 px) clips them.
const CARD_LABEL_W: f32 = 96.0; // LITERAL-PX-OK: watercolor card label column (descriptive names)

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

    // Master enable — off (default) makes a stroke byte-identical to a plain brush.
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_WATERCOLOR_ENABLE,
        "Enable",
        brush.watercolor,
    );
    if !brush.watercolor {
        return y;
    }

    // Three technique cards — each paints its rows off the snapshot and returns
    // the next `y`. Split out to keep this fn under the panel LOC cap.
    y = paint_wash_card(ctx, theme, x, content_w, y, &brush);
    y = paint_brush_card(ctx, theme, x, content_w, y, &brush);
    y = paint_water_card(ctx, theme, x, content_w, y, &brush);

    y
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
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Wash", 5);
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
    let _ = card_row(
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

/// Draw a titled bordered **card** (the Composite/Clone-card idiom) sized for `n_rows` number rows, and
/// return `(inner_x, inner_w, first_row_y, y_after_card)` — the caller paints the rows into
/// `[inner_x, inner_w]` starting at `first_row_y` with [`card_row`], then continues at `y_after_card`.
fn card_frame(
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
fn card_row(
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
