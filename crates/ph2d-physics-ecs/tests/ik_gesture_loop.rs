//! **O laço REAL do gesto de pose** (W-IK) — solve, escreve `Transform`, o frame
//! passa, repete.
//!
//! ⚠️ Este é o gate que os outros seis não podiam ser: os deles resolvem várias
//! vezes seguidas **sem o frame passar**, e é exatamente a passagem do frame que
//! quebrava. Escrever o `Transform` é o que posar É, e o dispatch seguinte vê a
//! pose AUTORADA mudada em repouso — a condição em que a ponte **re-descreve** o
//! corpo, dando handle NOVO. A árvore ficava apontando para uma arena que andou:
//! medido, `No element at index` dentro do rapier, no segundo frame do arrasto.
//!
//! Um pânico no meio de um gesto é o pior lugar possível para um, e nenhum gate
//! headless que não passasse o frame podia vê-lo.
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity")
}

#[test]
fn a_pose_survives_the_dispatch_that_follows_its_own_write() {
    let mut sim = SimWorld::new();
    for (n, x, k) in [
        ("Hook", 0.0, BodyKind::Static),
        ("L1", 0.5, BodyKind::Dynamic),
        ("L2", 1.5, BodyKind::Dynamic),
    ] {
        sim.world_mut().spawn((
            Name::new(n),
            RigidBody { kind: k },
            Collider {
                shape: ColliderShape::Cuboid { half_x: 0.5, half_y: 0.1 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
    }
    for (a, b, x) in [("Hook", "L1", 0.0), ("L1", "L2", 1.0)] {
        sim.world_mut().spawn((
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                kind: JointKind::Pin,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(x, 0.0)),
        ));
    }
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let tip = named(&mut sim, "L2");

    assert!(bridge.ik_begin(tip));
    let target = [0.6f32, 1.2];
    let mut tip_pos = [0.0f32, 0.0];
    for frame in 0..20 {
        let poses = bridge.ik_move(target, 0.0, Default::default());
        // ⚠️ **A metade que impede o gate de passar por morte silenciosa:** uma
        // sessão que se encerrasse sozinha devolveria vazio e não panicaria, o
        // que é indistinguível de "não quebrou" para um gate que só olha o
        // pânico.
        assert!(!poses.is_empty(), "frame {frame}: a sessão secou");
        for (e, p, r) in poses {
            if e == tip {
                tip_pos = p;
            }
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
                t.translation = Vec2::new(p[0], p[1]);
                t.rotation = r;
            }
        }
        bridge.dispatch(&mut sim, false, 0);
    }
    // E a metade que prova que ela CONTINUOU RESOLVENDO através dos re-describes.
    let d = ((tip_pos[0] - target[0]).powi(2) + (tip_pos[1] - target[1]).powi(2)).sqrt();
    assert!(d < 0.05, "a ponta parou a {d:.3} m do alvo através do laço real");
    bridge.ik_end();
}
