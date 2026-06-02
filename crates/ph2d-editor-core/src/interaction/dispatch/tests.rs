use super::*;
use crate::interaction::{HitIndex, InteractiveState, PainterLayerDrop};
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, ToggleState,
};
use crate::zones::Rect;
use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerEvent, PointerKind, PointerSource};

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
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
fn hierarchy_eye_companion_click_emits_click_event() {
    // Regression: companion NodeIds for the hierarchy eye-toggle
    // (and chevron) are registered in HitIndex only — never in
    // WidgetStore (the painter has no &mut store). Before the
    // M14.6A bugfix, the `is_focusable` gate in PointerKind::Down
    // rejected unregistered ids, so no `active` was captured and
    // Up emitted nothing. Now the dispatcher special-cases these
    // companions and routes them through the regular Up→Click
    // path; this test pins that behavior.
    use crate::ids;
    let mut store = WidgetStore::with_capacity(4);
    // Simulate a live hierarchy row (registered as Plain by
    // `hierarchy::populate_live`); only the companion is missing
    // from the store — which is the realistic scenario.
    let row_id = ph2d_a11y::NodeId(412);
    store.register(row_id, InteractiveState::Plain);
    let eye_id = ids::hier_eye_companion(row_id);
    let mut hits = HitIndex::new();
    hits.register(row_id, Rect::new(0.0, 0.0, 200.0, 20.0));
    hits.register(eye_id, Rect::new(170.0, 0.0, 24.0, 20.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 182.0, 10.0),
        &arena,
    );
    // Active must be set even though the companion isn't in store.
    assert_eq!(store.active_id(), Some(eye_id));
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 182.0, 10.0),
        &arena,
    );
    assert_eq!(evts, &[WidgetEvent::Click(eye_id)]);
    assert_eq!(store.active_id(), None);
}

#[test]
fn hierarchy_expand_companion_click_emits_click_event() {
    // Same contract as the eye test above, for the chevron
    // companion (collapse/expand). Lives separately so a
    // regression on one toggle bit doesn't silently break both.
    use crate::ids;
    let mut store = WidgetStore::with_capacity(4);
    let row_id = ph2d_a11y::NodeId(413);
    store.register(row_id, InteractiveState::Plain);
    let chev_id = ids::hier_expand_companion(row_id);
    let mut hits = HitIndex::new();
    hits.register(row_id, Rect::new(0.0, 0.0, 200.0, 20.0));
    hits.register(chev_id, Rect::new(4.0, 4.0, 12.0, 12.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 10.0, 10.0),
        &arena,
    );
    assert_eq!(store.active_id(), Some(chev_id));
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 10.0, 10.0),
        &arena,
    );
    assert_eq!(evts, &[WidgetEvent::Click(chev_id)]);
}

#[test]
fn hierarchy_drag_in_live_mode_emits_reparent_intent() {
    // Pre-M14.6B regression: dragging a live (ECS-bridge) row
    // used `is_hierarchy_entity_id` which only matched the
    // fixture range 400..=411 — so live rows (NodeIds in the
    // 100_000+ range) never became drag candidates and Up
    // emitted no `HierReparent`. This test pins the new
    // contract: the row set published via
    // `set_hierarchy_row_ids` is the single source of truth.
    let mut store = WidgetStore::with_capacity(8);
    // Two "live" rows from the bridge — far outside the
    // fixture range that the old code looked at.
    let parent_id = ph2d_a11y::NodeId(100_000);
    let dragged_id = ph2d_a11y::NodeId(100_001);
    store.register(parent_id, InteractiveState::Plain);
    store.register(dragged_id, InteractiveState::Plain);
    let mut row_set = std::collections::BTreeSet::new();
    row_set.insert(parent_id);
    row_set.insert(dragged_id);
    store.set_hierarchy_row_ids(row_set);
    store.set_hierarchy_order(vec![parent_id, dragged_id]);
    let mut hits = HitIndex::new();
    // Parent row at y=0..20, dragged row at y=30..50.
    hits.register(parent_id, Rect::new(0.0, 0.0, 200.0, 20.0));
    hits.register(dragged_id, Rect::new(0.0, 30.0, 200.0, 20.0));
    let arena = Bump::new();
    // Down on dragged row.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 100.0, 40.0),
        &arena,
    );
    // Move enough to cross the drag threshold (8 px in any axis;
    // bumping the cursor 50 px up clears it comfortably).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 100.0, 10.0),
        &arena,
    );
    // Up over the middle of the parent row → HierDrop::Inside,
    // which emits HierReparent { dragged, new_parent: Some(parent), before: None }.
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 100.0, 10.0),
        &arena,
    );
    assert!(
        evts.iter().any(|e| matches!(
            e,
            WidgetEvent::HierReparent {
                dragged,
                new_parent: Some(np),
                before: None,
                after: _,
            } if *dragged == dragged_id && *np == parent_id
        )),
        "expected HierReparent Inside({parent_id:?}); got {evts:?}"
    );
}

#[test]
fn painter_layer_drag_emits_reparent_with_resolved_drop() {
    // W3 T3.8: Primary Down on a painter layer row + a threshold-crossing
    // Move + Up over the middle of a target row resolves to
    // `PainterLayerDrop::Inside` and emits a `PainterLayerReparent`. The
    // dispatch does NO structure mutation — the painter tool applies it.
    let mut store = WidgetStore::with_capacity(8);
    let target_id = ph2d_a11y::NodeId(200_000);
    let dragged_id = ph2d_a11y::NodeId(200_001);
    store.register(target_id, InteractiveState::Plain);
    store.register(dragged_id, InteractiveState::Plain);
    let mut row_set = std::collections::BTreeSet::new();
    row_set.insert(target_id);
    row_set.insert(dragged_id);
    store.set_painter_layer_row_ids(row_set);
    let mut hits = HitIndex::new();
    // Target row y=0..20 (Inside band 6..14), dragged row y=30..50.
    hits.register(target_id, Rect::new(0.0, 0.0, 200.0, 20.0));
    hits.register(dragged_id, Rect::new(0.0, 30.0, 200.0, 20.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 100.0, 40.0),
        &arena,
    );
    // Move up past the 5px threshold (dy = 30).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 100.0, 10.0),
        &arena,
    );
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 100.0, 10.0),
        &arena,
    );
    assert!(
        evts.iter().any(|e| matches!(
            e,
            WidgetEvent::PainterLayerReparent {
                dragged,
                drop: PainterLayerDrop::Inside(t),
            } if *dragged == dragged_id && *t == target_id
        )),
        "expected PainterLayerReparent Inside({target_id:?}); got {evts:?}"
    );
}

#[test]
fn painter_layer_sub_threshold_click_emits_no_reparent() {
    // A Down+Up on the same row WITHOUT crossing the drag threshold is a
    // click, not a drag → no reparent (row select stays the panel's job).
    let mut store = WidgetStore::with_capacity(8);
    let row_id = ph2d_a11y::NodeId(200_010);
    store.register(row_id, InteractiveState::Plain);
    let mut row_set = std::collections::BTreeSet::new();
    row_set.insert(row_id);
    store.set_painter_layer_row_ids(row_set);
    let mut hits = HitIndex::new();
    hits.register(row_id, Rect::new(0.0, 0.0, 200.0, 20.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 100.0, 10.0),
        &arena,
    );
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 100.0, 11.0),
        &arena,
    );
    assert!(
        !evts
            .iter()
            .any(|e| matches!(e, WidgetEvent::PainterLayerReparent { .. })),
        "a sub-threshold click must not emit a reparent; got {evts:?}"
    );
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
        selection_anchor: None,
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
            selection_anchor: None,
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
            buffer: super::super::format_number(value),
            caret: super::super::format_number(value).len(),
            last_committed: value,
            selection_anchor: None,
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
    let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
    assert_eq!(buf, "1.25");
    assert!((value - 5.0).abs() < f64::EPSILON);
    // Enter commits.
    let evts = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(_)))
    );
    let (_, value, _, _, _) = store.number_input(NodeId(1)).unwrap();
    assert!((value - 1.25).abs() < 1e-9);
}

#[test]
fn number_input_escape_reverts_to_last_committed() {
    let mut store = make_number_store(7.0);
    let arena = Bump::new();
    for ch in ['9', '9'] {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    let (_, _, buf, _, _) = store.number_input(NodeId(1)).unwrap();
    assert_eq!(buf, "799");
    let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
    assert!(evts.iter().any(|e| matches!(e, WidgetEvent::Blur(_))));
    let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
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
    let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
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
    let (_, _, buf, _, _) = store.number_input(NodeId(1)).unwrap();
    assert_eq!(buf, "");
}

#[test]
fn number_input_set_value_syncs_buffer_when_unfocused() {
    let mut store = make_number_store(0.0);
    store.set_focus(None); // simulate unfocused
    store.set_number_value(NodeId(1), 0.42);
    let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
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
    let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
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
            selection_anchor: None,
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
    let (_, num_value, num_buf, _, _) = store.number_input(NodeId(2)).unwrap();
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
            selection_anchor: None,
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
            hsv_h: 0.0,
            hsv_s: 1.0,
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
            selection_anchor: None,
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
            caret: 0,
            selection_anchor: None,
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
            hsv_h: 0.07,
            hsv_s: 0.75,
        },
    );
    // Seed the picker's palette so swatch clicks have something
    // to read (the default 12 colors from `default_palette`).
    store.init_blender_palette(
        NodeId(100),
        crate::widget::default_palette().swatches.clone(),
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
    // Channel slider track rects — painter now registers only the
    // inner track (no label/value chip), so x=0..110 covers the
    // interactive region directly.
    for idx in 0u8..4 {
        hits.register(
            NodeId(200 + idx as u64),
            Rect::new(0.0, idx as f32 * 30.0, 110.0, 22.0),
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
    // Red slider track (NodeId 200) is x: 0..110. Click at x=55
    // (midpoint) → R ≈ 128 (0.5 * 255).
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 55.0, 11.0),
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

// ── Multi-line click mapping (TextArea) ────────────────────────────────

fn textarea_setup(initial: &str) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(42),
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: initial.to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let rect = Rect::new(100.0, 200.0, 240.0, 60.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(42), rect);
    (store, hits, rect)
}

#[test]
fn textarea_click_line2_places_caret_on_line2() {
    // Two lines: "abc" (3 bytes) + '\n' + "defgh" (5 bytes). Total 9.
    let (mut store, hits, rect) = textarea_setup("abc\ndefgh");
    let arena = Bump::new();
    // Click well into line 2's y range (line_h ~ 18, padding 8,
    // so y ≈ rect.y + 8 + 18 + 4 = rect.y + 30 hits line 2).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 12.0 + 1.0, rect.y + 32.0),
        &arena,
    );
    let caret = match store.get(NodeId(42)) {
        Some(InteractiveState::TextInput { caret, .. }) => *caret,
        _ => 0,
    };
    // Line 2 starts at byte 4 (`abc` + '\n'). Caret at byte 4 means
    // start of line 2 — exactly what the user wants when clicking
    // near the left of line 2.
    assert!(
        (4..=9).contains(&caret),
        "expected caret on line 2 (>= byte 4), got {caret}"
    );
}

#[test]
fn textarea_click_far_right_snaps_to_end_of_line() {
    // Line 1 is short ("abc"); clicking far right of line 1 must
    // not jump into line 2 — caret should land at byte 3 (end of
    // line 1).
    let (mut store, hits, rect) = textarea_setup("abc\ndefghijklmnop");
    let arena = Bump::new();
    // Click on line 1 (y ≈ rect.y + 12, inside first line band)
    // at the far right of the rect.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + rect.w - 4.0, rect.y + 12.0),
        &arena,
    );
    let caret = match store.get(NodeId(42)) {
        Some(InteractiveState::TextInput { caret, .. }) => *caret,
        _ => 99,
    };
    assert_eq!(
        caret, 3,
        "click past end of line 1 must snap to end-of-line (byte 3), got {caret}"
    );
}

// ── Combobox clear-✕ button ────────────────────────────────────────────

fn combobox_setup(initial_query: &str) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(55),
        InteractiveState::Combobox {
            state: crate::widget::ComboboxState::Normal,
            open: false,
            query: initial_query.to_string(),
            caret: initial_query.len(),
            selection_anchor: None,
        },
    );
    let rect = Rect::new(50.0, 100.0, 240.0, 32.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(55), rect);
    (store, hits, rect)
}

#[test]
fn combobox_clear_x_wipes_query_and_emits_text_changed() {
    let (mut store, hits, rect) = combobox_setup("spike");
    let arena = Bump::new();
    let probe = crate::widget::Combobox::new(NodeId(55), "", vec![]).query("spike");
    let xr = probe
        .clear_button_rect(rect)
        .expect("clear rect must exist");
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, xr.x + xr.w * 0.5, xr.y + xr.h * 0.5),
        &arena,
    );
    let q = match store.get(NodeId(55)) {
        Some(InteractiveState::Combobox { query, .. }) => query.clone(),
        _ => "<missing>".to_string(),
    };
    assert!(
        q.is_empty(),
        "expected empty query after X click, got {q:?}"
    );
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::TextChanged(id) if *id == NodeId(55))),
        "expected TextChanged(55) after clear, got {evts:?}"
    );
}

#[test]
fn combobox_no_clear_x_when_query_empty() {
    // Clicking on the right side of an empty Combobox should not
    // mutate any state (no clear, no error). It just focuses +
    // places caret at 0.
    let (mut store, hits, rect) = combobox_setup("");
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(
            PointerKind::Down,
            rect.x + rect.w - 8.0,
            rect.y + rect.h * 0.5,
        ),
        &arena,
    );
    // Still empty.
    let q = match store.get(NodeId(55)) {
        Some(InteractiveState::Combobox { query, .. }) => query.clone(),
        _ => "<missing>".to_string(),
    };
    assert!(q.is_empty());
}

// ── NumberInput stepper buttons ────────────────────────────────────────

fn number_input_setup(initial: f64) -> (WidgetStore, HitIndex, Rect) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(77),
        InteractiveState::NumberInput {
            state: crate::widget::TextInputState::Normal,
            value: initial,
            buffer: super::super::format_number(initial),
            caret: 0,
            last_committed: initial,
            selection_anchor: None,
        },
    );
    let rect = Rect::new(0.0, 0.0, 80.0, 28.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(77), rect);
    (store, hits, rect)
}

#[test]
fn number_input_up_arrow_increments_integer() {
    let (mut store, hits, rect) = number_input_setup(5.0);
    let arena = Bump::new();
    let probe = crate::widget::NumberInput::new(NodeId(77), "", 5.0);
    let up = probe.up_rect(rect);
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5),
        &arena,
    );
    let v = match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput { value, .. }) => *value,
        _ => -1.0,
    };
    assert!((v - 6.0).abs() < f64::EPSILON, "expected 6.0 got {v}");
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(77)))
    );
}

#[test]
fn number_input_down_arrow_decrements_fractional_by_001() {
    // Buffer "0.50" contains '.', so the step heuristic picks 0.01.
    let (mut store, hits, rect) = number_input_setup(0.5);
    let arena = Bump::new();
    let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.5);
    let down = probe.down_rect(rect);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(
            PointerKind::Down,
            down.x + down.w * 0.5,
            down.y + down.h * 0.5,
        ),
        &arena,
    );
    let v = match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput { value, .. }) => *value,
        _ => -1.0,
    };
    assert!((v - 0.49).abs() < 1e-6, "expected 0.49 got {v}");
}

/// Read NumberInput value via the store accessor — avoids
/// boilerplate in the M14.A drag tests below.
fn read_value(store: &WidgetStore, id: NodeId) -> f64 {
    store.number_value(id).expect("NumberInput value")
}

/// M14.A: Down on the body (NOT the stepper) seeds a drag
/// candidate. Move right past the threshold flips into slider
/// mode with the horizontal rate (50× step / px) — fast.
#[test]
fn number_input_body_drag_horizontal_uses_fast_rate() {
    let (mut store, hits, rect) = number_input_setup(5.0);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 1: cross the 4 px threshold. Promotion re-anchors
    // `last_x` to here so the threshold-crossing distance is NOT
    // added as a value jump (post-2026-05-24 canon).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 15.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 2: 10 px past the anchor → dx=10, dy=0 → delta = 10*50*1 = 500.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 25.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let v = read_value(&store, NodeId(77));
    assert!(
        (v - 505.0).abs() < 1e-6,
        "expected 505.0 (5 + 10*50*1) got {v}"
    );
}

/// M14.A: vertical drag uses the slow rate (5× step / px) and
/// inverts dy so cursor-up = positive delta (screen coords have
/// y growing down).
#[test]
fn number_input_body_drag_vertical_uses_slow_rate_and_inverts() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 1: -5 px crosses threshold + locks vertical, no delta
    // (promote re-anchors).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(
            PointerKind::Move,
            rect.x + 10.0,
            rect.y + rect.h * 0.5 - 5.0,
        ),
        &arena,
    );
    // Move 2: another -10 px → dy=-10 → delta = -(-10)*5*1 = 50.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(
            PointerKind::Move,
            rect.x + 10.0,
            rect.y + rect.h * 0.5 - 15.0,
        ),
        &arena,
    );
    let v = read_value(&store, NodeId(77));
    assert!((v - 50.0).abs() < 1e-6, "expected 50 (0 + 10*5*1) got {v}");
}

/// M14.A: holding Shift multiplies the delta by 0.001 — Blender-
/// style fine adjustment. With horizontal 50× × 0.001 = 0.05 / px,
/// a 10 px drag yields 0.5 step-units of change.
#[test]
fn number_input_body_drag_with_shift_uses_fine_rate() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    store.set_shift_held(true);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 1: cross threshold (no delta — post-2026-05-24 re-anchor).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 15.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 2: 10 px past anchor → delta = 10*50*0.001 = 0.5.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 25.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let v = read_value(&store, NodeId(77));
    assert!(
        (v - 0.5).abs() < 1e-6,
        "expected 0.5 (10*50*0.001*1) got {v}"
    );
}

/// M14.A: the axis lock survives off-axis wobble after the
/// threshold cross. User crosses with dx > dy → horizontal locks
/// → subsequent drift into the vertical direction is ignored,
/// because the only way to release the axis is a fresh Down.
#[test]
fn number_input_body_drag_locked_axis_persists_through_off_axis_wobble() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let down_x = rect.x + 10.0;
    let down_y = rect.y + rect.h * 0.5;
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, down_x, down_y),
        &arena,
    );
    // Move 1: cross threshold horizontally → axis locks horizontal,
    // promote re-anchors `last_x` to here, no delta.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 5.0, down_y),
        &arena,
    );
    assert!((read_value(&store, NodeId(77)) - 0.0).abs() < 1e-6);
    // Move 2: another 5 px right → step_dx=5 → delta = 5*50 = 250.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 10.0, down_y),
        &arena,
    );
    assert!((read_value(&store, NodeId(77)) - 250.0).abs() < 1e-6);
    // Now drift vertically a lot (dy=86 with x staying put). Without
    // the lock, vertical would dominate → delta = -(-86)*5 = 430 →
    // value would jump to 680. With horizontal lock, dy is zeroed
    // and step_dx=0 → value stays 250.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 10.0, down_y + 86.0),
        &arena,
    );
    let v = read_value(&store, NodeId(77));
    assert!(
        (v - 250.0).abs() < 1e-6,
        "horizontal axis lock leaked: expected 250.0 got {v}"
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, down_x + 10.0, down_y + 86.0),
        &arena,
    );
    assert!(store.number_input_drag().is_none());
}

/// M14.A: at the moment the threshold flips, the dominant axis
/// is decided and locked on the drag state. A drag that's
/// predominantly horizontal (dx 20, dy 5) ignores the dy
/// contribution; the formula uses dx only.
#[test]
fn number_input_body_drag_locks_to_dominant_axis() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let down_x = rect.x + 10.0;
    let down_y = rect.y + rect.h * 0.5;
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, down_x, down_y),
        &arena,
    );
    // Move 1: horizontal-dominant cross (dx=20, dy=5) → locks
    // horizontal, promote re-anchors to (down+20, down+5). No delta
    // applied this frame.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 20.0, down_y + 5.0),
        &arena,
    );
    assert!((read_value(&store, NodeId(77)) - 0.0).abs() < 1e-6);
    // Move 2: another 20 px right, 5 px down → step_dx=20, step_dy=5.
    // With axis locked horizontal, dy is zeroed → delta = 20*50 = 1000.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 40.0, down_y + 10.0),
        &arena,
    );
    let v = read_value(&store, NodeId(77));
    assert!(
        (v - 1000.0).abs() < 1e-6,
        "horizontal-dominant axis lock failed: expected 1000.0 got {v}"
    );
}

/// M14.A: during a drag-slider the displayed text in the field
/// MUST refresh every Move — not just `value`, but the `buffer`
/// that the focused-state painter renders. (Bypass the
/// `set_number_value` focus-guard via direct mutation.)
#[test]
fn number_input_body_drag_refreshes_buffer_in_realtime() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let down_x = rect.x + 10.0;
    let down_y = rect.y + rect.h * 0.5;
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, down_x, down_y),
        &arena,
    );
    // Move 1: cross threshold (no delta — re-anchor).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 5.0, down_y),
        &arena,
    );
    // Move 2: 10 px past anchor → delta = 10*50 = 500 → buffer = "500".
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 15.0, down_y),
        &arena,
    );
    let buffer = store.text(NodeId(77)).unwrap_or("").to_string();
    assert_eq!(
        buffer, "500",
        "buffer must refresh during drag-slider; got {buffer:?}"
    );
}

/// M14.A: Down + Up at (almost) the same position never crosses
/// the threshold → no ValueChanged, drag state cleared, focus is
/// retained from Down (edit mode = click→type behavior preserved).
#[test]
fn number_input_body_click_without_drag_preserves_edit_mode() {
    let (mut store, hits, rect) = number_input_setup(3.0);
    let arena = Bump::new();
    let down_x = rect.x + 10.0;
    let down_y = rect.y + rect.h * 0.5;
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, down_x, down_y),
        &arena,
    );
    // Move 2 px (< 4 px threshold) → drag stays pending.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, down_x + 2.0, down_y),
        &arena,
    );
    // Up — drag never crossed; edit mode stays active.
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, down_x + 2.0, down_y),
        &arena,
    );
    assert!(
        !evts
            .iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(_))),
        "no-drag click must not emit ValueChanged"
    );
    // Focus remained on the field (placed at Down by the existing
    // text-widget pathway). Drag candidate cleared.
    assert_eq!(store.focus_id(), Some(NodeId(77)));
    assert!(store.number_input_drag().is_none());
    // Value unchanged.
    assert!((read_value(&store, NodeId(77)) - 3.0).abs() < 1e-6);
}

/// M14.A: continuous-hold on the up arrow. Down fires the first
/// increment (already covered by `number_input_up_arrow_increments_integer`).
/// `dispatch_tick` skips while inside the initial 250 ms delay,
/// then fires repeats every 30 ms.
#[test]
fn number_stepper_hold_repeats_after_initial_delay() {
    use crate::interaction::drag::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};
    let (mut store, hits, rect) = number_input_setup(10.0);
    let arena = Bump::new();
    let probe = crate::widget::NumberInput::new(NodeId(77), "", 10.0);
    let up = probe.up_rect(rect);
    // Down at t=0 ns — first tick fires (10 → 11) via apply_number_stepper_if_hit.
    let mut down_evt = pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5);
    down_evt.timestamp_ns = 0;
    let _ = dispatch_pointer(&mut store, &hits, down_evt, &arena);
    assert!((read_value(&store, NodeId(77)) - 11.0).abs() < f64::EPSILON);
    // Tick at 100 ms — still inside the initial delay; nothing.
    let evts = dispatch_tick(&arena, &mut store, 100_000_000);
    assert!(evts.is_empty(), "no repeat inside initial delay");
    // Tick at 300 ms — past the delay → one repeat fires (11 → 12).
    let evts = dispatch_tick(
        &arena,
        &mut store,
        STEPPER_HOLD_INITIAL_DELAY_NS + 50_000_000,
    );
    assert_eq!(evts.len(), 1);
    assert!((read_value(&store, NodeId(77)) - 12.0).abs() < f64::EPSILON);
    // Another tick 50 ms later (> 30 ms repeat) → second repeat (12 → 13).
    let evts = dispatch_tick(
        &arena,
        &mut store,
        STEPPER_HOLD_INITIAL_DELAY_NS + 50_000_000 + STEPPER_REPEAT_INTERVAL_NS + 5_000_000,
    );
    assert_eq!(evts.len(), 1);
    assert!((read_value(&store, NodeId(77)) - 13.0).abs() < f64::EPSILON);
}

/// M14.A audit fix #1 (CRITICAL): Esc mid-drag must abort the
/// in-flight `number_input_drag` and `number_stepper_hold`. Old
/// behavior: Esc reverted the buffer but the drag stayed armed,
/// so the next Move would continue overwriting the value from a
/// stale `start_value`. This regression test pins the new
/// invariant.
#[test]
fn esc_clears_in_flight_number_input_drag() {
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    // Down on body — drag candidate seeded (focus also lands).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move past 4 px threshold — drag promoted to slider.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    assert!(store.number_input_drag().is_some(), "drag armed before Esc");
    // Esc clears it.
    let _ = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
    assert!(
        store.number_input_drag().is_none(),
        "Esc must clear in-flight drag"
    );
    assert!(
        store.number_stepper_hold().is_none(),
        "Esc must also clear any stepper hold"
    );
}

/// M14.A audit fix #2 (CRITICAL): while the drag-slider is
/// scrubbing, `last_committed` must stay anchored on the
/// pre-Down value so Esc rollback works. Only the Up commit
/// updates `last_committed`. Old behavior overwrote it on every
/// Move and silently destroyed the rollback target.
#[test]
fn drag_slider_last_committed_anchors_until_up_commits() {
    let (mut store, hits, rect) = number_input_setup(7.0);
    let arena = Bump::new();
    // Down + Move 1 (cross threshold, no delta) + Move 2 (apply delta).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 15.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput {
            value,
            last_committed,
            ..
        }) => {
            assert!(
                (*last_committed - 7.0).abs() < f64::EPSILON,
                "last_committed must stay at the pre-drag value during Move, got {last_committed}"
            );
            assert!(
                (*value - 7.0).abs() > f64::EPSILON,
                "value should already have moved during drag"
            );
        }
        _ => panic!("expected NumberInput state"),
    }
    // Up commits — last_committed now matches value.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput {
            value,
            last_committed,
            ..
        }) => {
            assert!(
                (*last_committed - *value).abs() < f64::EPSILON,
                "Up must commit last_committed = value"
            );
        }
        _ => panic!("expected NumberInput state"),
    }
}

/// Re-audit fix: `set_number_value` must NOT overwrite
/// `last_committed` when a drag-slider is actively scrubbing the
/// same field. Without this guard, the per-frame snapshot
/// republish (host path) silently moved the rollback anchor to
/// the latest dragged value, defeating audit fix #2.
#[test]
fn set_number_value_preserves_last_committed_during_drag() {
    let (mut store, hits, rect) = number_input_setup(7.0);
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Move 1: cross threshold (no delta). Move 2: apply delta.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 15.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    // Host-side snapshot republish: writes a value via
    // `set_number_value` while drag is active. Must NOT clobber
    // `last_committed` (still anchored at 7.0 = pre-drag).
    store.set_number_value(NodeId(77), 999.0);
    match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput { last_committed, .. }) => {
            assert!(
                (*last_committed - 7.0).abs() < f64::EPSILON,
                "set_number_value mid-drag must not move last_committed; got {last_committed}"
            );
        }
        _ => panic!("expected NumberInput state"),
    }
}

/// M14.A: pointer-Up clears the continuous-hold so subsequent
/// ticks (even at a time past the delay) do nothing — release
/// stops the repeat. Verified against the same fixture as the
/// previous test minus the trailing ticks.
#[test]
fn number_stepper_hold_ends_on_pointer_up() {
    use crate::interaction::drag::STEPPER_HOLD_INITIAL_DELAY_NS;
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.0);
    let up = probe.up_rect(rect);
    let mut down_evt = pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5);
    down_evt.timestamp_ns = 0;
    let _ = dispatch_pointer(&mut store, &hits, down_evt, &arena);
    // Up at t=10 ms — hold cleared.
    let mut up_evt = pointer(PointerKind::Up, up.x + up.w * 0.5, up.y + up.h * 0.5);
    up_evt.timestamp_ns = 10_000_000;
    let _ = dispatch_pointer(&mut store, &hits, up_evt, &arena);
    assert!(store.number_stepper_hold().is_none());
    // Tick at 500 ms (well past delay) — nothing fires.
    let evts = dispatch_tick(
        &arena,
        &mut store,
        STEPPER_HOLD_INITIAL_DELAY_NS + 250_000_000,
    );
    assert!(
        evts.is_empty(),
        "no repeat after pointer-Up cleared the hold"
    );
    // Value remained at the single Down-increment.
    assert!((read_value(&store, NodeId(77)) - 1.0).abs() < f64::EPSILON);
}

/// Audit fix #1 (CRITICAL): Esc clears any in-flight
/// `number_input_drag` AND `number_stepper_hold` regardless of
/// focus. Without this, the drag state stays armed and the next
/// Move would continue advancing `last_committed` from a stale
/// `start_value`.
#[test]
fn esc_clears_in_flight_drag_and_stepper_hold() {
    let (mut store, hits, rect) = number_input_setup(7.0);
    let arena = Bump::new();
    // 1) Start a drag-slider mid-scrub.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    assert!(store.number_input_drag().is_some());
    // 2) Esc cancels.
    let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
    let _ = evts;
    assert!(
        store.number_input_drag().is_none(),
        "Esc must clear number_input_drag"
    );

    // Same coverage for stepper hold.
    let (mut store, hits, rect) = number_input_setup(0.0);
    let arena = Bump::new();
    let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.0);
    let up = probe.up_rect(rect);
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5),
        &arena,
    );
    assert!(store.number_stepper_hold().is_some());
    let _ = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
    assert!(
        store.number_stepper_hold().is_none(),
        "Esc must clear number_stepper_hold"
    );
}

/// Audit fix #2 (CRITICAL): per-Move drag updates `value` +
/// `buffer` but leaves `last_committed` untouched until Up.
/// Otherwise Esc-revert would only roll back to the most recent
/// scrubbed value, not to the pre-Down anchor.
#[test]
fn drag_move_does_not_advance_last_committed() {
    let (mut store, hits, rect) = number_input_setup(42.0);
    let arena = Bump::new();
    // Pre-Down anchor is `last_committed = 42.0`.
    let initial_last_committed = match store.get(NodeId(77)) {
        Some(InteractiveState::NumberInput { last_committed, .. }) => *last_committed,
        _ => -1.0,
    };
    assert_eq!(initial_last_committed, 42.0);
    // Down → Move 1 (cross) → Move 2 (apply delta). Value advances,
    // last_committed must NOT.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 15.0, rect.y + rect.h * 0.5),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    if let Some(InteractiveState::NumberInput {
        value,
        last_committed,
        ..
    }) = store.get(NodeId(77))
    {
        assert!(
            (*value - 42.0).abs() > 1e-3,
            "value should have advanced during drag"
        );
        assert_eq!(
            *last_committed, 42.0,
            "last_committed must remain pre-Down anchor until Up"
        );
    }
    // Up commits last_committed to the scrubbed value.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, rect.x + 30.0, rect.y + rect.h * 0.5),
        &arena,
    );
    if let Some(InteractiveState::NumberInput {
        value,
        last_committed,
        ..
    }) = store.get(NodeId(77))
    {
        assert_eq!(*value, *last_committed);
    }
}

fn meta_key(kc: u32) -> KeyEvent {
    KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: true,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

fn focused_text_input(text: &str, caret: usize, anchor: Option<usize>) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(50),
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text: text.to_string(),
            caret,
            selection_anchor: anchor,
        },
    );
    store.set_focus(Some(NodeId(50)));
    store
}

#[test]
fn cmd_c_copies_selection_to_outbox() {
    let mut store = focused_text_input("hello world", 5, Some(0));
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, meta_key(KEY_KEY_C), &arena);
    assert_eq!(store.take_clipboard_copy().as_deref(), Some("hello"));
}

#[test]
fn cmd_c_without_selection_emits_nothing() {
    let mut store = focused_text_input("hello", 3, None);
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, meta_key(KEY_KEY_C), &arena);
    assert!(store.take_clipboard_copy().is_none());
}

#[test]
fn cmd_x_cuts_selection_and_emits_text_changed() {
    let mut store = focused_text_input("hello world", 11, Some(5));
    let arena = Bump::new();
    let evts = dispatch_key(&mut store, meta_key(KEY_KEY_X), &arena);
    assert_eq!(store.take_clipboard_copy().as_deref(), Some(" world"));
    match store.get(NodeId(50)) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            assert_eq!(text, "hello");
            assert_eq!(*caret, 5);
        }
        _ => panic!("expected TextInput"),
    }
    assert!(
        evts.iter()
            .any(|e| matches!(e, WidgetEvent::TextChanged(_)))
    );
}

#[test]
fn cmd_v_sets_paste_request() {
    let mut store = focused_text_input("abc", 3, None);
    let arena = Bump::new();
    let _ = dispatch_key(&mut store, meta_key(KEY_KEY_V), &arena);
    assert_eq!(store.take_clipboard_paste_request(), Some(NodeId(50)));
}

#[test]
fn apply_clipboard_paste_inserts_at_caret() {
    let mut store = focused_text_input("abxy", 2, None);
    let ok = apply_clipboard_paste(&mut store, NodeId(50), "cd");
    assert!(ok);
    match store.get(NodeId(50)) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            assert_eq!(text, "abcdxy");
            assert_eq!(*caret, 4);
        }
        _ => panic!(),
    }
}

#[test]
fn apply_clipboard_paste_replaces_selection() {
    let mut store = focused_text_input("hello world", 5, Some(0));
    apply_clipboard_paste(&mut store, NodeId(50), "Hi");
    match store.get(NodeId(50)) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            assert_eq!(text, "Hi world");
            assert_eq!(*caret, 2);
        }
        _ => panic!(),
    }
}
