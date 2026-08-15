//! **O que a normal do contato CUSTA** (`W-HitNormal`).
//!
//! O trabalho acrescentado é uma rotação de vetor por par de colliders ativo, por
//! sub-passo — ao lado de uma transformação de isometria (rotação **e**
//! translação) que o `active_pair` já fazia no ponto, sobre os mesmos dados. A
//! §0 manda medir mesmo assim: um argumento sobre o código não é um número.
//!
//! `cargo test -p ph2d-physics-ecs --release --test measure_contact_normal -- --ignored --nocapture`

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Uma pilha densa — muitos pares ativos ao mesmo tempo, que é o regime em que
/// um custo por-par aparece.
fn pile(rows: usize, cols: usize) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    for r in 0..rows {
        for c in 0..cols {
            sim.world_mut().spawn((
                Name::new("Box"),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.25,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(
                    (c as f32).mul_add(0.52, -10.0),
                    (r as f32).mul_add(0.51, 0.25),
                )),
            ));
        }
    }
    sim
}

#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_step_with_many_contacts() {
    for (rows, cols) in [(4, 20), (8, 40)] {
        let mut sim = pile(rows, cols);
        let mut bridge = PhysicsBridge::new();
        // Deixa a pilha assentar: é DEPOIS de assentar que os pares existem
        // todos ao mesmo tempo, e uma medição feita durante a queda mediria uma
        // cena com menos contatos do que a que se quer medir.
        for t in 1..=120u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let pairs = bridge.contacts().len();
        let start = Instant::now();
        let n = 120u64;
        for t in 121..=120 + n {
            bridge.dispatch(&mut sim, true, t);
        }
        let per = start.elapsed().as_secs_f64() * 1000.0 / n as f64;
        println!(
            "{:4} corpos | {pairs:4} pares tocando | {per:7.3} ms/tique",
            rows * cols
        );
    }
}
