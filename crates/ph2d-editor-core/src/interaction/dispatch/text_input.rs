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
use crate::ime::{ComposeKey, Composition};
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
    // O texto que chega SUBSTITUI a pré-edição (o `Commit` do IME, ou o `á` da tecla morta).
    store.set_composition(None);
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

/// `Ime::Preedit`: mostra `text` no caret do campo focado; vazio = a composição acaba. Nunca toca
/// o valor — só o `Commit` (via [`dispatch_text_input`]) escreve.
pub fn dispatch_preedit(store: &mut WidgetStore, text: &str, cursor: Option<(usize, usize)>) {
    let c = composing_field(store)
        .filter(|_| !text.is_empty())
        .map(|id| Composition {
            id,
            text: text.to_owned(),
            cursor,
            dead_key: false,
        });
    store.set_composition(c);
}

/// Uma tecla premida, vista pela composição (o caminho SEM IME, onde o xkb compõe e o winit só
/// entrega `Key::Dead`). `true` = consumida: o `Escape` que desfaz uma tecla morta pendente.
pub fn dispatch_compose_key(store: &mut WidgetStore, key: ComposeKey) -> bool {
    match key {
        ComposeKey::Dead(accent) => {
            if let Some(id) = composing_field(store) {
                store.set_composition(Some(Composition {
                    id,
                    text: accent.to_string(),
                    cursor: None,
                    dead_key: true,
                }));
            }
            false
        }
        ComposeKey::Modifier => false,
        ComposeKey::Escape | ComposeKey::Other => {
            let pending = store.composition().is_some_and(|c| c.dead_key);
            if pending {
                store.set_composition(None);
            }
            pending && key == ComposeKey::Escape
        }
    }
}

/// O campo focado que mostra uma composição — hoje o `TextInput` (o único pintor que a desenha).
fn composing_field(store: &WidgetStore) -> Option<ph2d_a11y::NodeId> {
    store
        .focus_id()
        .filter(|&id| matches!(store.get(id), Some(InteractiveState::TextInput { .. })))
}
