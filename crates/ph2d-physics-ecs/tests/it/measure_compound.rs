//! **Sonda: um corpo pode ter MAIS DE UM collider?**
//!
//! Roda com
//! `cargo test -p ph2d-physics-ecs --test measure_compound -- --ignored --nocapture`.
//!
//! O vão está nomeado desde o W-Offset (*"múltiplos colliders (composite)"*) e o
//! TIPO já o denuncia: a `BodyQuery` da ponte exige `RigidBody` **e** `Collider`
//! na MESMA entidade. A pergunta que esta sonda faz é o que acontece na prática
//! com as duas formas que um artista tentaria:
//!
//! 1. um **filho** que carrega só `Collider` (o `CollisionShape2D` do Godot);
//! 2. um **filho** que carrega `Collider` **e** `RigidBody` (o que sobra hoje).
//!
//! A segunda funciona e não é a mesma coisa: dois corpos ligados são duas MASSAS
//! que o solver pode separar, não uma peça só.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn box_collider(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        ..Collider::default()
    }
}

/// Um "L": um corpo com um braço horizontal, mais uma perna vertical pendurada
/// na hierarquia. Duas formas, uma peça — o caso canônico.
fn ell(child_is_a_body: bool) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        box_collider(20.0, 0.5),
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let arm = sim
        .world_mut()
        .spawn((
            Name::new("Arm"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            box_collider(1.0, 0.2),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    let mut leg = sim.world_mut().spawn((
        Name::new("Leg"),
        box_collider(0.2, 1.0),
        // Pendurada na ponta direita do braço, descendo.
        Transform::from_translation(Vec2::new(0.8, -1.0)),
        ChildOf(arm),
    ));
    if child_is_a_body {
        leg.insert(RigidBody {
            kind: BodyKind::Dynamic,
        });
    }
    let leg = leg.id();
    (sim, leg)
}

fn run(sim: &mut SimWorld, ticks: u64) {
    let mut bridge = PhysicsBridge::new();
    for t in 0..=ticks {
        bridge.dispatch(sim, true, t);
    }
}

fn world_pos(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = ph2d_ecs::world_transform(sim.world(), e).expect("transform");
    [t.translation.x, t.translation.y]
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

#[test]
#[ignore = "sonda de medição"]
fn measure_what_a_second_collider_does_today() {
    println!("\n=== 1. FILHO com só `Collider` (o CollisionShape2D do Godot) ===");
    let (mut sim, leg) = ell(false);
    let arm = named(&mut sim, "Arm");
    let (a0, l0) = (world_pos(&sim, arm), world_pos(&sim, leg));
    run(&mut sim, 180);
    let (a1, l1) = (world_pos(&sim, arm), world_pos(&sim, leg));
    println!("  braço  {a0:?} -> {a1:?}");
    println!("  perna  {l0:?} -> {l1:?}");
    println!(
        "  a perna atravessou o chão? {}  (o chão está em y=0,5)",
        l1[1] < 0.5
    );

    println!("\n=== 2. FILHO com `Collider` + `RigidBody` (o que sobra hoje) ===");
    let (mut sim, leg) = ell(true);
    let arm = named(&mut sim, "Arm");
    run(&mut sim, 180);
    let (a1, l1) = (world_pos(&sim, arm), world_pos(&sim, leg));
    println!("  braço  {a1:?}");
    println!("  perna  {l1:?}");
    // A pose RELATIVA é o que diz se as duas formas continuam sendo uma peça.
    println!(
        "  offset relativo: {:?}  (autorado: [0.8, -1.0])",
        [l1[0] - a1[0], l1[1] - a1[1]]
    );
}
