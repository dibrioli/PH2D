//! The **Paper Colors** ramp — a thin wrapper binding the shared ramp editor
//! ([`crate::paint_ramp_widget`]) to the Paper's fixed widget ids + the published `paper_color_ramp_*`
//! snapshot. Maps the paper tooth to a colour so the substrate can be tinted (cream / tan / grey papers).
//! Mirror of [`crate::paint_shape_ramp`]; the editor body lives in the shared module.

use crate::paint_brush::paint_dropdown_popover;
use crate::paint_ramp_widget::{RampIds, RampView, paint_color_ramp_section};
use crate::state;
use ph2d_editor_core::ids::{
    self as core_ids, painter_paper_ramp_alpha_option_id, painter_paper_ramp_handle_id,
    painter_paper_ramp_interp_option_id, painter_paper_ramp_mode_option_id,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::{BrushSettings, RampAlphaMode, RampColorMode, RampInterp};

/// The Paper ramp's fixed widget ids + state hooks (collapsible "Paper Colors").
fn paper_ramp_ids() -> RampIds {
    RampIds {
        section: core_ids::PAINTER_PAPER_RAMP_SECTION,
        section_color: core_ids::PAINTER_PAPER_RAMP_SECTION_COLOR,
        reset: core_ids::PAINTER_PAPER_RAMP_RESET,
        enable: core_ids::PAINTER_PAPER_RAMP_ENABLE,
        mode: core_ids::PAINTER_PAPER_RAMP_MODE,
        interp: core_ids::PAINTER_PAPER_RAMP_INTERP,
        alpha_mode: core_ids::PAINTER_PAPER_RAMP_ALPHA_MODE,
        bw: core_ids::PAINTER_PAPER_RAMP_BW,
        add: core_ids::PAINTER_PAPER_RAMP_ADD,
        remove: core_ids::PAINTER_PAPER_RAMP_REMOVE,
        invert: core_ids::PAINTER_PAPER_RAMP_INVERT,
        edit: core_ids::PAINTER_PAPER_RAMP_EDIT,
        swatch: core_ids::PAINTER_PAPER_RAMP_SWATCH,
        stop_index: core_ids::PAINTER_PAPER_RAMP_STOP_INDEX,
        stop_pos: core_ids::PAINTER_PAPER_RAMP_STOP_POS,
        handle: painter_paper_ramp_handle_id,
        set_pending_mode: state::set_pending_paper_ramp_mode_dd,
        set_pending_interp: state::set_pending_paper_ramp_interp_dd,
        set_pending_alpha: state::set_pending_paper_ramp_alpha_dd,
    }
}

/// Paint the Paper Colors ramp section at `y`, returning the next `y` (collapsible "Paper Colors").
pub(crate) fn paint_paper_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let count =
        (brush.paper_color_ramp_stop_count as usize).min(brush.paper_color_ramp_stops.len());
    let view = RampView {
        enabled: brush.paper_color_ramp_enabled,
        bw: brush.paper_color_ramp_bw,
        bw_locked: false,
        mode: brush.paper_color_ramp_mode,
        interp: brush.paper_color_ramp_interp,
        alpha_mode: brush.paper_color_ramp_alpha_mode,
        stops: &brush.paper_color_ramp_stops[..count],
        selected_id: state::selected_paper_ramp_stop(),
    };
    paint_color_ramp_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Paper Colors",
        &paper_ramp_ids(),
        view,
    )
}

/// Deferred paint of the Paper ramp's open Mode / Interpolation / Alpha dropdown popovers.
pub(crate) fn paint_paper_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_paper_ramp_mode_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_PAPER_RAMP_MODE,
            mode_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_paper_ramp_interp_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_PAPER_RAMP_INTERP,
            interp_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_paper_ramp_alpha_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            core_ids::PAINTER_PAPER_RAMP_ALPHA_MODE,
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
                painter_paper_ramp_mode_option_id(m),
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
                painter_paper_ramp_interp_option_id(i),
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
                painter_paper_ramp_alpha_option_id(m),
                m,
                RampAlphaMode::from_u8(m).name(),
            )
        })
        .collect()
}
