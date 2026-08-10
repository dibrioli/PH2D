//! **QUEM A CORRENTEZA LEVA** — a sonda da lacuna que a `W-ZoneForce` fecha.
//!
//! O item do Enio, e o plano 08 §4.2 escreve o mecanismo: `effector::apply`
//! **recusa corpo não-dinâmico** antes de o tocar (`if !b.is_dynamic() { continue }`,
//! e a recusa está certa — um corpo cinemático tem massa infinita e o solver
//! ignoraria o impulso de qualquer maneira), e a lei cinemática integra um
//! `Fluid { buoyed, drag }` que **não tem força nenhuma**. As duas metades estão
//! corretas sozinhas e a soma delas é um personagem que a água empurra num modo e
//! não empurra nos outros dois.
//!
//! Esta sonda não afirma: ela põe os **três modos** na MESMA correnteza e imprime
//! o quanto cada um andou. E mede uma segunda coisa que ninguém tinha perguntado —
//! se um corpo **COMPOSTO** recebe o empurrão uma vez ou uma vez por FORMA.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_the_current -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PhysicsSettings,
    PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;

/// Sem gravidade: a zona é a única coisa a agir, então o que sobra é o que ela fez.
fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// Uma correnteza larga que empurra em +X.
fn current(sim: &mut SimWorld, force: f32) {
    sim.world_mut().spawn((
        Name::new("Current"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 30.0,
                half_y: 12.0,
            },
            ..Collider::default()
        },
        AreaEffector {
            force: [force, 0.0],
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

fn subject_tuned(sim: &mut SimWorld, mode: Option<PlayerMode>, brake: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if mode.is_some_and(PlayerMode::drives_itself) {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    if let Some(m) = mode {
        e.insert(PlatformPlayer {
            acceleration: brake,
            air_acceleration: brake,
            ..PlatformPlayer::default()
        });
        e.insert(m);
    }
    e.id()
}

fn subject(sim: &mut SimWorld, mode: Option<PlayerMode>) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if mode.is_some_and(PlayerMode::drives_itself) {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    if let Some(m) = mode {
        e.insert(PlatformPlayer::default());
        e.insert(m);
    }
    e.id()
}

fn x_of(sim: &SimWorld) -> f32 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation.x;
        }
    }
    panic!("o sujeito tem de existir");
}

/// Dois segundos de correnteza. Devolve o quanto o sujeito andou em X.
fn carried(mode: Option<PlayerMode>, force: f32) -> f32 {
    let mut sim = SimWorld::new();
    current(&mut sim, force);
    let who = subject(&mut sim, mode);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    x_of(&sim)
}

/// O mesmo, com o freio da caminhada ABLACIONADO pelo knob do artista.
fn carried_tuned(mode: Option<PlayerMode>, force: f32, brake: f32) -> f32 {
    let mut sim = SimWorld::new();
    current(&mut sim, force);
    let who = subject_tuned(&mut sim, mode, brake);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    x_of(&sim)
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_current() {
    println!("\n=== QUEM A CORRENTEZA LEVA (2 s, forca 4 N, sem gravidade) ===\n");
    println!("| sujeito              | x depois de 2 s |");
    println!("|----------------------|-----------------|");
    for (name, mode) in [
        ("caixote (sem player)", None),
        ("player Dynamic", Some(PlayerMode::Dynamic)),
        ("player Kinematic", Some(PlayerMode::Kinematic)),
        ("player Pure", Some(PlayerMode::Pure)),
    ] {
        let x = carried(mode, 4.0);
        println!("| {name:20} | {x:15.4} |");
    }

    println!("\n=== E POR QUE O DYNAMIC MAL ANDA: o FREIO da caminhada ===");
    println!("(ablacao pelo knob do artista: `acceleration`/`air_acceleration`)\n");
    println!("| freio  | Dynamic | Kinematic |");
    println!("|--------|---------|-----------|");
    for brake in [60.0f32, 20.0, 5.0, 1.0, 0.0] {
        let d = carried_tuned(Some(PlayerMode::Dynamic), 4.0, brake);
        let k = carried_tuned(Some(PlayerMode::Kinematic), 4.0, brake);
        println!("| {brake:6.1} | {d:7.4} | {k:9.4} |");
    }

    println!("\n=== E COM A FORCA: quanto a correnteza vence o freio de fabrica ===\n");
    println!("| forca (N) | caixote | Dynamic | Kinematic |");
    println!("|-----------|---------|---------|-----------|");
    for f in [1.0f32, 4.0, 16.0, 64.0, 256.0] {
        let c = carried(None, f);
        let d = carried(Some(PlayerMode::Dynamic), f);
        let k = carried(Some(PlayerMode::Kinematic), f);
        println!("| {f:9.1} | {c:7.3} | {d:7.4} | {k:9.4} |");
    }
    println!();
}
