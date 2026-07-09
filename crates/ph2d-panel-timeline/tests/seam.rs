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
fn add_track_button_raises_click_for_the_shell_to_bind() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_ADDPROP_TX),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
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
