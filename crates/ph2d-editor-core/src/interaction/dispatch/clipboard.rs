//! Clipboard cut/copy/paste plumbing for text-editing widgets.
//!
//! Extracted from [`super`] (Track A3). The dispatcher itself doesn't
//! touch the OS clipboard — the shell does, via [`apply_clipboard_paste`]
//! and the `pending_clipboard_copy / pending_clipboard_paste` queues on
//! [`super::super::WidgetStore`]. These helpers manage the in-widget
//! side: extracting the selected text for a copy, replacing the
//! selection during a paste, and the selection-collapse primitive used
//! by both cut and the non-shift Arrow handlers.
//!
//! `apply_clipboard_paste` is `pub` (the shell calls it directly);
//! the others are `pub(super)` for use inside the dispatch module
//! tree only.

use super::super::{InteractiveState, WidgetStore};
use ph2d_a11y::NodeId;

/// Read the currently selected text from a focused TextInput /
/// NumberInput / Combobox. Returns `None` when the widget isn't a
/// text widget or has no active selection. Caret is treated as a
/// zero-length selection (returns `Some("")`) — caller decides
/// whether empty copies are interesting.
pub(super) fn clipboard_extract_selection(store: &WidgetStore, id: NodeId) -> Option<String> {
    let (text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => (text.as_str(), *caret, *selection_anchor),
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (buffer.as_str(), *caret, *selection_anchor),
        Some(InteractiveState::Combobox {
            query,
            caret,
            selection_anchor,
            ..
        }) => (query.as_str(), *caret, *selection_anchor),
        _ => return None,
    };
    let anchor = anchor?;
    let (start, end) = if anchor < caret {
        (anchor, caret)
    } else {
        (caret, anchor)
    };
    if start == end {
        return None;
    }
    let start = start.min(text.len());
    let end = end.min(text.len());
    Some(text[start..end].to_string())
}

/// Insert `text` at the caret of a focused text widget, replacing
/// any active selection. Shell calls this after reading the OS
/// clipboard in response to a pending paste request. Returns true
/// when something was inserted.
pub fn apply_clipboard_paste(store: &mut WidgetStore, id: NodeId, text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let _ = delete_selection_if_any(store, id);
    match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            text: buf, caret, ..
        }) => {
            buf.insert_str(*caret, text);
            *caret += text.len();
            true
        }
        Some(InteractiveState::Combobox { query, caret, .. }) => {
            query.insert_str(*caret, text);
            *caret += text.len();
            true
        }
        Some(InteractiveState::NumberInput { buffer, caret, .. }) => {
            // Filter non-numeric chars so paste can't put a NumberInput
            // into an unparsable state. Allowed: digits, '.', '-', '+'.
            let filtered: String = text
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+'))
                .collect();
            if filtered.is_empty() {
                return false;
            }
            buffer.insert_str(*caret, &filtered);
            *caret += filtered.len();
            true
        }
        _ => false,
    }
}

/// If the focused TextInput / NumberInput has a non-empty selection,
/// delete the selected range (replacing it with an empty cut at
/// `caret = sel_start`) and return true. The caller is then expected
/// to insert any pending character at the new caret position.
pub(super) fn delete_selection_if_any(store: &mut WidgetStore, id: NodeId) -> bool {
    let (text_ref, caret_ref, anchor_ref): (&mut String, &mut usize, &mut Option<usize>) =
        match store.get_mut(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) => (text, caret, selection_anchor),
            Some(InteractiveState::NumberInput {
                buffer,
                caret,
                selection_anchor,
                ..
            }) => (buffer, caret, selection_anchor),
            Some(InteractiveState::Combobox {
                query,
                caret,
                selection_anchor,
                ..
            }) => (query, caret, selection_anchor),
            _ => return false,
        };
    let Some(anchor) = *anchor_ref else {
        return false;
    };
    let (start, end) = if anchor < *caret_ref {
        (anchor, *caret_ref)
    } else {
        (*caret_ref, anchor)
    };
    if start == end {
        *anchor_ref = None;
        return false;
    }
    let start = start.min(text_ref.len());
    let end = end.min(text_ref.len());
    text_ref.replace_range(start..end, "");
    *caret_ref = start;
    *anchor_ref = None;
    true
}

/// Collapse any active selection on the focused TextInput /
/// NumberInput, optionally moving the caret to the left or right
/// edge of the original selection (matching standard text-editor
/// behavior for non-shift Arrow keys with an active selection).
/// Returns true iff a selection was collapsed.
pub(super) fn collapse_selection(
    store: &mut WidgetStore,
    id: NodeId,
    move_to_right_edge: bool,
) -> bool {
    let (caret, selection_anchor) = match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            caret,
            selection_anchor,
            ..
        })
        | Some(InteractiveState::NumberInput {
            caret,
            selection_anchor,
            ..
        })
        | Some(InteractiveState::Combobox {
            caret,
            selection_anchor,
            ..
        }) => (caret, selection_anchor),
        _ => return false,
    };
    let Some(anchor) = *selection_anchor else {
        return false;
    };
    let (lo, hi) = if anchor < *caret {
        (anchor, *caret)
    } else {
        (*caret, anchor)
    };
    *caret = if move_to_right_edge { hi } else { lo };
    *selection_anchor = None;
    true
}
