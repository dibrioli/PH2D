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
                        events.push(WidgetEvent::Blur(old));
                    }
                    store.set_focus(Some(id));
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
/// dispatcher can decide whether to emit `ValueChanged`).
fn update_drag_value(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> bool {
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
            // Vertical slider: y=top → value=1, y=bottom → value=0.
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
                apply_click(store, id, &mut events);
            }
        }
        KEY_ESCAPE => {
            if let Some(id) = store.focus_id() {
                store.set_focus(None);
                events.push(WidgetEvent::Blur(id));
            }
        }
        _ => {}
    }
    events.into_bump_slice()
}

/// Character input from the IME / keyboard. Phase C wires this to
/// TextInput / Combobox; Phase A returns empty.
pub fn dispatch_text_input<'frame>(
    store: &mut WidgetStore,
    ch: char,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let _ = (store, ch);
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    events.into_bump_slice()
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
        Some(InteractiveState::Button { .. }) | Some(InteractiveState::Plain) => {
            events.push(WidgetEvent::Click(id));
        }
        // Phases C-D add per-kind click semantics (Tabs select,
        // Modal dismiss, Dropdown open, etc.).
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
                events.push(WidgetEvent::Blur(old));
            }
            if store.focus_id() != Some(id) {
                store.set_focus(Some(id));
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
}
