//! Brush Studio `apply_event` — thin forwarder (ADR-0040 TG-B).
//!
//! The panel keeps no semantic mapping (slider id → which brush param). That
//! lives in `PainterTool::handle_panel_event`. Here we classify each
//! `WidgetEvent` into a tool-agnostic `PanelEvent`, push it via
//! `EditorAction::ToolPanelEvent`, and let the shell action-bus drain call
//! `Tool::handle_panel_event` on the active tool.

use crate::ids;
use crate::state::BrushStudioPanelState;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;

pub(crate) fn apply_event(
    _state: &mut BrushStudioPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider drag — forward the freshly-dispatched 0..1 value; the tool maps
        // it to the target brush field (`SetBrushParam` / `SetGrainDepth`).
        WidgetEvent::ValueChanged(id) if is_studio_slider(id) => {
            let value = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    value as f64,
                )));
            true
        }
        // Checkboxes — `Toggled` forwards as Click; the tool reads the live brush
        // and flips the matching bool (`handle_panel_event`).
        WidgetEvent::Toggled(id) if is_studio_checkbox(id) => {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Buttons (close X + grain/rendering cyclers) — forward Click.
        WidgetEvent::Click(id) if is_studio_button(id) => {
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        _ => false,
    }
}

#[inline]
fn is_studio_slider(id: NodeId) -> bool {
    // Engine-pending sliders (Roundness/Alpha) are not painted, so they never
    // dispatch — omitted here. See paint.rs notes.
    id == ids::SPACING_SLIDER
        || id == ids::SPACING_JITTER_SLIDER
        || id == ids::JITTER_LATERAL_SLIDER
        || id == ids::FALLOFF_SLIDER
        || id == ids::TAPER_SLIDER
        || id == ids::STREAMLINE_SLIDER
        || id == ids::STABILIZATION_SLIDER
        || id == ids::MOTION_FILTER_SLIDER
        || id == ids::MOTION_EXPR_SLIDER
        || id == ids::SPEED_SIZE_SLIDER
        || id == ids::SPEED_OPACITY_SLIDER
        || id == ids::SPEED_SPACING_SLIDER
        || id == ids::SHAPE_SCATTER_SLIDER
        || id == ids::SHAPE_COUNT_SLIDER
        || id == ids::SHAPE_COUNT_JITTER_SLIDER
        || id == ids::FLOW_SLIDER
        || id == ids::EDGE_INTENSITY_SLIDER
        || id == ids::PAPER_SLIDER
        || id == ids::GRAIN_SCALE_SLIDER
        || id == ids::GRAIN_DEPTH_SLIDER
        || id == ids::HUE_JITTER_SLIDER
        || id == ids::SAT_JITTER_SLIDER
        || id == ids::LIGHT_JITTER_SLIDER
        || id == ids::DARK_JITTER_SLIDER
        || id == ids::SIZE_JITTER_SLIDER
        || id == ids::OPACITY_JITTER_SLIDER
        || is_studio_watercolor_slider(id)
}

/// The 16 watercolor sliders (ADR-0079 + ADR-0078 S5) use index-derived ids (no consts), so they're
/// recognized by recomputing each — without this they paint + drag visually but their
/// `ValueChanged` is never forwarded to the tool (the slider "can't be manipulated").
#[inline]
fn is_studio_watercolor_slider(id: NodeId) -> bool {
    use ph2d_editor_core::ids as core_ids;
    (0..ph2d_tool_painter::WatercolorParams::COUNT)
        .any(|i| id == core_ids::painter_studio_watercolor_slider_id(i))
}

#[inline]
fn is_studio_checkbox(id: NodeId) -> bool {
    id == ids::SHAPE_ROTATION_FOLLOW
        || id == ids::SHAPE_RANDOMIZED
        || id == ids::SHAPE_FLIP_X
        || id == ids::SHAPE_FLIP_Y
        || id == ids::PIGMENT
        || id == ids::ACCUMULATE
        || id == ids::WET_EDGES
        || id == ids::BURNT_EDGES
        || id == ids::FLUID
}

#[inline]
fn is_studio_button(id: NodeId) -> bool {
    id == ids::CLOSE || id == ids::GRAIN_TYPE || id == ids::RENDERING_MODE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every painted watercolor slider (ADR-0079) MUST be recognized by `is_studio_slider`,
    /// else its drag `ValueChanged` is dropped: the thumb moves then snaps back next frame
    /// when the unchanged snapshot re-syncs it ("can't be manipulated"). This gate pins the
    /// index-derived ids to the forwarder.
    #[test]
    fn every_watercolor_slider_is_recognized() {
        use ph2d_editor_core::ids as core_ids;
        for i in 0..ph2d_tool_painter::WatercolorParams::COUNT {
            assert!(
                is_studio_slider(core_ids::painter_studio_watercolor_slider_id(i)),
                "watercolor slider {i} ({}) not in is_studio_slider — its drag won't forward",
                ph2d_tool_painter::WatercolorParams::CONTROLS[i].label
            );
        }
    }
}
