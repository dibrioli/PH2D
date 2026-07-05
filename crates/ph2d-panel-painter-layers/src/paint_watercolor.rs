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

/// Slider range bounds — the parameter domains (matching the tool's `set_brush_*` clamps), not design
/// tokens. The `0..1` params (Granulation / Mix) use the allowlisted `0.0`/`1.0` inline.
const EDGE_MAX: f32 = 8.0; // LITERAL-PX-OK: watercolor Edge-gain range bound (parameter domain)
const SPREAD_MIN: f32 = 1.0; // LITERAL-PX-OK: watercolor Spread blur-radius min (px)
const SPREAD_MAX: f32 = 24.0; // LITERAL-PX-OK: watercolor Spread blur-radius max (px)
const DEPTH_MIN: f32 = 0.1; // LITERAL-PX-OK: Beer–Lambert optical-depth min (parameter domain)
const DEPTH_MAX: f32 = 8.0; // LITERAL-PX-OK: Beer–Lambert optical-depth max (parameter domain)
const WARP_MAX: f32 = 24.0; // LITERAL-PX-OK: watercolor Warp displacement max (px)

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
        // Wash body (render-path optics): Fill = interior density, Depth = Beer–Lambert scale. These two
        // reconstruct the flat translucent glaze; Edge/Granulation/Warp add the wet character on top.
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Fill",
            core_ids::PAINTER_WATERCOLOR_FILL,
            brush.fill,
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
        );
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Depth",
            core_ids::PAINTER_WATERCOLOR_DEPTH,
            brush.depth,
            DEPTH_MIN,
            DEPTH_MAX,
            crate::number_field::FINE_STEP,
            2,
        );
        // #1 Edge darkening (the "fringe") — gain + blur radius.
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Edge",
            core_ids::PAINTER_WATERCOLOR_EDGE,
            brush.edge_gain,
            0.0,
            EDGE_MAX,
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
            brush.edge_spread,
            SPREAD_MIN,
            SPREAD_MAX,
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
            brush.granulation,
            0.0,
            1.0,
            crate::number_field::FINE_STEP,
            2,
        );
        // Warp — organic (ragged) wash boundary via fractal displacement of the coverage sampling.
        y = crate::number_field::paint_num_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Warp",
            core_ids::PAINTER_WATERCOLOR_WARP,
            brush.warp,
            0.0,
            WARP_MAX,
            crate::number_field::SIZE_STEP,
            1,
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
                brush.pigment_mix,
                0.0,
                1.0,
                crate::number_field::FINE_STEP,
                2,
            );
        }
    }
    y
}
