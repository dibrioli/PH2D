//! **The Physics toggle exists ON SCREEN, CLICKS, and reaches the shell.**
//!
//! One transport, two consumers — the animation curves and the rapier world
//! (ADR-0131). This switch is which of them the clock reaches, and unlike the
//! Speed toggle beside it, it is NOT panel-local: the flag lives in the timeline
//! state and the shell reads it to drive physics, so the click has to leave the
//! panel as a `PanelEvent`.
//!
//! The gate paints for real (`MockPanelHost::paint` returns the hit index the
//! paint registered) and drives a real pointer at the rect it finds, because
//! three different omissions all look like "the checkbox does nothing" and only
//! one of them is caught by asserting on a synthetic `WidgetEvent`:
//! painted-but-unregistered, registered-but-not-in-`populate` (so the Down never
//! makes it active — dead under the mouse), and routed-nowhere.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

fn timeline_events(host: &mut MockPanelHost) -> Vec<PanelEvent> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::TimelinePanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect()
}

/// A plain transport with the flag in a known state. `simulate_physics` is the
/// only field under test; everything else is whatever a fresh document has.
fn transport(simulate_physics: bool) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        simulate_physics,
        ..TimelineViewSnapshot::default()
    }
}

fn paint(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    snap: TimelineViewSnapshot,
) -> Vec<(ph2d_editor_core::NodeId, Rect)> {
    set_current_timeline(Some(snap));
    host.paint::<TimelinePanel>(state, VIEWPORT)
}

/// Paint it, find the rect the paint registered, put the pointer in it, and
/// require the click to reach the shell carrying its new state.
#[test]
fn the_physics_toggle_is_painted_and_clicks_through_to_the_shell() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    let regs = paint(&mut host, &mut state, transport(false));
    let r = regs
        .iter()
        .find(|(w, _)| *w == ids::TIMELINE_PHYSICS)
        .map(|(_, r)| *r)
        .expect("the Physics toggle was painted but never hit-registered");

    // Half one: a real pointer on that rect must actually toggle the widget.
    // This is what `populate` buys — an unregistered id is never made active by
    // the Down, so it paints, hit-tests, and stays dead under the mouse.
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evs = host.click_at(cx, cy);
    let toggled = evs
        .iter()
        .find(|e| matches!(e, WidgetEvent::Toggled(id) if *id == ids::TIMELINE_PHYSICS))
        .copied()
        .unwrap_or_else(|| {
            panic!(
                "the pointer landed on {:?} but no Toggled came out — got {evs:?}",
                host.hit_at(cx, cy)
            )
        });

    // Half two: the panel must route it OUT, carrying the new state.
    host.apply_panel_event::<TimelinePanel>(&mut state, toggled);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_PHYSICS, true)],
        "clicking Physics must reach the shell so it arms the simulation \
         (it is NOT panel-local like the Speed view toggle)"
    );
}

/// The painted switch follows the document, in both directions.
///
/// A toggle that always paints its stored state is a toggle that can disagree
/// with what the clock is actually driving — and this one is off by default, so
/// the disagreement that matters is the one that reads as "I armed physics and
/// the box still says off".
#[test]
fn the_painted_switch_shows_what_the_transport_is_driving() {
    for armed in [false, true] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        paint(&mut host, &mut state, transport(armed));

        let (_, on) = host
            .store()
            .toggle(ids::TIMELINE_PHYSICS)
            .expect("the Physics toggle is not registered — check populate.rs");
        assert_eq!(
            on, armed,
            "the painted Physics switch disagrees with the transport it reflects"
        );
    }
}
