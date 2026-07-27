//! **Quem é a raiz, e quais arestas existem** (W-IK) — os gates do plano.
//!
//! A árvore inteira decorre destas duas respostas, então elas são afirmadas
//! sozinhas, sem resolver nada: `ik_plan` é puro sobre o estado reconciliado.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

/// Um corpo nomeado com collider de caixa, na pose dada.
fn body(sim: &mut SimWorld, name: &str, x: f32, y: f32, kind: BodyKind) -> Entity {
    let _ = sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, y)),
    ));
    named(sim, name)
}

/// A entidade de um nome — o `spawn` do ECS não devolve `Entity`.
fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn joint(sim: &mut SimWorld, a: &str, b: &str, kind: JointKind, at: [f32; 2]) -> Entity {
    let n = format!("J-{a}-{b}");
    let _ = sim.world_mut().spawn((
        Name::new(&n),
        PhysicsJoint {
            body_a: stable_name_id(a),
            body_b: stable_name_id(b),
            kind,
            ..PhysicsJoint::of_kind(kind)
        },
        Transform::from_translation(Vec2::new(at[0], at[1])),
    ));
    named(sim, &n)
}

/// Gancho estático + três elos de 1 m, pinados ponta a ponta.
fn chain() -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let l1 = body(&mut sim, "L1", 0.5, 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, 0.0, BodyKind::Dynamic);
    let l3 = body(&mut sim, "L3", 2.5, 0.0, BodyKind::Dynamic);
    joint(&mut sim, "Hook", "L1", JointKind::Pin, [0.0, 0.0]);
    joint(&mut sim, "L1", "L2", JointKind::Pin, [1.0, 0.0]);
    joint(&mut sim, "L2", "L3", JointKind::Pin, [2.0, 0.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge, vec![hook, l1, l2, l3])
}

#[test]
fn a_static_neighbour_is_the_root() {
    let (_sim, bridge, e) = chain();
    let plan = bridge.ik_plan(e[3]).expect("a plan for the tip");
    assert_eq!(plan.root, e[0], "the static hook must be the root");
    assert_eq!(plan.edges.len(), 3);
    // Ordem pai→filho a partir da raiz: Hook→L1→L2→L3.
    assert_eq!(plan.edges[0].0, e[0]);
    assert_eq!(plan.edges[0].1, e[1]);
    assert_eq!(plan.edges[2].1, e[3]);
}

#[test]
fn a_free_chain_roots_at_the_far_end() {
    // Sem gancho, a raiz é o elo mais DISTANTE da ponta — e o rapier lhe dá
    // uma raiz livre, então a IK pode transladar o conjunto.
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", 0.5, 0.0, BodyKind::Dynamic);
    let _b = body(&mut sim, "B", 1.5, 0.0, BodyKind::Dynamic);
    let c = body(&mut sim, "C", 2.5, 0.0, BodyKind::Dynamic);
    joint(&mut sim, "A", "B", JointKind::Pin, [1.0, 0.0]);
    joint(&mut sim, "B", "C", JointKind::Pin, [2.0, 0.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let plan = bridge.ik_plan(c).expect("a plan");
    assert_eq!(plan.root, a);
    assert_eq!(plan.edges.len(), 2);
}

#[test]
fn a_spring_is_not_a_link() {
    // Uma mola é SOFT: a pose dela é resultado de forças, não coordenada de
    // junta. Ela não é aresta, então a cadeia PARA nela.
    let mut sim = SimWorld::new();
    let hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let _l1 = body(&mut sim, "L1", 0.5, 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, 0.0, BodyKind::Dynamic);
    joint(&mut sim, "Hook", "L1", JointKind::Pin, [0.0, 0.0]);
    joint(&mut sim, "L1", "L2", JointKind::Spring, [1.0, 0.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    // Pegar L2 não acha nada rígido a que ele pertença.
    assert!(bridge.ik_plan(l2).is_none());
    // E pegar o gancho tampouco (ele não é dinâmico).
    assert!(bridge.ik_plan(hook).is_none());
}

#[test]
fn a_lone_body_has_nothing_to_bend() {
    let mut sim = SimWorld::new();
    let a = body(&mut sim, "A", 0.0, 0.0, BodyKind::Dynamic);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(bridge.ik_plan(a).is_none());
}

#[test]
fn posing_bends_the_chain_and_reports_every_body() {
    let (_sim, mut bridge, e) = chain();
    assert!(bridge.ik_begin(e[3]));
    assert!(bridge.is_posing());
    assert_eq!(bridge.posing_tip(), Some(e[3]));
    assert_eq!(bridge.posing_bodies().len(), 4, "root + three links");
    let mut mid_y = 0.0;
    let mut tip = [0.0f32, 0.0];
    for _ in 0..30 {
        for (ent, p, _) in bridge.ik_move([1.2, 1.8], 0.0, Default::default()) {
            if ent == e[1] {
                mid_y = p[1];
            }
            if ent == e[3] {
                tip = p;
            }
        }
    }
    let d = ((tip[0] - 1.2).powi(2) + (tip[1] - 1.8).powi(2)).sqrt();
    assert!(d < 0.05, "tip stopped {d:.3} m from the target");
    assert!(mid_y > 0.2, "the middle link did not bend (y={mid_y:.3})");
    bridge.ik_end();
    assert!(!bridge.is_posing());
    assert!(
        bridge
            .ik_move([0.0, 0.0], 0.0, Default::default())
            .is_empty()
    );
}

#[test]
fn the_tip_can_be_grabbed_from_either_end_of_the_authored_pair() {
    // O BFS pode chegar a um joint pelo lado B, e aí as âncoras do `JointDesc`
    // estão trocadas em relação à ordem pai→filho. Este gate monta a corrente
    // com a autoria INVERTIDA (`L1` é o body B do joint com o gancho) e exige a
    // mesma pose — sem o swap, o elo pendura pela ponta errada.
    let mut sim = SimWorld::new();
    let _hook = body(&mut sim, "Hook", 0.0, 0.0, BodyKind::Static);
    let _l1 = body(&mut sim, "L1", 0.5, 0.0, BodyKind::Dynamic);
    let l2 = body(&mut sim, "L2", 1.5, 0.0, BodyKind::Dynamic);
    // Autoria invertida nos dois joints.
    joint(&mut sim, "L1", "Hook", JointKind::Pin, [0.0, 0.0]);
    joint(&mut sim, "L2", "L1", JointKind::Pin, [1.0, 0.0]);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    assert!(bridge.ik_begin(l2));
    let mut tip = [0.0f32, 0.0];
    for _ in 0..40 {
        for (ent, p, _) in bridge.ik_move([0.4, 1.5], 0.0, Default::default()) {
            if ent == l2 {
                tip = p;
            }
        }
    }
    let d = ((tip[0] - 0.4).powi(2) + (tip[1] - 1.5).powi(2)).sqrt();
    assert!(d < 0.1, "tip stopped {d:.3} m away with reversed authoring");
}
