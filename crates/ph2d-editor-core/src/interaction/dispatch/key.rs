//! Keyboard event dispatch.
//!
//! Extracted from [`super`] (Track A10). One public entry point —
//! [`dispatch_key`] — routes keycodes through the focused widget's
//! state machine. Tab / Shift+Tab traverse the focus chain;
//! Cmd/Ctrl+A/C/X/V are clipboard primitives; Enter / Space activate
//! the focused widget (with text-widget special cases for newline /
//! literal space insertion); Esc reverts edits + blurs;
//! Backspace deletes; arrow keys move the caret (NumberInput Up/Down
//! +/- by 1.0).

use super::super::{InteractiveState, WidgetEvent, WidgetStore, format_number};
use super::clipboard::{clipboard_extract_selection, collapse_selection, delete_selection_if_any};
use super::focus::{apply_click, cycle_focus};
use super::keymap::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_DELETE,
    KEY_ENTER, KEY_ESCAPE, KEY_F2, KEY_KEY_A, KEY_KEY_C, KEY_KEY_D, KEY_KEY_F, KEY_KEY_G,
    KEY_KEY_K, KEY_KEY_P, KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
};
use super::text_ops::{next_char_boundary, prev_char_boundary};
use super::{
    commit_hex_buffer, commit_number_buffer, reset_focused_visual_state, revert_number_buffer,
    select_all_in_text_widget, write_hex_canonical,
};
use crate::interaction::types::GraphKey;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{KeyEvent, KeyKind};

/// Entry point for key events. Tab / Shift+Tab traverse the focus
/// chain; Enter / Space activate the focused widget; Escape blurs.
pub fn dispatch_key<'frame>(
    store: &mut WidgetStore,
    event: KeyEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    if event.kind == KeyKind::Up {
        return events.into_bump_slice();
    }
    // Motion Nodes M0.T3 — while a graph surface holds keyboard focus and no
    // text widget is being edited, the graph shortcuts (Delete/F/A/Esc/K/P and
    // Ctrl/Cmd+D) are consumed here, ahead of the generic focus/text handlers.
    // The panel drains `graph_keys` and decides what each verb does.
    if store.graph_focused().is_some()
        && store.focus_id().is_none()
        && let Some(gk) = graph_key_for(
            event.keycode,
            event.modifiers.ctrl || event.modifiers.meta,
            event.modifiers.alt,
        )
    {
        store.push_graph_key(gk);
        return events.into_bump_slice();
    }
    match event.keycode {
        KEY_TAB => {
            if event.modifiers.shift {
                cycle_focus(store, false, &mut events);
            } else {
                cycle_focus(store, true, &mut events);
            }
        }
        KEY_KEY_A if event.modifiers.meta || event.modifiers.ctrl => {
            // Cmd/Ctrl+A on a focused TextInput / NumberInput selects
            // the whole buffer (same effect as double-click).
            if let Some(id) = store.focus_id() {
                select_all_in_text_widget(store, id);
            }
        }
        KEY_KEY_C if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && let Some(text) = clipboard_extract_selection(store, id)
            {
                store.set_clipboard_copy(text);
            }
        }
        KEY_KEY_X if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && let Some(text) = clipboard_extract_selection(store, id)
            {
                store.set_clipboard_copy(text);
                delete_selection_if_any(store, id);
                events.push(WidgetEvent::TextChanged(id));
            }
        }
        KEY_KEY_V if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && matches!(
                    store.get(id),
                    Some(InteractiveState::TextInput { .. })
                        | Some(InteractiveState::Combobox { .. })
                        | Some(InteractiveState::NumberInput { .. })
                )
            {
                store.set_clipboard_paste_request(id);
            }
        }
        KEY_ENTER | KEY_SPACE => {
            if let Some(id) = store.focus_id() {
                // For NumberInput, Enter commits the buffer (parses
                // and emits ValueChanged on success) and ends the
                // edit by blurring — matches the single-line TextInput
                // form-field convention. Without the blur the caret
                // stayed inside the field and the user had to click
                // outside to dismiss focus.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. }))
                    && event.keycode == KEY_ENTER
                {
                    commit_number_buffer(store, id, &mut events, true);
                    reset_focused_visual_state(store, id);
                    store.set_focus(None);
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // Hex TextInput linked to a BlenderPicker: parse the
                // buffer and apply the resulting color to the parent,
                // then blur (Enter ends the edit, like a form field).
                if event.keycode == KEY_ENTER
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                    && store.blender_hex_parent(id).is_some()
                {
                    commit_hex_buffer(store, id, &mut events);
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // Palette-name TextInput linked to a BlenderPicker: Enter renames the active palette
                // to the typed buffer, then blurs (like the hex field).
                if event.keycode == KEY_ENTER
                    && let Some(parent) = store.blender_palette_name_parent(id)
                {
                    let name = match store.get(id) {
                        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
                        _ => String::new(),
                    };
                    store.blender_rename_active_palette(parent, &name);
                    store.sync_blender_palette_name_buffer(parent); // normalise (trim) the shown name
                    store.close_context_menu(); // Enter commits + closes the rename modal
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // SPACE while focus is on a text widget MUST insert a
                // space character (handled by dispatch_text_input) —
                // we used to also fire `apply_click` here, which
                // emitted a stray Click event and made the user press
                // SPACE twice before the char registered. Skip
                // apply_click for TextInput / Combobox / NumberInput.
                let is_text_widget = matches!(
                    store.get(id),
                    Some(InteractiveState::TextInput { .. })
                        | Some(InteractiveState::Combobox { .. })
                        | Some(InteractiveState::NumberInput { .. })
                );
                // SPACE on a text widget inserts a literal ' '
                // directly here, bypassing winit's text-input
                // pipeline. The shell's IME path delivers ' ' as a
                // text event AFTER the key event, but on macOS the
                // FIRST press's text-event sometimes arrives empty
                // (IME init) — so the user had to press SPACE twice.
                // Insert it ourselves on the key event; the shell
                // suppresses the matching text-event for KEY_SPACE
                // so we never double-insert.
                if event.keycode == KEY_SPACE
                    && matches!(
                        store.get(id),
                        Some(InteractiveState::TextInput { .. })
                            | Some(InteractiveState::Combobox { .. })
                    )
                {
                    delete_selection_if_any(store, id);
                    match store.get_mut(id) {
                        Some(InteractiveState::TextInput { text, caret, .. }) => {
                            text.insert(*caret, ' ');
                            *caret += 1;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::Combobox { query, caret, .. }) => {
                            query.insert(*caret, ' ');
                            *caret += 1;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        _ => {}
                    }
                    return events.into_bump_slice();
                }
                // M14.7 polish: Enter on the rename TextInput
                // commits via `WidgetEvent::Submit` instead of
                // inserting a newline. Caller (hero apply_event)
                // reads the buffer and applies the rename.
                if event.keycode == KEY_ENTER
                    && id == crate::ids::HIER_RENAME_INPUT
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Submit(id));
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // Enter on a TextInput: behavior depends on whether
                // it's marked multi-line.
                //   - Multi-line (TextArea, note bodies): insert a
                //     literal newline so paragraphs can have hard
                //     breaks.
                //   - Single-line (default): Submit + Blur, matching
                //     form-field convention. Callers (hero apply_event)
                //     read the buffer on Submit and apply the value.
                if event.keycode == KEY_ENTER
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    if store.is_multiline_text(id) {
                        delete_selection_if_any(store, id);
                        if let Some(InteractiveState::TextInput { text, caret, .. }) =
                            store.get_mut(id)
                        {
                            text.insert(*caret, '\n');
                            *caret += 1;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                    } else {
                        if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                            *state = crate::widget::TextInputState::Normal;
                        }
                        store.set_focus(None);
                        events.push(WidgetEvent::Submit(id));
                        events.push(WidgetEvent::Blur(id));
                    }
                    return events.into_bump_slice();
                }
                if !is_text_widget {
                    apply_click(store, id, &mut events);
                }
            }
        }
        KEY_ESCAPE => {
            // Audit fix #1 (CRITICAL): Esc must also abort any
            // in-flight NumberInput drag-slider OR stepper-hold,
            // regardless of focus. Without this the drag state stays
            // armed; the next Move would continue advancing
            // `last_committed` from a stale `start_value`, and Esc
            // (which is supposed to revert) would no longer work.
            // Cleared unconditionally — these `end_*` calls are no-ops
            // when nothing is in flight.
            let _ = store.end_number_input_drag();
            store.end_number_stepper_hold();
            if let Some(id) = store.focus_id() {
                // Dropdowns close on ESC instead of losing focus.
                if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(id)
                    && *open
                {
                    *open = false;
                    return events.into_bump_slice();
                }
                // M14.7 polish: Esc on the rename TextInput emits
                // `Cancel` so the panel can drop the rename mode without
                // committing (hierarchy row rename + timeline marker rename).
                // The FIELD says whether Esc aborts it (`mark_cancel_on_escape`) — this used
                // to be a hardcoded list of two ids here, which every new field of the kind
                // had to be added to, in a file that knows nothing about it.
                if store.is_cancel_on_escape(id)
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Cancel(id));
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // NumberInput: revert buffer to last committed value.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. })) {
                    revert_number_buffer(store, id);
                }
                // Hex TextInput: revert buffer to canonical form of
                // the parent picker's current value.
                if matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                    && store.blender_hex_parent(id).is_some()
                {
                    write_hex_canonical(store, id);
                }
                store.set_focus(None);
                events.push(WidgetEvent::Blur(id));
            }
        }
        KEY_BACKSPACE => {
            if let Some(id) = store.focus_id() {
                if delete_selection_if_any(store, id) {
                    events.push(WidgetEvent::TextChanged(id));
                } else {
                    match store.get_mut(id) {
                        Some(InteractiveState::TextInput { text, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(text, *caret);
                            text.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::NumberInput { buffer, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(buffer, *caret);
                            buffer.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::Combobox { query, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(query, *caret);
                            query.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        _ => {}
                    }
                }
            }
        }
        KEY_ARROW_LEFT => {
            if let Some(id) = store.focus_id() {
                // Selection collapse takes precedence over caret motion.
                if collapse_selection(store, id, false) {
                    return events.into_bump_slice();
                }
                match store.get_mut(id) {
                    Some(InteractiveState::TextInput { text, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(text, *caret);
                    }
                    Some(InteractiveState::NumberInput { buffer, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(buffer, *caret);
                    }
                    Some(InteractiveState::Combobox { query, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(query, *caret);
                    }
                    _ => {}
                }
            }
        }
        KEY_ARROW_RIGHT => {
            if let Some(id) = store.focus_id() {
                if collapse_selection(store, id, true) {
                    return events.into_bump_slice();
                }
                match store.get_mut(id) {
                    Some(InteractiveState::TextInput { text, caret, .. })
                        if *caret < text.len() =>
                    {
                        *caret = next_char_boundary(text, *caret);
                    }
                    Some(InteractiveState::NumberInput { buffer, caret, .. })
                        if *caret < buffer.len() =>
                    {
                        *caret = next_char_boundary(buffer, *caret);
                    }
                    Some(InteractiveState::Combobox { query, caret, .. })
                        if *caret < query.len() =>
                    {
                        *caret = next_char_boundary(query, *caret);
                    }
                    _ => {}
                }
            }
        }
        KEY_ARROW_UP => {
            if let Some(id) = store.focus_id()
                && let Some(InteractiveState::NumberInput {
                    value,
                    buffer,
                    caret,
                    last_committed,
                    ..
                }) = store.get_mut(id)
            {
                *value += 1.0;
                *last_committed = *value;
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", format_number(*value));
                *caret = buffer.len();
                events.push(WidgetEvent::ValueChanged(id));
            }
        }
        KEY_ARROW_DOWN => {
            if let Some(id) = store.focus_id()
                && let Some(InteractiveState::NumberInput {
                    value,
                    buffer,
                    caret,
                    last_committed,
                    ..
                }) = store.get_mut(id)
            {
                *value -= 1.0;
                *last_committed = *value;
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", format_number(*value));
                *caret = buffer.len();
                events.push(WidgetEvent::ValueChanged(id));
            }
        }
        _ => {}
    }
    events.into_bump_slice()
}

/// Motion Nodes M0.T3 — map a keycode (+ whether Cmd/Ctrl and Alt are held) to a
/// graph shortcut. Letter verbs require NO command modifier so `Cmd/Ctrl+A` stays
/// select-all and only `Cmd/Ctrl+D` duplicates; Delete/Backspace/Escape apply
/// regardless. Returns `None` for keys the graph doesn't consume (they fall
/// through to the generic handlers).
///
/// **Group / Ungroup** (doc 57) are `Ctrl+G` / `Ctrl+Alt+G` — the chords Blender
/// AND Nuke both use for exactly these two verbs (Blender: Group / Ungroup; Nuke:
/// Collapse To Group / Expand Group). The alt arm is listed FIRST: `Ctrl+Alt+G`
/// also satisfies `cmd`, and a match that tested the plain chord first would eat it.
///
/// **This is the ONE map, and it is `pub` for that reason** (Enio, smoke 2026-07-13:
/// *"se o mouse estiver em Motion, o tool motion deve capturar os atalhos e não o
/// canvas"*). The shell has to consult it too — its own key router runs on the
/// CURSOR (which panel is under the pointer), not on the focus gate below, because a
/// stale focus used to swallow the graph's keys. It used to do that by re-listing the
/// verbs by hand, one bespoke arm per key, and that second list is exactly how
/// `Ctrl+G` ended up toggling the scene grid: the graph grew a verb the shell had
/// never heard of. Two derivations of one map is one map too many.
pub fn graph_key_for(keycode: u32, cmd: bool, alt: bool) -> Option<GraphKey> {
    Some(match keycode {
        KEY_DELETE | KEY_BACKSPACE => GraphKey::Delete,
        KEY_KEY_F if !cmd => GraphKey::Fit,
        KEY_KEY_A if !cmd => GraphKey::Add,
        KEY_KEY_A if cmd => GraphKey::SelectAll,
        KEY_ESCAPE => GraphKey::Escape,
        KEY_KEY_K if !cmd => GraphKey::Knife,
        KEY_KEY_P if !cmd => GraphKey::Probe,
        KEY_KEY_D if cmd => GraphKey::Duplicate,
        KEY_KEY_G if cmd && alt => GraphKey::Ungroup,
        KEY_KEY_G if cmd => GraphKey::Group,
        // Space is the transport. It lived ONLY in the shell's hand-written list
        // until now, which made it the proof of the problem: the same verb, in two
        // maps, and neither knew about the other's.
        KEY_SPACE if !cmd => GraphKey::TogglePlay,
        KEY_F2 => GraphKey::Rename,
        _ => return None,
    })
}
