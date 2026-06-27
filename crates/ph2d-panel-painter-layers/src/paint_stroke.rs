//! The Painter dock's **Stroke** sub-section (clean-room port of Blender's 2D-paint "Stroke"
//! panel): Stroke Method, Spacing, Adjust-Strength, Jitter + Jitter Unit, Dash Ratio/Length,
//! and Input Samples. (Smoothing is automatic — a Catmull-Rom spline in the engine — so there is
//! no Stabilize toggle.)
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this
//! module only paints them off the published [`BrushSettings`] snapshot and reuses the row/chip
//! helpers from [`crate::paint_brush`]. The slider tracks are `0..1`; the tool maps each onto its
//! real range (the `BRUSH_*_MAX` constants are the single source). Edits forward over the frozen
//! `PanelEvent` channel (drained in [`crate::event`]).

use crate::paint::register_button;
use crate::paint_brush::paint_dropdown_row;
use crate::paint_brush_top::{paint_checkbox_row, paint_slider_chip_row};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_jitter_unit_option_id, painter_brush_stroke_method_option_id,
};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_tool_painter::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_COUNT_SLIDER_MAX,
    BRUSH_JITTER_ABS_MAX_PX, BrushSettings, JitterUnit, StrokeMethod,
};

/// Paint the Stroke section starting at `y`, returning the next `y`. The two dropdowns (Method,
/// Jitter Unit) stash their open rects for the deferred [`paint_stroke_popovers`] pass.
pub(crate) fn paint_stroke_section(
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
        "Stroke",
        core_ids::PAINTER_BRUSH_STROKE_SECTION,
        core_ids::PAINTER_BRUSH_STROKE_SECTION_COLOR,
        core_ids::PAINTER_BRUSH_STROKE_RESET,
    );
    if collapsed {
        return y;
    }

    // Each param is gated by the method that actually consumes it — Blender hides
    // the irrelevant rows rather than showing dead controls. A row that is not
    // painted registers no hit rect this frame, so it can't be clicked; its stored
    // value persists in the WidgetStore, so switching methods round-trips it (DIRETIVA
    // §2: no silent no-ops). The predicate set is locked by `stroke_panel_visibility_
    // matches_blender` in `ph2d-painter-brush`.
    let method = StrokeMethod::from_u8(brush.stroke_method);

    // ── Method dropdown (always) ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Method",
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        brush.stroke_method,
        stroke_method_name(brush.stroke_method),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_stroke_method_dd(Some((r, brush.stroke_method)));
    }

    // ── Apply / Apply & Keep — only the methods with a persistent on-canvas shape editor (Curve /
    //    Free Hand / Circle / Polygon). Apply bakes the open shape and drops the editor; Apply & Keep
    //    bakes but keeps the handles for re-apply/reshape. Two buttons share one row, stacking to a
    //    column when the panel is too narrow (mirrors the ramp controls' responsive split). ──
    if method.has_open_shape() {
        y = paint_apply_row(ctx, theme, x, content_w, y);
    }

    // ── Edge to Edge — only Anchored (the drag-sized stamp spans anchor→cursor) ──
    if method.uses_edge_to_edge() {
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
            "Edge to Edge",
            brush.edge_to_edge,
        );
    }

    // ── Rate (airbrush timer period, seconds) — only Airbrush emits on a timer ──
    if method.uses_rate() {
        let span = BRUSH_AIRBRUSH_RATE_MAX_S - BRUSH_AIRBRUSH_RATE_MIN_S;
        let track = ((brush.airbrush_rate_s - BRUSH_AIRBRUSH_RATE_MIN_S) / span).clamp(0.0, 1.0);
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Rate",
            core_ids::PAINTER_BRUSH_RATE,
            core_ids::PAINTER_BRUSH_RATE_CHIP,
            track,
        );
    }

    // ── Spacing (% of diameter) + Adjust Strength — only the spacing-driven methods ──
    if method.uses_spacing() {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Spacing",
            core_ids::PAINTER_BRUSH_SPACING,
            core_ids::PAINTER_BRUSH_SPACING_CHIP,
            brush.spacing,
        );
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_BRUSH_SPACE_ATTEN,
            "Adjust Strength",
            brush.space_attenuation,
        );
    }
    // (Accumulate moved to the top-of-panel basics as a checkbox — Enio 2026-06-24.)

    // ── Jitter group — Position / Scale / Rotation scatter, inside a decorative card (all but Drag
    //    Dot/Anchored) ──
    if method.allows_jitter() {
        y = paint_jitter_card(ctx, theme, x, content_w, y, brush);
    }

    // ── Dash ratio + length — only the spacing-driven methods ──
    if method.uses_dash() {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Dash",
            core_ids::PAINTER_BRUSH_DASH_RATIO,
            core_ids::PAINTER_BRUSH_DASH_RATIO_CHIP,
            brush.dash_ratio,
        );
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Length",
            core_ids::PAINTER_BRUSH_DASH_LENGTH,
            core_ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            count_to_norm(brush.dash_samples),
        );
    }

    // ── Input samples (always) ──
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Samples",
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES_CHIP,
        count_to_norm(brush.input_samples),
    );

    // ── Stabilizer intensity — the single "how regular" knob (0% = raw path). Only the freehand
    //    Space method runs it; Drag Dot and the per-event/interactive methods place dabs raw. ──
    if method.uses_stabilizer() {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Stabilize",
            core_ids::PAINTER_BRUSH_STABILIZE,
            core_ids::PAINTER_BRUSH_STABILIZE_CHIP,
            brush.stabilizer,
        );
    }

    y
}

/// Paint the **Tiling** section — its own collapsible section (default collapsed), the last one in the
/// Brush body (Enio 2026-06-24). Seamless wrap-around painting (paint past an edge → wraps to the
/// opposite edge) + the Repeat-Image on-canvas 3×3 tile preview.
pub(crate) fn paint_tiling_section(
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
        "Tiling",
        core_ids::PAINTER_BRUSH_TILING_SECTION,
        core_ids::PAINTER_BRUSH_TILING_SECTION_COLOR,
        core_ids::PAINTER_BRUSH_TILING_RESET,
    );
    if collapsed {
        return y;
    }
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_TILING_X,
        "Tiling X",
        brush.tiling[0],
    );
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_TILING_Y,
        "Tiling Y",
        brush.tiling[1],
    );
    // Repeat Image: on-canvas 3×3 tile preview + its per-axis Aspect Ratio (shown when on).
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_REPEAT_IMAGE,
        "Repeat Image",
        brush.repeat_image,
    );
    y
}

/// Deferred paint of the Stroke section's open dropdown popovers (Method + Jitter Unit), drained
/// at the very end of the Brush body so they sit above every row.
pub(crate) fn paint_stroke_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_brush_stroke_method_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            stroke_method_options(),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_brush_jitter_unit_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
            jitter_unit_options(),
            chip_rect,
            cur,
        );
    }
}

/// Paint the **Jitter** group inside a decorative rounded-rect card: a titled panel holding the
/// per-dab **Position** scatter (unit-aware) + its Unit, the **Scale** scatter, and (texture only)
/// the **Rotation** scatter — so the three jitter modes read clearly. The card background is drawn
/// first (its height pre-computed from the row count), then the rows on top. Returns the next `y`.
fn paint_jitter_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let pad = Spacing::Sm.px();
    let xs = Spacing::Xs.px();
    let sm = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let title_h = ROW_H_PX;
    let has_rotation = brush.texture_kind != 0;
    // Pre-compute the card height (each row includes its own trailing spacing). Position + Scale +
    // Spacing are param rows (ROW_H + Xs); Unit is a dropdown row (ROW_H + Sm); Rotation (texture only)
    // is one more param row. The trailing `+ xs` is bottom breathing room.
    let mut rows_h = (ROW_H_PX + xs) + (ROW_H_PX + sm) + (ROW_H_PX + xs) + (ROW_H_PX + xs);
    if has_rotation {
        rows_h += ROW_H_PX + xs;
    }
    let card_h = pad + title_h + rows_h + xs;
    let card = Rect::new(x, y, content_w, card_h);
    fill_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        resolve(ColorToken::Bg1, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        card,
        Radius::Md.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let inner_x = x + pad;
    let inner_w = (content_w - pad * 2.0).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Jitter",
        inner_x,
        y + pad + (title_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut iy = y + pad + title_h;
    // Position: the main per-dab position scatter. The slider track is `0..1` in BOTH units (View maps
    // the absolute px onto the `0..MAX` track); the tool maps it back per the unit.
    let jval = if brush.jitter_unit == JitterUnit::View.to_u8() {
        brush.jitter_absolute_px / BRUSH_JITTER_ABS_MAX_PX
    } else {
        brush.jitter
    };
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Position",
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_CHIP,
        jval,
    );
    // Unit (Brush / View) for the Position scatter.
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Unit",
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        brush.jitter_unit,
        jitter_unit_name(brush.jitter_unit),
    );
    iy = ny;
    if let Some(r) = open {
        state::set_pending_brush_jitter_unit_dd(Some((r, brush.jitter_unit)));
    }
    // Scale: per-dab radius scatter.
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Scale",
        core_ids::PAINTER_BRUSH_JITTER_SCALE,
        core_ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
        brush.jitter_scale,
    );
    // Spacing: per-gap scatter of the dab spacing (always relevant — placement, not appearance).
    iy = paint_slider_chip_row(
        ctx,
        theme,
        inner_x,
        inner_w,
        iy,
        "Spacing",
        core_ids::PAINTER_BRUSH_JITTER_SPACING,
        core_ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
        brush.jitter_spacing,
    );
    // Rotation: per-dab texture-rotation scatter — only meaningful with a texture assigned.
    if has_rotation {
        iy = paint_slider_chip_row(
            ctx,
            theme,
            inner_x,
            inner_w,
            iy,
            "Rotation",
            core_ids::PAINTER_BRUSH_JITTER_ROTATE,
            core_ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
            brush.jitter_rotate,
        );
    }
    let _ = iy;
    y + card_h + Spacing::Sm.px()
}

/// A faint, left-aligned section divider label in a `ROW_H_PX` cell. `pub(crate)` so the Texture
/// section ([`crate::paint_texture`]) reuses the same divider.
pub(crate) fn section_header(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    text: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        content_w,
        resolve(ColorToken::Text3, theme),
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Map a count (`1..=BRUSH_COUNT_SLIDER_MAX`) onto the slider's `0..1` track. Inverse of the
/// tool's `count_from_norm`.
fn count_to_norm(n: u32) -> f32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    (n.saturating_sub(1) as f32 / span).clamp(0.0, 1.0)
}

/// Display name for a stroke-method wire discriminant.
fn stroke_method_name(m: u8) -> &'static str {
    match StrokeMethod::from_u8(m) {
        StrokeMethod::Dots => "Dots",
        StrokeMethod::Airbrush => "Airbrush",
        StrokeMethod::Anchored => "Anchored",
        StrokeMethod::Space => "Space",
        StrokeMethod::DragDot => "Drag Dot",
        StrokeMethod::Line => "Line",
        StrokeMethod::Curve => "Curve",
        StrokeMethod::Circle => "Circle",
        StrokeMethod::Polygon => "Polygon",
        StrokeMethod::FreeHand => "Free Hand",
    }
}

/// Display name for a jitter-unit wire discriminant.
fn jitter_unit_name(u: u8) -> &'static str {
    match JitterUnit::from_u8(u) {
        JitterUnit::Brush => "Brush",
        JitterUnit::View => "View",
    }
}

/// The stroke methods as dropdown options, in Blender's menu order (Dots, Drag Dot, Space, Airbrush,
/// Anchored, Line, Curve), then the PH2D Circle + Polygon + Free Hand shape extensions last.
fn stroke_method_options() -> Vec<DropdownOption<u8>> {
    [0u8, 4, 3, 1, 2, 5, 6, 7, 8, 9]
        .into_iter()
        .map(|m| {
            DropdownOption::new(
                painter_brush_stroke_method_option_id(m),
                m,
                stroke_method_name(m),
            )
        })
        .collect()
}

/// Paint the **Apply / Apply & Keep** button row, returning the next `y`. Both buttons share one row;
/// when the panel is too narrow for two readable buttons side by side, the second wraps to its own row
/// below (so the labels never clip). Mirrors the ramp controls' one-row/wrap split.
fn paint_apply_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    // Minimum readable width for the wider "Apply & Keep" label; below `2·min + gap` we stack.
    let min_w = 96.0;
    let one_row = content_w >= min_w * 2.0 + gap;
    if one_row {
        let w = (content_w - gap) * 0.5;
        apply_button(ctx, theme, Rect::new(x, y, w, ROW_H_PX), "Apply", false);
        apply_button(
            ctx,
            theme,
            Rect::new(x + w + gap, y, w, ROW_H_PX),
            "Apply & Keep",
            true,
        );
        y + ROW_H_PX + Spacing::Xs.px()
    } else {
        apply_button(
            ctx,
            theme,
            Rect::new(x, y, content_w, ROW_H_PX),
            "Apply",
            false,
        );
        let y2 = y + ROW_H_PX + gap;
        apply_button(
            ctx,
            theme,
            Rect::new(x, y2, content_w, ROW_H_PX),
            "Apply & Keep",
            true,
        );
        y2 + ROW_H_PX + Spacing::Xs.px()
    }
}

/// One Apply button: `keep` picks the [`core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP`] id (bake + keep the
/// editor) vs [`core_ids::PAINTER_BRUSH_STROKE_APPLY`] (bake + drop). Registers the Button slot so the
/// dispatch can emit `Click` (the tool routes it in `route_brush_dab_event`).
fn apply_button(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, r: Rect, label: &str, keep: bool) {
    let id = if keep {
        core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP
    } else {
        core_ids::PAINTER_BRUSH_STROKE_APPLY
    };
    fill_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        r,
        Radius::Sm.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    paint_text_centered(
        ctx.text_system,
        ctx.scene,
        label,
        r,
        TypeToken::Base.px(),
        resolve(ColorToken::Text1, theme),
    );
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, r);
}

/// The two jitter units as dropdown options.
fn jitter_unit_options() -> Vec<DropdownOption<u8>> {
    [0u8, 1]
        .into_iter()
        .map(|u| {
            DropdownOption::new(
                painter_brush_jitter_unit_option_id(u),
                u,
                jitter_unit_name(u),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
