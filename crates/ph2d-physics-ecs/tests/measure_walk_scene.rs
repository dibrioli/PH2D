//! **A CENA 81 medida, não desenhada de cabeça.**
//!
//! O report do Enio sobre o *Max Slope* veio de andar na cena `=81`, e antes de
//! escrever qualquer coisa sobre "ande até a rampa" é preciso saber se **dá para
//! chegar lá andando**. Esta sonda reconstrói a geometria daquela cena (mesmos
//! centros, meias-extensões e rotações) e caminha para os dois lados, imprimindo
//! onde o personagem PARA.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --test measure_walk_scene -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A altura de flutuação da cena de smoke (a mesma do `spawn_player`).
const FLOAT: f32 = 0.9;

fn slab(sim: &mut SimWorld, name: &str, at: Vec2, half: [f32; 2], rot: f32) {
    sim.world_mut().spawn((
        Name::new(name.to_string()),
        Transform {
            rotation: rot,
            ..Transform::from_translation(at)
        },
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            ..Collider::default()
        },
    ));
}

fn player(sim: &mut SimWorld, at: Vec2) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("player")
}

/// A geometria da cena 81 **como ela era** — a que a sonda reprovou.
fn scene_81_before() -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Floor", Vec2::new(0.0, -0.5), [10.0, 0.5], 0.0);
    slab(
        &mut sim,
        "Ramp30",
        Vec2::new(-13.0, 1.2),
        [4.5, 0.5],
        30.0_f32.to_radians(),
    );
    slab(
        &mut sim,
        "Ramp60",
        Vec2::new(13.0, 2.4),
        [3.5, 0.5],
        -60.0_f32.to_radians(),
    );
    let p = player(&mut sim, Vec2::new(0.0, 2.0));
    (sim, PhysicsBridge::new(), p)
}

/// A geometria da cena 81 **corrigida** — a rampa rasa passa a ser alcançável.
fn scene_81_after() -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Floor", Vec2::new(0.0, -0.5), [16.0, 0.5], 0.0);
    slab(&mut sim, "WallL", Vec2::new(-16.5, 2.0), [0.5, 2.5], 0.0);
    slab(&mut sim, "WallR", Vec2::new(16.5, 2.0), [0.5, 2.5], 0.0);
    slab(
        &mut sim,
        "Ramp30",
        Vec2::new(-7.0, 1.3),
        [4.0, 0.5],
        -30.0_f32.to_radians(),
    );
    slab(&mut sim, "Plateau", Vec2::new(-13.0, 3.3), [3.5, 0.41], 0.0);
    let p = player(&mut sim, Vec2::new(0.0, 2.0));
    (sim, PhysicsBridge::new(), p)
}

/// A geometria CANDIDATA da cena 88 — a ladeira A/B.
///
/// Os dois lados são o par que cerca o limite autorado (45): **40° à esquerda**
/// (um degrau abaixo, tem de subir) e **50° à direita** (um degrau acima, tem de
/// escorregar). O sinal da rotação é o que decide de que lado a rampa SOBE, e é
/// ele que a cena 81 errou: lá as duas rampas sobem para longe do chão e o
/// personagem passa por baixo delas e cai no vazio.
fn scene_88() -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Floor", Vec2::new(0.0, -0.5), [16.0, 0.5], 0.0);
    slab(&mut sim, "WallL", Vec2::new(-16.5, 2.0), [0.5, 2.5], 0.0);
    slab(&mut sim, "WallR", Vec2::new(16.5, 2.0), [0.5, 2.5], 0.0);
    slab(
        &mut sim,
        "Ramp40",
        Vec2::new(-6.0, 1.1),
        [3.0, 0.5],
        -40.0_f32.to_radians(),
    );
    slab(&mut sim, "Plateau", Vec2::new(-12.0, 3.0), [4.0, 0.41], 0.0);
    slab(
        &mut sim,
        "Ramp50",
        Vec2::new(5.0, 1.2),
        [3.0, 0.5],
        50.0_f32.to_radians(),
    );
    let p = player(&mut sim, Vec2::new(0.0, 2.0));
    (sim, PhysicsBridge::new(), p)
}

fn run_scene(
    build: fn() -> (SimWorld, PhysicsBridge, ph2d_ecs::Entity),
    drive: f32,
    seconds: u64,
) -> Vec<(f32, f32)> {
    let (mut sim, mut bridge, p) = build();
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.set_player_input(
        p,
        PlayerInput {
            drive,
            ..PlayerInput::default()
        },
    );
    let mut track = Vec::new();
    let total = seconds * 60;
    for tick in 31..=(30 + total) {
        bridge.dispatch(&mut sim, true, tick);
        if (tick - 30) % 60 == 0 {
            track.push(pose(&sim));
        }
    }
    track
}

#[test]
#[ignore = "sonda de medicao"]
fn measure_whether_the_ramps_of_scene_81_are_reachable_on_foot() {
    for (label, drive) in [
        ("ESQUERDA (rampa 30deg)", -1.0),
        ("DIREITA (rampa 60deg)", 1.0),
    ] {
        eprintln!("=== CENA 81 ANTES, andando para a {label} ===");
        for (i, (x, y)) in run_scene(scene_81_before, drive, 6).into_iter().enumerate() {
            eprintln!("  t={}s  x={x:+8.2}  y={y:+8.2}", i + 1);
        }
    }
    eprintln!("=== CENA 81 DEPOIS, andando para a ESQUERDA (rampa 30deg) ===");
    for (i, (x, y)) in run_scene(scene_81_after, -1.0, 6).into_iter().enumerate() {
        eprintln!("  t={}s  x={x:+8.2}  y={y:+8.2}", i + 1);
    }
    for (label, drive) in [
        ("ESQUERDA (rampa 40deg -- tem de SUBIR)", -1.0),
        ("DIREITA (rampa 50deg -- tem de ESCORREGAR)", 1.0),
    ] {
        eprintln!("=== CENA 88 candidata, andando para a {label} ===");
        for (i, (x, y)) in run_scene(scene_88, drive, 6).into_iter().enumerate() {
            eprintln!("  t={}s  x={x:+8.2}  y={y:+8.2}", i + 1);
        }
    }
}
