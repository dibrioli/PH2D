//! Behavioral SEAM test for the timeline panel transport → shell wire
//! (architecture_interactive_crate_has_behavioral_test).
//!
//! Unit tests cover `intent_for_transport` (the shell half) and `apply_intent`
//! (the runtime), but neither proves the panel's `event.rs` actually raises the
//! action on a real WidgetEvent. This drives the full seam headless:
//!   populate → WidgetEvent → apply_event → bus → assert the TimelinePanelEvent
//! (the exact payload the shell drain translates into a TimelineIntent).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_ui_testkit::MockPanelHost;

fn timeline_events(host: &mut MockPanelHost) -> Vec<PanelEvent> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::TimelinePanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect()
}

#[test]
fn play_button_click_raises_transport_event() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    let outcome =
        host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_PLAY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Play click — event.rs arm missing"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Click(ids::TIMELINE_PLAY)],
        "Play click must raise TimelinePanelEvent(Click(PLAY)) for the shell to map to TogglePlay"
    );
}

#[test]
fn add_track_button_toggles_the_dropdown_locally() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert!(!state.add_track_open);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_ADD_TRACK),
    );
    assert!(state.add_track_open, "+Track opens the dropdown");
    // Toggling is panel-local — no shell event.
    assert!(timeline_events(&mut host).is_empty());
}

#[test]
fn add_track_prop_raises_click_and_closes_the_dropdown() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        add_track_open: true,
        ..TimelinePanelState::default()
    };
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_ADDPROP_TX),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        !state.add_track_open,
        "picking a property closes the dropdown"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Click(ids::TIMELINE_ADDPROP_TX)],
        "a +Track prop click must reach the shell so it binds the selected sprite"
    );
}

#[test]
fn time_chip_edit_raises_set_value() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    host.set_number_value(ids::TIMELINE_TIME_NUM, 1.5);
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_TIME_NUM),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 1.5)],
        "seconds-chip edit must carry the real value for the shell to Scrub to it"
    );
}

#[test]
fn ruler_scrub_maps_value_to_time_and_raises_scrub() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    // Simulate what paint stored: 10 s visible from t=0. A drag to the middle
    // (0.5) must Scrub to 5 s.
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 10.0,
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::SetValue(ids::TIMELINE_RULER, 5.0)],
        "ruler scrub at 0.5 over a 10 s span must Scrub to 5 s"
    );
}

#[test]
fn speed_toggle_flips_the_view_locally() {
    // The speed-graph view is panel-local view state (like +Track), not a
    // document command: the Toggled event must flip `speed_view` and raise NO
    // shell event.
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert!(!state.speed_view, "value view by default");

    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SPEED));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Speed toggle arm is missing from event.rs"
    );
    assert!(state.speed_view, "Speed toggle flips the panel-local view");
    assert!(
        timeline_events(&mut host).is_empty(),
        "the speed view is not a document command — it must not reach the shell"
    );

    // A second toggle flips it back.
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SPEED));
    assert!(
        !state.speed_view,
        "toggling again returns to the value view"
    );
}

#[test]
fn snap_toggle_raises_toggle_event() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // Snap is registered on (default true); a Toggled event re-reads the store.
    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SNAP));
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_SNAP, true)],
        "snap toggle must carry its on-state for the shell to SetFrameSnap"
    );
}

#[test]
fn record_toggle_raises_toggle_event() {
    // The Record (performing) toggle must reach the shell as a document command
    // (→ SetPerforming), NOT stay panel-local like the Speed view toggle.
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // Turn it on in the store, then fire the event (dispatch re-reads the store).
    host.set_toggle_on(ids::TIMELINE_RECORD, true);
    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_RECORD));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Record toggle arm is missing from event.rs"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_RECORD, true)],
        "Record must reach the shell so it arms performing (SetPerforming)"
    );
}

/// A real document with one bound `(entity, prop)` row, its snapshot published
/// — the same objects the shell hands the live panel. Returns the row's raw
/// `AnimTarget`.
fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&st, &ph);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

#[test]
fn delete_track_menu_click_raises_an_unbind_for_that_row() {
    // The production flow: the Secondary Down opened the menu, the NEXT Down
    // closed it (parking the request in `last_context_menu`) and only then the
    // Click on the row arrives — reading only the OPEN menu here is exactly how
    // a context-menu item ships doing nothing.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_timeline::PropKind;

    let _ = ph2d_panel_timeline::drain_intents(); // isolate from sibling tests
    let entity = 7;
    let target = publish_one_track(entity, PropKind::TranslationX);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu(); // Down-before-Click parks it

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_DELETE_TRACK),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Delete Track arm is missing from event.rs"
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::Unbind {
            entity,
            prop: PropKind::TranslationX
        }],
        "Delete Track must raise an Unbind for the row's binding"
    );
    assert!(
        host.store().last_context_menu().is_none(),
        "the request is spent — a stray later Click must not delete again"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn every_track_menu_row_is_handled_by_the_panel() {
    // The anti-dead-item gate, executable: a row added to TIMELINE_TRACK_MENU
    // without an event.rs arm is a painted menu item that silently does
    // nothing. Drive each one through the real seam and demand the panel
    // consume it.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(3, ph2d_timeline::PropKind::Opacity);
    for (id, label, _) in ph2d_editor_core::ids::TIMELINE_TRACK_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        host.store_mut().open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::TimelineTrack { target },
        });
        host.store_mut().close_context_menu();
        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "track-menu row `{label}` is painted but has no event.rs arm"
        );
    }
    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn a_delete_for_a_row_gone_from_the_snapshot_expires_quietly() {
    // The row was deleted between the menu opening and the click landing (e.g.
    // an undo pulled the binding out) — the action's target is gone, so no
    // intent may fire, least of all against a RE-USED target id.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None); // empty snapshot: row gone

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineTrack { target: 999 },
    });
    host.store_mut().close_context_menu();

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_DELETE_TRACK),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "a dead row's delete must not raise an intent"
    );
}
