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
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_manual: false,
        mass: 1.0,
        dominance: 0,
        restitution_combine_tag: 0,
        friction_combine_tag: 0,
        linear_damping: 0.0,
        angular_damping: 0.0,
        damp_mode_tag: 0,
        one_way: false,
        force: [0.0, 0.0],
        area_drag: 0.0,
        area_density: 0.0,
        area_form_drag: 0.0,
        area_torque: 0.0,
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
    // Free | Locked (Freeze Rotation) — Dynamic-only, each side its own boolean.
    for (i, &id) in ids::INSP_PHYS_LOCKROT.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::LockRotation(i == 1),
            &format!("Lock-rotation toggle option {i}"),
        );
    }
    // Free | Locked (Freeze Position X) — Dynamic-only, each side its own boolean.
    for (i, &id) in ids::INSP_PHYS_LOCKX.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::LockPositionX(i == 1),
            &format!("Freeze-Position-X toggle option {i}"),
        );
    }
    // Free | Locked (Freeze Position Y) — the vertical sibling.
    for (i, &id) in ids::INSP_PHYS_LOCKY.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::LockPositionY(i == 1),
            &format!("Freeze-Position-Y toggle option {i}"),
        );
    }
    // Auto | Manual (Mass source) — each side its own boolean.
    for (i, &id) in ids::INSP_PHYS_MASSMODE.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::MassMode(i == 1),
            &format!("Mass-mode toggle option {i}"),
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
            ids::INSP_PHYS_OFFSET_X,
            0.375,
            PhysicsFieldEdit::OffsetX(0.375),
            "Collider Offset X",
        ),
        (
            ids::INSP_PHYS_OFFSET_Y,
            -0.875,
            PhysicsFieldEdit::OffsetY(-0.875),
            "Collider Offset Y",
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

/// **The Dominance row is offered and honoured only for a Dynamic body, and the
/// widget's float is rounded to the i8 priority** (W-Dominance).
///
/// A non-dynamic body is already at the maximum dominance, so the row would be a
/// control that cannot do anything — the same Dynamic-only rule as gravity/velocity.
/// Presence AND absence per kind, and the commit asserts the conversion: the widget
/// is a float, the edit is an `i8`, so `5.0` in becomes `Dominance(5)` out.
#[test]
fn dominance_is_offered_and_committed_only_for_a_dynamic_body() {
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
            rects.iter().any(|(n, _)| *n == ids::INSP_PHYS_DOMINANCE),
            offered,
            "kind_tag={tag}: the Dominance row's presence is wrong"
        );
        assert_eq!(
            !commit(info, ids::INSP_PHYS_DOMINANCE, 5.0).is_empty(),
            offered,
            "kind_tag={tag}: the event handler disagrees with the painter about \
             whether Dominance is offered"
        );
    }
    // The commit rounds the float to the i8 priority: 5.0 in → Dominance(5) out.
    expect(
        &commit(with_body(), ids::INSP_PHYS_DOMINANCE, 5.0),
        PhysicsFieldEdit::Dominance(5),
        "Dominance (float → i8)",
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

/// **The Lock-rotation toggle is offered and honoured only for a Dynamic body**
/// (Freeze Rotation, W-LockRot).
///
/// Only a body the solver rotates under forces has a rotation to freeze, so a
/// Static (never moves) or Kinematic (pose-driven) body has nothing to lock — the
/// same Dynamic-only rule as gravity / velocity / CCD. Presence AND absence per
/// kind, and both halves have to agree (the painter offers, the event arm honours;
/// a refusal that lives only in the paint loop is not a refusal).
#[test]
fn lock_rotation_is_offered_and_committed_only_for_a_dynamic_body() {
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
        for &id in ids::INSP_PHYS_LOCKROT.iter() {
            assert_eq!(
                rects.iter().any(|(n, _)| *n == id),
                offered,
                "kind_tag={tag}: the Lock-rotation toggle's presence is wrong"
            );
            assert_eq!(
                !click(info, id).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about \
                 whether Lock-rotation is offered"
            );
        }
    }
}

/// **The Mass source is Auto|Manual, Dynamic-only, and the Mass row REPLACES the
/// Density row in Manual mode** (W-Mass).
///
/// Density and mass are the same quantity by two roads, so exactly one is ever live
/// — showing both would be the two-doors bug. This asserts: the toggle's presence
/// per kind (Dynamic-only, since a Static/Kinematic body has infinite mass); that
/// Auto paints Density and NOT Mass, while Manual paints Mass and NOT Density
/// (presence AND absence — an absence gate alone stays green with nothing painted);
/// and that the Mass commit reaches the bus.
#[test]
fn mass_source_toggle_swaps_density_for_mass_and_is_dynamic_only() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };

    let painted = |info: InspectorPhysicsInfo| {
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        let has = |id| rects.iter().any(|(n, _)| *n == id);
        (
            has(ids::INSP_PHYS_MASSMODE[0]),
            has(ids::INSP_PHYS_DENSITY),
            has(ids::INSP_PHYS_MASS),
        )
    };

    // Dynamic Auto: the toggle + Density, NOT Mass.
    assert_eq!(
        painted(with_body()),
        (true, true, false),
        "Dynamic Auto should paint the Mass toggle + Density, not the Mass row"
    );
    // Dynamic Manual: the toggle + Mass, NOT Density (the swap).
    assert_eq!(
        painted(InspectorPhysicsInfo {
            mass_manual: true,
            ..with_body()
        }),
        (true, false, true),
        "Dynamic Manual should paint the Mass toggle + Mass row, not Density (the two \
         are one quantity, so exactly one is live)"
    );
    // Static: no toggle, plain Density (unchanged from before this existed).
    assert_eq!(
        painted(InspectorPhysicsInfo {
            kind_tag: 1,
            ..with_body()
        }),
        (false, true, false),
        "a Static body should keep the plain Density row and NOT offer the Mass toggle"
    );

    // The toggle is honoured only for a Dynamic body.
    for (tag, offered) in [(0u8, true), (1, false), (2, false)] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        for &id in ids::INSP_PHYS_MASSMODE.iter() {
            assert_eq!(
                !click(info, id).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about \
                 whether the Mass toggle is offered"
            );
        }
    }

    // The Mass commit reaches the bus (Manual mode, Dynamic) with its own value.
    expect(
        &commit(
            InspectorPhysicsInfo {
                mass_manual: true,
                ..with_body()
            },
            ids::INSP_PHYS_MASS,
            12.5,
        ),
        PhysicsFieldEdit::Mass(12.5),
        "Mass (kg)",
    );
}

/// **The two Freeze-Position toggles are offered and honoured only for a Dynamic
/// body** (Freeze Position, W-LockPos).
///
/// Only a body the solver MOVES under forces has a position to freeze, so a Static
/// (never moves) or Kinematic (pose-driven) body has nothing to lock — the same
/// Dynamic-only rule the other constraints follow. Presence AND absence per kind,
/// and both halves have to agree (the painter offers, the event arm honours; a
/// refusal that lives only in the paint loop is not a refusal). The X and Y groups
/// are swept independently, so a wiring that pointed both at one axis fails.
#[test]
fn freeze_position_is_offered_and_committed_only_for_a_dynamic_body() {
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
        for &id in ids::INSP_PHYS_LOCKX
            .iter()
            .chain(ids::INSP_PHYS_LOCKY.iter())
        {
            assert_eq!(
                rects.iter().any(|(n, _)| *n == id),
                offered,
                "kind_tag={tag}: a Freeze-Position toggle's presence is wrong"
            );
            assert_eq!(
                !click(info, id).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about \
                 whether Freeze Position is offered"
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

/// **The two combine controls are offered for EVERY body kind — not Dynamic-only —
/// and each of the four options reaches the bus with its OWN tag** (W-Material).
///
/// This is the property that separates them from gravity/velocity/dominance: a
/// combine rule is a collider MATERIAL property, so a static floor's rule matters
/// too. The gate would catch a copy-paste that gated them on `kind_tag == 0` like
/// their neighbours (they would then vanish on a Static floor — the exact body a
/// bouncy ball lands on). Each option asserts its own tag on its own group, so a
/// wiring that pointed both controls at one component field, or every chip at
/// `Average`, would fail. And the bodyless entity refuses them — dim is not a
/// refusal ([[feedback_disabled_button_still_dispatches]]).
#[test]
fn combine_rules_are_offered_for_every_kind_and_each_option_reaches_the_bus() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };

    // Painted for ALL three kinds — the "not Dynamic-only" property. A Static floor
    // is exactly where a Max-combine rule earns its keep, so gating it on Dynamic
    // would delete it from the one body the artist most needs it on.
    for tag in [0u8, 1, 2] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        for &id in ids::INSP_PHYS_REST_COMBINE
            .iter()
            .chain(ids::INSP_PHYS_FRIC_COMBINE.iter())
        {
            assert!(
                rects.iter().any(|(n, _)| *n == id),
                "kind_tag={tag}: a combine-rule chip was not painted — a material \
                 property must be offered for every body kind, not just Dynamic"
            );
        }
    }

    // Each option on each group dispatches its OWN tag onto its OWN edit. Distinct
    // per index so a wiring that sent every chip to Average (or crossed the two
    // groups) would fail.
    for (i, &id) in ids::INSP_PHYS_REST_COMBINE.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::RestitutionCombine(i as u8),
            &format!("Bounce Combine option {i}"),
        );
    }
    for (i, &id) in ids::INSP_PHYS_FRIC_COMBINE.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::FrictionCombine(i as u8),
            &format!("Friction Combine option {i}"),
        );
    }

    // A combine chip on a bodyless entity is refused at the event layer — there is
    // no collider to give a material to.
    for &id in ids::INSP_PHYS_REST_COMBINE
        .iter()
        .chain(ids::INSP_PHYS_FRIC_COMBINE.iter())
    {
        assert!(
            click(without_body(), id).is_empty(),
            "a combine-rule chip reached the bus on an entity with no body"
        );
    }
}

/// **The damping rows (2 values + the Combine|Replace mode) are Dynamic-only, and each
/// reaches the bus with its own field** (W-Damping).
///
/// Damping decays a velocity only a Dynamic body has, so a Static/Kinematic body must
/// not offer the rows — the same rule gravity/velocity follow. Presence AND absence per
/// kind (an absence gate alone stays green with nothing painted). The two values commit
/// distinct numbers and the two mode chips their own booleans, so a wiring that crossed
/// them (or sent both mode chips to one value) would fail.
#[test]
fn damping_rows_are_dynamic_only_and_each_reaches_the_bus() {
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
        // The two number boxes AND both mode chips are painted only for a Dynamic body.
        for &id in [
            ids::INSP_PHYS_LINEAR_DAMPING,
            ids::INSP_PHYS_ANGULAR_DAMPING,
            ids::INSP_PHYS_DAMPMODE[0],
            ids::INSP_PHYS_DAMPMODE[1],
        ]
        .iter()
        {
            assert_eq!(
                rects.iter().any(|(n, _)| *n == id),
                offered,
                "kind_tag={tag}: a damping control's presence is wrong"
            );
        }
        // The mode chips honour the click only for a Dynamic body.
        for &id in ids::INSP_PHYS_DAMPMODE.iter() {
            assert_eq!(
                !click(info, id).is_empty(),
                offered,
                "kind_tag={tag}: the event handler disagrees with the painter about the \
                 damp-mode toggle"
            );
        }
        // The value commits honour only for a Dynamic body.
        assert_eq!(
            !commit(info, ids::INSP_PHYS_LINEAR_DAMPING, 3.0).is_empty(),
            offered,
            "kind_tag={tag}: Linear Damping commit / painter disagree"
        );
    }

    // Each value commits its OWN field with its OWN number; each mode chip its OWN bool.
    expect(
        &commit(with_body(), ids::INSP_PHYS_LINEAR_DAMPING, 3.0),
        PhysicsFieldEdit::LinearDamping(3.0),
        "Linear Damping",
    );
    expect(
        &commit(with_body(), ids::INSP_PHYS_ANGULAR_DAMPING, 1.25),
        PhysicsFieldEdit::AngularDamping(1.25),
        "Angular Damping",
    );
    for (i, &id) in ids::INSP_PHYS_DAMPMODE.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::DampMode(i as u8),
            &format!("Damp Mode option {i}"),
        );
    }
}

/// **The One-Way toggle is offered for EVERY body kind — not Dynamic-only — and each
/// side reaches the bus with its own boolean** (W-OneWay).
///
/// A jump-through platform is a COLLIDER property and a platform is almost always
/// **Static**, so a Dynamic-only gate copied from its neighbours would delete the
/// control from the exact body the feature exists for. Presence per kind, plus each
/// side asserting its OWN boolean (a wiring that sent both chips to `true` — the
/// classic half-dead two-way widget — fails here), plus the bodyless refusal.
#[test]
fn one_way_is_offered_for_every_kind_and_each_option_reaches_the_bus() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };

    // Painted for ALL three kinds — a Static platform is the whole point.
    for tag in [0u8, 1, 2] {
        let info = InspectorPhysicsInfo {
            kind_tag: tag,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        for &id in ids::INSP_PHYS_ONEWAY.iter() {
            assert!(
                rects.iter().any(|(n, _)| *n == id),
                "kind_tag={tag}: a One-Way chip was not painted — a jump-through platform is \
                 usually STATIC, so gating this on Dynamic deletes it from its own use case"
            );
        }
    }

    // Each side its own boolean.
    for (i, &id) in ids::INSP_PHYS_ONEWAY.iter().enumerate() {
        expect(
            &click(with_body(), id),
            PhysicsFieldEdit::OneWay(i == 1),
            &format!("One-Way toggle option {i}"),
        );
    }

    // ⚠️ NOT offered on a SENSOR (W-Area made the two exclusive): one-way works by
    // modifying solver CONTACTS and a sensor generates none, so the chip would be dead.
    // Painted and dispatched are checked separately — dim is not a refusal.
    {
        let sensor = InspectorPhysicsInfo {
            is_sensor: true,
            ..with_body()
        };
        let mut host = Host::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_physics(Some(sensor));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_physics(None);
        for &id in ids::INSP_PHYS_ONEWAY.iter() {
            assert!(
                !rects.iter().any(|(n, _)| *n == id),
                "a One-Way chip was painted on a SENSOR, where it cannot do anything"
            );
            assert!(
                click(sensor, id).is_empty(),
                "a One-Way chip reached the bus on a SENSOR"
            );
        }
    }

    // Refused on a bodyless entity — there is no collider to make one-way.
    for &id in ids::INSP_PHYS_ONEWAY.iter() {
        assert!(
            click(without_body(), id).is_empty(),
            "a One-Way chip reached the bus on an entity with no body"
        );
    }
}

/// **The area rows (Force X/Y + Drag) are SENSOR-only, and each reaches the bus**
/// (W-Area, W-AreaDrag).
///
/// The first §11 control gated on another CONTROL rather than on `kind_tag`, so the
/// sweep has to ask a question it never asked before: the same body, the same kind,
/// two different Trigger settings. Offered for EVERY kind, because a wind column is
/// almost always Static — the mirror of the One-Way rule, in the other mode.
#[test]
fn the_force_rows_are_sensor_only_and_each_axis_reaches_the_bus() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_ui_testkit::MockPanelHost as Host;

    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 320.0,
        h: 2400.0,
    };
    // ⚠️ All THREE, including Drag (W-AreaDrag). A sweep that enumerates the rows it
    // knows about is the premise that rots: the next row added to the sensor block
    // must be in this list, and the seam is where its absence shows.
    const FORCE_IDS: [ph2d_a11y::NodeId; 6] = [
        ids::INSP_PHYS_FORCE_X,
        ids::INSP_PHYS_FORCE_Y,
        ids::INSP_PHYS_AREA_TORQUE,
        ids::INSP_PHYS_AREA_DRAG,
        ids::INSP_PHYS_AREA_DENSITY,
        ids::INSP_PHYS_AREA_FORM_DRAG,
    ];

    for tag in [0u8, 1, 2] {
        for is_sensor in [true, false] {
            let info = InspectorPhysicsInfo {
                kind_tag: tag,
                is_sensor,
                ..with_body()
            };
            let mut host = Host::with_panel::<InspectorPanel>();
            let mut state = InspectorState::default();
            set_current_inspector_physics(Some(info));
            let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
            set_current_inspector_physics(None);
            for &id in FORCE_IDS.iter() {
                assert_eq!(
                    rects.iter().any(|(n, _)| *n == id),
                    is_sensor,
                    "kind_tag={tag}, is_sensor={is_sensor}: a Force row's presence is wrong — \
                     a force zone is usually STATIC, and it is meaningless on a solid collider"
                );
            }
        }
    }

    // Each axis its own edit, on a sensor.
    let sensor = InspectorPhysicsInfo {
        is_sensor: true,
        ..with_body()
    };
    expect(
        &commit(sensor, ids::INSP_PHYS_FORCE_X, 12.5),
        PhysicsFieldEdit::ForceX(12.5),
        "Force X",
    );
    expect(
        &commit(sensor, ids::INSP_PHYS_FORCE_Y, -3.25),
        PhysicsFieldEdit::ForceY(-3.25),
        "Force Y",
    );
    // ⚠️ A NEGATIVE value on purpose (W-AreaTorque): the sign is the spin direction, so it
    // must survive the event layer intact — a clamp here (as the drag rows take) would
    // silently drop the clockwise half.
    expect(
        &commit(sensor, ids::INSP_PHYS_AREA_TORQUE, -7.5),
        PhysicsFieldEdit::AreaTorque(-7.5),
        "Torque",
    );
    expect(
        &commit(sensor, ids::INSP_PHYS_AREA_DRAG, 4.0),
        PhysicsFieldEdit::AreaDrag(4.0),
        "Area Drag",
    );
    expect(
        &commit(sensor, ids::INSP_PHYS_AREA_DENSITY, 6.0),
        PhysicsFieldEdit::AreaDensity(6.0),
        "Fluid Density",
    );
    expect(
        &commit(sensor, ids::INSP_PHYS_AREA_FORM_DRAG, 2.5),
        PhysicsFieldEdit::AreaFormDrag(2.5),
        "Shape Drag",
    );

    // Refused on a SOLID collider and on a bodyless entity — the narrow phase records
    // no overlap for either, so the force would be authored and inert.
    for &id in FORCE_IDS.iter() {
        assert!(
            commit(with_body(), id, 12.5).is_empty(),
            "a Force row reached the bus on a SOLID collider, where it does nothing"
        );
        assert!(
            commit(without_body(), id, 12.5).is_empty(),
            "a Force row reached the bus on an entity with no body"
        );
    }
}
