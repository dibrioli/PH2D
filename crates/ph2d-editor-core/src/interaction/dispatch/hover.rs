//! Hover + pressed/released state transitions for primitive widgets.
//!
//! Extracted from [`super`] (Track A6). These helpers consume a
//! pointer hit-test result (`Option<NodeId>` from
//! [`super::super::HitIndex::hit`]) and roll the matching widgets'
//! `InteractiveState::*::state` field between `Normal`, `Hovered`,
//! `Pressed`, etc., honoring the four primitive widget kinds:
//! Button, Toggle, Slider, Checkbox.
//!
//! ⚠️ **A exclusão dos campos de texto MORREU em 2026-08-15, e a nota que a declarava estava
//! certa sobre o mecanismo e errada sobre a conclusão.** Ela dizia que `TextInput`/`NumberInput`
//! *"têm máquinas de estado próprias e não são tocados aqui"* — verdade, o estado deles é
//! **semântico** (`Focused`/`Error`/`Disabled`) —, mas o pintor **sempre** soube desenhar
//! `Hovered` (`border_token` devolve `BorderEmph`) e **nada neste repositório o produzia**: todo
//! campo numérico e de texto do editor era inerte sob o ponteiro.
//!
//! ⚠️ **O que a exclusão de facto protegia era o BLUR:** sete sítios do despacho escreviam
//! `Normal` ao sair do foco **sem saber onde o ponteiro estava**, e um campo desfocado com o rato
//! em cima ficaria escuro até alguém mexer no rato. Isso é curado onde nasce — pela
//! [`blurred_field_state`], que é a porta ÚNICA do *"que estado tem um campo que acabou de perder
//! o foco?"*.
//!
//! O `BlenderPicker` continua de fora: ele não tem estado de hover.

use super::super::{InteractiveState, WidgetStore};
use crate::widget::{
    ButtonState, CheckboxState, DropdownState, SliderState, TagState, TextInputState, ToggleState,
};
use ph2d_a11y::NodeId;

/// **O estado de um campo que acabou de perder o foco.**
///
/// ⚠️ `Normal` é a resposta errada quando o ponteiro ainda está em cima: o campo apagava-se sob o
/// rato e só reacendia ao primeiro movimento. Perguntar aqui é o que torna o hover do campo uma
/// propriedade do STORE em vez de um remendo por-pintor.
#[must_use]
pub fn blurred_field_state(store: &WidgetStore, id: NodeId) -> TextInputState {
    if store.hot_id() == Some(id) {
        TextInputState::Hovered
    } else {
        TextInputState::Normal
    }
}

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
        // ⚠️ **As três famílias que o pintor sabia desenhar e ninguém acendia.** O guard
        // `== Normal` é o que mantém o estado SEMÂNTICO intacto: um campo focado continua focado
        // sob o ponteiro (o anel de `Accent` é o que diz onde a digitação cai), um dropdown
        // aberto continua aberto, e um `Disabled` que acendesse seria a interface a prometer o
        // que recusa.
        Some(
            InteractiveState::TextInput { state, .. } | InteractiveState::NumberInput { state, .. },
        ) if *state == TextInputState::Normal => *state = TextInputState::Hovered,
        Some(InteractiveState::Dropdown { state, .. }) if *state == DropdownState::Normal => {
            *state = DropdownState::Hovered
        }
        Some(InteractiveState::Tag { state }) if *state == TagState::Normal => {
            *state = TagState::Hovered
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
        Some(
            InteractiveState::TextInput { state, .. } | InteractiveState::NumberInput { state, .. },
        ) if *state == TextInputState::Hovered => *state = TextInputState::Normal,
        Some(InteractiveState::Dropdown { state, .. }) if *state == DropdownState::Hovered => {
            *state = DropdownState::Normal
        }
        Some(InteractiveState::Tag { state }) if *state == TagState::Hovered => {
            *state = TagState::Normal
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
