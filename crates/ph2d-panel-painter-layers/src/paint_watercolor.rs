//! The Painter dock's **Watercolor** section — the wet-media look (edge darkening + granulation +
//! pigment build-up; no fluid sim, `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). A master
//! "Wet edges" checkbox gates the section; the Edge / Spread / Granulation sliders + the Pigment
//! toggle (and its Mix slider) show only while it is on (Blender hides dead controls; DIRETIVA §2).
//!
//! All controls are fixed-id, tool-global widgets (registered in [`crate::populate`]); this module
//! only paints them off the published [`BrushSettings`] snapshot, reusing the row helpers. The four
//! sliders carry natural units and forward the real value as `SetValue`; the two toggles + the reset
//! forward as `PanelEvent::Click` (drained in [`crate::event`]).

use crate::paint_brush::paint_dropdown_row;
use crate::paint_brush_top::{paint_checkbox_row, paint_collapsible_section};
use crate::{number_field, state};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::{
    BrushSettings, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TextureKind, TextureMapping, param_specs,
};

/// Slider range bounds — the parameter domains (matching the tool's `set_brush_*` clamps), not design
/// tokens. The `0..1` params (Granulation / Mix) use the allowlisted `0.0`/`1.0` inline.
const EDGE_MAX: f32 = 8.0; // LITERAL-PX-OK: watercolor Edge-gain range bound (parameter domain)
const SPREAD_MIN: f32 = 1.0; // LITERAL-PX-OK: watercolor Spread blur-radius min (px)
const SPREAD_MAX: f32 = 24.0; // LITERAL-PX-OK: watercolor Spread blur-radius max (px)
const TEX_SIZE_MIN: f32 = 0.1; // LITERAL-PX-OK: paper/granulation Size min (mirrors TEX_SIZE_MIN)
const TEX_SIZE_MAX: f32 = 100.0; // LITERAL-PX-OK: paper/granulation Size max (mirrors TEX_SIZE_MAX)
const ANGLE_MAX: f32 = 360.0; // LITERAL-PX-OK: paper/granulation Angle range (degrees)
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

/// Paint the **Paper** + **Granulation** canvas-anchored slots (substrate tooth + mineral settling).
/// Paper: a kind picker + Size/Angle. Granulation: the amount, a "Same as Paper" checkbox, and — when
/// off — its own kind + Size/Angle. Returns the next `y`.
/// Paint the **Paper** section (the substrate the wash sits on) — a collapsible section ABOVE the Grain
/// section, shown only in watercolor mode. Full parity with the Grain section's controls: Kind picker +
/// Mapping + Rake + Random Angle + Angle + Offset + Size + Depth + per-pattern params. (The live preview
/// + the Color Ramp are a follow-up; the substrate is a grayscale height-field so a ramp is niche.)
pub(crate) fn paint_paper_section(
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
        "Paper",
        core_ids::PAINTER_WATERCOLOR_PAPER_SECTION,
        core_ids::PAINTER_WATERCOLOR_PAPER_SECTION_COLOR,
        core_ids::PAINTER_WATERCOLOR_PAPER_RESET,
    );
    if collapsed {
        return y;
    }
    let kind = TextureKind::from_u8(brush.paper_kind);
    // ── Kind picker ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Paper",
        core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
        brush.paper_kind,
        kind.name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_paper_kind_dd(Some((r, brush.paper_kind)));
    }
    if kind == TextureKind::None {
        return y;
    }
    // ── Mapping + Rake + Random Angle (per-dab rotation mappings only) ──
    let mapping = TextureMapping::from_u8(brush.paper_mapping);
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Mapping",
        core_ids::PAINTER_WATERCOLOR_PAPER_MAPPING,
        brush.paper_mapping,
        mapping.name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_paper_mapping_dd(Some((r, brush.paper_mapping)));
    }
    if mapping.uses_dab_rotation() {
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_WATERCOLOR_PAPER_RAKE,
            "Rake",
            brush.paper_rake,
        );
        y = paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            core_ids::PAINTER_WATERCOLOR_PAPER_RANDOM,
            "Random Angle",
            brush.paper_random,
        );
    }
    y = number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Angle",
        core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE,
        f32::from(brush.paper_angle),
        0.0,
        ANGLE_MAX,
        number_field::ANGLE_STEP,
        0,
    );
    // ── Offset + Size ──
    y = number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Offset",
        core_ids::PAINTER_WATERCOLOR_PAPER_OFFSET_X,
        brush.paper_offset[0],
        core_ids::PAINTER_WATERCOLOR_PAPER_OFFSET_Y,
        brush.paper_offset[1],
        TEX_OFFSET_MIN,
        TEX_OFFSET_MAX,
        number_field::FINE_STEP,
        2,
    );
    y = number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
        brush.paper_size[0],
        core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_Y,
        brush.paper_size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        number_field::SIZE_STEP,
        2,
    );
    // ── Depth (paper tooth strength) ──
    y = number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Depth",
        core_ids::PAINTER_WATERCOLOR_PAPER_DEPTH,
        brush.paper_depth.clamp(0.0, 1.0),
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    );
    // ── Per-pattern params (Contrast / Brightness / kind knobs) ──
    let pp: Vec<(&str, ph2d_a11y::NodeId, f32)> = param_specs(kind)
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                s.label,
                core_ids::PAINTER_WATERCOLOR_PAPER_PARAMS[i],
                brush.paper_params[i],
            )
        })
        .collect();
    number_field::paint_num_params(ctx, theme, x, content_w, y, &pp)
}

/// The **Grain**-section watercolor extras — shown at the top of the Grain section in watercolor mode,
/// since in that mode the Grain slot IS the granulation map: the "Same as Paper" toggle (settle into the
/// paper's own tooth instead) + the granulation **Amount**. Returns the next `y`.
pub(crate) fn paint_grain_watercolor_extras(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_WATERCOLOR_GRAN_SAME,
        "Same as Paper",
        brush.granulation_use_paper,
    );
    number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Amount",
        core_ids::PAINTER_WATERCOLOR_GRANULATION,
        brush.granulation,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    )
}

/// Drain the Watercolor **Paper** kind + mapping dropdown popovers (called from `paint_brush_popovers`,
/// after the body clip is popped, so the open list is never clipped).
pub(crate) fn paint_watercolor_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_paper_kind_dd() {
        let options: Vec<DropdownOption<u8>> = (0..TextureKind::COUNT)
            .map(|k| {
                DropdownOption::new(
                    core_ids::painter_paper_kind_option_id(k),
                    k,
                    TextureKind::from_u8(k).name(),
                )
            })
            .collect();
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
            options,
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_paper_mapping_dd() {
        let options: Vec<DropdownOption<u8>> = (0..TextureMapping::COUNT)
            .map(|m| {
                DropdownOption::new(
                    core_ids::painter_paper_mapping_option_id(m),
                    m,
                    TextureMapping::from_u8(m).name(),
                )
            })
            .collect();
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_WATERCOLOR_PAPER_MAPPING,
            options,
            chip_rect,
            cur,
        );
    }
}
