//! **The Motion Path toggle exists ON SCREEN, CLICKS, and reaches the shell.**
//!
//! Per object (ADR-0141): the switch reflects the SELECTED object's position mode
//! and, clicked, converts it (the shell resolves the entity and emits
//! `ConvertPositionMode`, the mirror of +Track). Unlike the flag-backed toggles
//! beside it, it is NOT panel-local, so the click has to leave the panel as a
//! `PanelEvent`.
//!
//! The gate paints for real and drives a real pointer at the rect it finds, because
//! three different omissions all look like "the switch does nothing" and only one is
//! caught by a synthetic `WidgetEvent`: painted-but-unregistered,
//! **registered-but-not-in-`populate`** (so the Down never makes it active — dead
//! under the mouse; this is the bug the Enio reported), and routed-nowhere.

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

fn transport(position_is_path: bool) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        position_is_path,
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

/// Paint it, find the rect the paint registered, put the pointer in it, and require
/// the click to reach the shell — which is where the per-object convert happens.
#[test]
fn the_motion_path_toggle_is_painted_and_clicks_through_to_the_shell() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // Object in Separate (off); clicking must ask for Path (on).
    let regs = paint(&mut host, &mut state, transport(false));
    let r = regs
        .iter()
        .find(|(w, _)| *w == ids::TIMELINE_MOTION_PATH)
        .map(|(_, r)| *r)
        .expect("the Motion Path toggle was painted but never hit-registered");

    // A real pointer on that rect must actually toggle the widget. This is what
    // `populate` buys — an unregistered id is never made active by the Down, so it
    // paints, hit-tests, and stays DEAD under the mouse (the reported bug).
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evs = host.click_at(cx, cy);
    let toggled = evs
        .iter()
        .find(|e| matches!(e, WidgetEvent::Toggled(id) if *id == ids::TIMELINE_MOTION_PATH))
        .copied()
        .unwrap_or_else(|| {
            panic!(
                "the pointer landed on {:?} but no Toggled came out — check populate.rs",
                host.hit_at(cx, cy)
            )
        });

    // And the panel must route it OUT, carrying the new state, so the shell converts.
    host.apply_panel_event::<TimelinePanel>(&mut state, toggled);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_MOTION_PATH, true)],
        "clicking Motion Path must reach the shell so it converts the selected object"
    );
}

/// The painted switch follows the SELECTION, in both directions — selecting a Path
/// object shows on, a Separate object shows off.
#[test]
fn the_painted_switch_reflects_the_selected_objects_mode() {
    for is_path in [false, true] {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        paint(&mut host, &mut state, transport(is_path));

        let (_, on) = host
            .store()
            .toggle(ids::TIMELINE_MOTION_PATH)
            .expect("the Motion Path toggle is not registered — check populate.rs");
        assert_eq!(
            on, is_path,
            "the painted Motion Path switch disagrees with the selected object's mode"
        );
    }
}
