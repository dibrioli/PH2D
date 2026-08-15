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
    /// **A PORTA ÚNICA de *«como este botão se pinta AGORA?»*** — o estado DISCRETO e quanto do
    /// hover está presente, numa pergunta só e num argumento só.
    ///
    /// ⚠️ **Ela nasceu porque o app já a tinha inventado QUATRO vezes em privado — e as quatro não
    /// eram a mesma.** Três (`context_menu_dialogs`, `fill_modal`, `onion_modal`) derivavam de
    /// `hot_id`/`active_id`; a quarta (o `paint_helpers` do grid-snap) repetia a lei do store. Esta
    /// porta **subsume as duas**, e a ordem entre elas é a correcção: o estado PRÓPRIO de um botão
    /// registado vence, e só quando não há registo é que o ponteiro responde.
    ///
    /// ⚠️ **A fallback não é cortesia — é o que torna um botão de MODAL vivo.** Um botão que o
    /// `populate` não registou não tem `InteractiveState::Button`, então a lei do store devolveria
    /// `Normal` para sempre: um botão pintado que nunca acende.
    ///
    /// ⚠️ **E o `t` sai do store, não de um segundo argumento** — ver [`Self::hover_live`]. O
    /// neutro é `1.0` = *assente no estado*: um id que o relógio nunca viu pinta exactamente o que
    /// pintava antes de a UI viva existir, que é o que mantém todo gate de painel a medir o que
    /// media.
    #[must_use]
    pub fn button_visual(&self, id: NodeId) -> (ButtonState, f32) {
        let state = self.button_state(id).unwrap_or_else(|| {
            if self.active_id() == Some(id) {
                ButtonState::Pressed
            } else if self.hot_id() == Some(id) {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            }
        });
        (state, self.hover_live(id))
    }

    /// Quanto do hover está PRESENTE neste id (`0.0`..=`1.0`); [`crate::motion::SETTLED`] para quem
    /// o relógio nunca viu.
    ///
    /// ⚠️ O neutro é `1.0` e não `0.0`: ele significa *assente no estado que o widget diz ter*, e é
    /// por isso que um id sem track pinta o mundo pré-UI-viva byte a byte. O nome mora no substrato
    /// para um sítio que FORÇA um estado duro poder dizer o mesmo neutro sem repetir o literal.
    #[must_use]
    pub fn hover_live(&self, id: NodeId) -> f32 {
        self.hover_live
            .get(&id)
            .copied()
            .unwrap_or(crate::motion::SETTLED)
    }

    /// **O tique publica aqui.** Único escritor, uma vez por quadro, imediatamente antes de todos os
    /// leitores — o gêmeo exacto do [`WidgetStore::set_panel_scroll_live`].
    pub fn set_hover_live(&mut self, id: NodeId, t: f32) {
        self.hover_live.insert(id, t);
    }

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
