//! Focus traversal + click dispatch helpers.
//!
//! Extracted from [`super`] (Track A7). Three responsibilities:
//!
//! 1. [`is_focusable`] — predicate consulted by Tab/Shift+Tab nav to
//!    skip widgets in `Disabled` state. Default-allow for unknown
//!    kinds (still useful for keyboard scrolling).
//! 2. [`apply_click`] — Up-on-Down completion. Promotes the gesture
//!    to a `DoubleClick` event when the Down flagged it, then
//!    applies the per-kind click semantics (Toggle flips `on`,
//!    Checkbox cycles through Unchecked/Checked/Indeterminate,
//!    Dropdown/Combobox toggle `open`, Button/Plain emit Click).
//! 3. [`cycle_focus`] — Tab / Shift+Tab traversal. Commits the
//!    current focus owner's edit buffers (NumberInput, hex
//!    TextInput), emits `Blur(old)`, then walks the focus order
//!    forward (or backward) skipping non-focusable widgets and
//!    settles on the next available one with a `Focus(new)` event.

use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::{ButtonState, CheckboxState, CheckboxValue, SliderState, ToggleState};
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

pub(super) fn is_focusable(store: &WidgetStore, id: NodeId) -> bool {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state != ButtonState::Disabled,
        Some(InteractiveState::Toggle { state, .. }) => *state != ToggleState::Disabled,
        Some(InteractiveState::Slider { state, .. }) => *state != SliderState::Disabled,
        Some(InteractiveState::Checkbox { state, .. }) => *state != CheckboxState::Disabled,
        // Plain rects (section headers without collapsibility, etc.)
        // are still focusable for keyboard nav purposes — they don't
        // emit click events but accept Tab focus.
        Some(InteractiveState::Plain) => true,
        // Phases C-D add per-kind focusability for the rest.
        Some(_) => true,
        None => false,
    }
}

pub(super) fn apply_click<'a>(
    store: &mut WidgetStore,
    id: NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    // Upgrade to `DoubleClick(id)` when the matching Down flagged
    // this as a double-click on the same id. Consumed once per
    // gesture so a single click after that doesn't carry the flag.
    let pending = store.take_pending_double_click();
    let click_event = if pending == Some(id) {
        WidgetEvent::DoubleClick(id)
    } else {
        WidgetEvent::Click(id)
    };
    match store.get_mut(id) {
        Some(InteractiveState::Toggle { on, .. }) => {
            *on = !*on;
            events.push(WidgetEvent::Toggled(id));
        }
        Some(InteractiveState::Checkbox { value, .. }) => {
            *value = match *value {
                CheckboxValue::Unchecked | CheckboxValue::Indeterminate => CheckboxValue::Checked,
                CheckboxValue::Checked => CheckboxValue::Unchecked,
            };
            events.push(WidgetEvent::Toggled(id));
        }
        Some(InteractiveState::Dropdown { open, .. }) => {
            *open = !*open;
            // No event — caller observes via store.get(id).
        }
        Some(InteractiveState::Combobox { open, .. }) => {
            *open = !*open;
        }
        Some(InteractiveState::Button { .. }) | Some(InteractiveState::Plain) => {
            events.push(click_event);
        }
        // Phase D adds per-kind click semantics (Tabs select,
        // Modal dismiss, TreeView select, ContextMenu item, etc.).
        _ => {
            events.push(click_event);
        }
    }
}

pub(super) fn cycle_focus<'a>(
    store: &mut WidgetStore,
    forward: bool,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let order = store.focus_order();
    if order.is_empty() {
        return;
    }
    let current_pos = match store.focus_id() {
        Some(id) => order.iter().position(|x| *x == id),
        None => None,
    };
    let len = order.len();
    let start = match current_pos {
        Some(p) => {
            if forward {
                (p + 1) % len
            } else {
                (p + len - 1) % len
            }
        }
        None => {
            if forward {
                0
            } else {
                len - 1
            }
        }
    };
    // Walk forward until we find a focusable widget. Stop after one
    // full cycle to avoid infinite loop if nothing is focusable.
    let mut idx = start;
    for _ in 0..len {
        let id = order[idx];
        if is_focusable(store, id) {
            if let Some(old) = store.focus_id()
                && old != id
            {
                super::commit_number_buffer(store, old, events);
                super::commit_hex_buffer(store, old, events);
                super::reset_focused_visual_state(store, old);
                events.push(WidgetEvent::Blur(old));
            }
            if store.focus_id() != Some(id) {
                store.set_focus(Some(id));
                super::init_number_buffer(store, id);
                events.push(WidgetEvent::Focus(id));
            }
            return;
        }
        idx = if forward {
            (idx + 1) % len
        } else {
            (idx + len - 1) % len
        };
    }
}
