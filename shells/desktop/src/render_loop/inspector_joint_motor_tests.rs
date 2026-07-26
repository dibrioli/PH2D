//! **The MOTOR's half of the §12 seam** (W-J6) — the mode, the two numbers, and
//! the unit each of them carries.
//!
//! Sibling of `inspector_joint_tests` for the shell's 600-LOC file cap, and the
//! cut is by subject: everything here is about *what a driven joint was told*,
//! which is a question three kinds now share.

use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor::JointFieldEdit;
use ph2d_physics_ecs::{JointKind, MotorMode, PhysicsJoint};

use super::inspector_joint::{build_joint_info, joint_with_edit, kind_of};

/// **A motor number round-trips in its KIND's own unit, and the stored value is
/// the one the solver reads.**
///
/// The exact shape of `a_limit_round_trips_in_its_kinds_own_unit`, and it is
/// here for the same reason that one exists: a round-trip alone is a weak
/// oracle, because a conversion pair that is *consistently wrong* (always
/// converting, ignoring the kind) goes out and back perfectly while the winch
/// reels in degrees. The second assertion is the one that can fail.
///
/// ⚠️ The Rope is the case that needed a SEPARATE door: `limits_in_metres` says
/// `false` for it (no limit range at all) and `motor_in_metres` says `true`.
///
/// Mutation: `motor_in`/`motor_out` reading `limits_in_metres` — RED on the Rope
/// row, and only on it.
#[test]
fn a_motor_number_round_trips_in_its_kinds_own_unit() {
    // (kind tag, typed into the row, what the component must hold)
    for (kind_tag, typed, stored) in [
        (0u8, 90.0_f32, std::f32::consts::FRAC_PI_2), // Pin: degrees -> radians
        (4, 0.75, 0.75),                              // Slider: metres, verbatim
        (2, 0.25, 0.25),                              // Rope (winch): metres, verbatim
    ] {
        let base = PhysicsJoint {
            kind: kind_of(kind_tag),
            motor_enabled: true,
            ..PhysicsJoint::default()
        };
        for (edit, read_stored, read_ui) in [
            (
                JointFieldEdit::MotorSpeed(typed),
                (|j: &PhysicsJoint| j.motor_speed) as fn(&PhysicsJoint) -> f32,
                (|i: &ph2d_editor::InspectorJointInfo| i.motor_speed_ui)
                    as fn(&ph2d_editor::InspectorJointInfo) -> f32,
            ),
            (
                JointFieldEdit::MotorTarget(typed),
                |j: &PhysicsJoint| j.motor_target,
                |i: &ph2d_editor::InspectorJointInfo| i.motor_target_ui,
            ),
        ] {
            let after = joint_with_edit(base, edit).expect("a motor edit lands");
            assert!(
                (read_stored(&after) - stored).abs() < 1e-4,
                "kind {kind_tag}: {typed} has to BECOME {stored} in the component, got {}",
                read_stored(&after)
            );
            let mut sim = SimWorld::new();
            let e = sim
                .world_mut()
                .spawn((Name::new("J"), after, Transform::default()))
                .id();
            let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
            assert!(
                (read_ui(&info) - typed).abs() < 1e-3,
                "kind {kind_tag}: typed {typed}, the row shows {}",
                read_ui(&info)
            );
        }
    }
}

/// **Changing the motor's UNIT re-seeds its numbers — and changing kind within
/// one unit does not.**
///
/// Twin of `changing_the_limit_unit_re_seeds_the_range_and_nothing_else_does`,
/// and deliberately gated on a DIFFERENT question: the limits' door and the
/// motor's disagree about a Rope. Sharing the condition would leave a winch
/// running at 114 m/s under a label reading metres — the limits' door never
/// flips on a Pin→Rope change.
///
/// Mutation: the re-seed gated on `limits_in_metres` instead — RED on the
/// Pin→Rope half, which is exactly the case the two doors differ on.
#[test]
fn changing_the_motor_unit_re_seeds_its_numbers_and_nothing_else_does() {
    let pin = PhysicsJoint {
        kind: JointKind::Pin,
        motor_enabled: true,
        motor_speed: 2.0, // rad/s
        motor_target: 0.4,
        ..PhysicsJoint::default()
    };
    // Pin -> Slider AND Pin -> Rope: the unit goes angular -> linear, so both
    // numbers are restated. The Rope is the half the limits' door cannot see.
    for tag in [4u8, 2] {
        let linear = joint_with_edit(pin, JointFieldEdit::Kind(tag)).expect("kind edit");
        let want = PhysicsJoint::default_motor_speed(kind_of(tag));
        assert!(
            (linear.motor_speed - want).abs() < 1e-6,
            "Pin->{tag} has to re-seed the speed in m/s, got {}",
            linear.motor_speed
        );
        assert!(
            linear.motor_target.abs() < 1e-6,
            "Pin->{tag} has to reset the target, got {}",
            linear.motor_target
        );
    }
    // Pin -> Weld: SAME unit, so the artist's numbers survive — the promise the
    // component makes about switching kinds.
    let weld = joint_with_edit(pin, JointFieldEdit::Kind(3)).expect("kind edit");
    assert!(
        (weld.motor_speed - 2.0).abs() < 1e-6 && (weld.motor_target - 0.4).abs() < 1e-6,
        "Pin->Weld has to PRESERVE the motor numbers, got {} / {}",
        weld.motor_speed,
        weld.motor_target
    );
}

/// **The mode chip reaches the component**, in both directions.
#[test]
fn the_mode_chip_writes_the_mode_and_the_snapshot_reads_it_back() {
    let base = PhysicsJoint {
        motor_enabled: true,
        ..PhysicsJoint::default()
    };
    assert_eq!(
        base.motor_mode,
        MotorMode::Velocity,
        "a fresh motor is a rate"
    );
    for (tag, want) in [(1u8, MotorMode::Position), (0, MotorMode::Velocity)] {
        let after = joint_with_edit(base, JointFieldEdit::MotorMode(tag)).expect("mode edit");
        assert_eq!(after.motor_mode, want, "tag {tag}");
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((Name::new("J"), after, Transform::default()))
            .id();
        let info = build_joint_info(&mut sim, e.to_bits(), 0).expect("info");
        assert_eq!(
            info.motor_mode_tag, tag,
            "the snapshot has to mirror it back"
        );
    }
}

/// **Criar um Slider e converter um Pin em Slider chegam ao MESMO joint** — e as
/// duas metades são afirmadas contra os NÚMEROS, não uma contra a outra.
///
/// Duas rotas para um estado que discordam é a forma que este arquivo inteiro
/// recusa, e ela existiu aqui: a troca de tipo re-semeava os números da unidade
/// desde o W-J5, e `create_joint` construía `PhysicsJoint { kind, ..default() }`
/// — os do Pin. *"Join As = Slider"* dava um trilho com ±0,785 **metros** de
/// curso, *"faça um Pin e troque"* dava ±0,5.
///
/// ⚠️ **A 1ª versão deste gate era VERDE POR CONSTRUÇÃO** e eu a escrevi na mesma
/// sessão em que documentei o padrão três vezes: ela comparava
/// `joint_with_edit(…)` com `PhysicsJoint::of_kind(…)`, e o re-seed da troca de
/// tipo LÊ o `of_kind` — então neutralizar essa função adoecia os dois lados
/// igualmente e a igualdade sobrevivia. Uma razão entre dois doentes. Agora cada
/// rota é comparada com a constante que ela deve produzir, e a igualdade entre
/// elas é a terceira asserção em vez da única.
///
/// Mutação: `of_kind` voltando a `Self { kind, ..default() }` — RED nas duas
/// rotas; `create_joint_at` voltando a `..default()` — RED só na de criação, que
/// é a metade que a mutação de fato quebra.
#[test]
fn creating_a_kind_and_converting_to_it_reach_the_same_joint() {
    // (tag, curso esperado, velocidade esperada) — as constantes, escritas aqui
    // e não derivadas da função sob teste.
    for (tag, stroke, speed) in [
        (2u8, PhysicsJoint::DEFAULT_LIMIT, -0.5f32), // Rope: sem faixa (fica em rad), winch recolhe
        (4, PhysicsJoint::DEFAULT_STROKE, 0.5),      // Slider: curso em metros, carro avança
    ] {
        let converted =
            joint_with_edit(PhysicsJoint::default(), JointFieldEdit::Kind(tag)).expect("kind edit");
        let created = create_of_kind(kind_of(tag));
        for (what, got, want) in [
            ("curso convertido", converted.limit_max, stroke),
            ("curso criado", created.limit_max, stroke),
            ("velocidade convertida", converted.motor_speed, speed),
            ("velocidade criada", created.motor_speed, speed),
        ] {
            assert!(
                (got - want).abs() < 1e-6,
                "tag {tag} {what}: esperado {want}, got {got}"
            );
        }
    }
}

/// O joint que o gesto de CRIAÇÃO de fato produz — pelo `create_joint` real, não
/// por uma re-derivação. É a rota que a mutação do `create_joint_at` quebra.
fn create_of_kind(kind: JointKind) -> PhysicsJoint {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((Name::new("A"), Transform::default()))
        .id();
    let b = sim
        .world_mut()
        .spawn((
            Name::new("B"),
            Transform::from_translation(ph2d_core::Vec2::new(1.0, 0.0)),
        ))
        .id();
    let j = super::inspector_joint::create_joint(&mut sim, a.to_bits(), b.to_bits(), kind)
        .expect("create");
    *sim.world().get::<PhysicsJoint>(j).expect("joint")
}
