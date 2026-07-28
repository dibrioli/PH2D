//! **A RODA, do lado do ECS** (W-Wheel) — a ponte dobra os dois graus de
//! liberdade, e a criação semeia a mola na escala CERTA.
//!
//! O kernel (que a roda de fato segura de lado, cede no eixo, gira, e que o
//! batente de compressão morde) é gateado em `ph2d-physics/tests/joint_wheel.rs`.
//! Aqui ficam as três perguntas que só existem deste lado da fronteira:
//!
//! 1. a ponte entrega ao solver a roda que o componente descreve;
//! 2. um rewind a **re-arma** (o `BodyDesc`/`JointDesc` é a receita de spawn, e
//!    o scrub reconstrói o mundo a partir dela);
//! 3. `of_kind` semeia **stiffness/damping na escala de uma SUSPENSÃO** — o
//!    número de uma Spring põe o veículo sentado no batente.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Altura de marcha autorada.
const RIDE: f32 = 0.5;
const WHEEL_R: f32 = 0.3;

/// Um carro de uma roda APOIADO no chão — uma suspensão só comprime com a roda
/// apoiada e o peso do chassi descendo sobre ela; solta no ar os dois corpos
/// caem juntos e a distância entre eles nunca muda.
fn rig(kind: JointKind, stiffness: f32, limits: Option<[f32; 2]>) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    sim.world_mut().spawn((
        Name::new("Chassis"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, WHEEL_R + RIDE)),
    ));
    sim.world_mut().spawn((
        Name::new("Hub"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: WHEEL_R },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, WHEEL_R)),
    ));
    // A rotação do joint É a direção da suspensão: para CIMA.
    let mut t = Transform::from_translation(Vec2::new(0.0, WHEEL_R));
    t.rotation = std::f32::consts::FRAC_PI_2;
    sim.world_mut().spawn((
        Name::new("Wheel Joint"),
        PhysicsJoint {
            body_a: stable_name_id("Chassis"),
            body_b: stable_name_id("Hub"),
            kind,
            stiffness,
            limits_enabled: limits.is_some(),
            limit_min: limits.map_or(0.0, |l| l[0]),
            limit_max: limits.map_or(0.0, |l| l[1]),
            ..PhysicsJoint::of_kind(kind)
        },
        t,
    ));
    sim
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("body alive")
}

/// A altura de marcha depois de `ticks`, e quanto ela afundou.
fn sag_after(sim: &mut SimWorld, bridge: &mut PhysicsBridge, ticks: u64) -> f32 {
    for t in 1..=ticks {
        bridge.dispatch(sim, false, t);
    }
    RIDE - (y_of(sim, "Chassis") - y_of(sim, "Hub"))
}

/// **A ponte entrega a roda ao solver — e um Pin é o controle.**
#[test]
fn the_bridge_folds_a_wheel_and_the_suspension_gives() {
    let mut sim = rig(JointKind::Wheel, 400.0, None);
    let mut bridge = PhysicsBridge::new();
    let sag = sag_after(&mut sim, &mut bridge, 240);
    assert!(
        sag > 0.02,
        "a suspensão tinha de ceder sob o chassi; afundou {sag:.4} m"
    );

    let mut sim = rig(JointKind::Pin, 400.0, None);
    let mut bridge = PhysicsBridge::new();
    let pin = sag_after(&mut sim, &mut bridge, 240);
    assert!(
        pin.abs() < 0.005,
        "o controle falhou: um PINO não cede, e a altura de marcha dele mudou {pin:.4} m"
    );
}

/// **Um rewind RE-ARMA a roda.** O scrub reconstrói o mundo a partir das
/// receitas de spawn, então uma propriedade que só existisse no build ao vivo
/// desapareceria ao arrastar a régua para trás.
#[test]
fn a_rewind_re_arms_the_wheel() {
    let mut sim = rig(JointKind::Wheel, 400.0, None);
    let mut bridge = PhysicsBridge::new();
    let live = sag_after(&mut sim, &mut bridge, 240);
    // Volta ao começo e re-simula o mesmo alcance.
    bridge.dispatch(&mut sim, false, 0);
    let replayed = sag_after(&mut sim, &mut bridge, 240);
    assert!(
        (live - replayed).abs() < 0.01,
        "o replay tinha de reproduzir a mesma suspensão; {live:.4} ao vivo contra \
         {replayed:.4} depois do rewind"
    );
}

/// **O BATENTE de compressão chega ao solver pela ponte** — a metade que só uma
/// roda tem, porque nela o limite linear não é acoplado.
#[test]
fn the_travel_limit_reaches_the_solver() {
    let stop = 0.02_f32;
    let mut sim = rig(JointKind::Wheel, 60.0, Some([-stop, stop]));
    let mut bridge = PhysicsBridge::new();
    let limited = sag_after(&mut sim, &mut bridge, 240);

    let mut sim = rig(JointKind::Wheel, 60.0, None);
    let mut bridge = PhysicsBridge::new();
    let free = sag_after(&mut sim, &mut bridge, 240);

    assert!(
        free > stop * 3.0,
        "a fixture não contém o fenômeno: sem batente o chassi tinha de afundar \
         bem além dele, e afundou {free:.4} m"
    );
    assert!(
        (limited - stop).abs() < 0.01,
        "o batente de {stop:.2} m tinha de segurar a suspensão; ela parou em {limited:.4}"
    );
}

/// **Uma roda nova nasce com a mola de uma SUSPENSÃO, não com a de uma mola.**
///
/// Os dois usam os MESMOS dois campos, e a escala que cada um quer difere por
/// uma ordem de grandeza: uma Spring é autorada para *pendurar* um corpo (o
/// afundamento é o efeito), uma suspensão segura um veículo de pé (o
/// afundamento é efeito colateral). Herdando o 30 de uma Spring, o veículo
/// senta no batente no primeiro tick — e nada na tela diria por quê.
#[test]
fn a_fresh_wheel_is_born_with_a_suspensions_spring() {
    let wheel = PhysicsJoint::of_kind(JointKind::Wheel);
    let spring = PhysicsJoint::of_kind(JointKind::Spring);
    assert_eq!(
        wheel.stiffness,
        ph2d_physics::JointDesc::WHEEL_STIFFNESS,
        "a roda tem de nascer com a rigidez MEDIDA da suspensão"
    );
    assert!(
        wheel.stiffness > spring.stiffness * 5.0,
        "a suspensão tinha de nascer bem mais rígida que uma mola; {} contra {}",
        wheel.stiffness,
        spring.stiffness
    );
    // E o curso tem régua PRÓPRIA — meio metro de suspensão é mais que a altura
    // de marcha de qualquer veículo que um artista monte.
    let slider = PhysicsJoint::of_kind(JointKind::Slider);
    assert!(
        wheel.limit_max < slider.limit_max,
        "o curso de uma roda tinha de ser menor que o de um trilho; {} contra {}",
        wheel.limit_max,
        slider.limit_max
    );

    // E a prova de que a escala IMPORTA: com a mola de uma Spring, o mesmo
    // veículo afunda várias vezes mais.
    let mut sim = rig(JointKind::Wheel, spring.stiffness, None);
    let mut bridge = PhysicsBridge::new();
    let soft = sag_after(&mut sim, &mut bridge, 240);
    let mut sim = rig(JointKind::Wheel, wheel.stiffness, None);
    let mut bridge = PhysicsBridge::new();
    let stiff = sag_after(&mut sim, &mut bridge, 240);
    assert!(
        soft > stiff * 3.0,
        "a mola de uma Spring tinha de deixar o veículo sentado; afundou {soft:.4} m \
         contra {stiff:.4} com a da suspensão"
    );
}

/// A cena do carro que ANDA, para medir o que o swap faz de fato.
fn car_rig(swap: bool, compensate: bool) -> SimWorld {
    let mut sim = rig(JointKind::Wheel, 400.0, None);
    let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &PhysicsJoint)>();
    let e = q.iter(sim.world()).map(|(e, _)| e).next().expect("joint");
    let mut j = *sim.world().get::<PhysicsJoint>(e).unwrap();
    j.motor_enabled = true;
    j.motor_speed = -8.0;
    j.motor_max_force = 40.0;
    if swap {
        j = if compensate {
            j.swapped()
        } else {
            // O swap CRU: só troca as duas pontas, sem compensar nada.
            PhysicsJoint {
                body_a: j.body_b,
                body_b: j.body_a,
                local_a: j.local_b,
                local_b: j.local_a,
                ..j
            }
        };
    }
    *sim.world_mut().get_mut::<PhysicsJoint>(e).unwrap() = j;
    sim
}

/// **Numa RODA, o swap não escolhe qual corpo é a roda — porque o joint não
/// sabe.** Ele troca qual ponta se chama A, e o carro segue andando para o
/// MESMO lado.
///
/// ⚠️ **MEDIDO, e o pedido que o motivou não é construível.** *"Que o Swap mude
/// de verdade qual objeto é a roda"* supõe que o tipo designe um; ele não
/// designa: as duas âncoras COINCIDEM (`shares_a_point`), o eixo é a mesma
/// direção de mundo nos dois frames, e a restrição é entre os dois frames. Quem
/// decide qual corpo gira é a MASSA e o contato com o chão, não o rótulo A/B.
///
/// O que um swap SEM compensação faz numa roda é exatamente uma coisa, medida:
/// **dirigir o carro para trás** — chassi em `−5,6028` contra `+5,6028`, o
/// espelho exato em todo horizonte (5, 15, 30, 60, 120 e 240 ticks). Ou seja o
/// pedido, implementado, seria um botão chamado *Swap A/B* cujo efeito visível é
/// *inverter a tração* — que é o sinal de `Motor Speed`, na linha que diz isso.
///
/// ⚠️ **A preservação é de SENTIDO, não bit a bit.** A tabela do `swapped()` foi
/// medida antes de a roda existir; num carro dirigido com atrito ela reproduz o
/// deslocamento a ~29% em 4 s (7,92 contra 5,60 m) e o giro relativo a ~1,5%
/// (−1,626 contra −1,602 em 1 s), porque a ordem dos corpos no solver do rapier
/// não é simétrica e o contato amplifica. O gate afirma o **sinal**, que é a
/// propriedade que o artista lê; um bar em bit-exatidão aqui não poderia passar.
#[test]
fn swapping_a_wheel_keeps_the_car_driving_the_same_way() {
    let travel = |swap: bool, compensate: bool| {
        let mut sim = car_rig(swap, compensate);
        let mut bridge = PhysicsBridge::new();
        for t in 1..=240 {
            bridge.dispatch(&mut sim, true, t);
        }
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .find(|(n, _)| n.as_str() == "Chassis")
            .map(|(_, t)| t.translation.x)
            .expect("chassis alive")
    };
    let authored = travel(false, true);
    let swapped = travel(true, true);
    let bare = travel(true, false);

    assert!(
        authored > 3.0,
        "a fixture não contém o fenômeno: o carro tinha de ANDAR, e andou {authored:.4} m"
    );
    assert!(
        swapped > 3.0,
        "trocar as pontas de uma roda não pode inverter o carro; ele andou {swapped:.4} m \
         contra {authored:.4} autorado"
    );
    // E o controle que dá sentido ao anterior: SEM a compensação ele inverte —
    // é essa a única coisa que um swap cru faz numa roda.
    assert!(
        bare < -3.0,
        "o controle falhou: um swap CRU tinha de dirigir o carro para trás, e ele \
         andou {bare:.4} m"
    );
}
