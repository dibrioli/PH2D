//! **The BREAK half of the §12 seam** (W-J7) — the switch, the two thresholds,
//! and the one thing that makes them different from every other row in the
//! section: no unit conversion.
//!
//! Sibling of `inspector_joint_motor_tests` for the shell's 600-LOC file cap,
//! cut by subject.

use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::JointFieldEdit;
use ph2d_physics_ecs::{JointKind, PhysicsJoint};

use super::inspector_joint::{build_joint_info, joint_with_edit, kind_of};

/// **The two thresholds go out and come back VERBATIM** — a newton means the
/// same thing on both sides of this boundary.
///
/// Stated as its own gate precisely because every neighbouring row does convert
/// (the limits and the motor carry degrees on one kind and metres on another), so
/// "no conversion" is the exceptional case and the one a future refactor would
/// most plausibly break by pattern-matching on its neighbours.
///
/// Mutation: routing either row through `motor_in`/`motor_out` — a Pin's 250 N
/// becomes 4.363 in the component, which is what a *degree* would have become.
#[test]
fn a_break_threshold_is_carried_verbatim_in_newtons() {
    for kind_tag in [0u8, 1, 2, 3, 4] {
        let base = PhysicsJoint {
            kind: kind_of(kind_tag),
            break_enabled: true,
            ..PhysicsJoint::default()
        };
        for (edit, stored, shown) in [
            (
                JointFieldEdit::BreakForce(250.0),
                (|j: &PhysicsJoint| j.break_force) as fn(&PhysicsJoint) -> f32,
                (|i: &ph2d_editor::InspectorJointInfo| i.break_force)
                    as fn(&ph2d_editor::InspectorJointInfo) -> f32,
            ),
            (
                JointFieldEdit::BreakTorque(12.5),
                |j: &PhysicsJoint| j.break_torque,
                |i: &ph2d_editor::InspectorJointInfo| i.break_torque,
            ),
        ] {
            let typed = match edit {
                JointFieldEdit::BreakForce(v) | JointFieldEdit::BreakTorque(v) => v,
                _ => unreachable!(),
            };
            let after = joint_with_edit(base, edit).expect("a break edit lands");
            assert!(
                (stored(&after) - typed).abs() < 1e-4,
                "kind {kind_tag}: {typed} has to stay {typed} in the component, got {}",
                stored(&after)
            );
            let mut sim = SimWorld::new();
            let e = sim
                .world_mut()
                .spawn((Name::new("J"), after, Transform::default()))
                .id();
            let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
            assert!(
                (shown(&info) - typed).abs() < 1e-4,
                "kind {kind_tag}: the row has to show {typed}, shows {}",
                shown(&info)
            );
        }
    }
}

/// **A negative threshold is refused at the door.**
///
/// It is not a merely odd number: a threshold below zero is crossed by EVERY
/// load, so the joint would part on its first frame — the artist would see the
/// rig fall apart and have no way to read "minus one" as the cause.
#[test]
fn a_negative_threshold_is_clamped_to_zero() {
    let base = PhysicsJoint {
        break_enabled: true,
        ..PhysicsJoint::default()
    };
    let after = joint_with_edit(base, JointFieldEdit::BreakForce(-40.0)).expect("edit");
    assert!(
        after.break_force >= 0.0,
        "a negative break force is crossed by everything, got {}",
        after.break_force
    );
}

/// **The switch reaches the component, and the snapshot mirrors it back.**
#[test]
fn the_breakable_switch_writes_and_reads_back() {
    let base = PhysicsJoint::default();
    assert!(
        !base.break_enabled,
        "a fresh joint is unbreakable — the `∞ = off` default of P7"
    );
    for want in [true, false] {
        let after = joint_with_edit(
            PhysicsJoint {
                break_enabled: !want,
                ..base
            },
            JointFieldEdit::BreakEnabled(want),
        )
        .expect("edit");
        assert_eq!(after.break_enabled, want);
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Name::new("J"), after, Transform::default()))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert_eq!(info.break_enabled, want, "the snapshot mirrors it back");
    }
}

/// **The snapshot answers "can this kind report a torque?" from the ENGINE.**
///
/// The panel is loose-coupled and never sees `ph2d-physics-ecs`, so the answer
/// has to travel in the snapshot rather than be re-derived from `kind_tag` on the
/// far side — a second copy of the rule would be a second thing to keep true, and
/// the measurement that decides it (rapier reports nothing for a locked angular
/// axis) lives on the engine side.
///
/// Mutation: hardcoding `true` — four kinds go red here and the panel's own
/// sweep goes red with them.
#[test]
fn the_snapshot_carries_the_engines_answer_about_torque() {
    for (kind, want) in [
        (JointKind::Pin, true),
        (JointKind::Spring, false),
        (JointKind::Rope, false),
        (JointKind::Weld, false),
        (JointKind::Slider, false),
        // Uma barra deixa os dois olhais girarem: eixo angular LIVRE, torque
        // estruturalmente zero.
        (JointKind::Rod, false),
        // ⚠️ Uma RODA reporta, e foi MEDIDO: o eixo angular dela é livre com o
        // motor desligado (0,0000 N.m) e motorizado com ele ligado (0,5125) —
        // e quem manda e o estado em que a row pode ser alcançada.
        (JointKind::Wheel, true),
    ] {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                Name::new("J"),
                PhysicsJoint {
                    kind,
                    ..PhysicsJoint::default()
                },
                Transform::default(),
            ))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert_eq!(
            info.breaks_on_torque, want,
            "{kind:?}: the snapshot has to carry the engine's answer"
        );
    }
}

/// **O toast NOMEIA a joint e diz a carga com que ela partiu.**
///
/// É o único lugar onde esse número chega ao artista: o overlay mostra ONDE (o
/// estouro) e QUE está rompida (o vermelho), e nenhum dos dois pode carregá-lo —
/// um instante depois a joint lê carga zero, porque não está segurando nada.
///
/// Mutação: `break_reports` devolvendo uma lista vazia — o rompimento acontece,
/// a cena muda, e nada diz por quê.
#[test]
fn the_toast_names_the_joint_and_the_load_it_broke_at() {
    use ph2d_core::Vec2;
    use ph2d_physics_ecs::{
        BodyKind, Collider, ColliderShape, MassOverride, PhysicsBridge, RigidBody,
    };

    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Load"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            ..Collider::default()
        },
        MassOverride(10.0),
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Thin Rope"),
        PhysicsJoint {
            body_a: ph2d_ecs::stable_name_id("Hook"),
            body_b: ph2d_ecs::stable_name_id("Load"),
            kind: JointKind::Rope,
            max_length: 1.0,
            break_enabled: true,
            break_force: 50.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    let mut bridge = PhysicsBridge::default();
    let mut said = Vec::new();
    for t in 1..=90 {
        bridge.dispatch(&mut sim, true, t);
        said.extend(super::physics_bridge::break_reports(&bridge, &sim));
    }
    assert_eq!(said.len(), 1, "uma joint, um anuncio: {said:?}");
    assert!(
        said[0].starts_with("Thin Rope broke at "),
        "o toast nomeia a JOINT: {:?}",
        said[0]
    );
    assert!(
        said[0].ends_with(" N"),
        "e diz a carga em newtons: {:?}",
        said[0]
    );
}
