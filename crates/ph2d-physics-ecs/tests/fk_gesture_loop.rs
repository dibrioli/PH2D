//! **O laço REAL do gesto de FK** (W-FK) — aplica, escreve `Transform`, o frame
//! passa, repete.
//!
//! O irmão deste gate no lado da IK existe porque lá o frame passando
//! INVALIDAVA a sessão (a árvore de multibody guarda handles, e escrever a pose
//! re-descreve os corpos). Aqui a afirmação é a oposta e igualmente necessária:
//! a sessão de FK **não** guarda nada da arena, então o frame passando não pode
//! mudar nada — e *"não pode"* é uma hipótese até um gate dirigir o laço.
//!
//! ⚠️ E há um segundo modo de falha que só o laço vê: o `settle` da ponte lê a
//! pose AUTORADA em repouso e a impõe ao corpo. Se a pose escrita e a pose que o
//! solver recebe discordassem, o gesto ficaria brigando com o dispatch —
//! avançando um pouco e sendo puxado de volta a cada frame, o que num gate
//! estático é invisível.

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
fn a_swing_survives_the_dispatches_that_follow_its_own_writes() {
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
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.1,
                },
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
    let l1 = named(&mut sim, "L1");
    let l2 = named(&mut sim, "L2");

    // Pega L1 pelo centro; a junta acima dele é a do gancho, em (0, 0).
    assert!(bridge.fk_begin(&sim, l1, [0.5, 0.0]));
    // Vinte frames varrendo de 0 a 90°, escrevendo e despachando a cada um.
    for frame in 0..=20 {
        let a = (frame as f32 / 20.0) * std::f32::consts::FRAC_PI_2;
        let (s, c) = (a.sin(), a.cos());
        let poses = bridge.fk_move([0.5 * c, 0.5 * s]);
        assert!(!poses.is_empty(), "frame {frame}: a sessão secou");
        for (e, p, r) in poses {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
                t.translation = Vec2::new(p[0], p[1]);
                t.rotation = r;
            }
        }
        bridge.dispatch(&mut sim, false, 0);
    }

    // A peça inteira girou 90° em torno de (0, 0): L1 de (0.5, 0) para (0, 0.5)
    // e L2 de (1.5, 0) para (0, 1.5).
    let p1 = sim.world().get::<Transform>(l1).expect("L1").translation;
    let p2 = sim.world().get::<Transform>(l2).expect("L2").translation;
    assert!(
        (p1.x).abs() < 1e-2 && (p1.y - 0.5).abs() < 1e-2,
        "L1 terminou em ({}, {}) — o dispatch puxou a pose de volta",
        p1.x,
        p1.y
    );
    assert!(
        (p2.x).abs() < 1e-2 && (p2.y - 1.5).abs() < 1e-2,
        "L2 terminou em ({}, {})",
        p2.x,
        p2.y
    );
    bridge.fk_end();
    assert!(!bridge.is_posing_fk());
}
