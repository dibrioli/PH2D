use super::*;

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
fn wheel_over_an_open_dropdown_popover_scrolls_it() {
    let mut store = WidgetStore::with_capacity(8);
    let dd = NodeId(50);
    store.register(
        dd,
        InteractiveState::Dropdown {
            state: crate::widget::DropdownState::Normal,
            open: true,
            selected_index: None,
        },
    );
    // 200px of content in a 60px popover (overflows → scrolls).
    store.set_dropdown_popover(dd, Rect::new(10.0, 10.0, 100.0, 60.0));
    store.set_panel_content_h(dd, 200.0);
    store.set_panel_visible_h(dd, 60.0);
    let arena = Bump::new();
    let wheel = |dy: f32| ph2d_host::WheelEvent {
        x: 50.0,
        y: 40.0,
        delta_x: 0.0,
        delta_y: dy,
        modifiers: ph2d_host::Modifiers::default(),
        timestamp_ns: 0,
    };
    let _ = crate::interaction::dispatch_wheel(&mut store, wheel(-20.0), &arena);
    assert!(
        (store.panel_scroll(dd) - 20.0).abs() < 0.01,
        "wheel scrolls the open popover"
    );
    // Wheeling far down clamps at content_h - visible_h = 140.
    let _ = crate::interaction::dispatch_wheel(&mut store, wheel(-1000.0), &arena);
    assert!(
        (store.panel_scroll(dd) - 140.0).abs() < 0.01,
        "clamped at the bottom"
    );
    // Once closed, the popover no longer captures the wheel.
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(dd) {
        *open = false;
    }
    let before = store.panel_scroll(dd);
    let _ = crate::interaction::dispatch_wheel(&mut store, wheel(50.0), &arena);
    assert!(
        (store.panel_scroll(dd) - before).abs() < 0.01,
        "closed dropdown ignores the wheel"
    );
}

#[test]
fn scrollbar_track_drag_begins_without_focus_and_scrolls() {
    // A scrollbar is hit-registered only (never a store `InteractiveState`) → NOT focusable. The Down
    // handler must still begin the drag — the regression was that the begin sat inside the
    // `is_focusable` block, so no scrollbar could ever be dragged.
    let mut store = WidgetStore::with_capacity(8);
    let panel = crate::ids::PAINTER_LAYERS_PANEL;
    store.set_panel_content_h(panel, 300.0);
    store.set_panel_visible_h(panel, 100.0);
    let mut hits = HitIndex::new();
    hits.register(
        crate::widget::PAINTER_LAYERS_SCROLLBAR_ID,
        Rect::new(200.0, 0.0, 10.0, 100.0), // the full track
    );
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 205.0, 10.0),
        &arena,
    );
    assert!(
        store.scrollbar_drag().is_some(),
        "Down on the track begins a scrollbar drag (no focus required)"
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 205.0, 70.0),
        &arena,
    );
    assert!(
        store.panel_scroll(panel) > 0.0,
        "dragging the track down scrolled the panel"
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 205.0, 70.0),
        &arena,
    );
    assert!(store.scrollbar_drag().is_none(), "Up ends the drag");
}
