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
        || id == ids::STREAMLINE_SLIDER
        || id == ids::STABILIZATION_SLIDER
        || id == ids::SHAPE_SCATTER_SLIDER
        || id == ids::SHAPE_COUNT_SLIDER
        || id == ids::SHAPE_COUNT_JITTER_SLIDER
        || id == ids::FLOW_SLIDER
        || id == ids::GRAIN_SCALE_SLIDER
        || id == ids::GRAIN_DEPTH_SLIDER
        || id == ids::HUE_JITTER_SLIDER
        || id == ids::SAT_JITTER_SLIDER
        || id == ids::LIGHT_JITTER_SLIDER
        || id == ids::DARK_JITTER_SLIDER
        || id == ids::SIZE_JITTER_SLIDER
        || id == ids::OPACITY_JITTER_SLIDER
}

#[inline]
fn is_studio_checkbox(id: NodeId) -> bool {
    // Wet/Burnt Edges deferred — a correct stroke-silhouette edge effect needs a
    // coverage-mask pass, not a per-dab one (the per-dab version was wrong).
    id == ids::SHAPE_ROTATION_FOLLOW
        || id == ids::SHAPE_RANDOMIZED
        || id == ids::SHAPE_FLIP_X
        || id == ids::SHAPE_FLIP_Y
        || id == ids::PIGMENT
        || id == ids::ACCUMULATE
}

#[inline]
fn is_studio_button(id: NodeId) -> bool {
    id == ids::CLOSE || id == ids::GRAIN_TYPE || id == ids::RENDERING_MODE
}
