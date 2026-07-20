use super::*;

fn store_with(states: &[(u64, InteractiveState)]) -> WidgetStore {
    let mut s = WidgetStore::with_capacity(64);
    for (id, st) in states {
        s.register(NodeId(*id), st.clone());
    }
    s
}

#[test]
fn blank_number_input_clears_buffer_but_keeps_value_and_respects_focus() {
    let ni = |v: f64| InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value: v,
        buffer: format!("{v}"),
        caret: 0,
        last_committed: v,
        selection_anchor: None,
    };
    let mut store = store_with(&[(1, ni(42.0)), (2, ni(7.0))]);

    // BulkSelect "Mixed": blank the display, preserve value/committed.
    store.blank_number_input(NodeId(1));
    match store.get(NodeId(1)) {
        Some(InteractiveState::NumberInput {
            buffer,
            value,
            last_committed,
            ..
        }) => {
            assert!(buffer.is_empty(), "buffer not blanked: {buffer:?}");
            assert_eq!(*value, 42.0, "value must survive (clean blur revert)");
            assert_eq!(*last_committed, 42.0);
        }
        _ => panic!("not a NumberInput"),
    }

    // No-op while the field is focused (must not fight live typing).
    store.set_focus(Some(NodeId(2)));
    store.blank_number_input(NodeId(2));
    match store.get(NodeId(2)) {
        Some(InteractiveState::NumberInput { buffer, .. }) => {
            assert_eq!(buffer, "7", "focused field must not be blanked");
        }
        _ => panic!("not a NumberInput"),
    }
}

#[test]
fn register_grows_focus_order_to_match() {
    let mut store = WidgetStore::with_capacity(16);
    for i in 0..16 {
        store.register(NodeId(i as u64), InteractiveState::Plain);
    }
    assert_eq!(store.focus_order().len(), 16);
    assert_eq!(store.len(), 16);
}

#[test]
fn focus_order_matches_registration_order() {
    let store = store_with(&[
        (1, InteractiveState::Plain),
        (5, InteractiveState::Plain),
        (3, InteractiveState::Plain),
    ]);
    assert_eq!(store.focus_order(), &[NodeId(1), NodeId(5), NodeId(3)]);
}

#[test]
fn re_register_overwrites_without_growing_focus_order() {
    let mut store = WidgetStore::with_capacity(8);
    store.register(NodeId(1), InteractiveState::Plain);
    store.register(
        NodeId(1),
        InteractiveState::Button {
            state: ButtonState::Hovered,
        },
    );
    assert_eq!(store.focus_order().len(), 1);
    assert_eq!(store.button_state(NodeId(1)), Some(ButtonState::Hovered));
}

#[test]
fn collapsed_defaults_to_false() {
    let store = WidgetStore::with_capacity(4);
    assert!(!store.is_collapsed(NodeId(99)));
}

#[test]
fn collapsed_set_and_toggle() {
    let mut store = WidgetStore::with_capacity(4);
    store.set_collapsed(NodeId(7), true);
    assert!(store.is_collapsed(NodeId(7)));
    store.toggle_collapsed(NodeId(7));
    assert!(!store.is_collapsed(NodeId(7)));
    store.toggle_collapsed(NodeId(8));
    assert!(store.is_collapsed(NodeId(8)));
}

#[test]
fn convenience_getters_return_none_for_wrong_kind() {
    let store = store_with(&[(1, InteractiveState::Plain)]);
    assert!(store.button_state(NodeId(1)).is_none());
    assert!(store.slider(NodeId(1)).is_none());
}

#[test]
fn hot_active_focus_round_trip() {
    let mut store = WidgetStore::with_capacity(4);
    store.set_hot(Some(NodeId(2)));
    store.set_active(Some(NodeId(3)));
    store.set_focus(Some(NodeId(4)));
    assert_eq!(store.hot_id(), Some(NodeId(2)));
    assert_eq!(store.active_id(), Some(NodeId(3)));
    assert_eq!(store.focus_id(), Some(NodeId(4)));
}

#[test]
fn slider_convenience_round_trip() {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(1),
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.42,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let (st, v) = store.slider(NodeId(1)).unwrap();
    assert_eq!(st, SliderState::Normal);
    assert!((v - 0.42).abs() < f32::EPSILON);
}

/// **`set_slider_value` recentra o slider E o chip ligado.** É o que devolve o Offset ao
/// "sem offset" após um commit: sem a segunda metade (o número), o chip mostraria o valor
/// velho ao ser aberto para edição.
#[test]
fn set_slider_value_recenters_the_slider_and_its_linked_chip() {
    let (slider, number) = (NodeId(1), NodeId(2));
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.9,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        number,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 3.2,
            buffer: format!("{}", 3.2),
            caret: 0,
            last_committed: 3.2,
            selection_anchor: None,
        },
    );
    // Mapa afim `display = track * 8 - 4` (a faixa bipolar do Offset: track 0.5 ⇒ 0).
    store.link_slider_number_mapped(slider, number, 8.0, -4.0);

    store.set_slider_value(slider, 0.5);
    let (_, v) = store.slider(slider).unwrap();
    assert!((v - 0.5).abs() < f32::EPSILON, "o track foi para {v}");
    assert!(
        (store.number_value(number).unwrap() - 0.0).abs() < 1e-6,
        "o chip devia mostrar 0 (0.5·8−4), não {:?}",
        store.number_value(number)
    );

    // Num id que não é slider, é no-op (não entra em pânico).
    store.set_slider_value(NodeId(99), 0.5);
}

#[test]
fn hierarchy_parent_round_trip_and_depth() {
    let mut store = WidgetStore::with_capacity(4);
    assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
    assert!(store.hierarchy_set_parent(NodeId(11), Some(NodeId(10))));
    assert!(store.hierarchy_set_parent(NodeId(12), Some(NodeId(11))));
    assert_eq!(store.hierarchy_parent_of(NodeId(11)), Some(NodeId(10)));
    assert_eq!(store.hierarchy_parent_of(NodeId(12)), Some(NodeId(11)));
    assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
    assert_eq!(store.hierarchy_depth_of(NodeId(12)), 2);
}

#[test]
fn hierarchy_set_parent_rejects_cycles() {
    let mut store = WidgetStore::with_capacity(4);
    // Build: 12 → 11 → 10 (12 is grandchild of 10)
    store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
    store.hierarchy_set_parent(NodeId(12), Some(NodeId(11)));
    // Attempt to parent 10 under 12 (a descendant) → rejected.
    assert!(!store.hierarchy_set_parent(NodeId(10), Some(NodeId(12))));
    assert_eq!(store.hierarchy_parent_of(NodeId(10)), None);
    // Self-parent is also rejected.
    assert!(!store.hierarchy_set_parent(NodeId(11), Some(NodeId(11))));
}

#[test]
fn hierarchy_set_parent_none_detaches() {
    let mut store = WidgetStore::with_capacity(4);
    store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
    assert!(store.hierarchy_set_parent(NodeId(11), None));
    assert_eq!(store.hierarchy_parent_of(NodeId(11)), None);
    assert_eq!(store.hierarchy_depth_of(NodeId(11)), 0);
}
