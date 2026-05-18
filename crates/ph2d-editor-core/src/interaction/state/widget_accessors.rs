//! Read-only convenience accessors on [`WidgetStore`] for each
//! widget kind.
//!
//! Extracted from [`super`] (Track D7). All these methods are pure
//! reads — they match against the [`InteractiveState`] variant and
//! return the relevant fields. Painters lean on these heavily to
//! avoid open-coding the same match arms everywhere.

use super::WidgetStore;
use crate::interaction::InteractiveState;
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderState, TextInputState, ToggleState,
};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// Convenience: read button state.
    pub fn button_state(&self, id: NodeId) -> Option<ButtonState> {
        match self.states.get(&id) {
            Some(InteractiveState::Button { state }) => Some(*state),
            _ => None,
        }
    }

    /// Convenience: read toggle on/off + state.
    pub fn toggle(&self, id: NodeId) -> Option<(ToggleState, bool)> {
        match self.states.get(&id) {
            Some(InteractiveState::Toggle { state, on }) => Some((*state, *on)),
            _ => None,
        }
    }

    /// Convenience: read slider value + state.
    pub fn slider(&self, id: NodeId) -> Option<(SliderState, f32)> {
        match self.states.get(&id) {
            Some(InteractiveState::Slider { state, value, .. }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// Convenience: read checkbox value + state.
    pub fn checkbox(&self, id: NodeId) -> Option<(CheckboxState, CheckboxValue)> {
        match self.states.get(&id) {
            Some(InteractiveState::Checkbox { state, value }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// Convenience: read text input contents.
    pub fn text(&self, id: NodeId) -> Option<&str> {
        match self.states.get(&id) {
            Some(InteractiveState::TextInput { text, .. }) => Some(text.as_str()),
            Some(InteractiveState::Combobox { query, .. }) => Some(query.as_str()),
            Some(InteractiveState::NumberInput { buffer, .. }) => Some(buffer.as_str()),
            _ => None,
        }
    }

    /// Convenience: read number-input full state (state + value +
    /// editing buffer + caret + selection anchor). Returns `None`
    /// for non-number widgets.
    #[allow(clippy::type_complexity)]
    pub fn number_input(
        &self,
        id: NodeId,
    ) -> Option<(TextInputState, f64, &str, usize, Option<usize>)> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput {
                state,
                value,
                buffer,
                caret,
                selection_anchor,
                ..
            }) => Some((*state, *value, buffer.as_str(), *caret, *selection_anchor)),
            _ => None,
        }
    }

    /// Read just the current numeric value (committed). Useful for
    /// linked sliders that don't care about the in-progress buffer.
    pub fn number_value(&self, id: NodeId) -> Option<f64> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput { value, .. }) => Some(*value),
            _ => None,
        }
    }
}
