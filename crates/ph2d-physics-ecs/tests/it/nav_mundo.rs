//! ⭐⭐⭐ **O MUNDO QUE MUDA** (plano 30, W6) — a costura da malha por mosaicos com a ponte.
//!
//! O que só a ponte pode partir: quem é obstáculo NESTE tique (o cinemático parado sim, o que anda
//! não, o personagem nunca), e quem esquece o caminho quando a malha muda (só os agentes dela).
//! As leis da malha (por mosaicos = inteira, incremental = a frio) têm gates na `ph2d-navmesh`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_nav::Status;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PhysicsBridge, RigidBody,
    TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

fn corpo(
    sim: &mut SimWorld,
    nome: &str,
    kind: BodyKind,
    em: (f32, f32),
    meio: (f32, f32),
) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: meio.0,
                    half_y: meio.1,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn regiao(sim: &mut SimWorld, centro: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(centro.0, centro.1)),
    ));
}

fn mover(default_controls: bool) -> TopDownPlayer {
    TopDownPlayer::from_law(TopDownLaw {
        default_controls,
        direction: DirectionMode::Free,
        ..TopDownLaw::default()
    })
}

fn agente(sim: &mut SimWorld, nome: &str, em: (f32, f32), alvo: NavTarget) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            mover(false),
            NavAgent {
                target: alvo,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn poe(sim: &mut SimWorld, e: Entity, em: (f32, f32)) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("o corpo")
        .translation = Vec2::new(em.0, em.1);
}

/// A área andável de todas as malhas.
fn area(b: &PhysicsBridge) -> f64 {
    b.nav_meshes().map(|(_, _, m)| m.area()).sum()
}

/// O caminho do agente CRUZA a recta `x = 0` dentro do vão (`|y| < 1`)?
fn pela_porta(b: &PhysicsBridge, e: Entity) -> bool {
    b.nav_agent(e).is_some_and(|rt| {
        rt.path.windows(2).any(|w| {
            let (a, c) = (w[0], w[1]);
            if (a[0] < 0.0) == (c[0] < 0.0) {
                return false;
            }
            let y = a[1] + (c[1] - a[1]) * (0.0 - a[0]) / (c[0] - a[0]);
            y.abs() < 1.0
        })
    })
}

/// Duas salas e uma parede em `x = 0` com um vão de 2 m no meio; a porta (cinemática) começa
/// escondida dentro da parede de cima.
fn duas_salas() -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (0.0, 0.0));
    corpo(
        &mut sim,
        "Parede de cima",
        BodyKind::Static,
        (0.0, 3.5),
        (0.3, 2.5),
    );
    corpo(
        &mut sim,
        "Parede de baixo",
        BodyKind::Static,
        (0.0, -3.5),
        (0.3, 2.5),
    );
    let porta = corpo(
        &mut sim,
        "Porta",
        BodyKind::Kinematic,
        (0.0, 4.0),
        (0.3, 1.2),
    );
    let quem = agente(
        &mut sim,
        "Guarda",
        (-5.0, 0.0),
        NavTarget::Point([5.0, 0.0]),
    );
    (sim, PhysicsBridge::new(), porta, quem)
}

#[test]
fn a_porta_que_desliza_e_para_fecha_o_caminho_e_abrir_devolve_o() {
    let (mut sim, mut b, porta, quem) = duas_salas();
    for t in 1..=10 {
        b.dispatch(&mut sim, true, t);
    }
    let aberta = area(&b);
    assert!(
        pela_porta(&b, quem),
        "com a porta aberta o caminho é pelo vão"
    );
    assert_eq!(b.nav_agent(quem).map(|r| r.status), Some(Status::Moving));

    poe(&mut sim, porta, (0.0, 0.0));
    for t in 11..=14 {
        b.dispatch(&mut sim, true, t);
    }
    assert!(
        area(&b) < aberta - 1.0,
        "a porta parada no vão não recortou a malha ({} contra {aberta})",
        area(&b)
    );
    assert!(
        !pela_porta(&b, quem),
        "o caminho ainda atravessa a porta fechada"
    );
    assert_eq!(
        b.nav_agent(quem).map(|r| r.status),
        Some(Status::MovingPartial),
        "com a porta fechada o outro lado é outra ilha"
    );

    poe(&mut sim, porta, (0.0, 4.0));
    for t in 15..=18 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(area(&b), aberta, "reaberta, a malha é a de antes ao bit");
    assert!(pela_porta(&b, quem), "reaberta, o caminho volta ao vão");
}

#[test]
fn uma_porta_a_andar_nao_recorta() {
    let (mut sim, mut b, porta, _) = duas_salas();
    for t in 1..=5 {
        b.dispatch(&mut sim, true, t);
    }
    let aberta = area(&b);
    // A porta desce até ao meio do vão a andar SEM parar: nunca é parede (é o desvio que a evita).
    for t in 6..=35u64 {
        let y = 3.0 - 0.1 * (t - 5) as f32;
        poe(&mut sim, porta, (0.0, y));
        b.dispatch(&mut sim, true, t);
        assert_eq!(
            area(&b),
            aberta,
            "tique {t}: a porta a andar recortou a malha"
        );
    }
    // O CONTROLO: parada a meio do vão, recorta.
    for t in 36..=39 {
        b.dispatch(&mut sim, true, t);
    }
    assert!(area(&b) < aberta - 1.0, "o controlo (parada) não recortou");
}

#[test]
fn um_personagem_parado_nao_vira_parede() {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (0.0, 0.0));
    let heroi = corpo(
        &mut sim,
        "Herói",
        BodyKind::Kinematic,
        (3.0, 0.0),
        (0.4, 0.4),
    );
    sim.world_mut().entity_mut(heroi).insert(mover(true));
    let caixa = corpo(
        &mut sim,
        "Caixa",
        BodyKind::Kinematic,
        (-3.0, 3.0),
        (0.4, 0.4),
    );
    let quem = agente(
        &mut sim,
        "Guarda",
        (-5.0, 0.0),
        NavTarget::Named(stable_name_id("Herói")),
    );
    let mut b = PhysicsBridge::new();
    for t in 1..=4 {
        b.dispatch(&mut sim, true, t);
    }
    let com_caixa = area(&b);
    // O CONTROLO: tirar a caixa (cinemática SEM mover, parada) devolve área — ela recortava.
    sim.world_mut().despawn(caixa);
    for t in 5..=8 {
        b.dispatch(&mut sim, true, t);
    }
    let sem_caixa = area(&b);
    assert!(sem_caixa > com_caixa + 0.5, "a caixa parada não era parede");
    // O herói parado tem o MESMO corpo que a caixa: se recortasse, a região inteira menos ele não
    // seria a área toda (16 × 12 recuada pelo raio do guarda, arredondado PARA CIMA a 1/256 m).
    let r = (0.3f64 * 256.0).ceil() / 256.0;
    let toda = (16.0 - 2.0 * r) * (12.0 - 2.0 * r);
    assert!(
        (sem_caixa - toda).abs() < 1e-3,
        "o herói parado recortou a malha ({sem_caixa} contra {toda})"
    );
    assert!(pela_ilha_do_heroi(&b, quem));
}

fn pela_ilha_do_heroi(b: &PhysicsBridge, quem: Entity) -> bool {
    b.nav_agent(quem)
        .is_some_and(|rt| !rt.partial && rt.status != Status::NoPath)
}

#[test]
fn so_os_agentes_da_malha_que_mudou_refazem_o_caminho() {
    let mut sim = SimWorld::new();
    regiao(&mut sim, (-10.0, 0.0));
    regiao(&mut sim, (10.0, 0.0));
    let esq = agente(
        &mut sim,
        "Esquerda",
        (-15.0, 0.0),
        NavTarget::Point([-5.0, 0.0]),
    );
    let dir = agente(
        &mut sim,
        "Direita",
        (5.0, 0.0),
        NavTarget::Point([15.0, 0.0]),
    );
    let mut b = PhysicsBridge::new();
    for t in 1..=5 {
        b.dispatch(&mut sim, true, t);
    }
    let procuras = |b: &PhysicsBridge, e| b.nav_agent(e).map_or(0, |r| r.searches);
    let (e0, d0) = (procuras(&b, esq), procuras(&b, dir));
    corpo(&mut sim, "Pedra", BodyKind::Static, (-8.0, 4.0), (0.5, 0.5));
    for t in 6..=7 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        procuras(&b, esq),
        e0 + 1,
        "a malha da esquerda mudou e o agente dela não refez o caminho"
    );
    assert_eq!(
        procuras(&b, dir),
        d0,
        "a malha da direita não mudou e o agente dela refez o caminho"
    );
}
