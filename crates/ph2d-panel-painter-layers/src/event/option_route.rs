//! Dropdown-popover **option-pick** routing for the Brush section (split from `event.rs` for the LOC
//! cap). A picked option closes its chip + forwards the chosen wire `u8` to the tool. Table-driven:
//! each entry pairs an option-id decoder with the chip it targets; the first decoder that matches the
//! clicked id wins (the id spaces are disjoint, so order is irrelevant). Add a dropdown ⇒ add one row.

use super::decode::{
    decode_brush_blend_option, decode_brush_falloff_option, decode_brush_preset_option,
    decode_jitter_unit_option, decode_paper_kind_option, decode_paper_mapping_option,
    decode_shape_kind_option, decode_shape_ramp_alpha_option, decode_shape_ramp_interp_option,
    decode_shape_ramp_mode_option, decode_stroke_method_option, decode_texture_kind_option,
    decode_texture_mapping_option, decode_texture_ramp_alpha_option,
    decode_texture_ramp_interp_option, decode_texture_ramp_mode_option,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

/// One row of the dispatch table: an option-id decoder + the chip it targets.
type OptionRoute = (fn(ph2d_a11y::NodeId) -> Option<u8>, ph2d_a11y::NodeId);

/// Route a `Click(id)` against the Brush-section dropdown options. Returns `Some(true)` when `id` was a
/// known option (consumed), else `None` (the caller's `Click` fall-through continues).
pub(super) fn route_brush_dropdown_option(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<bool> {
    let routes: [OptionRoute; 16] = [
        (decode_brush_preset_option, core_ids::PAINTER_BRUSH_PRESET),
        (
            decode_paper_kind_option,
            core_ids::PAINTER_WATERCOLOR_PAPER_KIND,
        ),
        (
            decode_paper_mapping_option,
            core_ids::PAINTER_WATERCOLOR_PAPER_MAPPING,
        ),
        (decode_brush_blend_option, core_ids::PAINTER_BRUSH_BLEND),
        (decode_brush_falloff_option, core_ids::PAINTER_BRUSH_FALLOFF),
        (
            decode_stroke_method_option,
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
        ),
        (
            decode_jitter_unit_option,
            core_ids::PAINTER_BRUSH_JITTER_UNIT,
        ),
        (decode_shape_kind_option, core_ids::PAINTER_SHAPE_KIND),
        (
            decode_texture_kind_option,
            core_ids::PAINTER_BRUSH_TEXTURE_KIND,
        ),
        (
            decode_texture_mapping_option,
            core_ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        ),
        (
            decode_texture_ramp_mode_option,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        ),
        (
            decode_texture_ramp_interp_option,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        ),
        (
            decode_texture_ramp_alpha_option,
            core_ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
        ),
        (
            decode_shape_ramp_mode_option,
            core_ids::PAINTER_SHAPE_RAMP_MODE,
        ),
        (
            decode_shape_ramp_interp_option,
            core_ids::PAINTER_SHAPE_RAMP_INTERP,
        ),
        (
            decode_shape_ramp_alpha_option,
            core_ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
        ),
    ];
    routes.iter().find_map(|&(decode, target)| {
        decode(id).map(|v| {
            forward_dropdown_option(host, target, v);
            true
        })
    })
}

/// Close the dropdown `chip_id` + forward the chosen option `value` (wire u8) to the tool.
fn forward_dropdown_option(
    host: &mut dyn PanelHostInternal,
    chip_id: ph2d_a11y::NodeId,
    value: u8,
) {
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(chip_id)
    {
        *open = false;
        *selected_index = Some(value as usize);
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            chip_id,
            value.to_string(),
        )));
}
