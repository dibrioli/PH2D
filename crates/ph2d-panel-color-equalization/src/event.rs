//! Color Equalization panel `apply_event` — thin forwarder.
//!
//! Mirrors the post-ADR-0040 TG-C pattern (vide `ph2d-panel-padding`):
//! the panel no longer holds the semantic mapping (`slider 0..1 →
//! natural unit`). That lives in `ColorEqualizationTool::handle_panel_event`
//! over in the tool crate. Here we just classify each `WidgetEvent` into
//! a tool-agnostic [`PanelEvent`], push it through
//! `EditorAction::ToolPanelEvent`, and let the action-bus drain in the
//! shell call `Tool::handle_panel_event` on the active tool. Cancel maps
//! to `EditorAction::CancelActiveTool`.
//!
//! The panel still owns the slider ↔ chip MIRROR (the chip's stored
//! number value follows a slider drag, and the slider's stored value
//! follows a chip commit). That is UI-state-local to the widget store,
//! not authoritative tool state.

use crate::ColorEqualizationPanelState;
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_tool_color_equalization::params::{
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, saturation_to_slider,
    slider_to_brightness, slider_to_clip_limit, slider_to_contrast, slider_to_saturation,
    slider_to_tile_grid, tile_grid_to_slider,
};

/// Pair: slider id ↔ chip id (chip displays the natural unit).
fn chip_for_slider(slider: NodeId) -> Option<NodeId> {
    if slider == ids::CEQ_CLIP_LIMIT {
        Some(ids::CEQ_CLIP_LIMIT_NUM)
    } else if slider == ids::CEQ_TILE_GRID {
        Some(ids::CEQ_TILE_GRID_NUM)
    } else if slider == ids::CEQ_BRIGHTNESS {
        Some(ids::CEQ_BRIGHTNESS_NUM)
    } else if slider == ids::CEQ_CONTRAST {
        Some(ids::CEQ_CONTRAST_NUM)
    } else if slider == ids::CEQ_SATURATION {
        Some(ids::CEQ_SATURATION_NUM)
    } else {
        None
    }
}

fn slider_for_chip(chip: NodeId) -> Option<NodeId> {
    if chip == ids::CEQ_CLIP_LIMIT_NUM {
        Some(ids::CEQ_CLIP_LIMIT)
    } else if chip == ids::CEQ_TILE_GRID_NUM {
        Some(ids::CEQ_TILE_GRID)
    } else if chip == ids::CEQ_BRIGHTNESS_NUM {
        Some(ids::CEQ_BRIGHTNESS)
    } else if chip == ids::CEQ_CONTRAST_NUM {
        Some(ids::CEQ_CONTRAST)
    } else if chip == ids::CEQ_SATURATION_NUM {
        Some(ids::CEQ_SATURATION)
    } else {
        None
    }
}

/// Map a normalized slider track (`0..1`) into the chip's natural unit
/// for that slider. The tool's `params` module owns the canonical
/// projections; we forward through them so the panel + tool never drift.
fn slider_to_chip_value(slider: NodeId, track: f32) -> f64 {
    if slider == ids::CEQ_CLIP_LIMIT {
        slider_to_clip_limit(track) as f64
    } else if slider == ids::CEQ_TILE_GRID {
        slider_to_tile_grid(track) as f64
    } else if slider == ids::CEQ_BRIGHTNESS {
        slider_to_brightness(track) as f64
    } else if slider == ids::CEQ_CONTRAST {
        slider_to_contrast(track) as f64
    } else if slider == ids::CEQ_SATURATION {
        slider_to_saturation(track) as f64
    } else {
        track as f64
    }
}

/// Inverse: project a chip's natural-unit value onto the paired slider's
/// `0..1` track.
fn chip_to_slider_track(slider: NodeId, value: f64) -> f32 {
    let v = value as f32;
    if slider == ids::CEQ_CLIP_LIMIT {
        clip_limit_to_slider(v)
    } else if slider == ids::CEQ_TILE_GRID {
        tile_grid_to_slider(v as u32)
    } else if slider == ids::CEQ_BRIGHTNESS {
        brightness_to_slider(v)
    } else if slider == ids::CEQ_CONTRAST {
        contrast_to_slider(v)
    } else if slider == ids::CEQ_SATURATION {
        saturation_to_slider(v)
    } else {
        v
    }
}

pub(crate) fn apply_event(
    _state: &mut ColorEqualizationPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Slider dragged — forward the normalized track to the tool and
        // mirror the projected natural-unit value into the paired chip's
        // stored number value so the chip text matches the thumb in real
        // time.
        WidgetEvent::ValueChanged(id) if chip_for_slider(id).is_some() => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    track as f64,
                )));
            if let Some(chip) = chip_for_slider(id) {
                host.store_mut()
                    .set_number_value(chip, slider_to_chip_value(id, track));
            }
            true
        }
        // Chip committed — forward the natural-unit value to the tool and
        // mirror the inverse projection back onto the slider's track so
        // the thumb follows the chip.
        WidgetEvent::ValueChanged(id) if slider_for_chip(id).is_some() => {
            let value = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, value,
                )));
            if let Some(slider) = slider_for_chip(id)
                && let Some(InteractiveState::Slider { value: track, .. }) =
                    host.store_mut().get_mut(slider)
            {
                *track = chip_to_slider_track(slider, value);
            }
            true
        }
        // Auto-WB toggle — forward as a tool-side click event (the tool
        // flips its bool).
        WidgetEvent::Click(id) if id == ids::CEQ_AUTO_WB => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Apply — bake at full resolution against every selected sprite
        // (shell broadcast).
        WidgetEvent::Click(id) if id == ids::CEQ_APPLY => {
            reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // Cancel — abandon + deactivate the tool. Shell switches back to
        // the default tool via `CancelActiveTool`, hiding this panel and
        // restoring the Inspector.
        WidgetEvent::Click(id) if id == ids::CEQ_CANCEL => {
            reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        _ => false,
    }
}

fn reset_button(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
