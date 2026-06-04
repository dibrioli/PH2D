use super::*;

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
