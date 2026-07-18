//! **Behavioral SEAM sweep for §11 Physics Body** (ADR-0131 D8).
//!
//! The architecture gates cover the *focusability* half — a control painted
//! and hit-registered but never `store.register`ed is dead on click, and
//! `architecture_panel_wiring_parity` catches that. They deliberately do NOT
//! assert that a registered control has an event arm, so **a row nothing
//! dispatches passes every one of them**. This file is the other half.
//!
//! It is a SWEEP, not a sample: every control the section paints is clicked
//! or committed here and its exact action asserted. Picking "the fullest
//! card" and trusting it to cover the rest is the premise that has rotted
//! twice in this repo ([[feedback_the_fullest_card_premise_rots]]).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel};
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_physics};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0xABCD_1234;

/// An entity that already carries a body — a Box, because that is what the
/// Add button produces and therefore the state an artist is actually in.
fn with_body() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        entity_bits: ENTITY,
        has_body: true,
        kind_tag: 0,
        shape_tag: 1,
        radius: 0.4,
        half_x: 0.3,
        half_y: 0.2,
        density: 1.0,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        can_join: false,
    }
}

/// A plain sprite: the empty face, where the only control is Add.
fn without_body() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        has_body: false,
        ..with_body()
    }
}

fn click(info: InspectorPhysicsInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(info));
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(id));
    let out = host.drained_actions();
    set_current_inspector_physics(None);
    out
}

fn commit(info: InspectorPhysicsInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_physics(None);
    out
}

fn expect(actions: &[EditorAction], edit: PhysicsFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorPhysicsEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} did not reach the bus as exactly one InspectorPhysicsEdit({edit:?}) — the \
         control is painted and focusable but its event arm is missing or wrong"
    );
}

/// **The door.** Add is the only control on a plain sprite, and before it
/// existed nothing in the editor could make an entity physical.
#[test]
fn add_physics_body_reaches_the_bus() {
    expect(
        &click(without_body(), ids::INSP_PHYS_ADD),
        PhysicsFieldEdit::Add,
        "Add Physics Body",
    );
}

#[test]
fn remove_physics_body_reaches_the_bus() {
    expect(
        &click(with_body(), ids::INSP_PHYS_REMOVE),
        PhysicsFieldEdit::Remove,
        "Remove Physics Body",
    );
}

/// Both segmented groups, every option — a two-way control where only one
/// side is wired is the classic half-dead widget.
#[test]
fn every_segmented_option_reaches_the_bus() {
    for (i, &id) in ids::INSP_PHYS_KIND.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::Kind(i as u8),
            &format!("Body kind option {i}"),
        );
    }
    for (i, &id) in ids::INSP_PHYS_SHAPE.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::Shape(i as u8),
            &format!("Collider shape option {i}"),
        );
    }
    // All EIGHT layer chips, each asserting its OWN index — a sweep that only
    // checked "some Layer edit came out" would pass with every chip wired to
    // layer 0, which is the copy-paste this loop shape invites.
    for (i, &id) in ids::INSP_PHYS_LAYER.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::Layer(i as u8),
            &format!("Collision layer chip {i}"),
        );
    }
}

/// **A layer chip on an entity with no body is refused at the event layer.**
///
/// The chips are only painted for a body, but painting is not refusing — dim
/// still dispatches ([[feedback_disabled_button_still_dispatches]]), and an
/// edit naming a collider that does not exist would reach the bus and be
/// dropped somewhere further along, where nobody is looking.
#[test]
fn a_layer_chip_without_a_body_is_refused() {
    for &id in ids::INSP_PHYS_LAYER.iter() {
        let actions = click(without_body(), id);
        assert!(
            actions.is_empty(),
            "a layer chip clicked on an entity with NO physics body reached the \
             bus: {actions:?}"
        );
    }
}

/// Every dimension box. The values are distinct on purpose: a wiring mistake
/// that routes two fields to the same edit would still pass if they all
/// committed the same number.
#[test]
fn every_dimension_field_reaches_the_bus() {
    for (id, v, edit, what) in [
        (
            ids::INSP_PHYS_RADIUS,
            0.75,
            PhysicsFieldEdit::Radius(0.75),
            "Radius",
        ),
        (
            ids::INSP_PHYS_HALF_X,
            0.25,
            PhysicsFieldEdit::HalfX(0.25),
            "Half Width",
        ),
        (
            ids::INSP_PHYS_HALF_Y,
            0.125,
            PhysicsFieldEdit::HalfY(0.125),
            "Half Height",
        ),
        (
            ids::INSP_PHYS_DENSITY,
            2.5,
            PhysicsFieldEdit::Density(2.5),
            "Density",
        ),
        (
            ids::INSP_PHYS_RESTITUTION,
            0.875,
            PhysicsFieldEdit::Restitution(0.875),
            "Bounce",
        ),
        (
            ids::INSP_PHYS_FRICTION,
            1.5,
            PhysicsFieldEdit::Friction(1.5),
            "Friction",
        ),
    ] {
        expect(&commit(with_body(), id, v), edit, what);
    }
}

/// **The refusal lives in the event layer, not the paint loop.** The painter
/// only ever offers one of Add/Remove, but "the button isn't drawn" is not a
/// refusal — a stale hit rect, a scripted event, or a future layout change
/// can all deliver the click anyway
/// ([[feedback_disabled_button_still_dispatches]]).
#[test]
fn the_wrong_half_of_add_remove_is_refused_at_the_event_layer() {
    assert!(
        click(with_body(), ids::INSP_PHYS_ADD).is_empty(),
        "Add fired on an entity that ALREADY has a body — it would overwrite the artist's \
         collider with a fresh sprite-shaped one"
    );
    assert!(
        click(without_body(), ids::INSP_PHYS_REMOVE).is_empty(),
        "Remove fired on an entity with no body"
    );
}

/// A dimension commit on a bodyless entity must not reach the bus either:
/// there is no collider to write the number into.
#[test]
fn a_dimension_commit_without_a_body_is_refused() {
    assert!(
        commit(without_body(), ids::INSP_PHYS_DENSITY, 3.0).is_empty(),
        "a density commit fired on an entity with no collider"
    );
}

/// The section is offered for a plain sprite at all — the precondition every
/// gate above rests on. If the snapshot were built only for entities that
/// already have a body, `add_physics_body_reaches_the_bus` would still pass
/// while the button was unreachable in the product.
#[test]
fn the_panel_consumes_the_add_click_so_the_section_is_reachable() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(without_body()));
    let outcome = host
        .apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(ids::INSP_PHYS_ADD));
    set_current_inspector_physics(None);
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Add click was not consumed — the §11 arm is unreachable from apply_event"
    );
    let _ = InspectorPanel::ID;
}

/// **Join Selected Bodies is offered only when the shell says two bodies are
/// selected, and it is CLICKABLE when it is.**
///
/// `can_join` is computed by the shell — the only half that can see a
/// selection — and read twice: the painter decides whether to offer the
/// button, the event arm decides whether to honour the click. Both halves are
/// asserted here, because a button that is painted and not honoured (or
/// honoured and not painted) is the same bug from either side.
#[test]
fn join_is_offered_and_dispatched_only_for_two_selected_bodies() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    let joinable = InspectorPhysicsInfo {
        can_join: true,
        ..with_body()
    };

    // Painted only when joinable...
    for can in [false, true] {
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(InspectorPhysicsInfo {
            can_join: can,
            ..with_body()
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        assert_eq!(
            rects.iter().any(|(n, _)| *n == ids::INSP_PHYS_JOIN),
            can,
            "the Join button was painted for can_join={can}"
        );
    }

    // ...and honoured only when joinable. Dim is not a refusal
    // ([[feedback_disabled_button_still_dispatches]]): the id is in the store
    // all session, so the arm has to check for itself.
    expect(
        &click(joinable, ids::INSP_PHYS_JOIN),
        PhysicsFieldEdit::Join,
        "Join Selected Bodies",
    );
    assert!(
        click(with_body(), ids::INSP_PHYS_JOIN).is_empty(),
        "Join fired with can_join=false — the refusal lives only in the paint \
         loop, which is not a refusal"
    );
}
