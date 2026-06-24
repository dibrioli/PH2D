use super::*;

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

#[test]
fn palette_io_pending_round_trips_and_replace_caps() {
    use crate::interaction::PaletteIoKind;
    let (mut store, _) = blender_picker_setup();
    // No request until a button flags one; the dispatch arm calls `set_palette_io_pending`.
    assert!(store.take_palette_io_pending().is_none());
    store.set_palette_io_pending(NodeId(100), PaletteIoKind::Import);
    assert_eq!(
        store.take_palette_io_pending(),
        Some((NodeId(100), PaletteIoKind::Import)),
        "the host drains the pending request",
    );
    assert!(store.take_palette_io_pending().is_none(), "drained once");
    // Import APPENDS a new named palette + activates it, truncated to the 27 hit slots.
    let many: Vec<_> = (0..40)
        .map(|i| ph2d_tokens::ColorValue::from_rgba8(i, 0, 0, 255))
        .collect();
    store.blender_import_palette(NodeId(100), "Sunset", many);
    assert_eq!(
        store.blender_palette_set(NodeId(100)).map(<[_]>::len),
        Some(2),
        "import adds a palette (seed + imported), not replace",
    );
    assert_eq!(
        store.blender_palette(NodeId(100)).map(<[_]>::len),
        Some(27),
        "the imported palette is active + capped at the swatch-slot count",
    );
}

#[test]
fn named_palette_crud_new_select_rename_delete() {
    let (mut store, _) = blender_picker_setup();
    let id = NodeId(100);
    let count = |s: &WidgetStore| s.blender_palette_set(id).map(<[_]>::len);
    let active = |s: &WidgetStore| s.blender_picker(id).unwrap().3;
    assert_eq!(count(&store), Some(1), "seeded with one palette");
    // New → appended and made active.
    store.blender_new_palette(id);
    assert_eq!(count(&store), Some(2));
    assert_eq!(active(&store), 1, "the new palette is active");
    // Select the first, rename it.
    store.blender_select_palette(id, 0);
    assert_eq!(active(&store), 0);
    store.blender_rename_active_palette(id, "Warm");
    assert_eq!(store.blender_palette_set(id).unwrap()[0].name, "Warm");
    // Delete the active → back to one; deleting the last is a no-op (always keep ≥1).
    store.blender_delete_active_palette(id);
    assert_eq!(count(&store), Some(1));
    store.blender_delete_active_palette(id);
    assert_eq!(count(&store), Some(1), "always keeps at least one palette");
}

#[test]
fn palette_name_field_syncs_to_active_and_renames() {
    use crate::widget::TextInputState;
    let (mut store, _) = blender_picker_setup();
    let (id, field) = (NodeId(100), NodeId(700));
    store.register(
        field,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.link_blender_palette_name(id, field);
    let field_text = |s: &WidgetStore| match s.get(field) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    };
    // Sync pulls the active palette's name into the field buffer.
    store.sync_blender_palette_name_buffer(id);
    assert_eq!(
        field_text(&store),
        "Palette",
        "the field shows the active palette name"
    );
    // Rename (the Enter-commit path) updates the set + trims whitespace.
    store.blender_rename_active_palette(id, "  Warm  ");
    assert_eq!(
        store.blender_palette_set(id).unwrap()[0].name,
        "Warm",
        "rename trims whitespace"
    );
    // Switching palettes re-syncs the field to the new active name.
    store.blender_new_palette(id);
    store.sync_blender_palette_name_buffer(id);
    assert_eq!(
        field_text(&store),
        "Palette 2",
        "switching palettes re-syncs the field"
    );
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
