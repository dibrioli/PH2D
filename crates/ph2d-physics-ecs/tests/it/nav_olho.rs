//! **A parede de olho fechado não barra o caminho** (dono 09/11): um corpo com TODAS as formas
//! desligadas (o olho da Hierarquia) fica no solver sem tocar nada — e a navegação lê o mesmo: não
//! recorta a malha, a área proibida dele some, e o agente passa a direito. Reabrir o olho volta a
//! barrar. O controlo de cada régua é a mesma cena de olho aberto.

use ph2d_core::Vec2;
use ph2d_ecs::row_flag::{RowFlag, flip};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavCostArea, NavRegion, NavTarget, PhysicsBridge,
    RigidBody, TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

/// A região `16 × 12` e uma «parede» estática em `x = 0` (`0,6 × 8`) cuja forma é uma FILHA — o
/// idioma do editor (ADR-0199). `proibida` põe-lhe uma `NavCostArea` proibida em vez de ser sólida
/// (sensor): o furo vem da área, não do colisor.
fn cena(proibida: bool) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [8.0, 6.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let parede = w
        .spawn((
            Name::new("Parede"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    if proibida {
        w.entity_mut(parede).insert(NavCostArea {
            forbidden: true,
            ..NavCostArea::default()
        });
    }
    let forma = w
        .spawn((
            Name::new("Collision Shape"),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 4.0,
                },
                is_sensor: proibida,
                ..Collider::default()
            },
            Transform::IDENTITY,
            ChildOf(parede),
        ))
        .id();
    w.spawn((
        Name::new("Ag"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            ..Collider::default()
        },
        TopDownPlayer::from_law(TopDownLaw {
            default_controls: false,
            direction: DirectionMode::Free,
            ..TopDownLaw::default()
        }),
        NavAgent {
            target: NavTarget::Point([4.0, 0.0]),
            arrive_distance: 0.1,
            ..NavAgent::default()
        },
        Transform::from_translation(Vec2::new(-4.0, 0.0)),
    ));
    (sim, parede, forma)
}

/// A área da (única) malha depois do tique `t`.
fn area(sim: &mut SimWorld, bridge: &mut PhysicsBridge, t: u64) -> f64 {
    bridge.dispatch(sim, true, t);
    let malhas: Vec<f64> = bridge.nav_meshes().map(|(_, _, m)| m.area()).collect();
    assert_eq!(malhas.len(), 1, "uma região, um raio");
    malhas[0]
}

/// ⭐ **De olho fechado a parede (e a área proibida) não abrem furo; reaberto, abrem de novo** — a
/// meio do play, pela mesma ponte.
#[test]
fn a_parede_de_olho_fechado_nao_recorta_a_malha_e_reaberta_recorta() {
    for proibida in [false, true] {
        let (mut sim, _parede, forma) = cena(proibida);
        let mut bridge = PhysicsBridge::new();
        let aberta = area(&mut sim, &mut bridge, 1);
        flip(sim.world_mut(), forma, RowFlag::Visible);
        let fechada = area(&mut sim, &mut bridge, 2);
        flip(sim.world_mut(), forma, RowFlag::Visible);
        let reaberta = area(&mut sim, &mut bridge, 3);
        let tipo = if proibida {
            "a área proibida"
        } else {
            "a parede"
        };
        assert!(
            fechada > aberta + 3.0,
            "{tipo} de olho fechado ainda recorta: {fechada} contra {aberta}"
        );
        assert!(
            (reaberta - aberta).abs() < 1e-6,
            "{tipo} reaberta não voltou a recortar: {reaberta} contra {aberta}"
        );
    }
}

/// ⭐ **O agente atravessa a parede de olho fechado em linha recta** — o controlo (olho aberto) dá a
/// volta por cima ou por baixo.
#[test]
fn o_agente_passa_a_direito_pela_parede_de_olho_fechado() {
    for fechado in [false, true] {
        let (mut sim, _parede, forma) = cena(false);
        if fechado {
            flip(sim.world_mut(), forma, RowFlag::Visible);
        }
        let ag = {
            let w = sim.world_mut();
            let mut q = w.query::<(Entity, &NavAgent)>();
            q.iter(w).map(|(e, _)| e).next().expect("o agente")
        };
        let mut bridge = PhysicsBridge::new();
        let mut maior_y = 0.0_f32;
        let mut fim = (0.0, 0.0);
        for t in 1..=300 {
            bridge.dispatch(&mut sim, true, t);
            let p = sim
                .world()
                .get::<Transform>(ag)
                .expect("o agente")
                .translation;
            maior_y = maior_y.max(p.y.abs());
            fim = (p.x, p.y);
        }
        assert!(
            (fim.0 - 4.0).hypot(fim.1) < 0.2,
            "não chegou (olho {}): {fim:?}",
            if fechado { "fechado" } else { "aberto" }
        );
        if fechado {
            assert!(
                maior_y < 0.05,
                "deu a volta a uma parede desligada: |y| foi a {maior_y}"
            );
        } else {
            assert!(
                maior_y > 4.0,
                "o controlo não deu a volta à parede: |y| foi a {maior_y}"
            );
        }
    }
}

/// ⭐ **Um corpo que ANDA, de olho fechado, sai do DESVIO** (o filtro `shape_off` de `nav_desvio`): um
/// herói parado no caminho (Kinematic com mover — não recorta a malha, é obstáculo só do desvio) com a
/// forma FILHA. De olho aberto o agente contorna-o; de olho fechado passa a direito.
#[test]
fn um_corpo_que_anda_de_olho_fechado_nao_e_desviado() {
    use super::nav_desvio::{agente, corre, mover, regiao};
    for fechado in [false, true] {
        let mut sim = SimWorld::new();
        regiao(&mut sim);
        let heroi = sim
            .world_mut()
            .spawn((
                Name::new("Hero"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                mover(),
                Transform::from_translation(Vec2::new(0.0, 0.0)),
            ))
            .id();
        let forma = sim
            .world_mut()
            .spawn((
                Name::new("Collision Shape"),
                Collider {
                    shape: ColliderShape::Ball { radius: 0.6 },
                    ..Collider::default()
                },
                Transform::IDENTITY,
                ChildOf(heroi),
            ))
            .id();
        if fechado {
            flip(sim.world_mut(), forma, RowFlag::Visible);
        }
        let quem = agente(
            &mut sim,
            "A",
            (-4.0, 0.0),
            NavTarget::Point([4.0, 0.0]),
            true,
        );
        let c = corre(&mut sim, &mut PhysicsBridge::new(), &[quem], 1, 300);
        let fora = c.iter().map(|ps| ps[0].1.abs()).fold(0.0f32, f32::max);
        let fim = c.last().expect("a corrida")[0];
        assert!(
            (fim.0 - 4.0).hypot(fim.1) < 0.2,
            "não chegou (olho {}): {fim:?}",
            if fechado { "fechado" } else { "aberto" }
        );
        if fechado {
            assert!(fora < 0.05, "desviou-se de um corpo desligado: |y| {fora}");
        } else {
            assert!(fora > 0.5, "o CONTROLO não se desviou do herói: |y| {fora}");
        }
    }
}
