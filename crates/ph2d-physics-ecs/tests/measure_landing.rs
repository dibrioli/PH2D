//! **SONDA — o POUSO.** Dois reports do Enio (2026-08-07), medidos antes de
//! qualquer diagnóstico:
//!
//! 1. *"quando o player está pousando de seu pulo, a desaceleração ao encostar
//!    no chão é muito lenta e fica artificial"*.
//! 2. *"ao pular na jangada sobre a água, ao primeiro toque, em vez de empurrar
//!    a jangada para baixo, a jangada é ATRAÍDA para o player"*.
//!
//! Esta sonda **não conserta nada** — ela nomeia os dois com números.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_landing -- --ignored --nocapture --test-threads=1`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge,
    PlatformPlayer, RigidBody,
};

const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
const FLOAT: f32 = 0.9;
/// O passo do relógio do módulo.
const DT: f32 = 1.0 / 60.0;

fn capsule() -> Collider {
    Collider {
        shape: ColliderShape::Capsule {
            half_height: HALF_H,
            radius: RADIUS,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// Chão plano com o topo em `y = 0`.
fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
}

fn player(sim: &mut SimWorld, y: f32, damping: f32) -> Entity {
    player_k(sim, y, damping, PlatformPlayer::default().spring_strength)
}

fn player_k(sim: &mut SimWorld, y: f32, damping: f32, strength: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            capsule(),
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                spring_damping: damping,
                spring_strength: strength,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, y)),
        ))
        .id()
}

fn y_of(sim: &SimWorld, who: &str) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some(t.translation.y);
        }
    }
    found.expect("o corpo tem de existir")
}

/// **O DEFAULT que o artista recebe** — porque o doc da mola e o
/// `STARTING_POINT` discordavam, e um deles está a mentir.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_shipping_default() {
    let p = PlatformPlayer::default();
    println!("\n=== O DEFAULT DE `PlatformPlayer::default()` ===");
    println!("  spring_damping  = {:.4}", p.spring_damping);
    println!("  spring_strength = {:.1}", p.spring_strength);
    println!("  float_height    = {:.3}", p.float_height);
    println!("  cling_distance  = {:.3}", p.cling_distance);
    println!("  fall_gravity    = {:.2}", p.fall_gravity);
    println!("  peak_gravity    = {:.2}", p.peak_gravity);
    println!("  peak_speed      = {:.2}", p.peak_speed);
}

/// **O PERFIL DO POUSO** — tick a tick, do primeiro toque ao repouso.
///
/// A régua: quantos tiques da PRIMEIRA vez que o sensor vê chão até o corpo
/// estar a menos de 1 cm da altura de repouso **e** abaixo de 0,05 m/s.
#[test]
#[ignore = "sonda de medição"]
fn measure_landing_profile() {
    println!("\n=== PERFIL DO POUSO (queda de 3 m, damping do produto) ===");
    let damping = PlatformPlayer::default().spring_damping;
    let mut sim = SimWorld::new();
    floor(&mut sim);
    player(&mut sim, 3.0, damping);
    let mut bridge = PhysicsBridge::new();

    let reach = FLOAT + PlatformPlayer::default().cling_distance;
    let mut prev = 3.0_f32;
    let mut touch: Option<u64> = None;
    let mut settled: Option<u64> = None;
    let mut lowest = f32::INFINITY;
    println!("{:>5} {:>9} {:>9} {:>8}", "tick", "y", "vy", "fase");
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
        let y = y_of(&sim, "Player");
        let vy = (y - prev) / DT;
        prev = y;
        lowest = lowest.min(y);
        if touch.is_none() && y <= reach {
            touch = Some(t);
        }
        if touch.is_some() && settled.is_none() && (y - FLOAT).abs() < 0.01 && vy.abs() < 0.05 {
            settled = Some(t);
        }
        if let Some(t0) = touch
            && t <= t0 + 40
        {
            {
                println!(
                    "{t:>5} {y:>9.4} {vy:>9.3} {:>8}",
                    if settled.is_some_and(|s| t >= s) {
                        "repouso"
                    } else {
                        "pousando"
                    }
                );
            }
        }
    }
    let t0 = touch.expect("tem de tocar");
    let ts = settled.expect("tem de assentar");
    println!(
        "\n  toque no tick {t0} · repouso no tick {ts} => {} tiques = {:.3} s",
        ts - t0,
        (ts - t0) as f32 * DT
    );
    println!(
        "  ponto mais baixo {lowest:.4} (repouso {FLOAT:.3}) => afunda {:.1} cm abaixo do repouso",
        (FLOAT - lowest) * 100.0
    );
}

/// **O QUE O KNOB COMPRA** — a mesma queda, varrendo o amortecimento.
#[test]
#[ignore = "sonda de medição"]
fn measure_landing_against_damping() {
    println!("\n=== POUSO vs `spring_damping` (queda de 3 m) ===");
    println!(
        "{:>9} {:>10} {:>12} {:>12} {:>14}",
        "damping", "tiques", "segundos", "afunda (cm)", "deriva 30° (m)"
    );
    for d in [
        0.25_f32, 0.40, 0.45, 0.50, 0.55, 0.60, 0.667, 0.75, 0.90, 1.0,
    ] {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        player(&mut sim, 3.0, d);
        let mut bridge = PhysicsBridge::new();
        let reach = FLOAT + PlatformPlayer::default().cling_distance;
        let mut prev = 3.0_f32;
        let (mut touch, mut settled) = (None, None);
        let mut lowest = f32::INFINITY;
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
            let y = y_of(&sim, "Player");
            let vy = (y - prev) / DT;
            prev = y;
            lowest = lowest.min(y);
            if touch.is_none() && y <= reach {
                touch = Some(t);
            }
            if touch.is_some() && settled.is_none() && (y - FLOAT).abs() < 0.01 && vy.abs() < 0.05 {
                settled = Some(t);
            }
        }
        let (t0, ts) = (touch.unwrap(), settled.unwrap_or(600));
        // ⚠️ A coluna da deriva NÃO é medida aqui — ela vem da LEI já publicada
        // e gateada (`RideConfig::STARTING_POINT`, varredura de 05/08):
        // `deriva(10 s) = 0,153 · sen θ · (1 − d)`. Re-medi-la seria uma segunda
        // resposta para um número que já tem dono.
        let drift = 0.153 * (30.0_f32.to_radians().sin()) * (1.0 - d);
        println!(
            "{d:>9.3} {:>10} {:>12.3} {:>12.1} {drift:>14.4}",
            ts - t0,
            (ts - t0) as f32 * DT,
            (FLOAT - lowest) * 100.0
        );
    }
}

/// **O TERCEIRO EIXO** — com o amortecimento no teto (o que shipa), a RIGIDEZ.
///
/// ⚠️ Com `damping = 1` o boost apaga a velocidade relativa INTEIRA a cada
/// tique, então o único movimento que sobra é o que a mola produz **num** tique:
/// `Δoffset = k · offset · dt²` ⇒ o resto decai por `1 − k·dt²`, uma progressão
/// geométrica. A rigidez é quem manda nessa razão, e o amortecimento **não**.
#[test]
#[ignore = "sonda de medição"]
fn measure_landing_against_stiffness() {
    println!("\n=== POUSO vs `spring_strength`, com `damping = 1,0` (o que shipa) ===");
    println!(
        "{:>10} {:>10} {:>12} {:>12} {:>10}",
        "strength", "tiques", "segundos", "afunda (cm)", "1 - k*dt²"
    );
    for k in [400.0_f32, 800.0, 1200.0, 1600.0, 2000.0, 3000.0] {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        player_k(&mut sim, 3.0, 1.0, k);
        let mut bridge = PhysicsBridge::new();
        let reach = FLOAT + PlatformPlayer::default().cling_distance;
        let mut prev = 3.0_f32;
        let (mut touch, mut settled) = (None, None);
        let mut lowest = f32::INFINITY;
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
            let y = y_of(&sim, "Player");
            let vy = (y - prev) / DT;
            prev = y;
            lowest = lowest.min(y);
            if touch.is_none() && y <= reach {
                touch = Some(t);
            }
            if touch.is_some() && settled.is_none() && (y - FLOAT).abs() < 0.01 && vy.abs() < 0.05 {
                settled = Some(t);
            }
        }
        let (t0, ts) = (touch.unwrap(), settled.unwrap_or(600));
        println!(
            "{k:>10.0} {:>10} {:>12.3} {:>12.1} {:>10.3}",
            ts - t0,
            (ts - t0) as f32 * DT,
            (FLOAT - lowest) * 100.0,
            1.0 - k * DT * DT
        );
    }
}

/// **A JANGADA** — o report 2, tick a tick no primeiro toque.
///
/// A pergunta: no instante em que o sensor do personagem alcança a jangada, ela
/// sobe ou desce?
#[test]
#[ignore = "sonda de medição"]
fn measure_first_touch_on_a_raft() {
    println!("\n=== PRIMEIRO TOQUE NUMA JANGADA (poça + jangada a boiar) ===");
    let mut sim = SimWorld::new();
    // A poça: sensor com empuxo e arrasto, topo em y = 0.
    sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(4.0),
        AreaDrag(0.6),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Raft"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.5,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, 0.5)),
    ));
    let mut bridge = PhysicsBridge::new();
    // Deixa a jangada assentar na linha d'água ANTES de o personagem existir.
    for t in 1..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let rest = y_of(&sim, "Raft");
    println!("  jangada em repouso: y = {rest:.4}");

    // O personagem cai de 3 m acima dela.
    player(
        &mut sim,
        rest + 3.0,
        PlatformPlayer::default().spring_damping,
    );
    let reach = FLOAT + PlatformPlayer::default().cling_distance;

    println!(
        "{:>5} {:>9} {:>10} {:>10} {:>9}",
        "tick", "player y", "jangada y", "delta (mm)", "dist"
    );
    let mut touch: Option<u64> = None;
    let mut highest = rest;
    for t in 301..=420u64 {
        bridge.dispatch(&mut sim, true, t);
        let py = y_of(&sim, "Player");
        let ry = y_of(&sim, "Raft");
        // A distância que o sensor de chão mede: do centro do player ao topo da
        // jangada (a cápsula tem meia-altura + raio abaixo do centro).
        let dist = py - (ry + 0.2);
        if touch.is_none() && dist <= reach {
            touch = Some(t);
            println!("  --- o sensor alcanca a jangada ---");
        }
        if touch.is_some() {
            highest = highest.max(ry);
            println!(
                "{t:>5} {py:>9.4} {ry:>10.4} {:>10.2} {dist:>9.4}",
                (ry - rest) * 1000.0
            );
        }
        if touch.is_some_and(|t0| t > t0 + 25) {
            break;
        }
    }
    println!(
        "\n  a jangada SOBE {:.2} mm acima do repouso antes de descer",
        (highest - rest) * 1000.0
    );
}
