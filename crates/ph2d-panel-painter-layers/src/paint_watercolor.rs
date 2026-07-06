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
use ph2d_tool_painter::{BrushSettings, TextureKind};

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
        // Paper (substrate tooth) + Granulation (mineral settling) — two canvas-anchored slots.
        y = paint_paper_granulation(ctx, theme, x, content_w, y, brush);
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
fn paint_paper_granulation(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    brush: BrushSettings,
) -> f32 {
    // ── Paper (substrate) ──
    let (ny, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Paper",
        core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
        brush.paper_kind,
        TextureKind::from_u8(brush.paper_kind).name(),
    );
    y = ny;
    if let Some(r) = open {
        state::set_pending_paper_kind_dd(Some((r, brush.paper_kind)));
    }
    if brush.paper_kind != 0 {
        y = paint_slot_size_angle(
            ctx,
            theme,
            x,
            content_w,
            y,
            brush.paper_size,
            core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X,
            core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_Y,
            brush.paper_angle,
            core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE,
        );
    }

    // ── Granulation (mineral settling) ──
    y = number_field::paint_num_row(
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
        number_field::FINE_STEP,
        2,
    );
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
    if !brush.granulation_use_paper {
        let (ny, open) = paint_dropdown_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Grain",
            core_ids::PAINTER_WATERCOLOR_GRAN_KIND,
            brush.granulation_kind,
            TextureKind::from_u8(brush.granulation_kind).name(),
        );
        y = ny;
        if let Some(r) = open {
            state::set_pending_gran_kind_dd(Some((r, brush.granulation_kind)));
        }
        if brush.granulation_kind != 0 {
            y = paint_slot_size_angle(
                ctx,
                theme,
                x,
                content_w,
                y,
                brush.granulation_size,
                core_ids::PAINTER_WATERCOLOR_GRAN_SIZE_X,
                core_ids::PAINTER_WATERCOLOR_GRAN_SIZE_Y,
                brush.granulation_angle,
                core_ids::PAINTER_WATERCOLOR_GRAN_ANGLE,
            );
        }
    }
    y
}

/// Paint a canvas-anchored slot's **Size X/Y** + **Angle** rows (shared by Paper + Granulation).
#[allow(clippy::too_many_arguments)]
fn paint_slot_size_angle(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    size: [f32; 2],
    size_x_id: ph2d_a11y::NodeId,
    size_y_id: ph2d_a11y::NodeId,
    angle: u16,
    angle_id: ph2d_a11y::NodeId,
) -> f32 {
    y = number_field::paint_num_xy(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        size_x_id,
        size[0],
        size_y_id,
        size[1],
        TEX_SIZE_MIN,
        TEX_SIZE_MAX,
        number_field::SIZE_STEP,
        2,
    );
    number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Angle",
        angle_id,
        f32::from(angle),
        0.0,
        ANGLE_MAX,
        number_field::SIZE_STEP,
        0,
    )
}

/// The `TextureKind` options for the Paper / Granulation kind dropdowns, each in its own id namespace
/// (so a click never collides with the Grain picker). Drained by [`paint_watercolor_popovers`].
fn kind_options(option_id: fn(u8) -> ph2d_a11y::NodeId) -> Vec<DropdownOption<u8>> {
    (0..TextureKind::COUNT)
        .map(|k| DropdownOption::new(option_id(k), k, TextureKind::from_u8(k).name()))
        .collect()
}

/// Drain the Watercolor Paper / Granulation kind dropdown popovers (called from `paint_brush_popovers`,
/// after the body clip is popped, so the open list is never clipped).
pub(crate) fn paint_watercolor_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_paper_kind_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
            kind_options(core_ids::painter_paper_kind_option_id),
            chip_rect,
            cur,
        );
    }
    if let Some((chip_rect, cur)) = state::take_pending_gran_kind_dd() {
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_WATERCOLOR_GRAN_KIND,
            kind_options(core_ids::painter_granulation_kind_option_id),
            chip_rect,
            cur,
        );
    }
}
