//! The **Wet Paint** section (ADR-0134 — the fluid engine as a paint mode): sibling of
//! `paint_watercolor.rs`. Today it carries only the **Enable** master checkbox — the persistent
//! ARM that makes the Brush paint WET and survives tool round-trips (eraser / selection and back;
//! Enio 2026-07-21), exactly like the Watercolor and Impasto enables. The W3 knob curation lands
//! its ~6 sliders here.

use crate::PaintCtx;
use ph2d_editor_core::ids as core_ids;
use ph2d_tool_painter::BrushSettings;

pub(crate) fn paint_wetpaint_section(
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
        "Wet Paint",
        core_ids::PAINTER_WETPAINT_SECTION,
        core_ids::PAINTER_WETPAINT_SECTION_COLOR,
        core_ids::PAINTER_WETPAINT_RESET,
    );
    if collapsed {
        return y;
    }
    // Master enable — the ARM. Off (default) leaves every stroke byte-identical to a plain
    // brush; on, the Brush IS the fluid until unchecked (leaving to another tool keeps it).
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_WETPAINT_ENABLE,
        "Enable",
        brush.wetpaint,
    );
    y
}
