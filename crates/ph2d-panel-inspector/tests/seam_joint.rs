//! **Behavioral SEAM sweep for §12 Physics Joint** (W3).
//!
//! A SWEEP, not a sample: every control the section paints is exercised here
//! and its exact action asserted. Choosing "the fullest state" and trusting it
//! to cover the rest is the premise that has rotted twice in this repo
//! ([[feedback_the_fullest_card_premise_rots]]) — and here it would rot
//! immediately, because no single kind paints all the rows.
//!
//! ⚠️ **The click sweep drives `click_at` — the real dispatcher — and not a
//! synthetic `WidgetEvent`.** A synthetic event skips the store's focusability
//! check, so a widget left out of `populate` stays painted, hit-registered,
//! arm-wired and **dead under the mouse**, with a green test beside it. W2c
//! shipped that exact hole for a day and the fix was this.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::screens::hero::{InspectorJointInfo, JointFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_joint};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_0042;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

/// A joint of `kind_tag`, with both Pin switches ON so the section paints
/// every row it has. `bound`, because a connected joint is the state an artist
/// is in when they tune one.
fn joint(kind_tag: u8) -> InspectorJointInfo {
    InspectorJointInfo {
        entity_bits: ENTITY,
        kind_tag,
        body_a_name: "Hook".into(),
        body_b_name: "Plank".into(),
        bound: true,
        limits_enabled: true,
        limit_min_deg: -45.0,
        limit_max_deg: 45.0,
        motor_enabled: true,
        motor_speed_deg: 114.0,
        motor_max_force: 10.0,
        rest_length: 1.0,
        stiffness: 30.0,
        damping: 0.5,
        max_length: 2.0,
    }
}

/// Paint the section, then click the middle of the widget with `id` — through
/// the real pointer dispatcher. Returns the actions the panel raised.
fn click_real(info: InspectorJointInfo, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let rect = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("§12 never painted the widget {id:?}"));
    let events = host.click_at(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "clicking the middle of {id:?} produced {events:?} — the widget is \
         painted and hit-registered but the store does not consider it \
         focusable, so it is dead under the mouse"
    );
    for ev in events {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut state, ev);
    }
    let out = host.drained_actions();
    set_current_inspector_joint(None);
    out
}

fn commit(info: InspectorJointInfo, id: ph2d_a11y::NodeId, v: f64) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(Some(info));
    host.set_number_value(id, v);
    let _ = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::ValueChanged(id));
    let out = host.drained_actions();
    set_current_inspector_joint(None);
    out
}

#[track_caller]
fn expect(actions: &[EditorAction], edit: JointFieldEdit, what: &str) {
    assert_eq!(
        actions,
        [EditorAction::InspectorJointEdit {
            entity_bits: ENTITY,
            edit,
        }],
        "{what} did not raise the edit it is supposed to"
    );
}

/// **Every kind chip is clickable and picks its own kind.**
#[test]
fn the_kind_chips_each_pick_their_own_kind() {
    for (i, &id) in ids::INSP_JOINT_KIND.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::Kind(i as u8),
            &format!("kind chip {i}"),
        );
    }
}

/// **The two Pin switches are switches** — Off writes `false`, On writes
/// `true`, and each is reached by clicking it.
#[test]
fn the_pin_switches_write_the_side_they_are_on() {
    for (i, &id) in ids::INSP_JOINT_LIMITS.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::LimitsEnabled(i == 1),
            &format!("limits switch {i}"),
        );
    }
    for (i, &id) in ids::INSP_JOINT_MOTOR.iter().enumerate() {
        expect(
            &click_real(joint(0), id),
            JointFieldEdit::MotorEnabled(i == 1),
            &format!("motor switch {i}"),
        );
    }
}

/// **Delete Joint is wired.**
#[test]
fn delete_joint_is_dispatched() {
    expect(
        &click_real(joint(0), ids::INSP_JOINT_REMOVE),
        JointFieldEdit::Remove,
        "Delete Joint",
    );
}

/// **Every number box commits to its own field.**
///
/// The values are distinct on purpose: a handler that routed two boxes to the
/// same field would pass a sweep that committed the same number everywhere.
#[test]
fn each_number_box_commits_to_its_own_field() {
    let cases: [(u8, ph2d_a11y::NodeId, f64, JointFieldEdit); 8] = [
        (
            0,
            ids::INSP_JOINT_LIMIT_MIN,
            -30.0,
            JointFieldEdit::LimitMinDeg(-30.0),
        ),
        (
            0,
            ids::INSP_JOINT_LIMIT_MAX,
            60.0,
            JointFieldEdit::LimitMaxDeg(60.0),
        ),
        (
            0,
            ids::INSP_JOINT_MOTOR_SPEED,
            90.0,
            JointFieldEdit::MotorSpeedDeg(90.0),
        ),
        (
            0,
            ids::INSP_JOINT_MOTOR_FORCE,
            7.5,
            JointFieldEdit::MotorMaxForce(7.5),
        ),
        (
            1,
            ids::INSP_JOINT_REST_LENGTH,
            2.5,
            JointFieldEdit::RestLength(2.5),
        ),
        (
            1,
            ids::INSP_JOINT_STIFFNESS,
            42.0,
            JointFieldEdit::Stiffness(42.0),
        ),
        (
            1,
            ids::INSP_JOINT_DAMPING,
            3.25,
            JointFieldEdit::Damping(3.25),
        ),
        (
            2,
            ids::INSP_JOINT_MAX_LENGTH,
            4.75,
            JointFieldEdit::MaxLength(4.75),
        ),
    ];
    for (kind, id, v, edit) in cases {
        expect(&commit(joint(kind), id, v), edit, &format!("{id:?}"));
    }
}

/// **Every row the section paints for a kind belongs to that kind.**
///
/// The anti-dead-knob sweep, asked of EACH kind rather than of "the fullest
/// one": a rope must not paint a stiffness box, a spring must not paint a
/// motor. The rule comes from `JointKind::is_hinge`/`has_length`, which the
/// bridge also asks before handing parameters to the solver — so a knob
/// painted here that the solver ignores would be a knob that does nothing.
#[test]
fn each_kind_paints_only_the_rows_it_uses() {
    const PIN_ONLY: [ph2d_a11y::NodeId; 4] = [
        ids::INSP_JOINT_LIMIT_MIN,
        ids::INSP_JOINT_LIMIT_MAX,
        ids::INSP_JOINT_MOTOR_SPEED,
        ids::INSP_JOINT_MOTOR_FORCE,
    ];
    const SPRING_ONLY: [ph2d_a11y::NodeId; 3] = [
        ids::INSP_JOINT_REST_LENGTH,
        ids::INSP_JOINT_STIFFNESS,
        ids::INSP_JOINT_DAMPING,
    ];
    const ROPE_ONLY: [ph2d_a11y::NodeId; 1] = [ids::INSP_JOINT_MAX_LENGTH];

    // 0..4 includes Weld (kind 3), which owns NO parameter rows: it never
    // equals an owner below, so every PIN/SPRING/ROPE row must go unpainted for
    // it. A Weld that leaked a param row (a length, a motor) would fail here.
    for kind in 0u8..4 {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(joint(kind)));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);

        for (owner, ids_) in [
            (0u8, &PIN_ONLY[..]),
            (1, &SPRING_ONLY[..]),
            (2, &ROPE_ONLY[..]),
        ] {
            for &id in ids_ {
                assert_eq!(
                    painted(id),
                    kind == owner,
                    "kind {kind} {} {id:?}, which belongs to kind {owner}",
                    if kind == owner {
                        "must paint"
                    } else {
                        "must not paint"
                    }
                );
            }
        }
        // The kind chips and Delete are painted for every kind — the control
        // group that is always there. Without this the assertions above would
        // be satisfied by a section that painted nothing at all.
        for id in ids::INSP_JOINT_KIND.iter().chain(&[ids::INSP_JOINT_REMOVE]) {
            assert!(painted(*id), "kind {kind} did not paint {id:?}");
        }
    }
}

/// **The switches' rows appear only when the switch is ON.**
///
/// A limit box for a hinge with limits off is a control that changes a number
/// nothing reads — the §11 rule about a radius on a box, one level in.
#[test]
fn the_pin_rows_follow_their_switches() {
    for (limits, motor) in [(false, false), (true, false), (false, true)] {
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        set_current_inspector_joint(Some(InspectorJointInfo {
            limits_enabled: limits,
            motor_enabled: motor,
            ..joint(0)
        }));
        let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        set_current_inspector_joint(None);
        let painted = |id: ph2d_a11y::NodeId| rects.iter().any(|(n, _)| *n == id);
        assert_eq!(painted(ids::INSP_JOINT_LIMIT_MIN), limits);
        assert_eq!(painted(ids::INSP_JOINT_LIMIT_MAX), limits);
        assert_eq!(painted(ids::INSP_JOINT_MOTOR_SPEED), motor);
        assert_eq!(painted(ids::INSP_JOINT_MOTOR_FORCE), motor);
    }
}

/// **With no joint selected, none of these ids does anything.**
///
/// The ids live in the store for the whole session, so an event arm that did
/// not check the section is live would fire on whatever entity happened to be
/// selected — writing joint parameters onto a sprite.
#[test]
fn the_joint_arms_are_silent_when_nothing_is_a_joint() {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_joint(None);
    for id in [
        ids::INSP_JOINT_KIND[1],
        ids::INSP_JOINT_REMOVE,
        ids::INSP_JOINT_LIMITS[1],
        ids::INSP_JOINT_MOTOR[0],
    ] {
        let outcome = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(id));
        assert!(
            !matches!(outcome, EventOutcome::Consumed),
            "{id:?} was consumed with no joint selected"
        );
    }
    assert!(
        host.drained_actions().is_empty(),
        "the §12 arms raised an action with no joint selected"
    );
}
