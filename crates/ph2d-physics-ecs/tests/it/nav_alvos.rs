//! ⭐⭐⭐ **Os alvos da W6** (plano 30) — *a tag mais perto* e *a patrulha*, pela porta do produto.
//!
//! A tag: o mais perto que pertence à tag (com a subárvore), em linha recta, e muda quando outro
//! fica mais perto; sem a árvore entregue ninguém (falha FECHADO). A patrulha: visita os pontos do
//! `NavRoute` por ordem (fechada dá voltas), e um scrub devolve a ronda ao bit (ela vai no anel).

use ph2d_core::Vec2;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavRoute, NavTarget, PhysicsBridge,
    RigidBody, TopDownPlayer,
};
use ph2d_tags::TagTree;
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

fn regiao(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [10.0, 8.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

fn agente(sim: &mut SimWorld, em: (f32, f32), alvo: NavTarget) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Guarda"),
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
                target: alvo,
                avoidance: false,
                ..NavAgent::default()
            },
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn marcado(sim: &mut SimWorld, nome: &str, em: (f32, f32), tags: Tags) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            tags,
            Transform::from_translation(Vec2::new(em.0, em.1)),
        ))
        .id()
}

fn pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("o corpo");
    [t.translation.x, t.translation.y]
}

/// O fim do caminho que o agente segue agora.
fn destino(b: &PhysicsBridge, e: Entity) -> Option<[f64; 2]> {
    b.nav_agent(e).and_then(|r| r.path.last().copied())
}

#[test]
fn a_tag_mais_perto_com_a_subarvore_e_muda_quando_outro_fica_mais_perto() {
    let mut arvore = TagTree::new();
    let inimigo = arvore.create("Enemy").expect("a tag");
    let voador = arvore.create("Enemy/Flying").expect("a subtag");
    let outra = arvore.create("Coin").expect("outra tag");
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let longe = marcado(&mut sim, "Longe", (8.0, 0.0), Tags::from_ids([inimigo]));
    // O mais perto só pertence à tag pela SUBÁRVORE.
    let perto = marcado(&mut sim, "Perto", (-6.0, 0.0), Tags::from_ids([voador]));
    // O CONTROLO: o mais perto de todos tem OUTRA tag e não conta.
    marcado(&mut sim, "Moeda", (-1.0, 0.0), Tags::from_ids([outra]));
    let quem = agente(&mut sim, (0.0, 0.0), NavTarget::NearestTagged(inimigo.0));
    let mut b = PhysicsBridge::new();
    // Sem a árvore, a tag não alcança ninguém (falha FECHADO).
    b.dispatch(&mut sim, true, 1);
    assert_eq!(
        destino(&b, quem),
        None,
        "sem a árvore o agente achou alguém"
    );
    b.set_tag_tree(&arvore);
    b.dispatch(&mut sim, true, 2);
    assert_eq!(
        destino(&b, quem),
        Some([-6.0, 0.0]),
        "não foi atrás do mais perto"
    );
    // O «Perto» afasta-se para lá do «Longe»: o alvo muda.
    sim.world_mut()
        .get_mut::<Transform>(perto)
        .expect("o corpo")
        .translation = Vec2::new(-9.5, 7.5);
    for t in 3..=4 {
        b.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        destino(&b, quem),
        Some([f64::from(pos(&sim, longe)[0]), 0.0]),
        "o alvo não mudou para o que ficou mais perto"
    );
}

/// A ronda de um quadrado de 8 × 6 m à volta do centro.
fn ronda() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    regiao(&mut sim);
    let quem = agente(&mut sim, (-4.0, -3.0), NavTarget::Patrol(7));
    sim.world_mut().entity_mut(quem).insert(NavRoute {
        points: vec![[-4.0, -3.0], [4.0, -3.0], [4.0, 3.0], [-4.0, 3.0]],
        closed: true,
    });
    (sim, PhysicsBridge::new(), quem)
}

#[test]
fn a_patrulha_fechada_da_voltas_pelos_pontos() {
    let (mut sim, mut b, quem) = ronda();
    let mut vistos: Vec<u32> = Vec::new();
    let mut chegou = 0;
    for t in 1..=900u64 {
        b.dispatch(&mut sim, true, t);
        if let Some(r) = b.nav_ronda(quem)
            && vistos.last() != Some(&r.ponto)
        {
            vistos.push(r.ponto);
        }
        chegou += b
            .nav_events()
            .iter()
            .filter(|e| e.agent == quem && e.kind == ph2d_nav::Event::Arrived)
            .count();
    }
    // Começa no ponto onde está (o 0), já alcançado ⇒ vai para o 1, e dá a volta pelo menos uma vez.
    assert!(
        vistos.windows(5).any(|w| w == [1, 2, 3, 0, 1]),
        "a ronda não deu a volta: {vistos:?}"
    );
    assert_eq!(chegou, 0, "o `On Arrived` falou a meio da ronda");
}

#[test]
fn um_scrub_devolve_a_ronda_ao_bit() {
    let (mut sim, mut b, quem) = ronda();
    let mut primeira = Vec::new();
    for t in 1..=400u64 {
        b.dispatch(&mut sim, true, t);
        primeira.push((pos(&sim, quem), b.nav_ronda(quem)));
    }
    for meio in [350u64, 130] {
        b.dispatch(&mut sim, false, meio);
        assert_eq!(
            (pos(&sim, quem), b.nav_ronda(quem)),
            primeira[(meio - 1) as usize],
            "o scrub a {meio}"
        );
    }
}
