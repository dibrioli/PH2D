//! Behavioral SEAM test for the Inspector panel's `apply_event` — the
//! test class the 2026-06-20 diagnosis found missing (blindagem Fase 0.1).
//!
//! The Inspector has NO paired tool: its `apply_event` (+ the §7 sibling
//! `event_ordering.rs`) emits `EditorAction::Inspector*Edit` variants onto
//! the action bus, and the Close (X) button toggles the panel's own
//! `host.set_panel_visible`. `populate.rs` proves the widgets are
//! registered and the `*_contract_surface` gates count symbols — but
//! NEITHER proves the wire between a widget event and the bus / visibility
//! flip is intact. A forgotten `event.rs`/`event_ordering.rs` arm leaves
//! the field painted, draggable and SILENTLY DEAD while every gate stays
//! green.
//!
//! These tests run the full headless path the desktop shell runs:
//!   populate → publish the live snapshot → set the widget value →
//!   apply_event → assert the EXACT EditorAction on the bus (a), or the
//!   EXACT panel_visible flip (b).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::screens::hero::{
    InspectorOrderingInfo, InspectorOrderingMixed, OrderingFieldEdit,
};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_ordering};
use ph2d_ui_testkit::MockPanelHost;

/// A canonical entity-bits sentinel the snapshot carries and the emitted
/// action must echo back verbatim.
const ENTITY: u64 = 0xABCD_1234;

/// Build a clean §7 Ordering snapshot for `ENTITY`. The §7 handlers gate
/// on `state::current_inspector_ordering()` being `Some(_)` — that
/// thread-local is what the host publishes before each paint. No real ECS
/// selection is needed.
fn ordering_snapshot() -> InspectorOrderingInfo {
    InspectorOrderingInfo {
        entity_bits: ENTITY,
        z_index: None,
        z_as_relative: true,
        show_behind_parent: false,
        sorting_layer: 2,
        order_in_layer: 0,
        y_sort_enabled: false,
        y_sort_point: 0,
        y_sort_axis: [0.0, 0.0],
        sorting_group: false,
        sort_at_root: false,
        top_level: false,
        selected_count: 1,
        mixed: InspectorOrderingMixed::default(),
    }
}

/// (a) A REAL edit reaches the bus: committing the §7 "Z Index" integer
/// field must push exactly
/// `InspectorOrderingEdit { entity_bits: ENTITY, edit: ZIndex(Some(5)) }`.
///
/// This is the clean, selection-free edit: it needs only the published
/// ordering snapshot (no constructed ECS entity), and exercises every site
/// from `populate` (NumberInput slot) → `apply_event` → `event_ordering`
/// arm → `host.bus_mut().push(...)`. The non-zero value is load-bearing:
/// the arm maps `0 → None` (detach) and any other value `→ Some(v)`.
#[test]
fn z_index_edit_reaches_the_bus() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut panel_state = InspectorState::default();

    // The host publishes the live ordering snapshot before paint/event.
    set_current_inspector_ordering(Some(ordering_snapshot()));

    // A drag/commit writes the NumberInput's value, THEN the dispatch
    // emits ValueChanged. Simulate both. 5 is non-zero so the arm yields
    // `ZIndex(Some(5))` (the override path), not `None` (the detach path).
    host.set_number_value(ids::INSP_ORDER_Z_INDEX, 5.0);
    let outcome = host.apply_panel_event::<InspectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::INSP_ORDER_Z_INDEX),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real Z Index commit — the `event_ordering.rs` arm for \
         INSP_ORDER_Z_INDEX is missing or unreachable"
    );

    // The shell drains the bus each frame. Assert the EXACT variant — not
    // merely that "an action was pushed".
    let actions = host.drained_actions();
    assert_eq!(
        actions,
        vec![EditorAction::InspectorOrderingEdit {
            entity_bits: ENTITY,
            edit: OrderingFieldEdit::ZIndex(Some(5)),
        }],
        "Z Index commit reached the seam but produced the wrong action / payload \
         (expected exactly one InspectorOrderingEdit{{ ZIndex(Some(5)) }})"
    );

    // Cleanup the thread-local so a re-run in the same test thread starts
    // from a clean snapshot.
    set_current_inspector_ordering(None);
}

/// (b) Close toggles visibility: with the Inspector visible, a click on
/// the X (INSP_CLOSE) must flip `panel_visible("inspector")` to EXACTLY
/// `false`. This arm needs no selection at all — it reads/writes the
/// panel's own visibility through the host trait.
#[test]
fn close_button_hides_the_inspector() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut panel_state = InspectorState::default();

    // Precondition: the panel is currently visible.
    host.set_panel_visible(InspectorPanel::ID, true);
    assert!(
        host.panel_visible(InspectorPanel::ID),
        "precondition failed: Inspector should start visible"
    );

    let outcome = host
        .apply_panel_event::<InspectorPanel>(&mut panel_state, WidgetEvent::Click(ids::INSP_CLOSE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close (X) click was ignored — the `event.rs` arm for INSP_CLOSE is missing"
    );

    // The observable effect: visibility flipped to EXACTLY false.
    assert!(
        !host.panel_visible(InspectorPanel::ID),
        "Close click was consumed but the Inspector is still visible — the \
         set_panel_visible seam in the INSP_CLOSE arm is dead"
    );

    // INSP_CLOSE is a toggle (`!host.panel_visible`); the Close arm pushes
    // no EditorAction of its own, so the bus stays empty.
    assert!(
        host.drained_actions().is_empty(),
        "the Close arm must not emit an EditorAction — it only flips visibility"
    );
}
