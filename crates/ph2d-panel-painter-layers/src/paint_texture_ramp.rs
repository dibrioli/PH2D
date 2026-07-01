//! The **Grain** Colour Ramp — a thin wrapper that binds the shared ramp editor
//! ([`crate::paint_ramp_widget`]) to the Grain's fixed widget ids + the published `texture_ramp_*`
//! snapshot. The editor body, the B&W filter + the responsive layout all live in the shared module;
//! here we only build the id-bundle + view and drain the Grain's deferred dropdown popovers.

use crate::paint_brush::paint_dropdown_popover;
use crate::paint_ramp_widget::{RampIds, RampView, paint_color_ramp_section};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_brush_texture_ramp_alpha_option_id,
    painter_brush_texture_ramp_handle_id, painter_brush_texture_ramp_interp_option_id,
    painter_brush_texture_ramp_mode_option_id,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::{BrushSettings, RampAlphaMode, RampColorMode, RampInterp};

/// The Grain ramp's fixed widget ids + state hooks (collapsible under "Grain Colors" / "Shape Color").
fn grain_ramp_ids() -> RampIds {
    RampIds {
        section: core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION,
        section_color: core_ids::PAINTER_BRUSH_COLOR_RAMP_SECTION_COLOR,
        reset: core_ids::PAINTER_BRUSH_COLOR_RAMP_RESET,
        enable: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        mode: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        interp: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        alpha_mode: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
        bw: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_BW,
        add: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        remove: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
        invert: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INVERT,
        edit: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_EDIT,
        swatch: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
        stop_index: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX,
        stop_pos: core_ids::PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS,
        handle: painter_brush_texture_ramp_handle_id,
        set_pending_mode: state::set_pending_ramp_mode_dd,
        set_pending_interp: state::set_pending_ramp_interp_dd,
        set_pending_alpha: state::set_pending_ramp_alpha_dd,
    }
}

/// Paint the Grain Color Ramp editor at `y`, returning the next `y`. `title` names the collapsible —
/// "Grain Colors" under the Grain, or "Shape Color" when this same `texture_ramp` object backs the
/// Shape's ramp slot. (The Shape's OWN ramp is [`crate::paint_shape_ramp`].)
pub(crate) fn paint_texture_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
    title: &str,
) -> f32 {
    let count = (brush.texture_ramp_stop_count as usize).min(brush.texture_ramp_stops.len());
    // Smear/Blur/Clone/Eraser paint no colour → the ramp is a B&W coverage tone; force B&W checked + locked.
    let bw_locked = brush.paints_no_color() || brush.eraser;
    let view = RampView {
        enabled: brush.texture_ramp_enabled,
        bw: brush.texture_ramp_bw || bw_locked,
        bw_locked,
        mode: brush.texture_ramp_mode,
        interp: brush.texture_ramp_interp,
        alpha_mode: brush.texture_ramp_alpha_mode,
        stops: &brush.texture_ramp_stops[..count],
        selected_id: state::selected_ramp_stop(),
    };
    paint_color_ramp_section(ctx, theme, x, content_w, y, title, &grain_ramp_ids(), view)
}

/// Deferred paint of the Grain ramp's open Mode / Interpolation / Alpha dropdown popovers.
pub(crate) fn paint_texture_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_ramp_mode_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
            ramp_mode_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_ramp_interp_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
            ramp_interp_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_ramp_alpha_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
            ramp_alpha_options(),
            chip,
            cur,
        );
    }
}

fn ramp_alpha_options() -> Vec<DropdownOption<u8>> {
    (0..RampAlphaMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_brush_texture_ramp_alpha_option_id(m),
                m,
                RampAlphaMode::from_u8(m).name(),
            )
        })
        .collect()
}

fn ramp_mode_options() -> Vec<DropdownOption<u8>> {
    (0..RampColorMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_brush_texture_ramp_mode_option_id(m),
                m,
                RampColorMode::from_u8(m).name(),
            )
        })
        .collect()
}

fn ramp_interp_options() -> Vec<DropdownOption<u8>> {
    (0..RampInterp::COUNT)
        .map(|i| {
            DropdownOption::new(
                painter_brush_texture_ramp_interp_option_id(i),
                i,
                RampInterp::from_u8(i).name(),
            )
        })
        .collect()
}
