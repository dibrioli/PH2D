//! **Uma corda não pode ser mais curta que o caminho que ela enfia** (W-Pulley,
//! 2026-07-29).
//!
//! A wave do raio fechou a explosão por uma PORTA (`reseat_wheel_geometry`),
//! chamada pelos três gestos que a conhecem: arrastar o centro, arrastar o aro,
//! digitar o raio. Mas o `L0` é derivado da rota, e a rota tem mais entradas que
//! essas três — *uma condição que enumera seus leitores apodrece*. Medido com o
//! `L0` parado (sonda `measure_pulley_route_gestures`):
//!
//! | gesto | violação | maior salto num tique |
//! |---|---|---|
//! | (controle: ninguém tocou) | +0,0000 | 0,0817 |
//! | acrescentar uma roldana | **+2,8816** | **13,97** (raio 0,60: **55,45**) |
//! | mover o centro dela para o lado | **+4,1908** | **25,27** |
//! | digitar `Rope Length = 5` | **+6,9650** | **46,58** |
//! | mover o centro para baixo | −1,3832 | 0,0813 |
//! | **apagar** uma roldana | −2,1953 | 0,0785 |
//!
//! ⚠️ **A assimetria É o desenho, e ela foi medida, não escolhida:** violação
//! POSITIVA explode; a negativa é folga e mede o salto do CONTROLE. Então a cura é
//! um **PISO** (`L0 ≥ L(rota)` no estado autorado), nunca uma re-derivação: para
//! baixo ela clobbaria a row `Rope Length (m)`, que é editável numa polia.
//!
//! ⚠️ **E apagar uma roldana nunca poderia passar por uma porta** — o delete da
//! Hierarquia não sabe o que é uma corda. É por isso que a cura mora onde a
//! resposta já mora (o reconcile, que já computa a rota), e não num quarto
//! chamador.
//!
//! ⚠️ **O oráculo dos saltos é o CONTROLE, não um literal:** a cena assenta com
//! um salto próprio, e o que se afirma é que o gesto **não muda a ordem de
//! grandeza** dele.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// Um elevador: carga de 3 kg e contrapeso de 1 kg por duas roldanas no alto.
fn rig() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, mass: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: mass / (std::f32::consts::PI * 0.04),
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Floor", 0.0, -4.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 3.0);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 1.0);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counter"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.5, 2.0)),
    ));
    for (i, x) in [-1.5f32, 1.5].into_iter().enumerate() {
        sim.world_mut().spawn((
            Name::new(format!("Rope Wheel {}", i + 1)),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("duas roldanas"),
                radius: 0.3,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }
    sim
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity_of(sim, name);
    sim.world().get::<Transform>(e).expect("t").translation.y
}

fn rope_l0(sim: &mut SimWorld) -> f32 {
    let e = entity_of(sim, "Rope");
    sim.world().get::<PhysicsJoint>(e).expect("j").max_length
}

/// O comprimento que a rota de fato desenha, pela porta do solver.
fn route_len(bridge: &PhysicsBridge) -> f32 {
    let Some(v) = bridge.joint_views().next() else {
        return f32::NAN;
    };
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).map_or(f32::NAN, |r| r.length)
}

/// Roda 60 tiques e devolve o maior passo que um corpo deu num tique só.
fn worst_jump(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> f32 {
    let mut prev = (y_of(sim, "Load"), y_of(sim, "Counter"));
    let mut worst = 0.0f32;
    for t in 1..=60u64 {
        bridge.dispatch(sim, true, t);
        let now = (y_of(sim, "Load"), y_of(sim, "Counter"));
        worst = worst.max((now.0 - prev.0).abs().max((now.1 - prev.1).abs()));
        prev = now;
    }
    worst
}

/// A cena intocada, assentando — o oráculo de todos os gestos abaixo.
fn control_jump() -> f32 {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    worst_jump(&mut sim, &mut bridge)
}

/// Acrescenta a 3ª roldana como o botão "Add Wheel" do shell faz: no meio do
/// último trecho, herdando o raio da anterior.
fn add_third_wheel(sim: &mut SimWorld, radius: f32) {
    sim.world_mut().spawn((
        Name::new("Rope Wheel 3"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 2,
            radius,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
}

/// **Nenhum gesto que CRESCE a rota deixa a corda violada.**
///
/// Os três que não passam pela porta, mais o quarto que nunca poderia passar.
/// Mutação: `(true, _) => authored` (o piso removido) ⇒ violações de +2,88 / +4,19
/// / +6,97 m e saltos de 13,97 / 25,27 / 46,58 — os números da sonda.
#[test]
fn no_gesture_that_grows_the_route_leaves_the_rope_violated() {
    let control = control_jump();
    assert!(
        control < 0.2,
        "o CONTROLE já salta ({control:.4} m) — a fixture não mede o defeito"
    );

    type Gesture = (&'static str, fn(&mut SimWorld));
    let gestures: [Gesture; 4] = [
        ("acrescentar uma roldana", |sim| add_third_wheel(sim, 0.3)),
        ("acrescentar uma roldana grande", |sim| {
            add_third_wheel(sim, 0.6)
        }),
        ("mover o centro de uma roldana", |sim| {
            let w = entity_of(sim, "Rope Wheel 1");
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(w) {
                t.translation = Vec2::new(-4.5, 6.0);
            }
        }),
        ("digitar um Rope Length curto", |sim| {
            let e = entity_of(sim, "Rope");
            if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
                j.max_length = 5.0;
            }
        }),
    ];

    for (label, gesture) in gestures {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        gesture(&mut sim);
        bridge.dispatch(&mut sim, false, 0);

        let (l0, route) = (rope_l0(&mut sim), route_len(&bridge));
        assert!(
            route - l0 < 1.0e-3,
            "{label}: a corda nasce violada em {:+.4} m (L0 {l0:.4}, rota {route:.4}) \
             — o solver come isso num tique",
            route - l0
        );
        let worst = worst_jump(&mut sim, &mut bridge);
        assert!(
            worst < control * 1.5,
            "{label}: maior salto {worst:.4} m contra {control:.4} do controle"
        );
    }
}

/// **Um comprimento autorado MAIOR que a rota sobrevive** — é um piso, não uma
/// re-derivação.
///
/// ⚠️ Este é o gate que separa as duas leis: com uma re-derivação
/// (`(true, Some(r)) => r`) a corda de 20 m viraria 11,97 e a row `Rope Length`
/// seria um controle morto.
#[test]
fn an_authored_length_longer_than_the_route_survives() {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let seeded = rope_l0(&mut sim);
    let e = entity_of(&mut sim, "Rope");
    if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
        j.max_length = 20.0;
    }
    bridge.dispatch(&mut sim, false, 0);
    let l0 = rope_l0(&mut sim);
    assert!(
        (l0 - 20.0).abs() < f32::EPSILON,
        "o comprimento autorado (20,0) foi clobbado para {l0:.4} \
         (a rota semeada era {seeded:.4}) — o piso virou re-derivação"
    );
}

/// **Tirar geometria deixa FOLGA, não um tranco** — a metade negativa da
/// assimetria, pinada para ninguém "completar" o piso numa re-derivação.
///
/// Apagar uma roldana encurta a rota; o `L0` autorado fica, e a corda sobra. O
/// salto medido é o do CONTROLE (0,0785 contra 0,0817), então não há nada a
/// consertar aqui — e consertar clobbaria a autoria.
#[test]
fn removing_geometry_leaves_slack_not_a_jolt() {
    let control = control_jump();
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let seeded = rope_l0(&mut sim);

    let w = entity_of(&mut sim, "Rope Wheel 2");
    let _ = sim.world_mut().despawn(w);
    bridge.dispatch(&mut sim, false, 0);

    let (l0, route) = (rope_l0(&mut sim), route_len(&bridge));
    assert!(
        (l0 - seeded).abs() < f32::EPSILON,
        "apagar uma roldana reescreveu o comprimento ({seeded:.4} -> {l0:.4})"
    );
    assert!(
        route < l0,
        "a rota encurtou (rota {route:.4}, L0 {l0:.4})? o gesto não mede folga"
    );
    let worst = worst_jump(&mut sim, &mut bridge);
    assert!(
        worst < control * 1.5,
        "folga deu tranco: maior salto {worst:.4} m contra {control:.4} do controle"
    );
}

/// **O piso NÃO caminha enquanto a sim corre** — a premissa em que o desenho se
/// apoia, medida em vez de assumida.
///
/// O piso mora no reconcile, sem porta de relógio, e isso só é seguro porque a
/// rota é função do estado **AUTORADO** inteiro: as âncoras saem de
/// `world_from_local_at_pose(rest_a, ...)` e os centros das roldanas da pose de
/// repouso. Logo o número é constante durante o play, e a escrita do piso é
/// idempotente lá.
///
/// ⚠️ **Se alguém trocar a rota para a pose VIVA**, o piso passa a crescer a cada
/// tique em que a carga desce — uma corda que ESTICA — e nada mais nesta suíte
/// veria: um dispatch pausado não distingue as duas poses. Este gate é o único
/// que roda o relógio.
///
/// ⚠️ Ele observa o VALOR, não a contagem de escritas — e é o certo: o diff do
/// undo também compara bytes, então uma escrita idempotente é invisível
/// exatamente onde ela importaria.
///
/// ⛔ **Mutação INVÁLIDA, registrada para ninguém repetir:** uma margem de
/// segurança (`r * 1.001`) **não** caminha — o valor gravado já é maior que a
/// rota, então a condição fecha no dispatch seguinte e ela escreve uma vez só.
#[test]
fn the_floored_length_does_not_walk_while_the_sim_runs() {
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    add_third_wheel(&mut sim, 0.3);
    bridge.dispatch(&mut sim, false, 0);
    let floored = rope_l0(&mut sim);
    assert!(floored > 14.0, "a fixture não floorou nada ({floored:.4})");

    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
        let now = rope_l0(&mut sim);
        assert!(
            now.to_bits() == floored.to_bits(),
            "a corda ESTICOU durante a corrida: {floored:.6} -> {now:.6} no tique {t} \
             (a rota deixou de ser função do estado autorado)"
        );
    }
}
