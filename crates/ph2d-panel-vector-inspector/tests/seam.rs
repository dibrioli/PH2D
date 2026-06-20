//! Behavioral SEAM test for the Vector Inspector panel's `apply_event` ↔ host.
//!
//! Unit tests in `state.rs` / `ids.rs` exercise the thread-local stores and the
//! id table directly, and `populate.rs` asserts widgets are registered — but
//! NEITHER proves the wire from a `WidgetEvent::Click` through `apply_event`
//! into a host side-effect is intact. A dropped `event.rs` arm, a wrong id
//! guard, or a `set_panel_visible` regression all leave the Close button
//! painted, hittable and SILENTLY DEAD while every unit test stays green.
//!
//! These tests run the full headless path the desktop shell runs:
//!   populate → apply_panel_event → assert the host's panel-visible flips.
//!
//! NOTE: the Close handler is NOT a toggle — it unconditionally drives
//! `set_panel_visible(VectorInspectorPanel::ID, false)` (see `src/event.rs`).
//! So we seed `true` first to give it something observable to turn OFF.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::panel::Panel; // brings `VectorInspectorPanel::ID` into scope
use ph2d_editor_core::panel::PanelHostInternal; // brings panel_visible / set_panel_visible
use ph2d_panel_vector_inspector::state::VectorInspectorPanelState;
use ph2d_panel_vector_inspector::{VectorInspectorPanel, ids, take_pending_shape_selection};
use ph2d_ui_testkit::MockPanelHost;

/// Clicking the Close (X) button must drive `host.set_panel_visible(false)` —
/// proving the `WidgetEvent::Click(VECTOR_INSPECTOR_CLOSE)` arm reaches the host.
#[test]
fn close_click_hides_the_panel_through_the_host() {
    let mut host = MockPanelHost::with_panel::<VectorInspectorPanel>();
    let mut panel_state = VectorInspectorPanelState;

    // Seed the panel as visible so the Close handler has something to turn OFF.
    host.set_panel_visible(VectorInspectorPanel::ID, true);

    // Non-vacuous precondition: it really is visible before the click.
    assert!(
        host.panel_visible(VectorInspectorPanel::ID),
        "test setup failed — panel must be visible before the Close click"
    );

    let outcome = host.apply_panel_event::<VectorInspectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_INSPECTOR_CLOSE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close click was ignored — `event.rs` arm for VECTOR_INSPECTOR_CLOSE is missing"
    );

    // The observable side-effect: the panel is now hidden via the host seam.
    assert!(
        !host.panel_visible(VectorInspectorPanel::ID),
        "Close click never reached `host.set_panel_visible(.., false)` — the panel→host seam is dead"
    );
}

/// Clicking a Shape-kind option must record a pending selection that the shell
/// bridge drains (`take_pending_shape_selection`) — proving the second
/// `apply_event` arm (SHAPE_OPTION_IDS) reaches its observable state change.
#[test]
fn shape_option_click_records_pending_selection() {
    let mut host = MockPanelHost::with_panel::<VectorInspectorPanel>();
    let mut panel_state = VectorInspectorPanelState;

    // Non-vacuous precondition: nothing pending before the click. (`take` is
    // single-shot, so this also clears any stale thread-local from prior tests.)
    assert_eq!(
        take_pending_shape_selection(),
        None,
        "test setup failed — no shape selection should be pending before the click"
    );

    // Click the Polygon option (index 2 in `ShapeKind::ALL` / SHAPE_OPTION_IDS).
    let polygon_id = ids::SHAPE_OPTION_IDS[2];
    let outcome = host.apply_panel_event::<VectorInspectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(polygon_id),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Shape-option click was ignored — `event.rs` SHAPE_OPTION_IDS arm is missing"
    );

    // The observable side-effect: the bridge drains exactly the clicked index.
    assert_eq!(
        take_pending_shape_selection(),
        Some(2),
        "Shape-option click never recorded the pending kind index — the panel→bridge seam is dead"
    );
}
