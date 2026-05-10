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

use super::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, ToggleState,
};
use crate::zones::Rect;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{KeyEvent, KeyKind, PointerEvent, PointerKind};

/// Keycodes the editor cares about. We don't pull in winit here —
/// the shell normalizes its keycodes to these constants before
/// forwarding to [`dispatch_key`]. Values mirror common
/// platform-independent keycodes (matches the shell's
/// `KeyEvent::keycode` field documentation).
pub const KEY_TAB: u32 = 0x09;
pub const KEY_ENTER: u32 = 0x0D;
pub const KEY_SPACE: u32 = 0x20;
pub const KEY_ESCAPE: u32 = 0x1B;
pub const KEY_BACKSPACE: u32 = 0x08;
pub const KEY_ARROW_UP: u32 = 0xF700;
pub const KEY_ARROW_DOWN: u32 = 0xF701;
pub const KEY_ARROW_LEFT: u32 = 0xF702;
pub const KEY_ARROW_RIGHT: u32 = 0xF703;

/// Entry point for pointer events. Updates [`WidgetStore`] hover /
/// active / focus cursors based on the hit-test, transitions the
/// per-widget interactive state, and emits widget events into the
/// caller's frame-local arena.
///
/// Returns the events emitted for this single dispatch call. Caller
/// drains synchronously; after the frame ends, caller resets the
/// arena (deallocates events for the next frame).
pub fn dispatch_pointer<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);

    match event.kind {
        PointerKind::Move => {
            // While a Slider is being dragged, every Move computes a
            // fresh value from the pointer position relative to the
            // active rect. Hover tracking is suppressed (the active
            // widget keeps its Pressed state regardless of where the
            // cursor went).
            if let Some(active) = store.active_id() {
                if let Some(rect) = store.active_rect()
                    && update_drag_value(store, active, rect, event.x, event.y)
                {
                    events.push(WidgetEvent::ValueChanged(active));
                }
            } else {
                let hit = hit_index.hit(event.x, event.y);
                update_hover(store, hit);
            }
        }
        PointerKind::Down => {
            if let Some((id, rect)) = hit_index.hit_with_rect(event.x, event.y)
                && is_focusable(store, id)
            {
                store.set_active(Some(id));
                store.set_active_rect(Some(rect));
                let prev_focus = store.focus_id();
                if prev_focus != Some(id) {
                    if let Some(old) = prev_focus {
                        commit_number_buffer(store, old, &mut events);
                        events.push(WidgetEvent::Blur(old));
                    }
                    store.set_focus(Some(id));
                    init_number_buffer(store, id);
                    events.push(WidgetEvent::Focus(id));
                }
                set_widget_pressed(store, id);
                // For sliders, the initial Down also sets value
                // (jump-to-clicked-position behavior).
                if matches!(store.get(id), Some(InteractiveState::Slider { .. }))
                    && update_drag_value(store, id, rect, event.x, event.y)
                {
                    events.push(WidgetEvent::ValueChanged(id));
                }
                // BlenderColorPicker sub-control hits route into the
                // parent's stored state mutation.
                if let Some(parent) = apply_blender_hit(store, id, rect, event.x, event.y) {
                    events.push(WidgetEvent::ValueChanged(parent));
                }
            }
        }
        PointerKind::Up => {
            if let Some(active) = store.active_id() {
                let hit = hit_index.hit(event.x, event.y);
                let still_hot = hit == Some(active);
                // Sliders emit no Click on release — they emitted
                // ValueChanged events throughout the drag. Buttons,
                // Toggles, and Checkboxes only count Click if the
                // pointer ended inside the original widget.
                let is_drag_widget =
                    matches!(store.get(active), Some(InteractiveState::Slider { .. }));
                if still_hot && !is_drag_widget {
                    apply_click(store, active, &mut events);
                }
                set_widget_released(store, active, still_hot);
                store.set_active(None);
                store.set_active_rect(None);
            }
        }
    }

    events.into_bump_slice()
}

/// Recompute slider value from pointer position relative to its
/// active rect. Returns true iff the value actually changed (so
/// dispatcher can decide whether to emit `ValueChanged`). Mirrors
/// the new value into a linked NumberInput, if one is registered.
fn update_drag_value(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> bool {
    let (changed, propagated) = {
        let Some(InteractiveState::Slider {
            state,
            value,
            orientation,
        }) = store.get_mut(id)
        else {
            return false;
        };
        let new_value = match *orientation {
            SliderOrientation::Horizontal => {
                if rect.w <= 0.0 {
                    0.0
                } else {
                    ((px - rect.x) / rect.w).clamp(0.0, 1.0)
                }
            }
            SliderOrientation::Vertical => {
                if rect.h <= 0.0 {
                    0.0
                } else {
                    (1.0 - (py - rect.y) / rect.h).clamp(0.0, 1.0)
                }
            }
        };
        let changed = (new_value - *value).abs() > f32::EPSILON;
        *value = new_value;
        *state = SliderState::Dragging;
        (changed, new_value as f64)
    };
    if changed && let Some(number_id) = store.linked_number(id) {
        store.set_number_value(number_id, propagated);
    }
    changed
}

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
    match event.keycode {
        KEY_TAB => {
            if event.modifiers.shift {
                cycle_focus(store, false, &mut events);
            } else {
                cycle_focus(store, true, &mut events);
            }
        }
        KEY_ENTER | KEY_SPACE => {
            if let Some(id) = store.focus_id() {
                // For NumberInput, Enter commits the buffer (parses)
                // and emits ValueChanged on success. Falls through to
                // apply_click for everything else.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. }))
                    && event.keycode == KEY_ENTER
                {
                    commit_number_buffer(store, id, &mut events);
                    return events.into_bump_slice();
                }
                apply_click(store, id, &mut events);
            }
        }
        KEY_ESCAPE => {
            if let Some(id) = store.focus_id() {
                // Dropdowns close on ESC instead of losing focus.
                if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(id)
                    && *open
                {
                    *open = false;
                    return events.into_bump_slice();
                }
                // NumberInput: revert buffer to last committed value.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. })) {
                    revert_number_buffer(store, id);
                }
                store.set_focus(None);
                events.push(WidgetEvent::Blur(id));
            }
        }
        KEY_BACKSPACE => {
            if let Some(id) = store.focus_id() {
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
                    _ => {}
                }
            }
        }
        KEY_ARROW_LEFT => {
            if let Some(id) = store.focus_id() {
                match store.get_mut(id) {
                    Some(InteractiveState::TextInput { text, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(text, *caret);
                    }
                    Some(InteractiveState::NumberInput { buffer, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(buffer, *caret);
                    }
                    _ => {}
                }
            }
        }
        KEY_ARROW_RIGHT => {
            if let Some(id) = store.focus_id() {
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
                let _ = write!(buffer, "{}", super::state::format_number(*value));
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
                let _ = write!(buffer, "{}", super::state::format_number(*value));
                *caret = buffer.len();
                events.push(WidgetEvent::ValueChanged(id));
            }
        }
        _ => {}
    }
    events.into_bump_slice()
}

fn prev_char_boundary(s: &str, mut i: usize) -> usize {
    if i == 0 {
        return 0;
    }
    i -= 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    if i >= len {
        return len;
    }
    i += 1;
    while i < len && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

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
    match store.get_mut(id) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            text.insert(*caret, ch);
            *caret += ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        Some(InteractiveState::Combobox { query, .. }) => {
            query.push(ch);
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

/// Filter for chars allowed in a NumberInput buffer: digits, sign,
/// decimal point, and scientific-notation `e`/`E`/`+`. Anything else
/// (letters, spaces, control chars) is dropped silently.
fn is_numeric_input_char(ch: char) -> bool {
    matches!(ch, '0'..='9' | '.' | '-' | '+' | 'e' | 'E')
}

/// On focus arrival into a NumberInput, sync `buffer` from `value`
/// using the same formatter the painter uses, and place the caret at
/// the end so the user can append digits immediately.
pub(super) fn init_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::state::format_number(*value));
        *caret = buffer.len();
        *last_committed = *value;
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
                let _ = write!(buffer, "{}", super::state::format_number(*value));
                *caret = buffer.len();
            }
            _ => {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::state::format_number(*last_committed));
                *value = *last_committed;
                *caret = buffer.len();
            }
        }
    }
    if let Some(v) = new_value
        && let Some(slider_id) = store.linked_slider(id)
        && let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id)
    {
        *value = (v as f32).clamp(0.0, 1.0);
        events.push(WidgetEvent::ValueChanged(slider_id));
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
        let _ = write!(buffer, "{}", super::state::format_number(*last_committed));
        *caret = buffer.len();
    }
}

// ---------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------

fn is_focusable(store: &WidgetStore, id: ph2d_a11y::NodeId) -> bool {
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

fn update_hover(store: &mut WidgetStore, hit: Option<ph2d_a11y::NodeId>) {
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

fn enter_hover(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
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

fn leave_hover(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
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

fn set_widget_pressed(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) => *state = ButtonState::Pressed,
        Some(InteractiveState::Toggle { state, .. }) => *state = ToggleState::Pressed,
        Some(InteractiveState::Slider { state, .. }) => *state = SliderState::Dragging,
        Some(InteractiveState::Checkbox { state, .. }) => *state = CheckboxState::Pressed,
        _ => {}
    }
}

fn set_widget_released(store: &mut WidgetStore, id: ph2d_a11y::NodeId, still_hot: bool) {
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

fn apply_click<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
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
            events.push(WidgetEvent::Click(id));
        }
        // Phase D adds per-kind click semantics (Tabs select,
        // Modal dismiss, TreeView select, ContextMenu item, etc.).
        _ => {
            events.push(WidgetEvent::Click(id));
        }
    }
}

fn cycle_focus<'a>(store: &mut WidgetStore, forward: bool, events: &mut BumpVec<'a, WidgetEvent>) {
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
                commit_number_buffer(store, old, events);
                events.push(WidgetEvent::Blur(old));
            }
            if store.focus_id() != Some(id) {
                store.set_focus(Some(id));
                init_number_buffer(store, id);
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

/// If `id` is a [`InteractiveState::BlenderHit`] sub-control, apply
/// the click to its parent [`InteractiveState::BlenderPicker`] and
/// return the parent's id (so the caller can emit `ValueChanged`).
/// Returns `None` for non-picker hits.
fn apply_blender_hit(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> Option<ph2d_a11y::NodeId> {
    use crate::interaction::BlenderHitKind;
    use crate::widget::{
        ChannelMode, InterpolationMode, apply_blender_value_pick, apply_blender_wheel_pick,
    };
    let (parent, kind) = match store.get(id)? {
        InteractiveState::BlenderHit { parent, kind } => (*parent, *kind),
        _ => return None,
    };
    match kind {
        BlenderHitKind::Wheel => {
            apply_blender_wheel_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::ValueSlider => {
            apply_blender_value_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::InterpolationLinear => {
            store.set_blender_interpolation(parent, InterpolationMode::Linear);
            Some(parent)
        }
        BlenderHitKind::InterpolationPerceptual => {
            store.set_blender_interpolation(parent, InterpolationMode::Perceptual);
            Some(parent)
        }
        BlenderHitKind::ChannelRgb => {
            store.set_blender_channel_mode(parent, ChannelMode::Rgb);
            Some(parent)
        }
        BlenderHitKind::ChannelHsv => {
            store.set_blender_channel_mode(parent, ChannelMode::Hsv);
            Some(parent)
        }
        BlenderHitKind::ChannelSlider(idx) => {
            // Compute normalised 0..1 from pointer x relative to the
            // slider track rect. The full rect is the track hit area
            // registered by `paint_blender_color_picker_with_store`.
            let norm = if rect.w > 0.0 {
                ((px - rect.x) / rect.w).clamp(0.0, 1.0)
            } else {
                0.0
            };
            store.set_blender_channel(parent, idx, norm);
            Some(parent)
        }
        BlenderHitKind::Hex => {
            // `Hex` hits redirect focus to the hex TextInput sibling
            // registered with the same parent id hierarchy. The
            // BlenderHit NodeId IS the hex TextInput id in our
            // registration scheme (BLENDER_HEX = NodeId 604 which is
            // registered as TextInput, not as BlenderHit). This arm
            // is a no-op: focus was already set by the Down handler.
            // Return None so no spurious ValueChanged fires.
            None
        }
        BlenderHitKind::PaletteSwatch(swatch_idx) => {
            // Read the swatch colour from the static default palette
            // (no alloc: slice index into a const array).
            let palette = crate::widget::default_palette();
            if let Some(color) = palette.swatches.get(swatch_idx as usize).copied() {
                store.set_blender_value(parent, color);
                Some(parent)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractiveState;
    use crate::widget::ButtonState;
    use crate::zones::Rect;
    use ph2d_a11y::NodeId;
    use ph2d_host::{Modifiers, PointerSource};

    fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            timestamp_ns: 0,
        }
    }

    fn key(kc: u32, shift: bool) -> KeyEvent {
        KeyEvent {
            keycode: kc,
            modifiers: Modifiers {
                shift,
                ctrl: false,
                alt: false,
                meta: false,
            },
            kind: KeyKind::Down,
            timestamp_ns: 0,
        }
    }

    fn one_button_setup() -> (WidgetStore, HitIndex) {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        (store, hits)
    }

    #[test]
    fn pointer_move_into_widget_sets_hot_id_and_hover_state() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 50.0, 25.0),
            &arena,
        );
        assert_eq!(store.hot_id(), Some(NodeId(7)));
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Hovered));
    }

    #[test]
    fn pointer_move_out_clears_hot_and_reverts_state() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 50.0, 25.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 500.0, 500.0),
            &arena,
        );
        assert_eq!(store.hot_id(), None);
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Normal));
    }

    #[test]
    fn button_down_sets_pressed_and_emits_focus() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Pressed));
        assert_eq!(store.active_id(), Some(NodeId(7)));
        assert_eq!(evts, &[WidgetEvent::Focus(NodeId(7))]);
    }

    #[test]
    fn button_down_then_up_emits_click_and_clears_active() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Click(NodeId(7))]);
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn button_down_then_drag_out_then_up_does_not_click() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 500.0, 500.0),
            &arena,
        );
        assert_eq!(evts, &[]);
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn disabled_button_does_not_focus_or_press_on_down() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Button {
                state: ButtonState::Disabled,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[]);
        assert_eq!(store.active_id(), None);
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Disabled));
    }

    #[test]
    fn toggle_click_flips_on_and_emits_toggled() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Toggle {
                state: ToggleState::Normal,
                on: false,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Toggled(NodeId(7))]);
        let (_, on) = store.toggle(NodeId(7)).unwrap();
        assert!(on);
    }

    #[test]
    fn tab_cycles_focus_forward() {
        let mut store = WidgetStore::with_capacity(4);
        for id in [1, 2, 3] {
            store.register(
                NodeId(id),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Focus(NodeId(1))]);
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(2)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(3)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(1)), "wraps around");
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut store = WidgetStore::with_capacity(4);
        for id in [1, 2, 3] {
            store.register(
                NodeId(id),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_TAB, true), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(3)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, true), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(2)));
    }

    #[test]
    fn enter_on_focused_button_emits_click() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Click(NodeId(1))]);
    }

    #[test]
    fn escape_blurs_focus() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Blur(NodeId(1))]);
        assert_eq!(store.focus_id(), None);
    }

    #[test]
    fn slider_down_jumps_to_pointer_and_emits_value_changed() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 75.0, 10.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(7)))
        );
        let (state, v) = store.slider(NodeId(7)).unwrap();
        assert_eq!(state, SliderState::Dragging);
        assert!((v - 0.75).abs() < 0.01, "expected 0.75, got {v}");
    }

    #[test]
    fn slider_drag_emits_value_changed_per_move() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 25.0, 10.0),
            &arena,
        );
        // Drag the cursor outside the rect — value still updates,
        // because active drag persists.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 90.0, 200.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_)))
        );
        let (_, v) = store.slider(NodeId(7)).unwrap();
        assert!((v - 0.90).abs() < 0.01);
    }

    #[test]
    fn slider_release_clears_active_and_does_not_emit_click() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 10.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 10.0),
            &arena,
        );
        assert!(
            !evts.iter().any(|e| matches!(e, WidgetEvent::Click(_))),
            "Slider should not emit Click on release"
        );
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn vertical_slider_inverts_y_to_value() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Vertical,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 20.0, 100.0));
        let arena = Bump::new();
        // Down at the top of the rect → value should be near 1.0.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 10.0, 5.0),
            &arena,
        );
        let (_, v) = store.slider(NodeId(7)).unwrap();
        assert!((v - 0.95).abs() < 0.01, "expected ~0.95 at top, got {v}");
    }

    #[test]
    fn checkbox_click_cycles_unchecked_to_checked() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 18.0, 18.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 9.0, 9.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 9.0, 9.0),
            &arena,
        );
        assert!(evts.iter().any(|e| matches!(e, WidgetEvent::Toggled(_))));
        let (_, v) = store.checkbox(NodeId(7)).unwrap();
        assert_eq!(v, CheckboxValue::Checked);
    }

    #[test]
    fn checkbox_indeterminate_then_click_yields_checked() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Indeterminate,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 18.0, 18.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 9.0, 9.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 9.0, 9.0),
            &arena,
        );
        let (_, v) = store.checkbox(NodeId(7)).unwrap();
        assert_eq!(v, CheckboxValue::Checked);
    }

    #[test]
    fn key_up_event_is_ignored() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), InteractiveState::Plain);
        let arena = Bump::new();
        let evts = dispatch_key(
            &mut store,
            KeyEvent {
                keycode: KEY_TAB,
                modifiers: Modifiers::default(),
                kind: KeyKind::Up,
                timestamp_ns: 0,
            },
            &arena,
        );
        assert_eq!(evts, &[]);
    }

    // -----------------------------------------------------------------
    // Phase C — TextInput / NumberInput / Combobox / Dropdown
    // -----------------------------------------------------------------

    use crate::widget::TextInputState;

    fn text_input(text: &str) -> InteractiveState {
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: text.into(),
            caret: text.len(),
        }
    }

    #[test]
    fn text_input_char_insert_advances_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input(""));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_text_input(&mut store, 'a', &arena);
        assert!(matches!(evts, [WidgetEvent::TextChanged(_)]));
        let evts2 = dispatch_text_input(&mut store, 'b', &arena);
        assert!(matches!(evts2, [WidgetEvent::TextChanged(_)]));
        assert_eq!(store.text(NodeId(1)), Some("ab"));
    }

    #[test]
    fn text_input_backspace_at_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input("hello"));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        assert!(matches!(evts, [WidgetEvent::TextChanged(_)]));
        assert_eq!(store.text(NodeId(1)), Some("hell"));
    }

    #[test]
    fn text_input_arrow_left_moves_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input("xyz"));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_ARROW_LEFT, false), &arena);
        // The caret moved (no text changed). Reading caret directly:
        if let Some(InteractiveState::TextInput { caret, .. }) = store.get(NodeId(1)) {
            assert_eq!(*caret, 2);
        } else {
            panic!("expected TextInput");
        }
    }

    #[test]
    fn text_input_unfocused_ignores_input() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input(""));
        let arena = Bump::new();
        let evts = dispatch_text_input(&mut store, 'x', &arena);
        assert_eq!(evts, &[]);
        assert_eq!(store.text(NodeId(1)), Some(""));
    }

    #[test]
    fn number_input_arrow_up_increments() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 5.0,
                buffer: "5".into(),
                caret: 1,
                last_committed: 5.0,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ARROW_UP, false), &arena);
        assert!(matches!(evts, [WidgetEvent::ValueChanged(_)]));
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get(NodeId(1)) {
            assert!((value - 6.0).abs() < f64::EPSILON);
        } else {
            panic!("expected NumberInput");
        }
    }

    fn make_number_store(value: f64) -> WidgetStore {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value,
                buffer: super::super::state::format_number(value),
                caret: super::super::state::format_number(value).len(),
                last_committed: value,
            },
        );
        store.set_focus(Some(NodeId(1)));
        store
    }

    #[test]
    fn number_input_typing_replaces_buffer_and_commits_on_enter() {
        let mut store = make_number_store(5.0);
        let arena = Bump::new();
        // Erase '5' then type "1.25".
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['1', '.', '2', '5'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        // Buffer reflects edits but value has not yet committed.
        let (_, value, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "1.25");
        assert!((value - 5.0).abs() < f64::EPSILON);
        // Enter commits.
        let evts = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_)))
        );
        let (_, value, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 1.25).abs() < 1e-9);
    }

    #[test]
    fn number_input_escape_reverts_to_last_committed() {
        let mut store = make_number_store(7.0);
        let arena = Bump::new();
        for ch in ['9', '9'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let (_, _, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "799");
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert!(evts.iter().any(|e| matches!(e, WidgetEvent::Blur(_))));
        let (_, value, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 7.0).abs() < f64::EPSILON);
        assert_eq!(buf, "7");
    }

    #[test]
    fn number_input_unparsable_buffer_reverts_on_commit() {
        let mut store = make_number_store(3.0);
        let arena = Bump::new();
        // Replace the existing single digit with garbage.
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['e', 'e', 'e'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, value, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 3.0).abs() < f64::EPSILON);
        assert_eq!(buf, "3");
    }

    #[test]
    fn number_input_filters_non_numeric_chars() {
        let mut store = make_number_store(0.0);
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        // Typing letters should be filtered.
        for ch in ['a', 'b', 'X', '!', ' '] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let (_, _, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "");
    }

    #[test]
    fn number_input_set_value_syncs_buffer_when_unfocused() {
        let mut store = make_number_store(0.0);
        store.set_focus(None); // simulate unfocused
        store.set_number_value(NodeId(1), 0.42);
        let (_, value, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 0.42).abs() < 1e-9);
        assert_eq!(buf, "0.420");
    }

    #[test]
    fn number_input_set_value_preserves_buffer_when_focused() {
        let mut store = make_number_store(0.0);
        // Type a partial edit.
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['1', '.', '2'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        // While focused, programmatic set_number_value should NOT
        // clobber the in-progress buffer.
        store.set_number_value(NodeId(1), 9.99);
        let (_, value, buf, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 9.99).abs() < 1e-9);
        assert_eq!(buf, "1.2");
    }

    #[test]
    fn slider_drag_propagates_to_linked_number_input() {
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".into(),
                caret: 1,
                last_committed: 0.0,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        let mut hits = HitIndex::new();
        hits.register(NodeId(1), Rect::new(0.0, 0.0, 100.0, 30.0));
        let arena = Bump::new();
        // Down at x=50 → value 0.5 → number value 0.5.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let (_, num_value, num_buf, _) = store.number_input(NodeId(2)).unwrap();
        assert!((num_value - 0.5).abs() < 1e-6);
        assert_eq!(num_buf, "0.500");
    }

    #[test]
    fn number_commit_propagates_to_linked_slider() {
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value: 0.0,
                buffer: "0".into(),
                caret: 1,
                last_committed: 0.0,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        store.set_focus(Some(NodeId(2)));
        let arena = Bump::new();
        // Erase '0' then type "0.75".
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['0', '.', '7', '5'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, sv) = store.slider(NodeId(1)).unwrap();
        assert!((sv - 0.75).abs() < 1e-5);
    }

    #[test]
    fn blender_wheel_click_mutates_picker_value() {
        use crate::interaction::BlenderHitKind;
        use crate::widget::{ChannelMode, InterpolationMode};
        use ph2d_tokens::ColorValue;
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(100),
            InteractiveState::BlenderPicker {
                value: ColorValue::from_rgba8(231, 231, 231, 255),
                channel_mode: ChannelMode::Rgb,
                interpolation: InterpolationMode::Perceptual,
                active_palette: 0,
            },
        );
        store.register(
            NodeId(101),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::Wheel,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(101), Rect::new(0.0, 0.0, 100.0, 100.0));
        let arena = Bump::new();
        // Click right-edge → hue ≈ 0°, sat ≈ 1.0 → red-leaning value.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 95.0, 50.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_))),
            "expected a ValueChanged event from wheel click"
        );
        let (new_value, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        // Value should have rotated away from neutral grey.
        assert!(
            new_value.rgba != [231, 231, 231, 255],
            "picker value should change after wheel click"
        );
    }

    #[test]
    fn linked_number_value_clamps_into_slider_range() {
        // NumberInput accepts arbitrary f64; the slider snapshot
        // clamps to [0..1] without panicking on out-of-range commits.
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value: 0.5,
                buffer: "0.5".into(),
                caret: 3,
                last_committed: 0.5,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        store.set_focus(Some(NodeId(2)));
        let arena = Bump::new();
        for _ in 0..3 {
            let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        }
        for ch in ['9', '9'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, sv) = store.slider(NodeId(1)).unwrap();
        assert!((sv - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dropdown_click_toggles_open() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Dropdown {
                state: crate::widget::DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(1), Rect::new(0.0, 0.0, 100.0, 30.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 15.0),
            &arena,
        );
        let open_after_first = matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        assert!(open_after_first);
        // Second click closes it.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 15.0),
            &arena,
        );
        let open_after_second = matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        assert!(!open_after_second);
    }

    #[test]
    fn escape_closes_open_dropdown_without_blur() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Dropdown {
                state: crate::widget::DropdownState::Normal,
                open: true,
                selected_index: None,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert_eq!(evts, &[]); // closing the dropdown does not blur
        assert_eq!(store.focus_id(), Some(NodeId(1)));
        assert!(matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: false, .. })
        ));
    }

    #[test]
    fn combobox_text_input_appends_to_query() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Combobox {
                state: crate::widget::ComboboxState::Normal,
                open: false,
                query: String::new(),
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let _ = dispatch_text_input(&mut store, 's', &arena);
        let _ = dispatch_text_input(&mut store, 'p', &arena);
        assert_eq!(store.text(NodeId(1)), Some("sp"));
    }

    // -----------------------------------------------------------------
    // BlenderColorPicker sub-control dispatch (B4 fix)
    // -----------------------------------------------------------------

    fn blender_picker_setup() -> (WidgetStore, HitIndex) {
        use crate::interaction::BlenderHitKind;
        use crate::widget::{ChannelMode, InterpolationMode};
        use ph2d_tokens::ColorValue;

        let mut store = WidgetStore::with_capacity(32);
        store.register(
            NodeId(100),
            InteractiveState::BlenderPicker {
                value: ColorValue::from_rgba8(128, 64, 32, 255),
                channel_mode: ChannelMode::Rgb,
                interpolation: InterpolationMode::Perceptual,
                active_palette: 0,
            },
        );
        // Channel slider shims (0..3 = R, G, B, A).
        for idx in 0u8..4 {
            store.register(
                NodeId(200 + idx as u64),
                InteractiveState::BlenderHit {
                    parent: NodeId(100),
                    kind: BlenderHitKind::ChannelSlider(idx),
                },
            );
        }
        // Interpolation toggle shims.
        store.register(
            NodeId(210),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::InterpolationLinear,
            },
        );
        store.register(
            NodeId(211),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::InterpolationPerceptual,
            },
        );
        // Channel mode toggle shims.
        store.register(
            NodeId(212),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::ChannelRgb,
            },
        );
        store.register(
            NodeId(213),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::ChannelHsv,
            },
        );
        // Palette swatch shims.
        for swatch in 0u8..4 {
            store.register(
                NodeId(220 + swatch as u64),
                InteractiveState::BlenderHit {
                    parent: NodeId(100),
                    kind: BlenderHitKind::PaletteSwatch(swatch),
                },
            );
        }
        let mut hits = HitIndex::new();
        // Channel slider rects (100 px wide each).
        for idx in 0u8..4 {
            hits.register(
                NodeId(200 + idx as u64),
                Rect::new(0.0, idx as f32 * 30.0, 100.0, 22.0),
            );
        }
        // Toggle half-rects.
        hits.register(NodeId(210), Rect::new(0.0, 200.0, 100.0, 28.0));
        hits.register(NodeId(211), Rect::new(100.0, 200.0, 100.0, 28.0));
        hits.register(NodeId(212), Rect::new(0.0, 240.0, 100.0, 28.0));
        hits.register(NodeId(213), Rect::new(100.0, 240.0, 100.0, 28.0));
        // Swatch rects.
        for swatch in 0u8..4 {
            hits.register(
                NodeId(220 + swatch as u64),
                Rect::new(swatch as f32 * 30.0, 300.0, 24.0, 24.0),
            );
        }
        (store, hits)
    }

    #[test]
    fn channel_slider_down_mutates_red_channel() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Red slider (NodeId 200) rect is x: 0..100. Click at x=50 → R ≈ 128.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 11.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from channel slider hit"
        );
        let (v, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert!(
            (v.rgba[0] as f32 / 255.0 - 0.5).abs() < 0.01,
            "red channel should be ≈128 (0.5 * 255), got {}",
            v.rgba[0]
        );
        // Other channels should be unchanged.
        assert_eq!(v.rgba[1], 64, "green channel unchanged");
        assert_eq!(v.rgba[2], 32, "blue channel unchanged");
    }

    #[test]
    fn channel_slider_down_mutates_alpha_channel() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Alpha slider (NodeId 203) rect is y offset at 90. Click at x=0 → A = 0.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 0.0, 101.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from alpha channel slider"
        );
        let (v, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(v.rgba[3], 0, "alpha channel should be 0 after click at x=0");
    }

    #[test]
    fn interp_toggle_linear_switches_mode() {
        use crate::widget::InterpolationMode;
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 214.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100)))
        );
        let (_, _, interp, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(interp, InterpolationMode::Linear);
    }

    #[test]
    fn channel_mode_toggle_hsv_switches_mode() {
        use crate::widget::ChannelMode;
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 150.0, 254.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100)))
        );
        let (_, mode, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(mode, ChannelMode::Hsv);
    }

    #[test]
    fn palette_swatch_click_changes_picker_value() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Click swatch 2 (NodeId 222), which maps to default_palette().swatches[2].
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 60.0 + 12.0, 312.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from swatch click"
        );
        let (new_val, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        let expected = crate::widget::default_palette().swatches[2];
        assert_eq!(
            new_val.rgba, expected.rgba,
            "picker value should match swatch 2 of default palette"
        );
    }
}
