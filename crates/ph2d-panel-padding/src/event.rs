//! Padding panel `apply_event` — routes the panel's widget events out to
//! the shell over `EditorAction::PaddingUiEdit` / `PaddingCancel`, and
//! keeps each edge's slider + px chip in lock-step.
//!
//! The authoritative `PaddingTool` lives in the shell's `ToolRegistry`,
//! so the panel can't mutate the spec directly — every edit becomes a
//! bus action the shell drains into `PaddingTool::apply_ui_edit`.
//!
//! Real-time slider ⟷ chip link (done MANUALLY rather than via
//! `link_slider_number`, which would force both into the same `0..1`
//! space and clobber the px chip):
//! - slider drag → push the px edit + mirror the px into the chip's
//!   stored value (so focusing the chip shows the right number);
//! - chip commit/scrub → push the px edit + mirror the normalized track
//!   position into the slider's stored value (so the thumb tracks).

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tools::padding::{PaddingUiEdit, px_to_slider, slider_to_px};
use ph2d_editor_core::widget::ButtonState;

use crate::state::PaddingPanelState;

/// `(slider_id, chip_id)` → the edit constructor for that edge.
fn edge_edit(slider_or_chip: NodeId, px: i32) -> Option<PaddingUiEdit> {
    if slider_or_chip == ids::PAD_TOP || slider_or_chip == ids::PAD_TOP_NUM {
        Some(PaddingUiEdit::Top(px))
    } else if slider_or_chip == ids::PAD_RIGHT || slider_or_chip == ids::PAD_RIGHT_NUM {
        Some(PaddingUiEdit::Right(px))
    } else if slider_or_chip == ids::PAD_BOTTOM || slider_or_chip == ids::PAD_BOTTOM_NUM {
        Some(PaddingUiEdit::Bottom(px))
    } else if slider_or_chip == ids::PAD_LEFT || slider_or_chip == ids::PAD_LEFT_NUM {
        Some(PaddingUiEdit::Left(px))
    } else {
        None
    }
}

/// Chip id paired with an edge slider id.
fn chip_for_slider(slider: NodeId) -> Option<NodeId> {
    if slider == ids::PAD_TOP {
        Some(ids::PAD_TOP_NUM)
    } else if slider == ids::PAD_RIGHT {
        Some(ids::PAD_RIGHT_NUM)
    } else if slider == ids::PAD_BOTTOM {
        Some(ids::PAD_BOTTOM_NUM)
    } else if slider == ids::PAD_LEFT {
        Some(ids::PAD_LEFT_NUM)
    } else {
        None
    }
}

/// Slider id paired with an edge px chip id.
fn slider_for_chip(chip: NodeId) -> Option<NodeId> {
    if chip == ids::PAD_TOP_NUM {
        Some(ids::PAD_TOP)
    } else if chip == ids::PAD_RIGHT_NUM {
        Some(ids::PAD_RIGHT)
    } else if chip == ids::PAD_BOTTOM_NUM {
        Some(ids::PAD_BOTTOM)
    } else if chip == ids::PAD_LEFT_NUM {
        Some(ids::PAD_LEFT)
    } else {
        None
    }
}

pub(crate) fn apply_event(
    _state: &mut PaddingPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // Edge slider dragged — read the live track, map to px, push the
        // edit, and mirror the px into the paired chip's stored value.
        WidgetEvent::ValueChanged(id) if chip_for_slider(id).is_some() => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            let px = slider_to_px(track);
            if let Some(edit) = edge_edit(id, px) {
                host.bus_mut().push(EditorAction::PaddingUiEdit(edit));
            }
            if let Some(chip) = chip_for_slider(id) {
                host.store_mut().set_number_value(chip, px as f64);
            }
            true
        }
        // Edge px chip edited (keyboard commit or drag-scrub) — read the
        // px value, push the edit, and mirror the normalized track onto
        // the paired slider's stored value so the thumb follows.
        WidgetEvent::ValueChanged(id) if slider_for_chip(id).is_some() => {
            let px = host.store().number_value(id).unwrap_or(0.0).round() as i32;
            if let Some(edit) = edge_edit(id, px) {
                host.bus_mut().push(EditorAction::PaddingUiEdit(edit));
            }
            if let Some(slider) = slider_for_chip(id)
                && let Some(InteractiveState::Slider { value, .. }) =
                    host.store_mut().get_mut(slider)
            {
                *value = px_to_slider(px);
            }
            true
        }
        // Pivot-mode toggle — flips recenter/keep. Reset the pressed
        // state; the active look is the per-frame snapshot in `paint`.
        WidgetEvent::Click(id) if id == ids::PAD_PIVOT_RECENTER => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::PaddingUiEdit(
                PaddingUiEdit::TogglePivotRecenter,
            ));
            true
        }
        // Apply button — bake at full resolution.
        WidgetEvent::Click(id) if id == ids::PAD_APPLY => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut()
                .push(EditorAction::PaddingUiEdit(PaddingUiEdit::Apply));
            true
        }
        // Cancel — abandon + deactivate the tool (shell switches back to
        // the default tool, hiding this panel and restoring the Inspector).
        WidgetEvent::Click(id) if id == ids::PAD_CANCEL => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut().push(EditorAction::PaddingCancel);
            true
        }
        _ => false,
    }
}
