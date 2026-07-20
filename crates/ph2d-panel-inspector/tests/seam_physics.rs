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
        bake_seconds: 5.0,
        is_sensor: false,
        bake_channels_tag: 0,
        gravity_scale: 1.0,
        cap_half_height: 0.25,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
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
    // Solid | Sensor. Each side asserts its OWN boolean, so a wiring that sent
    // both to `Sensor(false)` (or both to true) would fail — the two-way half-
    // dead widget this whole test exists to catch.
    for (i, &id) in ids::INSP_PHYS_SENSOR.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::Sensor(i == 1),
            &format!("Sensor toggle option {i}"),
        );
    }
    // Bake channel selector (All / Position / Rotation) — a Dynamic body, since
    // the selector is offered only there. Each option asserts its own tag.
    for (i, &id) in ids::INSP_PHYS_BAKE_CH.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::BakeChannels(i as u8),
            &format!("Bake channel option {i}"),
        );
    }
    // Discrete | Continuous (W-CCD) — a Dynamic body, since the toggle is offered
    // only there. Each side asserts its OWN boolean, so a wiring that sent both to
    // the same value would fail (the two-way half-dead widget this test catches).
    for (i, &id) in ids::INSP_PHYS_CCD.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::Ccd(i == 1),
            &format!("CCD toggle option {i}"),
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
        (
            ids::INSP_PHYS_CAP_HALF_H,
            0.625,
            PhysicsFieldEdit::CapHalfHeight(0.625),
            "Capsule Half Height",
        ),
        (
            ids::INSP_PHYS_LINVEL_X,
            3.5,
            PhysicsFieldEdit::LinvelX(3.5),
            "Init Vel X",
        ),
        (
            ids::INSP_PHYS_LINVEL_Y,
            -2.0,
            PhysicsFieldEdit::LinvelY(-2.0),
            "Init Vel Y",
        ),
    ] {
        expect(&commit(with_body(), id, v), edit, what);
    }
}

/// **The capsule's rows are offered only for a capsule** — and the box's only
/// for a box (W-capsule).
///
/// Each shape paints its OWN dimensions and no others: a Half Width box on a
/// capsule is a control that cannot do anything, and a Half Height row that
/// meant the box's half-extent on one shape and the capsule's straight segment
/// on another would be one control meaning two things. The ids are distinct, so
/// this asserts presence AND absence per shape — an absence gate without its
/// presence sibling would stay green with nothing painted at all
/// ([[feedback_absence_gate_needs_a_presence_sibling]]).
#[test]
fn each_shape_paints_only_its_own_dimension_rows() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    // shape_tag → (radius, half_x, half_y, capsule half-height)
    for (tag, want) in [
        (0u8, [true, false, false, false]),
        (1, [false, true, true, false]),
        (2, [true, false, false, true]),
    ] {
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(InspectorPhysicsInfo {
            shape_tag: tag,
            ..with_body()
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        let painted = |id| rects.iter().any(|(n, _)| *n == id);
        let got = [
            painted(ids::INSP_PHYS_RADIUS),
            painted(ids::INSP_PHYS_HALF_X),
            painted(ids::INSP_PHYS_HALF_Y),
            painted(ids::INSP_PHYS_CAP_HALF_H),
        ];
        assert_eq!(
            got, want,
            "shape_tag={tag}: wrong dimension rows painted \
             [radius, half_x, half_y, cap_half_height]"
        );
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

/// **The Bake button is painted, FOCUSABLE, and reaches the bus.** (W4.)
///
/// Driven through `click_at` — a real pointer Down/Up at the painted rect,
/// through the real dispatcher — rather than through `apply_event` directly.
/// The difference is the whole point: `apply_event` skips the store's
/// focusability check, so a button missing from `populate.rs` would be painted,
/// hit-registered and dead under the mouse while a synthetic-event gate stayed
/// green. That is the bug the W2c layer matrix was rewritten to catch, and this
/// gate's first version claimed to check focusability while doing exactly what
/// that rewrite was undoing.
///
/// It also asserts the LABEL carries the range: the number is the only thing
/// telling the artist how much of the timeline they are about to write, and a
/// button that silently baked five seconds when the document said two would be
/// worse than one that asked.
#[test]
fn the_bake_button_is_painted_and_reaches_the_bus() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    let mut host = Host::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_physics(Some(with_body()));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == ids::INSP_PHYS_BAKE)
        .map(|(_, r)| *r)
        .expect("the Bake button was never painted, so nothing can be clicked");

    // The real pointer path: this is what fails if the id is not registered.
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    set_current_inspector_physics(None);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(id) if *id == ids::INSP_PHYS_BAKE)),
        "a real click on the painted Bake rect produced no Click for it — the \
         id is not focusable in the store, so the button is dead under the mouse"
    );

    expect(
        &click(with_body(), ids::INSP_PHYS_BAKE),
        PhysicsFieldEdit::Bake,
        "Bake to Timeline",
    );

    // The label carries the resolved range — the painter builds it from this
    // same function, so a label that stopped showing the number would have to
    // change here.
    let label = ph2d_panel_inspector::bake_label(2.5);
    assert!(
        label.contains("2.5"),
        "the Bake button's label does not show the range it would cover: {label:?}"
    );

    // The empty face has no motion to bake, and offers nothing.
    assert!(
        click(without_body(), ids::INSP_PHYS_BAKE).is_empty(),
        "Bake fired on an entity with no body — there is no simulation to read"
    );
}

/// **Gravity Scale is offered and honoured only for a Dynamic body** (W8).
///
/// rapier applies gravity to Dynamic bodies only, so a Gravity Scale row on a
/// Static/Kinematic body would be a control that cannot do anything. Both
/// halves have to agree, the same as Bake: the painter decides whether to OFFER
/// the row and the event arm decides whether to HONOUR the commit, and a
/// refusal that lives only in the paint loop is not a refusal
/// ([[feedback_disabled_button_still_dispatches]]). The committed value is
/// distinct (`0.3`) so a wiring mistake routing it to another field would fail.
#[test]
fn gravity_scale_is_offered_and_committed_only_for_a_dynamic_body() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    // 0 Dynamic (offered) · 1 Static · 2 Kinematic.
    for (tag, offered) in [(0u8, true), (1, false), (2, false)] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        assert_eq!(
            rects
                .iter()
                .any(|(n, _)| *n == ids::INSP_PHYS_GRAVITY_SCALE),
            offered,
            "kind_tag={tag}: the Gravity Scale row's presence is wrong"
        );
        let actions = commit(info, ids::INSP_PHYS_GRAVITY_SCALE, 0.3);
        assert_eq!(
            !actions.is_empty(),
            offered,
            "kind_tag={tag}: the event handler disagrees with the painter about \
             whether Gravity Scale is offered"
        );
        if offered {
            expect(
                &actions,
                PhysicsFieldEdit::GravityScale(0.3),
                "Gravity Scale",
            );
        }
    }
}

/// **The three initial-velocity rows are offered and honoured only for a Dynamic
/// body, and the angular one converts deg/s → rad/s at the panel boundary** (W9).
///
/// Static/Kinematic bodies don't move under forces, so the launch is meaningless
/// there — the same Dynamic-only rule as gravity. Presence AND absence per kind
/// (an absence gate alone stays green with nothing painted). The angular commit
/// asserts the conversion: the panel shows deg/s, the component takes radians, so
/// a wiring that forgot the conversion would ship a body spinning 57× too fast.
#[test]
fn initial_velocity_is_offered_and_committed_only_for_a_dynamic_body() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    let rows = [
        ids::INSP_PHYS_LINVEL_X,
        ids::INSP_PHYS_LINVEL_Y,
        ids::INSP_PHYS_ANGVEL,
    ];
    // 0 Dynamic (offered) · 1 Static · 2 Kinematic.
    for (tag, offered) in [(0u8, true), (1, false), (2, false)] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let painted = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        for id in rows {
            assert_eq!(
                painted.iter().any(|(n, _)| *n == id),
                offered,
                "kind_tag={tag}: an initial-velocity row's presence is wrong"
            );
            assert_eq!(
                !commit(info, id, 1.0).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about \
                 an initial-velocity row"
            );
        }
    }

    // The angular commit converts: 90 deg/s in → Angvel(π/2) out.
    expect(
        &commit(with_body(), ids::INSP_PHYS_ANGVEL, 90.0),
        PhysicsFieldEdit::Angvel(90.0_f32.to_radians()),
        "Init Spin (deg/s → rad/s)",
    );
}

/// **The CCD toggle is offered and honoured only for a Dynamic body** (W-CCD).
///
/// Only a body the solver moves fast can tunnel, so a Static (never moves) or
/// Kinematic (pose-driven) body has nothing to sweep — the same Dynamic-only rule
/// as gravity and initial velocity. Presence AND absence per kind (an absence gate
/// alone stays green with nothing painted, [[feedback_absence_gate_needs_a_presence_sibling]]),
/// and both halves have to agree: the painter offers the row, the event arm honours
/// the click, and a refusal that lives only in the paint loop is not a refusal.
#[test]
fn ccd_is_offered_and_committed_only_for_a_dynamic_body() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    // 0 Dynamic (offered) · 1 Static · 2 Kinematic. The group id is the label
    // owner; the two option chips are what a click lands on.
    for (tag, offered) in [(0u8, true), (1, false), (2, false)] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        for &id in ids::INSP_PHYS_CCD.iter() {
            assert_eq!(
                rects.iter().any(|(n, _)| *n == id),
                offered,
                "kind_tag={tag}: the CCD toggle's presence is wrong"
            );
            assert_eq!(
                !click(info, id).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about \
                 whether CCD is offered"
            );
        }
    }
}

/// **Bake is neither offered nor honoured for a body with no simulated motion.**
///
/// A `Static` body never moves and a `Kinematic` one is already driven by the
/// scene, so a bake of either can only ever report "nothing moved" — a button
/// promising 5 seconds of work that is impossible for the body it points at.
/// Both halves have to agree: the painter decides whether to OFFER it and the
/// event handler decides whether to HONOUR it, and a refusal that lives only in
/// the paint loop is not a refusal.
#[test]
fn bake_is_offered_only_for_a_body_the_solver_moves() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    // 0 Dynamic (offered) · 1 Static · 2 Kinematic.
    for (tag, offered) in [(0u8, true), (1, false), (2, false)] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        assert_eq!(
            rects.iter().any(|(n, _)| *n == ids::INSP_PHYS_BAKE),
            offered,
            "kind_tag={tag}: the Bake button's presence is wrong"
        );
        assert_eq!(
            !click(info, ids::INSP_PHYS_BAKE).is_empty(),
            offered,
            "kind_tag={tag}: the event handler disagrees with the painter about \
             whether this body can be baked"
        );
    }
}
