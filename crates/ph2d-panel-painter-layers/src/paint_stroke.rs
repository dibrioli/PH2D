//! The Painter dock's **Stroke** sub-section (clean-room port of Blender's 2D-paint "Stroke"
//! panel): Stroke Method, Spacing, Adjust-Strength, Jitter + Jitter Unit, Dash Ratio/Length,
//! Input Samples, and Stabilize Stroke (+ Radius/Factor when on).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this
//! module only paints them off the published [`BrushSettings`] snapshot and reuses the row/chip
//! helpers from [`crate::paint_brush`]. The slider tracks are `0..1`; the tool maps each onto its
//! real range (the `BRUSH_*_MAX` constants are the single source). Edits forward over the frozen
//! `PanelEvent` channel (drained in [`crate::event`]).

use crate::paint_brush::{ParamRow, paint_dropdown_row, paint_param_row, paint_toggle_row};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_jitter_unit_option_id, painter_brush_stroke_method_option_id,
};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::{
    BRUSH_COUNT_SLIDER_MAX, BRUSH_JITTER_ABS_MAX_PX, BRUSH_SMOOTH_RADIUS_MAX_PX, BrushSettings,
    JitterUnit, StrokeMethod,
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
    let mut y = section_header(ctx, theme, x, content_w, y, "Stroke");

    // ── Method dropdown ──
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

    // ── Spacing (% of diameter) + Adjust Strength for Spacing ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Spacing",
        id: core_ids::PAINTER_BRUSH_SPACING,
        value: brush.spacing,
        readout: &format!("{:.0}%", brush.spacing * 100.0),
    });
    y = paint_toggle_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        "Adjust Strength",
        brush.space_attenuation,
    );

    // ── Jitter (unit-aware track + readout) + Jitter Unit dropdown ──
    let view = brush.jitter_unit == JitterUnit::View.to_u8();
    let (jval, jread) = if view {
        (
            brush.jitter_absolute_px / BRUSH_JITTER_ABS_MAX_PX,
            format!("{:.0}px", brush.jitter_absolute_px),
        )
    } else {
        (brush.jitter, format!("{:.2}", brush.jitter))
    };
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Jitter",
        id: core_ids::PAINTER_BRUSH_JITTER,
        value: jval,
        readout: &jread,
    });
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Unit",
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        brush.jitter_unit,
        jitter_unit_name(brush.jitter_unit),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_brush_jitter_unit_dd(Some((r, brush.jitter_unit)));
    }

    // ── Dash ratio + length ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Dash",
        id: core_ids::PAINTER_BRUSH_DASH_RATIO,
        value: brush.dash_ratio,
        readout: &format!("{:.2}", brush.dash_ratio),
    });
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Length",
        id: core_ids::PAINTER_BRUSH_DASH_LENGTH,
        value: count_to_norm(brush.dash_samples),
        readout: &brush.dash_samples.to_string(),
    });

    // ── Input samples ──
    y = paint_param_row(ParamRow {
        ctx,
        theme,
        x,
        content_w,
        y,
        label: "Samples",
        id: core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        value: count_to_norm(brush.input_samples),
        readout: &brush.input_samples.to_string(),
    });

    // ── Stabilize Stroke (+ Radius / Factor sub-sliders only while on) ──
    y = paint_toggle_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_BRUSH_STABILIZE,
        "Stabilize",
        brush.smooth_stroke,
    );
    if brush.smooth_stroke {
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Radius",
            id: core_ids::PAINTER_BRUSH_STABILIZE_RADIUS,
            value: brush.smooth_radius_px / BRUSH_SMOOTH_RADIUS_MAX_PX,
            readout: &format!("{:.0}", brush.smooth_radius_px),
        });
        y = paint_param_row(ParamRow {
            ctx,
            theme,
            x,
            content_w,
            y,
            label: "Factor",
            id: core_ids::PAINTER_BRUSH_STABILIZE_FACTOR,
            value: brush.smooth_factor,
            readout: &format!("{:.2}", brush.smooth_factor),
        });
    }
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

/// A faint, left-aligned section divider label in a `ROW_H_PX` cell.
fn section_header(
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
    }
}

/// Display name for a jitter-unit wire discriminant.
fn jitter_unit_name(u: u8) -> &'static str {
    match JitterUnit::from_u8(u) {
        JitterUnit::Brush => "Brush",
        JitterUnit::View => "View",
    }
}

/// The 7 stroke methods as dropdown options, in Blender's menu order
/// (Dots, Drag Dot, Space, Airbrush, Anchored, Line, Curve).
fn stroke_method_options() -> Vec<DropdownOption<u8>> {
    [0u8, 4, 3, 1, 2, 5, 6]
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
