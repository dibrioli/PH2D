//! Padding panel `apply_event` — routes the panel's widget events out to
//! the shell over `EditorAction::PaddingUiEdit` / `PaddingCancel`.
//!
//! The authoritative `PaddingTool` lives in the shell's `ToolRegistry`,
//! so the panel can't mutate the spec directly — every edit becomes a
//! bus action the shell drains into `PaddingTool::apply_ui_edit`.
//! Mirrors `ph2d-panel-bgremoval`.

use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::tools::padding::PaddingUiEdit;
use ph2d_editor_core::widget::ButtonState;

use crate::state::PaddingPanelState;

pub(crate) fn apply_event(
    _state: &mut PaddingPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        // One of the four edge fields edited (keyboard commit or
        // drag-scrub) — read the committed NumberInput value and forward
        // it as a signed pixel count.
        WidgetEvent::ValueChanged(id)
            if id == ids::PAD_TOP
                || id == ids::PAD_RIGHT
                || id == ids::PAD_BOTTOM
                || id == ids::PAD_LEFT =>
        {
            let value = host.store().number_value(id).unwrap_or(0.0).round() as i32;
            let edit = if id == ids::PAD_TOP {
                PaddingUiEdit::Top(value)
            } else if id == ids::PAD_RIGHT {
                PaddingUiEdit::Right(value)
            } else if id == ids::PAD_BOTTOM {
                PaddingUiEdit::Bottom(value)
            } else {
                PaddingUiEdit::Left(value)
            };
            host.bus_mut().push(EditorAction::PaddingUiEdit(edit));
            true
        }
        // Apply button — bake the resized canvas at full resolution. The
        // shell drains `Apply` into `apply_ui_edit` (arms pending_apply)
        // and then pushes `EditorAction::Padding` for the active selection.
        WidgetEvent::Click(id) if id == ids::PAD_APPLY => {
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            host.bus_mut()
                .push(EditorAction::PaddingUiEdit(PaddingUiEdit::Apply));
            true
        }
        // Cancel — abandon the spec and deactivate the tool (the shell
        // switches back to the default tool, hiding this panel and
        // restoring the Inspector).
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
