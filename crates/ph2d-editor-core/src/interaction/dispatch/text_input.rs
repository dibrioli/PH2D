//! IME / printable-character dispatch into focused text widgets.
//!
//! Extracted from [`super`] (Track A11). One public entry point —
//! [`dispatch_text_input`] — receives one `char` per IME commit / key
//! press and inserts it at the caret of the focused TextInput,
//! Combobox query, or NumberInput buffer (the last gated by
//! [`super::number_input::is_numeric_input_char`]).

use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use super::clipboard::delete_selection_if_any;
use super::number_input::is_numeric_input_char;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

/// Character input from the IME / keyboard. Inserts `ch` at the
/// caret of a focused [`InteractiveState::TextInput`] or appends to
/// a focused [`InteractiveState::Combobox::query`]. Other widget
/// kinds ignore the character.
pub fn dispatch_text_input<'frame>(
    store: &mut WidgetStore,
    ch: char,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    // Filter control characters; only printable text gets inserted.
    if ch.is_control() {
        return events.into_bump_slice();
    }
    let Some(id) = store.focus_id() else {
        return events.into_bump_slice();
    };
    // If the focused widget has an active selection, replacing it
    // is the first half of "type to overwrite". For NumberInput we
    // additionally require the typed char to be a valid numeric
    // character — otherwise we drop the char without touching
    // selection state.
    let should_replace_selection = match store.get(id) {
        Some(InteractiveState::TextInput { .. }) | Some(InteractiveState::Combobox { .. }) => true,
        Some(InteractiveState::NumberInput { .. }) => is_numeric_input_char(ch),
        _ => false,
    };
    if should_replace_selection {
        delete_selection_if_any(store, id);
    }
    match store.get_mut(id) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            text.insert(*caret, ch);
            *caret += ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        Some(InteractiveState::Combobox { query, caret, .. }) => {
            let pos = (*caret).min(query.len());
            query.insert(pos, ch);
            *caret = pos + ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        Some(InteractiveState::NumberInput { buffer, caret, .. }) if is_numeric_input_char(ch) => {
            buffer.insert(*caret, ch);
            *caret += ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        _ => {}
    }
    events.into_bump_slice()
}
