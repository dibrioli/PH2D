//! The **Shape** Colour Ramp — a thin wrapper binding the shared ramp editor
//! ([`crate::paint_ramp_widget`]) to the Shape's fixed widget ids + the published `shape_color_ramp_*`
//! snapshot. The Shape ramp is the colour twin of the Grain ramp: with its **B&W** filter off it
//! colourises the silhouette; on, it is the silhouette's grayscale tone (auto-on with a Grain). The
//! body, B&W filter + responsive layout all live in the shared module — here we build the id-bundle +
//! view and drain the Shape ramp's own deferred dropdown popovers.

use crate::paint_brush::paint_dropdown_popover;
use crate::paint_ramp_widget::{RampIds, RampView, paint_color_ramp_section};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_shape_ramp_alpha_option_id, painter_shape_ramp_handle_id,
    painter_shape_ramp_interp_option_id, painter_shape_ramp_mode_option_id,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::{BrushSettings, RampAlphaMode, RampColorMode, RampInterp};

/// The Shape ramp's fixed widget ids + state hooks (collapsible "Shape Color").
fn shape_ramp_ids() -> RampIds {
    RampIds {
        section: core_ids::PAINTER_SHAPE_RAMP_SECTION,
        section_color: core_ids::PAINTER_SHAPE_RAMP_SECTION_COLOR,
        reset: core_ids::PAINTER_SHAPE_RAMP_RESET,
        enable: core_ids::PAINTER_SHAPE_RAMP_ENABLE,
        mode: core_ids::PAINTER_SHAPE_RAMP_MODE,
        interp: core_ids::PAINTER_SHAPE_RAMP_INTERP,
        alpha_mode: core_ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
        bw: core_ids::PAINTER_SHAPE_RAMP_BW,
        add: core_ids::PAINTER_SHAPE_RAMP_ADD,
        remove: core_ids::PAINTER_SHAPE_RAMP_REMOVE,
        invert: core_ids::PAINTER_SHAPE_RAMP_INVERT,
        edit: core_ids::PAINTER_SHAPE_RAMP_EDIT,
        swatch: core_ids::PAINTER_SHAPE_RAMP_SWATCH,
        stop_index: core_ids::PAINTER_SHAPE_RAMP_STOP_INDEX,
        stop_pos: core_ids::PAINTER_SHAPE_RAMP_STOP_POS,
        handle: painter_shape_ramp_handle_id,
        set_pending_mode: state::set_pending_shape_ramp_mode_dd,
        set_pending_interp: state::set_pending_shape_ramp_interp_dd,
        set_pending_alpha: state::set_pending_shape_ramp_alpha_dd,
    }
}

/// Paint the Shape Colour Ramp section at `y`, returning the next `y`. Always shown (it is the Shape
/// silhouette's colour, or — B&W on — its tone), titled "Shape Color".
pub(crate) fn paint_shape_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let count =
        (brush.shape_color_ramp_stop_count as usize).min(brush.shape_color_ramp_stops.len());
    // Smear/Blur/Clone/Eraser paint no colour → the ramp is a B&W coverage tone; force B&W checked + locked.
    let bw_locked = brush.paints_no_color() || brush.eraser || brush.is_mask;
    let view = RampView {
        enabled: brush.shape_color_ramp_enabled,
        bw: brush.shape_color_ramp_bw || bw_locked,
        bw_locked,
        mode: brush.shape_color_ramp_mode,
        interp: brush.shape_color_ramp_interp,
        alpha_mode: brush.shape_color_ramp_alpha_mode,
        stops: &brush.shape_color_ramp_stops[..count],
        selected_id: state::selected_shape_ramp_stop(),
    };
    paint_color_ramp_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Shape Color",
        &shape_ramp_ids(),
        view,
    )
}

/// Deferred paint of the Shape ramp's open Mode / Interpolation / Alpha dropdown popovers.
pub(crate) fn paint_shape_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_shape_ramp_mode_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_RAMP_MODE,
            mode_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_shape_ramp_interp_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_RAMP_INTERP,
            interp_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_shape_ramp_alpha_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
            alpha_options(),
            chip,
            cur,
        );
    }
}

fn mode_options() -> Vec<DropdownOption<u8>> {
    (0..RampColorMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_shape_ramp_mode_option_id(m),
                m,
                RampColorMode::from_u8(m).name(),
            )
        })
        .collect()
}

fn interp_options() -> Vec<DropdownOption<u8>> {
    (0..RampInterp::COUNT)
        .map(|i| {
            DropdownOption::new(
                painter_shape_ramp_interp_option_id(i),
                i,
                RampInterp::from_u8(i).name(),
            )
        })
        .collect()
}

fn alpha_options() -> Vec<DropdownOption<u8>> {
    (0..RampAlphaMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_shape_ramp_alpha_option_id(m),
                m,
                RampAlphaMode::from_u8(m).name(),
            )
        })
        .collect()
}
