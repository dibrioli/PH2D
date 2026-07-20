//! The brush panel's **Impasto** section — the paint's own thickness, the ten tools that shape it, and
//! the light that shows it (`docs/Painter/16_impasto_plano_implementacao.md`).
//!
//! Since 2026-07-19 this is the section's **single home** (Enio: *"os tools de Impasto estão espalhados
//! em 3 lugares … vamos unificar"*). The order down the panel is the order of the questions:
//!
//! 1. **Adjust Last Stroke** — does moving a knob reach the paint already on the canvas? It governs
//!    every slider below, so it belongs to none of the boxes and sits above all of them.
//! 2. **TOOL** — the ten operations on the body of the paint ([`crate::paint_impasto_tool`]).
//! 3. the selected tool's properties, and no others.
//! 4. **Material** — what the paint IS. Per-brush, baked with the deposit, so Deposit-only.
//! 5. **Lighting** — the room. Per-CANVAS, one light for the whole document (like the paper colour), so
//!    it is painted for EVERY tool.
//!
//! The card box itself is the shared [`crate::card`]; mirrors `paint_watercolor.rs`.
//!
//! The section is **not painted at all** in modes that do not touch relief — and a card that is not
//! painted registers no hit, so its ids are inert. The predicate is `brush.impasto_section_applies`,
//! published by the tool: the panel does not re-derive it (that is how a UI and its engine come to
//! disagree about when a feature is live). ⚠️ Note it is NOT `impasto_applies`, which is the narrower
//! *does this brush deposit body?* and now gates the Deposit tool's card alone — conflating the two is
//! precisely what used to hide the Lighting card from Sculpt and the Smear.

use crate::card::{card_frame, card_row};
use crate::number_field;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{
    ColorSwatch, SegmentedAdaptive, SegmentedOption, SwatchSize, SwatchState, paint_color_swatch,
    paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// Width of the Wax-colour swatch — the same square the lamp's colour uses.
const SWATCH_W: f32 = 28.0; // LITERAL-PX-OK: swatch box, sized to the row height
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
    // ONE predicate, and it is the tool's (`impasto_section_applies`): the three modes that act on the
    // body of the paint — Paint (Deposit), Smear (Knife), Sculpt (the eight verbs). It used to be
    // `impasto_applies`, which is `Paint` alone, and that is what left the Lighting card unreachable from
    // the two modes that shape relief without depositing it.
    if !brush.impasto_section_applies {
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

    // **Enable** is the section's master and is painted FIRST, because that is its rank (Enio,
    // 2026-07-19: *"é quem habilita esse modo de pintura"*). It briefly lived inside the Deposit card —
    // the one tool whose engine actually reads it — and that put the switch for the whole subject below
    // the list of things it governs.
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
        // The Lighting card is Impasto's OWN, so it goes with the rest when the section is off (Enio,
        // 2026-07-19: *"o card Lighting é próprio de Impasto. só deve aparecer se impasto estiver ativo"*).
        // This reverses an earlier exemption of mine — I had kept the light reachable with Enable off, on
        // the theory that its controls belong to the canvas rather than the brush; Enio's call is that the
        // whole card is part of the Impasto subject and hides with it.
        //
        // ⚠️ The light PASS is untouched and still runs: `impasto_visible` reads `impasto_show` and
        // whether any relief exists, never `brush.impasto`. So relief already on the canvas stays lit —
        // only its CONTROLS are hidden until Impasto is switched back on. Hiding the card is not the same
        // as putting the light out, and the engine keeps them separate.
        return y;
    }
    // Governs every slider below it, so it sits outside the boxes rather than inside one: a control that
    // reaches across every card does not belong in any of them. Unticked by default (Enio 2026-07-19) —
    // finished paint stays finished.
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_IMPASTO_LIVE_EDIT,
        "Adjust Last Stroke",
        brush.impasto_live_edit,
    );
    // …and directly beneath it, the ten tools (Enio: *"as tools todas devem ser organizadas logo abaixo
    // de Adjust Last Stroke"*), then the properties of the one that is selected.
    y = crate::paint_impasto_tool::paint_tool_card(ctx, theme, x, content_w, y, &brush);
    let (mut y, wants_material) =
        crate::paint_impasto_tool::paint_tool_body(ctx, theme, x, content_w, y, &brush);
    if wants_material {
        y = paint_material_card(ctx, theme, x, content_w, y, &brush);
    }
    // Lighting is last and is painted for EVERY tool: it is the canvas's, not the brush's — one light for
    // the whole document, like the paper colour.
    paint_lighting_card(ctx, theme, x, content_w, y, &brush)
}

/// Card 2: MATERIAL — what the paint IS, as opposed to what shape it has (per-brush).
///
/// It sits between Body and Lighting because that is the order of the question: how thick is the paint
/// (Body) · what is the paint (Material) · what light falls on it (Lighting). The first two are the
/// BRUSH's and are baked into the canvas with the stroke; the third is the ROOM's and stays live and
/// canvas-wide. Shine used to sit in the Lighting card — it never belonged there (Enio, 2026-07-13:
/// *"Shine parece ser a intensidade do brilho mas é global e não por lâmpada"*; it is neither the
/// lamp's nor the canvas's, it is the paint's).
fn paint_material_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (ix, iw, mut ry, next_y) = card_frame(ctx, theme, x, content_w, y, "Material", 4);
    ry = card_row(
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
    // Roughness is the knob that did not exist — the exponent was a constant. It is a different percept
    // from Shine (how BROAD the glint is, not how strong), which is why it is a second row and not a
    // second gain on the first.
    ry = card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "Roughness",
        core_ids::PAINTER_IMPASTO_ROUGHNESS,
        brush.impasto_roughness,
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
        "Metallic",
        core_ids::PAINTER_IMPASTO_METALLIC,
        brush.impasto_metallic,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    // ── Row: Wax + its COLOUR swatch, side by side — the same shape the lamp's Intensity row has, and
    //    for the same reason: the two are one thought ("how much of what light"). Here it reads "how
    //    deep the light goes, and what it picks up on the way" (Enio, 2026-07-13).
    //
    //    The swatch is a FILTER, and its neutral is WHITE. That is not a UI convenience — it is the only
    //    honest way to put this control in a square: a *replacement* tint would have "the paint's own
    //    colour" as its neutral, and that is a value which differs per pixel and cannot be shown.
    let box_w = iw - SWATCH_W - Spacing::Xs.px();
    let after = card_row(
        ctx,
        theme,
        ix,
        box_w,
        ry,
        "Wax",
        core_ids::PAINTER_IMPASTO_WAX,
        brush.impasto_wax,
        0.0,
        UNIT_MAX,
        number_field::FINE_STEP,
        2,
    );
    let sw_id = core_ids::PAINTER_IMPASTO_WAX_COLOR;
    let sr = Rect::new(ix + box_w + Spacing::Xs.px(), ry, SWATCH_W, ROW_H_PX);
    let open = ctx.host.store().picker_target() == Some(sw_id);
    let enc = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    let wc = brush.impasto_wax_color;
    paint_color_swatch(
        &ColorSwatch {
            id: sw_id,
            label: String::new(),
            rgba: [enc(wc[0]), enc(wc[1]), enc(wc[2]), 255],
            state: if open {
                SwatchState::Focused
            } else {
                SwatchState::Normal
            },
            size: SwatchSize::Sm,
        },
        sr,
        ctx.scene,
        theme,
    );
    crate::paint::register_button(ctx.host.store_mut(), sw_id);
    ctx.host.hit_index_mut().register(sw_id, sr);
    // Read-back: the shared picker writes the pick onto the swatch's widget colour; forward it to the
    // tool ONLY when it actually differs, or every frame with the picker open would be an undo step.
    if open
        && let Some(picked) = ctx.host.store().widget_color(sw_id)
        && [enc(wc[0]), enc(wc[1]), enc(wc[2])] != [picked[0], picked[1], picked[2]]
    {
        ctx.host
            .bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                sw_id,
                format!("{},{},{}", picked[0], picked[1], picked[2]),
            )));
    }
    let _ = after;
    next_y
}

/// Card 1: BODY — how this brush deposits thickness (per-brush). Painted for the **Deposit** tool.
pub(crate) fn paint_body_card(
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
    // Rows: Show · Light (1 2 3 4) · [Enable] · Angle · Elevation · Intensity+colour. (Shine LEFT: it
    // is the paint's, not the room's — it lives in the Material card now.)
    let rows = if brush.impasto_rig.selected > 0 { 6 } else { 5 };
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
    let _ = crate::paint_impasto_rig::paint_light_rows(ctx, theme, ix, iw, ry, brush);
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

/// The **Knife**'s card: **Plow**, and nothing else.
///
/// Deliberately NOT the Body card with rows greyed out. A knife has no Depth, no Draw To and no Depth
/// Source — showing them disabled would be showing the artist four controls to explain why three of
/// them do not apply. (And a dimmed control is cosmetic: it still hit-registers. The rule of the house
/// is that a control which does not apply is not painted.)
///
/// Until the tools were unified this function painted its own section header and the Smear got *only*
/// this row — no Material, and no **Lighting**, so the knife could move relief the artist had no way to
/// light. It is a card among the others now.
pub(crate) fn paint_knife_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
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
    next_y
}
