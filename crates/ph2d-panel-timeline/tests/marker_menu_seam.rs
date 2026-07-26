//! Behavioral SEAM test for the marker right-click menu (ADR-0143).
//!
//! The pennant's whole edit surface — Rename Marker / Set Signal / Delete Marker —
//! lives on a context menu now (it used to hide behind double-click /
//! Shift+double-click / Alt+click). A menu router is only proven by driving the
//! REAL seam: the Secondary Down opened the menu, the next Down closed it (parking
//! the request in `last_context_menu`), and only THEN the Click on a row arrives.
//! Reading only the OPEN menu is exactly how a context-menu item ships doing nothing.

use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_ui_testkit::MockPanelHost;

/// Park a `TimelineMarker { index }` menu request the way the production Down does:
/// open it, then close it (the Down-before-Click stores it in `last_context_menu`).
fn park_marker_menu(host: &mut MockPanelHost, index: usize) {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineMarker { index },
    });
    host.store_mut().close_context_menu();
}

#[test]
fn rename_marker_menu_click_arms_the_label_editor() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    park_marker_menu(&mut host, 2);

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_RENAME_MARKER),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Rename Marker arm is missing from event.rs"
    );
    assert_eq!(
        state
            .marker_rename
            .map(|m| (m.index, m.opened, m.editing_signal)),
        Some((2, false, false)),
        "Rename Marker arms the inline editor for the parked marker in LABEL mode"
    );
    assert!(
        host.store().last_context_menu().is_none(),
        "the request is spent — a stray later Click must not re-open the editor"
    );
}

#[test]
fn set_signal_menu_click_arms_the_signal_editor() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    park_marker_menu(&mut host, 2);

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_SET_SIGNAL),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        state
            .marker_rename
            .map(|m| (m.index, m.opened, m.editing_signal)),
        Some((2, false, true)),
        "Set Signal arms the SAME editor in SIGNAL mode (ADR-0143), not the label"
    );
}

#[test]
fn delete_marker_menu_click_removes_that_marker() {
    let _ = ph2d_panel_timeline::drain_intents(); // isolate from sibling tests
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    park_marker_menu(&mut host, 3);

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_DELETE_MARKER),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::RemoveMarker { index: 3 }],
        "Delete Marker must raise a RemoveMarker for the parked index"
    );
    assert!(
        state.marker_rename.is_none(),
        "deleting a marker never opens the rename editor"
    );
    assert!(
        host.store().last_context_menu().is_none(),
        "the request is spent — a stray later Click must not delete again"
    );
}

#[test]
fn every_marker_menu_row_is_handled_by_the_panel() {
    // The anti-dead-item gate, executable: a row added to TIMELINE_MARKER_MENU
    // without a `marker_menu::route` arm is a painted menu item that silently does
    // nothing. Drive each one through the real seam and demand the panel consume it.
    let _ = ph2d_panel_timeline::drain_intents();
    for (id, label, _) in ids::TIMELINE_MARKER_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        park_marker_menu(&mut host, 0);
        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "marker-menu row `{label}` is painted but has no marker_menu::route arm"
        );
    }
    let _ = ph2d_panel_timeline::drain_intents();
}
