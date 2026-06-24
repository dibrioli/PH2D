//! Brush panel — the reorganized **top basics** (canonical slider-with-chip rows + a checkbox) and
//! the first **collapsible section** ("Randomize Color"), built on the shared Inspector widgets so
//! the Brush panel matches the rest of the app (editable numeric chips, ALL-CAPS section header with
//! a collapse chevron + assignable colour dot). Split from [`crate::paint_brush`] for the LOC cap.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Checkbox, CheckboxValue, SectionHeader, color_circle_hit_rect, paint_checkbox,
    paint_section_header, paint_slider_with_chip,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// One canonical "label · slider · editable chip" row (the Widget-Gallery / Inspector look). The chip
/// is `link_slider_number`-linked to the slider in `populate`, so typing in it forwards as the
/// slider's `ValueChanged` over the existing brush-slider channel. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_slider_chip_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &str,
    slider_id: ph2d_a11y::NodeId,
    chip_id: ph2d_a11y::NodeId,
    value: f32,
) -> f32 {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_slider_with_chip(
        Rect::new(x, y, content_w, ROW_H_PX),
        label,
        value,
        slider_id,
        chip_id,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// A canonical checkbox row (box + label) driven by the brush snapshot. The id stays a `Button` in the
/// store (the click forwards over the existing Click channel — the tool toggles the bool); only the
/// VISUAL is a checkbox. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_checkbox_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    checked: bool,
) -> f32 {
    let value = if checked {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let cb = Checkbox::new(id, label).value(value);
    let rect = Rect::new(x, y, content_w, ROW_H_PX);
    paint_checkbox(&cb, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    y + ROW_H_PX + Spacing::Sm.px()
}

/// Paint a collapsible section header (Inspector pattern: ALL-CAPS label + collapse chevron +
/// assignable colour dot) and register the header (collapse-toggle on click) + the colour dot (opens
/// the picker to assign it). Returns `(next_y, collapsed)` — the caller paints no body when collapsed.
/// Shared by every Brush-panel section (Randomize / Texture / Color Ramp / Stroke / Tiling).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_collapsible_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label: &str,
    section_id: ph2d_a11y::NodeId,
    color_id: ph2d_a11y::NodeId,
) -> (f32, bool) {
    let header_h = TypeToken::Md.px() + Spacing::Md.px();
    let collapsed = ctx.host.store().is_collapsed(section_id);
    let rgba = ctx
        .host
        .store()
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for an unconfigured section dot
    let header = SectionHeader::new(section_id, label)
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, content_w, header_h);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_section_header(&header, header_rect, scene, text_system, theme);
    }
    ctx.host.hit_index_mut().register(section_id, header_rect);
    if let Some(circle) = color_circle_hit_rect(&header, header_rect) {
        ctx.host.hit_index_mut().register(color_id, circle);
    }
    (y + header_h + Spacing::Xs.px(), collapsed)
}

/// Paint the collapsible "Randomize Color" section: ALL-CAPS header + collapse chevron + assignable
/// colour dot (Inspector pattern). Collapsed → just the header. Expanded → Hue / Saturation / Value
/// editable slider rows (the effect activates when any amount > 0; there is no enable toggle).
pub(crate) fn paint_randomize_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let (mut y, collapsed) = paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Randomize Color",
        core_ids::PAINTER_BRUSH_RANDOMIZE_SECTION,
        core_ids::PAINTER_BRUSH_RANDOMIZE_SECTION_COLOR,
    );
    if collapsed {
        return y;
    }
    for (slot, label) in ["Hue", "Saturation", "Value"].into_iter().enumerate() {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            label,
            core_ids::PAINTER_BRUSH_RANDOMIZE_SLIDERS[slot],
            core_ids::PAINTER_BRUSH_RANDOMIZE_CHIPS[slot],
            brush.color_jitter[slot],
        );
    }
    y
}
