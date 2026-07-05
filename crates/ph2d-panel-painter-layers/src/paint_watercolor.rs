//! The Painter dock's **Watercolor** section — the wet-media look (edge darkening + granulation +
//! pigment build-up; no fluid sim, `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). A master
//! "Wet edges" checkbox gates the section; the Edge / Spread / Granulation sliders + the Pigment
//! toggle (and its Mix slider) show only while it is on (Blender hides dead controls; DIRETIVA §2).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module
//! only paints them off the published [`BrushSettings`] snapshot, reusing the row helpers. The four
//! sliders carry natural units and forward the real value as `SetValue`; the two toggles + the reset
//! forward as `PanelEvent::Click` (drained in [`crate::event`]).

use crate::paint_brush_top::{paint_checkbox_row, paint_collapsible_section};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_tool_painter::BrushSettings;

/// Paint the Watercolor section starting at `y`, returning the next `y`.
pub(crate) fn paint_watercolor_section(
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
        "Watercolor",
        core_ids::PAINTER_WATERCOLOR_SECTION,
        core_ids::PAINTER_WATERCOLOR_SECTION_COLOR,
        core_ids::PAINTER_WATERCOLOR_RESET,
    );
    if collapsed {
        return y;
    }

    // Master enable — off (default) makes a stroke byte-identical to a plain brush.
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_WATERCOLOR_ENABLE,
        "Wet edges",
        brush.watercolor,
    );

    if brush.watercolor {
        // #1 Edge darkening (the "fringe") — gain + blur radius.
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Edge",
            core_ids::PAINTER_WATERCOLOR_EDGE,
            brush.edge_gain.clamp(0.0, 8.0),
            0.0,
            8.0,
            crate::number_field::FINE_STEP,
            2,
        );
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Spread",
            core_ids::PAINTER_WATERCOLOR_SPREAD,
            brush.edge_spread.clamp(1.0, 24.0),
            1.0,
            24.0,
            crate::number_field::SIZE_STEP,
            1,
        );
        // #2 Granulation — deposit gate into the paper tooth (pair with a Tiled Grain).
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Granulation",
            core_ids::PAINTER_WATERCOLOR_GRANULATION,
            brush.granulation.clamp(0.0, 1.0),
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
        );
        // #3 Pigment — subtractive Kubelka–Munk wet-on-wet mixing + its amount.
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_WATERCOLOR_PIGMENT,
            "Pigment",
            brush.pigment,
        );
        if brush.pigment {
            y = crate::number_field::paint_num_row(
                ctx,
                theme,
                x,
                content_w,
                y,
                "Mix",
                core_ids::PAINTER_WATERCOLOR_MIX,
                brush.pigment_mix.clamp(0.0, 1.0),
                0.0,
                1.0,
                crate::number_field::FINE_STEP,
                2,
            );
        }
    }
    y
}
