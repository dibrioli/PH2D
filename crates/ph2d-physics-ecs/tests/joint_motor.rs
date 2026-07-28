//! **The servo and the winch, through the bridge** (W-J6) — the ECS half of the
//! motor that the wrapper's `joint_motor.rs` gates on the solver side.
//!
//! Three things live here and nowhere else: the *doors* that answer "which kinds
//! are driven, and in what unit", the bridge FOLDING `motor_mode`/`motor_target`
//! into the descriptor, and a rewind re-arming both.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MotorMode, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// A hook at `(0, 6)` and an arm hanging a metre under it, joined by `joint`.
fn rig(joint: PhysicsJoint, joint_rot: f32) -> SimWorld {
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
        Name::new("Arm"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.15 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 5.0)),
    ));
    let mut t = Transform::from_translation(Vec2::new(0.0, 6.0));
    t.rotation = joint_rot;
    sim.world_mut().spawn((
        Name::new("Joint"),
        PhysicsJoint {
            body_a: stable_name_id("Hook"),
            body_b: stable_name_id("Arm"),
            ..joint
        },
        t,
    ));
    sim
}

fn arm_y(sim: &mut SimWorld, b: &mut PhysicsBridge, from: u64, ticks: u64) -> f32 {
    for t in from + 1..=from + ticks {
        b.dispatch(sim, false, t);
    }
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    let (_, t) = q
        .iter(sim.world())
        .find(|(n, _)| n.as_str() == "Arm")
        .expect("Arm alive");
    t.translation.y
}

/// **As portas de unidade, uma linha por TIPO — e agora exaustivas.**
///
/// A Rope é o caso que prova que `limits_in_metres` e `motor_in_metres` são
/// perguntas diferentes num sentido (faixa nenhuma + motor linear), e o Wheel é
/// o que prova no outro (limite em METROS + motor em GRAUS, no mesmo joint,
/// porque são dois graus de liberdade). Fundidas numa porta só, uma das duas
/// sempre sai na unidade errada.
///
/// ⚠️ **A tabela é um `match` EXAUSTIVO, e a mudança não é cosmética:** a versão
/// anterior era uma lista de cinco tuplas, o [`JointKind::Rod`] chegou depois e
/// **ninguém a estendeu** — ele atravessou uma wave sem ter uma única destas
/// respostas conferida. Agora um tipo novo **não compila** até declarar as cinco.
#[test]
fn the_unit_doors_disagree_about_a_rope_and_that_is_the_point() {
    use JointKind::*;
    /// `(translates, has_limits, limits_in_metres, has_motor, motor_in_metres)`
    fn expected(k: JointKind) -> (bool, bool, bool, bool, bool) {
        match k {
            Pin => (false, true, false, true, false),
            Spring => (false, false, false, false, false),
            Rope => (true, false, false, true, true),
            Weld => (false, false, false, false, false),
            Slider => (true, true, true, true, true),
            // Uma barra é UM número: nem faixa, nem motor, nem eixo.
            Rod => (false, false, false, false, false),
            // A roda: desliza (o curso, em metros) E gira (a tração, em graus).
            Wheel => (true, true, true, true, false),
            // A polia translada (a corda) e nada mais: a faixa dela É o
            // comprimento, e um guincho seria motor de outra wave.
            Pulley => (true, false, false, false, false),
        }
    }
    for k in JointKind::ALL {
        let (translates, has_limits, limits_m, has_motor, motor_m) = expected(k);
        assert_eq!(k.translates(), translates, "{k:?} translates()");
        assert_eq!(k.has_limits(), has_limits, "{k:?} has_limits()");
        assert_eq!(k.limits_in_metres(), limits_m, "{k:?} limits_in_metres()");
        assert_eq!(k.has_motor(), has_motor, "{k:?} has_motor()");
        assert_eq!(k.motor_in_metres(), motor_m, "{k:?} motor_in_metres()");
    }
    // As duas afirmações em duas linhas: cada uma sozinha só mostra metade.
    assert_ne!(
        Rope.limits_in_metres(),
        Rope.motor_in_metres(),
        "a Rope não tem faixa de limite e tem motor LINEAR — uma porta só daria \
         ao guincho um alvo em graus"
    );
    assert_ne!(
        Wheel.limits_in_metres(),
        Wheel.motor_in_metres(),
        "uma roda limita uma TRANSLAÇÃO (o curso) e motoriza uma ROTAÇÃO (o giro) \
         — uma porta só daria à tração uma velocidade em metros por segundo"
    );
}

/// **A lista [`JointKind::ALL`] cobre cada discriminante exatamente uma vez.**
///
/// É ela que o gate acima percorre, então uma lista curta o deixaria verde
/// sobre um tipo que ninguém conferiu — que é literalmente o que aconteceu com
/// o Rod.
#[test]
fn the_kind_list_covers_every_discriminant_once() {
    let mut seen: Vec<u8> = JointKind::ALL
        .iter()
        .map(|k| postcard::to_allocvec(k).expect("serialisable")[0])
        .collect();
    seen.sort_unstable();
    let expected: Vec<u8> = (0..JointKind::ALL.len() as u8).collect();
    assert_eq!(
        seen, expected,
        "JointKind::ALL tem de ter um elemento por discriminante, sem repetido"
    );
}

/// **The bridge folds the servo, and the arm holds a place it would otherwise
/// fall from.**
///
/// The joint is a Pin whose servo is told `+1.0 rad`; the arm starts hanging.
/// The control is the identical component with `motor_enabled: false`.
///
/// Mutation: `joint_desc` dropping `mode` (always `Velocity`) — RED, the arm
/// hangs.
#[test]
fn the_bridge_folds_a_servo_and_the_arm_holds_its_place() {
    let servo = PhysicsJoint {
        kind: JointKind::Pin,
        motor_enabled: true,
        motor_mode: MotorMode::Position,
        motor_target: 1.0,
        ..PhysicsJoint::default()
    };
    let mut b = PhysicsBridge::new();
    let held = arm_y(&mut rig(servo, 0.0), &mut b, 0, 240);
    let mut b2 = PhysicsBridge::new();
    let free = arm_y(
        &mut rig(
            PhysicsJoint {
                motor_enabled: false,
                ..servo
            },
            0.0,
        ),
        &mut b2,
        0,
        240,
    );
    // Hanging, the arm sits a metre below the hook (y = 5). The pin shares a
    // point 1 m above the arm's centre, so holding the relative rotation at
    // +1.0 rad swings the arm up and aside to `6 - cos(1) = 5.46`.
    assert!(
        free < 5.05,
        "the control has to hang under the hook, got {free:.3}"
    );
    assert!(
        held > free + 0.35,
        "the servo has to hold the arm above where it would hang: \
         {free:.3} -> {held:.3}"
    );
}

/// **A winch reels, through the bridge** — and this is the kind that had no
/// motor at all before W-J6.
///
/// Mutation: `joint_desc` gating on `is_hinge()` (as it did until this wave) —
/// RED, the Rope's motor never reaches the solver.
#[test]
fn the_bridge_folds_a_winch_onto_a_rope() {
    let winch = PhysicsJoint {
        kind: JointKind::Rope,
        max_length: 1.0,
        motor_enabled: true,
        motor_mode: MotorMode::Position,
        motor_target: 0.25,
        ..PhysicsJoint::default()
    };
    let mut b = PhysicsBridge::new();
    let reeled = arm_y(&mut rig(winch, 0.0), &mut b, 0, 300);
    let mut b2 = PhysicsBridge::new();
    let slack = arm_y(
        &mut rig(
            PhysicsJoint {
                motor_enabled: false,
                ..winch
            },
            0.0,
        ),
        &mut b2,
        0,
        300,
    );
    assert!(
        (slack - 5.0).abs() < 0.1,
        "a passive 1 m rope hangs the arm at y = 5, got {slack:.3}"
    );
    assert!(
        reeled > 5.6,
        "a winch told 0.25 m should haul it to about y = 5.75, got {reeled:.3}"
    );
}

/// **A rewind re-arms the motor** — the descriptor a joint is rebuilt from has to
/// carry the mode and the target, or scrubbing back replays a different scene.
///
/// Mutation: `joint_desc` dropping `target` (always `0.0`) — RED, the replay
/// holds a different angle than the live run did.
#[test]
fn a_rewind_re_arms_the_servo() {
    let servo = PhysicsJoint {
        kind: JointKind::Pin,
        motor_enabled: true,
        motor_mode: MotorMode::Position,
        motor_target: 1.0,
        ..PhysicsJoint::default()
    };
    let mut sim = rig(servo, 0.0);
    let mut b = PhysicsBridge::new();
    let live = arm_y(&mut sim, &mut b, 0, 240);
    // Scrub back to the start and replay the same span. `rewind_to` rebuilds
    // every body AND joint from its rest descriptor, so anything the descriptor
    // does not carry is silently lost here.
    b.dispatch(&mut sim, false, 0);
    let replayed = arm_y(&mut sim, &mut b, 0, 240);
    assert!(
        (live - replayed).abs() < 1e-3,
        "a replay has to reproduce the servo: {live:.4} vs {replayed:.4}"
    );
    // The control: it held something, rather than both runs hanging.
    assert!(
        live > 5.35,
        "the servo has to have lifted the arm at all, got {live:.3}"
    );
}

/// **Uma winch recém-criada RECOLHE no seu default** — e o default anterior era
/// um no-op.
///
/// Report do Enio (2026-07-26): *"Em Rope:Motor:ON: Velocity parâmetros Speed e
/// Max Force não afetam a simulação"*. Era fiel, e o mecanismo está no solver do
/// rapier (`motor_linear_coupled`): os limites **CLAMPAM o alvo do próprio
/// motor** — `target_vel = clamp(target_vel, (min−dist)/dt, (max−dist)/dt)` — e
/// uma carga pendurada senta EXATAMENTE em `max_length`, então todo alvo positivo
/// vira **zero**. Com alvo zero não há erro em que gastar força, e é por isso que
/// os DOIS knobs pareciam mortos: uma causa só.
///
/// O oráculo tem as duas metades: o default **levanta**, e a taxa oposta (a que
/// era o default) **não move nada**. Sem a segunda o gate ficaria verde num mundo
/// onde qualquer motor levanta e o sinal não significa coisa alguma.
///
/// Mutação: `default_motor_speed(Rope)` sem o sinal — RED, a carga fica em 5,000.
#[test]
fn a_fresh_winch_reels_in_at_its_default_speed() {
    let winch = |speed: f32| PhysicsJoint {
        max_length: 1.0,
        motor_enabled: true,
        motor_speed: speed,
        ..PhysicsJoint::of_kind(JointKind::Rope)
    };
    let mut b = PhysicsBridge::new();
    let mut sim = rig(
        winch(PhysicsJoint::default_motor_speed(JointKind::Rope)),
        0.0,
    );
    let hauled = arm_y(&mut sim, &mut b, 0, 180);
    assert!(
        hauled > 5.6,
        "o default de uma winch tem de RECOLHER a carga (o gancho está em y=6), \
         got {hauled:.3}"
    );
    // A metade que nomeia o mecanismo: a taxa OPOSTA — a que era o default — não
    // move nada, porque a corda já está no comprimento máximo dela.
    let mut b2 = PhysicsBridge::new();
    let paid_out = arm_y(
        &mut rig(winch(PhysicsJoint::DEFAULT_LINEAR_MOTOR_SPEED), 0.0),
        &mut b2,
        0,
        180,
    );
    assert!(
        (paid_out - 5.0).abs() < 0.1,
        "soltar corda que já está toda solta não move nada, got {paid_out:.3}"
    );
}

/// **Um joint nasce com os números da SUA unidade** — e nascer e trocar de tipo
/// chegam ao mesmo lugar.
///
/// `PhysicsJoint::default()` não tem tipo a que perguntar, então ele semeia os do
/// Pin: ±45° de faixa e 2 rad/s de motor. Lidos como Slider isso é **±0,785
/// metros** de curso e **2 m/s** de carrinho. A troca de tipo re-semeia por essas
/// portas desde o W-J5; a CRIAÇÃO não semeava, então *"Join As = Slider"* e
/// *"faça um Pin e troque para Slider"* produziam joints diferentes.
///
/// Mutação: `of_kind` voltando a `Self { kind, ..default() }` — RED nas duas
/// metades.
#[test]
fn a_joint_is_born_with_its_own_kinds_numbers() {
    let rail = PhysicsJoint::of_kind(JointKind::Slider);
    assert!(
        (rail.limit_max - PhysicsJoint::DEFAULT_STROKE).abs() < 1e-6,
        "um trilho nasce com curso em METROS, got {}",
        rail.limit_max
    );
    assert!(
        (rail.motor_speed - PhysicsJoint::DEFAULT_LINEAR_MOTOR_SPEED).abs() < 1e-6,
        "e com velocidade em m/s, got {}",
        rail.motor_speed
    );
    assert!(
        PhysicsJoint::of_kind(JointKind::Rope).motor_speed < 0.0,
        "uma winch nasce recolhendo"
    );
    // O CONTROLE: uma dobradiça continua nascendo em radianos.
    let hinge = PhysicsJoint::of_kind(JointKind::Pin);
    assert!((hinge.limit_max - PhysicsJoint::DEFAULT_LIMIT).abs() < 1e-6);
    assert!((hinge.motor_speed - PhysicsJoint::DEFAULT_MOTOR_SPEED).abs() < 1e-6);
}

/// **As três portas que a [`JointKind::Pulley`] SEPAROU**, exaustivas pela mesma
/// razão da tabela acima: onde o comprimento MORA, se ele é um RAIO em volta de
/// uma âncora, se as duas pontas partilham um ponto, e se o tipo pode PARTIR.
///
/// As quatro tinham uma resposta derivada de outra até a polia chegar —
/// `shares_a_point` era `!has_length()`, e o anel de canvas saía direto do
/// `length_field`. Uma polia tem comprimento (a corda), pontas separadas, nenhum
/// raio a desenhar, e nada que meça a reação dela.
#[test]
fn the_length_and_break_doors_are_declared_per_kind() {
    use JointKind::*;
    use ph2d_physics_ecs::LengthField;
    fn expected(k: JointKind) -> (Option<LengthField>, bool, bool, bool) {
        // (length_field, length_is_a_radius, shares_a_point, can_break)
        match k {
            Pin => (None, false, true, true),
            Spring => (Some(LengthField::Rest), true, false, true),
            Rope => (Some(LengthField::Max), true, false, true),
            Weld => (None, false, true, true),
            Slider => (None, false, true, true),
            Rod => (Some(LengthField::Max), true, false, true),
            Wheel => (None, false, true, true),
            // A corda tem comprimento, não tem raio, tem pontas separadas, e não
            // parte: ela não está no `ImpulseJointSet`, então nada mede a reação.
            Pulley => (Some(LengthField::Max), false, false, false),
        }
    }
    for k in JointKind::ALL {
        let (field, radius, shares, breaks) = expected(k);
        assert_eq!(k.length_field(), field, "{k:?} length_field()");
        assert_eq!(k.length_is_a_radius(), radius, "{k:?} length_is_a_radius()");
        assert_eq!(k.shares_a_point(), shares, "{k:?} shares_a_point()");
        assert_eq!(k.can_break(), breaks, "{k:?} can_break()");
    }
    // A afirmação que a polia tornou necessária, dita direto: ter comprimento
    // deixou de implicar que um anel o autora.
    assert!(
        Pulley.has_length() && !Pulley.length_is_a_radius(),
        "a polia é exatamente o caso em que as duas perguntas divergem"
    );
}
