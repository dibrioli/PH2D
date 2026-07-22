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

// Knob ranges/steps: each mirrors the `set_wet_knob_value` clamp in the tool
// (which mirrors SPEC §16's `KNOB_DEFS` slider table) — parameter domains of
// the fluid engine, not design values.
const PIGMENT_MAX: f32 = 2000.0; // LITERAL-PX-OK: wet-paint Pigment-per-dab range bound (SPEC §16)
const PIGMENT_STEP: f64 = 10.0; // LITERAL-PX-OK: wet-paint Pigment slider step (SPEC §16)
const PICKUP_MAX: f32 = 0.2; // LITERAL-PX-OK: wet-paint Pickup range bound (SPEC §16)
const PICKUP_STEP: f64 = 0.001; // LITERAL-PX-OK: wet-paint Pickup slider step (SPEC §16)
const DRY_SPEED_MAX: f32 = 8.0; // LITERAL-PX-OK: wet-paint Evaporation range bound (SPEC §16)
const DRY_SPEED_STEP: f64 = 0.05; // LITERAL-PX-OK: wet-paint Evaporation slider step (SPEC §16)
const EDGE_MAX: f32 = 200.0; // LITERAL-PX-OK: wet-paint Edge-darkening range bound (SPEC §16)
const GRAVITY_MAX: f32 = 0.05; // LITERAL-PX-OK: wet-paint Gravity range bound (SPEC §16)
const GRAVITY_STEP: f64 = 0.0005; // LITERAL-PX-OK: wet-paint Gravity slider step (SPEC §16)

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
                k.pigment() as f32,
                0.0,
                PIGMENT_MAX,
                PIGMENT_STEP,
                0,
            ),
            (
                "Pickup",
                core_ids::PAINTER_WETPAINT_PICKUP,
                k.pickup() as f32,
                0.0,
                PICKUP_MAX,
                PICKUP_STEP,
                3,
            ),
            (
                "Dry Speed",
                core_ids::PAINTER_WETPAINT_DRY_SPEED,
                k.dry_speed() as f32,
                0.0,
                DRY_SPEED_MAX,
                DRY_SPEED_STEP,
                2,
            ),
            (
                "Edge Darkening",
                core_ids::PAINTER_WETPAINT_EDGE,
                k.edge_darkening() as f32,
                0.0,
                EDGE_MAX,
                1.0,
                0,
            ),
            (
                "Gravity",
                core_ids::PAINTER_WETPAINT_GRAVITY,
                k.gravity() as f32,
                0.0,
                GRAVITY_MAX,
                GRAVITY_STEP,
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
