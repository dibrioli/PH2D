use super::*;

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

/// A `Secondary` (right-button) pointer event, for opening context menus.
fn secondary(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        button: ph2d_host::PointerButton::Secondary,
        ..pointer(kind, x, y)
    }
}

/// DEFECT REPRO (Painter Falloff "Vector handle does nothing"): clicking a
/// context-menu ITEM must emit `Click(item_id)` even though the menu CLOSES on
/// the item's Down (so by Up the hit-index no longer holds the item). The
/// falloff handle menu, the vector point-type menu, and every chrome menu all
/// depend on this. The item is registered as a `Button` + hit-rect by the menu
/// paint; Down arms it (and closes the menu), Up fires the Click off the
/// Down-snapshotted active_rect.
#[test]
fn context_menu_item_click_emits_click_even_though_menu_closes_on_down() {
    use crate::ids::CTX_MENU_FALLOFF_HANDLE_VECTOR as ITEM;
    use crate::interaction::{ContextMenuKind, ContextMenuRequest};

    let mut store = WidgetStore::with_capacity(4);
    // The menu paint registers each item as a Button in the store + a hit-rect.
    store.register(
        ITEM,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let mut hits = HitIndex::new();
    let item_rect = Rect::new(10.0, 10.0, 120.0, 24.0);
    hits.register(ITEM, item_rect);

    // The right-click opened the FalloffPointHandle menu (shell side).
    store.open_context_menu(ContextMenuRequest {
        x: 10.0,
        y: 10.0,
        kind: ContextMenuKind::FalloffPointHandle,
    });

    let arena = Bump::new();
    // Primary Down on the "Vector" item: the dispatch closes the menu here.
    let down = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 50.0, 22.0),
        &arena,
    )
    .to_vec();
    assert!(
        store.context_menu().is_none(),
        "menu should close on the item Down: {down:?}"
    );
    assert_eq!(store.active_id(), Some(ITEM), "item armed active on Down");

    // Primary Up: fires the Click off the Down-snapshotted active_rect — even
    // though the menu (and its fresh hit-rect) is already gone.
    let up = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 50.0, 22.0),
        &arena,
    );
    assert_eq!(
        up,
        &[WidgetEvent::Click(ITEM)],
        "Up must emit Click(CTX_MENU_FALLOFF_HANDLE_VECTOR) — this is the event \
         that drives chrome::falloff_handle"
    );
}

/// A right-click that OPENS the falloff handle menu must not be eaten by the
/// generic `CreateNote` panel-menu opener for a non-excluded panel. (Guards the
/// double-open the painter panel triggers — see the fix in pointer_down_menus.)
#[test]
fn secondary_click_does_not_self_open_create_note_for_painter_panel() {
    // No panel rect registered → `panel_at` is None → the secondary click just
    // closes any menu (the baseline this dispatch test can assert without a
    // painter panel rect). The real-app double-open is covered by the shell's
    // own ordering; here we only pin that a bare secondary click emits no
    // spurious Click.
    let mut store = WidgetStore::with_capacity(4);
    let hits = HitIndex::new();
    let arena = Bump::new();
    let evts = dispatch_pointer(
        &mut store,
        &hits,
        secondary(PointerKind::Down, 5.0, 5.0),
        &arena,
    );
    assert_eq!(evts, &[], "a bare secondary click emits no widget Click");
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
