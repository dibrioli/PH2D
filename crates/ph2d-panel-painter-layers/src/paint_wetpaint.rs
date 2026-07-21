//! The **Wet Paint** section (ADR-0134 — the fluid engine as a paint mode): sibling of
//! `paint_watercolor.rs`. The **Enable** master checkbox — the persistent ARM that makes the
//! Brush paint WET and survives tool round-trips (eraser / selection and back; Enio 2026-07-21),
//! exactly like the Watercolor and Impasto enables — plus the W3 **curated knobs** (SPEC §16's
//! tuning table curated to the seven an artist reaches for), painted only while armed. Each knob
//! is a [`crate::card::card_row`] number chip forwarding as `SetValue` via
//! `PAINTER_WETPAINT_FIELDS` + `number_field::is_param_field`.

use crate::PaintCtx;
use crate::card::card_row;
use crate::number_field;
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
    // The W3 knobs — only while ARMED (a knob for an engine that is not running is a dead
    // control wearing a live one's clothes). Ranges mirror the tool's clamps (SPEC §16 /
    // `KNOB_DEFS` slider ranges); decimals sized to each knob's granularity.
    if brush.wetpaint {
        let k = brush.wet_knobs;
        let rows: [(&str, ph2d_a11y::NodeId, f32, f32, f32, f64, usize); 7] = [
            (
                "Water",
                core_ids::PAINTER_WETPAINT_WATER,
                k.water as f32,
                0.0,
                1.0,
                number_field::FINE_STEP,
                2,
            ),
            (
                "Pigment",
                core_ids::PAINTER_WETPAINT_PIGMENT,
                k.pigment as f32,
                0.0,
                2000.0,
                10.0,
                0,
            ),
            (
                "Pickup",
                core_ids::PAINTER_WETPAINT_PICKUP,
                k.pickup as f32,
                0.0,
                0.2,
                0.001,
                3,
            ),
            (
                "Dry Speed",
                core_ids::PAINTER_WETPAINT_DRY_SPEED,
                k.dry_speed as f32,
                0.0,
                8.0,
                0.05,
                2,
            ),
            (
                "Edge Darkening",
                core_ids::PAINTER_WETPAINT_EDGE,
                k.edge_darkening as f32,
                0.0,
                200.0,
                1.0,
                0,
            ),
            (
                "Gravity",
                core_ids::PAINTER_WETPAINT_GRAVITY,
                k.gravity as f32,
                0.0,
                0.05,
                0.0005,
                4,
            ),
            (
                "Erase Strength",
                core_ids::PAINTER_WETPAINT_ERASE,
                k.erase as f32,
                0.0,
                1.0,
                number_field::FINE_STEP,
                2,
            ),
        ];
        for (label, id, value, min, max, step, decimals) in rows {
            y = card_row(
                ctx, theme, x, content_w, y, label, id, value, min, max, step, decimals,
            );
        }
    }
    y
}
