//! **Sonda: uma PEÇA marcada como Sensor vira um trigger?**
//!
//! Roda com
//! `cargo test -p ph2d-physics-ecs --test measure_part_sensor -- --ignored --nocapture`.
//!
//! A W-PartFace passou a pintar o chip **Solid | Sensor** na face de peça, e a
//! política de fechamento desta linha exige que a *sequência* leve a algum lugar
//! — não basta o clique chegar ao barramento. O caso de uso é o mais comum que
//! existe num módulo 2D: o **sensor de pé** de um personagem (um corpo com o
//! collider sólido do tronco mais uma peça-sensor embaixo, que responde *"estou
//! no chão?"*). Box2D e Unity o expressam exatamente assim, com uma *fixture*
//! sensora no mesmo corpo.
//!
//! Três perguntas, cada uma com uma consequência diferente:
//!
//! 1. o collider da peça chega ao solver **como sensor** (atravessa)?
//! 2. o canal de trigger da ponte (`is_triggered` / `bodies_inside` /
//!    `triggered_sensors`) a enxerga?
//! 3. o dono continua sólido — ou seja, marcar a peça não desliga o corpo?

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

fn box_collider(half_x: f32, half_y: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid { half_x, half_y },
        ..Collider::default()
    }
}

/// Um personagem: tronco sólido (o corpo) + uma peça embaixo dele, e uma
/// plataforma estática logo abaixo para a peça sobrepor.
///
/// `part_is_sensor` é a única diferença entre os dois braços.
fn character(part_is_sensor: bool) -> (SimWorld, Entity, Entity, Entity) {
    let mut sim = SimWorld::new();
    let ground = sim
        .world_mut()
        .spawn((
            Name::new("Ground"),
            RigidBody {
                kind: BodyKind::Static,
            },
            box_collider(20.0, 0.5),
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    // Tronco: meia-altura 1,0, assentado sobre o chão (topo do chão em 0,5).
    let torso = sim
        .world_mut()
        .spawn((
            Name::new("Torso"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            box_collider(0.4, 1.0),
            Transform::from_translation(Vec2::new(0.0, 1.5)),
        ))
        .id();
    // O pé: uma peça larga e chata, pendurada 1,0 abaixo do centro do tronco —
    // ou seja MERGULHADA no chão, para haver sobreposição a reportar.
    let foot = sim
        .world_mut()
        .spawn((
            Name::new("Foot"),
            Collider {
                is_sensor: part_is_sensor,
                ..box_collider(0.5, 0.2)
            },
            Transform::from_translation(Vec2::new(0.0, -1.0)),
            ChildOf(torso),
        ))
        .id();
    (sim, ground, torso, foot)
}

fn run(sim: &mut SimWorld, ticks: u64) -> PhysicsBridge {
    let mut bridge = PhysicsBridge::new();
    for t in 0..=ticks {
        bridge.dispatch(sim, true, t);
    }
    bridge
}

fn world_y(sim: &SimWorld, e: Entity) -> f32 {
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

#[test]
#[ignore = "sonda de medição"]
fn measure_part_sensor() {
    println!("\n=== uma PEÇA marcada Sensor ===");
    for part_is_sensor in [false, true] {
        let (mut sim, ground, torso, foot) = character(part_is_sensor);
        let bridge = run(&mut sim, 120);
        println!(
            "\n  peça sensor = {part_is_sensor}\n    tronco y = {:.4}\n    \
             is_triggered(pé)     = {}\n    is_triggered(tronco) = {}\n    \
             bodies_inside(pé)     = {:?}\n    bodies_inside(tronco) = {:?}\n    \
             triggered_sensors()   = {:?}\n    (chão = {ground:?}, tronco = {torso:?}, pé = {foot:?})",
            world_y(&sim, torso),
            bridge.is_triggered(foot),
            bridge.is_triggered(torso),
            bridge.bodies_inside(foot),
            bridge.bodies_inside(torso),
            bridge.triggered_sensors(),
        );
    }
    println!(
        "\n  Leitura: se o tronco assenta no MESMO y nos dois braços, a peça-sensor\n  \
         não está atravessando (o chip não chega ao solver). Se ele difere mas\n  \
         `triggered_sensors()` fica vazio, o solver a honra e o CANAL não a vê."
    );
}
