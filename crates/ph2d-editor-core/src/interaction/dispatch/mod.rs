//! Pointer + key dispatchers — translate raw shell events into
//! [`super::WidgetStore`] state mutations and [`super::WidgetEvent`]
//! emissions.
//!
//! Every dispatcher takes a `&'frame bumpalo::Bump` and returns a
//! `&'frame [WidgetEvent]` slice — the arena is the frame-local
//! event allocator. Caller drains the slice in the same frame and
//! resets the arena before the next frame.
//!
//! Phase A wires Button + Toggle. Slider/RadioGroup/Checkbox arrive
//! in Phase B; TextInput/NumberInput/Combobox in Phase C;
//! TreeView/ContextMenu/ColorPicker/Modal/Tabs in Phase D.

mod blender;
pub mod clipboard;
mod focus;
pub mod hierarchy;
mod hover;
pub mod key;
pub mod keymap;
mod number_input;
mod pointer;
pub mod scroll;
pub mod text_input;
mod text_ops;
pub mod tick;

use blender::{apply_blender_channel_value, derive_blender_channel_value};
pub use clipboard::apply_clipboard_paste;
pub use key::dispatch_key;
pub use keymap::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_ENTER,
    KEY_ESCAPE, KEY_KEY_A, KEY_KEY_C, KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
};
pub use pointer::{dispatch_pointer, dispatch_pointer_with_text};
pub use scroll::dispatch_wheel;
pub use text_input::dispatch_text_input;
pub use tick::dispatch_tick;

use super::{InteractiveState, WidgetEvent, WidgetStore};
use crate::zones::Rect;
use bumpalo::collections::Vec as BumpVec;

pub(super) fn init_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    // BlenderColorPicker channel chip: seed `value` from the parent
    // picker's current channel value (the chip's stored `value` is
    // stale; the painter renders the live derived value every
    // frame, but on focus the buffer needs to start from the
    // visible value, not the stale stored one).
    if let Some((parent, idx)) = store.blender_channel_chip(id) {
        let derived = derive_blender_channel_value(store, parent, idx);
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(id) {
            *value = derived;
        }
    }
    if let Some(InteractiveState::NumberInput {
        state,
        value,
        buffer,
        caret,
        last_committed,
        selection_anchor,
    }) = store.get_mut(id)
    {
        *state = crate::widget::TextInputState::Focused;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::format_number(*value));
        *caret = buffer.len();
        *last_committed = *value;
        *selection_anchor = None;
    }
}

/// True iff `id` belongs to a section header that supports the
/// right-click → SectionOutline context menu. Used by the right-click
/// dispatcher to decide whether to open that menu vs the create-note
/// menu.
///
/// Two source-of-truth arrays:
/// - [`crate::ids::SECTION_IDS`] — 10 Widget
///   Gallery (showcase) section headers.
/// - [`crate::ids::LIVE_SECTION_IDS`] — 4 live
///   Inspector section headers (Name / Visibility / Transform /
///   Render Source). Restored in Wave 4.1 so the section outline
///   affordance reaches the canonical Inspector, not just the demo
///   gallery.
///
/// Wave 2 PR 11.3 migrated NodeIds from numeric ranges to FNV-1a
/// hashes; this function used to test `350..=359` which became dead
/// after the migration (every hash falls outside that range), silently
/// breaking the affordance until Wave 4.1's audit caught it.
pub(super) fn is_section_header_id(id: ph2d_a11y::NodeId) -> bool {
    crate::ids::SECTION_IDS.contains(&id) || crate::ids::LIVE_SECTION_IDS.contains(&id)
}

/// Reset the focused visual state of a text-editing widget at `id`
/// to its `Normal` variant. Used on every blur path (Down handler,
/// `cycle_focus`, ESC, hex commit) so the painter stops drawing the
/// caret + focus border once the widget loses focus. Combobox uses
/// its own `ComboboxState` enum so it gets a separate match arm.
pub(super) fn reset_focused_visual_state(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::NumberInput { state, .. })
        | Some(InteractiveState::TextInput { state, .. }) => {
            *state = crate::widget::TextInputState::Normal;
        }
        Some(InteractiveState::Combobox { state, .. }) => {
            *state = crate::widget::ComboboxState::Normal;
        }
        _ => {}
    }
}

/// Set `selection_anchor = Some(0)` and `caret = text.len()` on the
/// focused TextInput / NumberInput widget at `id`. Triggered by
/// double-click and by Cmd/Ctrl+A. No-op for any other widget kind.
pub(super) fn select_all_in_text_widget(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = text.len();
        }
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = buffer.len();
        }
        Some(InteractiveState::Combobox {
            query,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = query.len();
        }
        _ => {}
    }
}

/// On focus departure (Blur, Tab away, Enter commit) from a
/// NumberInput, parse `buffer.trim()`. On success → update `value` +
/// `last_committed` and emit `ValueChanged`. On failure → revert the
/// buffer to the formatted `last_committed`. After committing,
/// mirrors the new value into a linked Slider (clamped to [0..1]).
pub(super) fn commit_number_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let mut new_value: Option<f64> = None;
    {
        let Some(InteractiveState::NumberInput {
            value,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
        else {
            return;
        };
        match buffer.trim().parse::<f64>() {
            Ok(parsed) if parsed.is_finite() => {
                if (parsed - *value).abs() > f64::EPSILON {
                    *value = parsed;
                    *last_committed = parsed;
                    events.push(WidgetEvent::ValueChanged(id));
                    new_value = Some(parsed);
                }
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::format_number(*value));
                *caret = buffer.len();
            }
            _ => {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::format_number(*last_committed));
                *value = *last_committed;
                *caret = buffer.len();
            }
        }
    }
    if let Some(v) = new_value
        && let Some(slider_id) = store.linked_slider(id)
    {
        // Inverse-project the chip's display-space value into the
        // slider's 0..1 storage. Identity mapping (the default) leaves
        // `v` unchanged — equivalent to the pre-2026-05-27 behavior.
        // Non-identity mapping (`link_slider_number_mapped`) translates
        // e.g. Grow display "+0.20" (signed) into slider 0.6, so the
        // next paint's `display_override` recomputes to the same
        // "+0.20" the user typed.
        let (scale, offset) = store.linked_slider_mapping(id);
        let storage = ((v as f32) - offset) / scale;
        if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id) {
            *value = storage.clamp(0.0, 1.0);
            events.push(WidgetEvent::ValueChanged(slider_id));
        }
    }
    // BlenderColorPicker channel chip: write the parsed value back
    // into the parent picker's RGBA / HSVA dimension at `idx`.
    if let Some(v) = new_value
        && let Some((parent, idx)) = store.blender_channel_chip(id)
    {
        apply_blender_channel_value(store, parent, idx, v as f32);
        events.push(WidgetEvent::ValueChanged(parent));
    }
}

/// Restore a NumberInput's buffer to its last committed value
/// without emitting any event. Used by Escape.
pub(super) fn revert_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *value = *last_committed;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::format_number(*last_committed));
        *caret = buffer.len();
    }
}

/// Parse the hex `TextInput` buffer at `id` and apply the resulting
/// color to the linked parent BlenderPicker (via
/// [`WidgetStore::link_blender_hex`]). Whether the parse succeeds or
/// not, the buffer is normalised to the canonical `#RRGGBBAA` form
/// of the parent's resulting value, so the painter always shows a
/// consistent string after commit. No-op if `id` is not a TextInput
/// or has no linked parent.
pub(super) fn commit_hex_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let buf_owned: String = match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => return,
    };
    if let Some(color) = crate::widget::parse_hex(&buf_owned) {
        store.set_blender_value(parent, color);
        events.push(WidgetEvent::ValueChanged(parent));
    }
    write_hex_canonical(store, id);
}

/// Rewrite the hex `TextInput` buffer at `id` with the canonical
/// `#RRGGBBAA` form of the linked parent BlenderPicker's current
/// value. Used by both commit (after parse + apply) and revert
/// (ESC) so the visible text always matches the parent state.
pub(super) fn write_hex_canonical(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let Some((cv, ..)) = store.blender_picker(parent) else {
        return;
    };
    let [r, g, b, a] = cv.rgba;
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id) {
        text.clear();
        use std::fmt::Write;
        let _ = write!(text, "#{r:02X}{g:02X}{b:02X}{a:02X}");
        *caret = text.len();
    }
}

#[cfg(test)]
mod tests;
