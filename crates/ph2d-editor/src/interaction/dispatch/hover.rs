//! Hover + pressed/released state transitions for primitive widgets.
//!
//! Extracted from [`super`] (Track A6). These helpers consume a
//! pointer hit-test result (`Option<NodeId>` from
//! [`super::super::HitIndex::hit`]) and roll the matching widgets'
//! `InteractiveState::*::state` field between `Normal`, `Hovered`,
//! `Pressed`, etc., honoring the four primitive widget kinds:
//! Button, Toggle, Slider, Checkbox.
//!
//! Text-editing widgets (TextInput / NumberInput / Combobox) and the
//! BlenderPicker have their own state machines and are *not* touched
//! here.

use super::super::{InteractiveState, WidgetStore};
use crate::widget::{ButtonState, CheckboxState, SliderState, ToggleState};
use ph2d_a11y::NodeId;

/// Transition the hovered widget. Called on every pointer Move with
/// the new hit-tested id. Reverts the previously-hovered widget's
/// state, optionally promotes the new one to Hovered (skipped if
/// the widget is the currently-active drag target — its state must
/// stay Pressed for the duration of the drag).
pub(super) fn update_hover(store: &mut WidgetStore, hit: Option<NodeId>) {
    let prev = store.hot_id();
    if prev == hit {
        return;
    }
    if let Some(old) = prev {
        // Revert previous widget's state from Hovered → Normal
        // (unless it's currently Pressed/Disabled, which we leave
        // alone).
        leave_hover(store, old);
    }
    if let Some(new) = hit {
        // Skip hover state on the active (dragging) widget — its
        // state stays Pressed.
        if store.active_id() != Some(new) {
            enter_hover(store, new);
        }
    }
    store.set_hot(hit);
}

pub(super) fn enter_hover(store: &mut WidgetStore, id: NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) if *state == ButtonState::Normal => {
            *state = ButtonState::Hovered
        }
        Some(InteractiveState::Toggle { state, .. }) if *state == ToggleState::Normal => {
            *state = ToggleState::Hovered
        }
        Some(InteractiveState::Slider { state, .. }) if *state == SliderState::Normal => {
            *state = SliderState::Hovered
        }
        Some(InteractiveState::Checkbox { state, .. }) if *state == CheckboxState::Normal => {
            *state = CheckboxState::Hovered
        }
        _ => {}
    }
}

pub(super) fn leave_hover(store: &mut WidgetStore, id: NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) if *state == ButtonState::Hovered => {
            *state = ButtonState::Normal
        }
        Some(InteractiveState::Toggle { state, .. }) if *state == ToggleState::Hovered => {
            *state = ToggleState::Normal
        }
        Some(InteractiveState::Slider { state, .. }) if *state == SliderState::Hovered => {
            *state = SliderState::Normal
        }
        Some(InteractiveState::Checkbox { state, .. }) if *state == CheckboxState::Hovered => {
            *state = CheckboxState::Normal
        }
        _ => {}
    }
}

pub(super) fn set_widget_pressed(store: &mut WidgetStore, id: NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) => *state = ButtonState::Pressed,
        Some(InteractiveState::Toggle { state, .. }) => *state = ToggleState::Pressed,
        Some(InteractiveState::Slider { state, .. }) => *state = SliderState::Dragging,
        Some(InteractiveState::Checkbox { state, .. }) => *state = CheckboxState::Pressed,
        _ => {}
    }
}

pub(super) fn set_widget_released(store: &mut WidgetStore, id: NodeId, still_hot: bool) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) => {
            *state = if still_hot {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
        }
        Some(InteractiveState::Toggle { state, .. }) => {
            *state = if still_hot {
                ToggleState::Hovered
            } else {
                ToggleState::Normal
            };
        }
        Some(InteractiveState::Slider { state, .. }) => {
            *state = if still_hot {
                SliderState::Hovered
            } else {
                SliderState::Normal
            };
        }
        Some(InteractiveState::Checkbox { state, .. }) => {
            *state = if still_hot {
                CheckboxState::Hovered
            } else {
                CheckboxState::Normal
            };
        }
        _ => {}
    }
}
