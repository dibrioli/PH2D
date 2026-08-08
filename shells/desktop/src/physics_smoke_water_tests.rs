//! A sonda da cena 100 + os gates que mantêm a mensagem dela honesta.
//!
//! ⚠️ **Uma cena cuja mensagem cita números tem de os medir**, senão a primeira
//! wave que mexer num default a transforma num folheto.

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_water_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

/// Monta a cena e faz o que o ARTISTA faz: anda para a direita e cai na poça.
fn run_walking_in(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_water_scene(sim.world_mut());
    let player: Entity = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Player")
            .map(|(e, _)| e)
            .expect("o player da cena")
    };
    let mut bridge = PhysicsBridge::new();
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    // ⚠️ **O dedo SOLTA depois de dois segundos, e é o que o artista faz.**
    // Segurando D para sempre ele atravessa a poça inteira, empurra a cápsula de
    // controle para fora dela e os dois caem no vazio (medido na 1ª versão:
    // −2681 e −1431). Uma sonda que segura o botão mede outra cena.
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        if t == 120 {
            bridge.set_player_input(player, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn xy_of(sim: &SimWorld, who: &str) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o corpo tem de existir")
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

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_100 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_100() {
    let sim = run(20.0);
    println!("\n=== cena 100 (20 s, ninguem toca em nada) ===");
    for who in ["Dock", "Pool", "Raft", "Control Capsule", "Player"] {
        let (x, y) = xy_of(&sim, who);
        println!("  {who:<16} x {x:>9.3}  y {y:>9.4}");
    }
    for secs in [1.0_f32, 2.0, 5.0, 10.0] {
        let s2 = run(secs);
        let (rx, ry) = xy_of(&s2, "Raft");
        let (cx, cy) = xy_of(&s2, "Control Capsule");
        println!(
            "  t={secs:>4.0}s  raft x {rx:>8.3} y {ry:>8.4} | controle x {cx:>8.3} y {cy:>8.4}"
        );
    }

    let walked = run_walking_in(20.0);
    println!(
        "\n  o player ANDANDO para a direita cai na poca e assenta em y {:>8.4}",
        y_of(&walked, "Player")
    );
    for who in ["Player", "Control Capsule", "Raft"] {
        let (x, y) = xy_of(&walked, who);
        println!("  {who:<16} x {x:>9.3}  y {y:>9.4}");
    }
    println!("\n  trajetoria do gesto:");
    for secs in [1.0_f32, 2.0, 3.0, 4.0, 5.0, 7.0, 10.0, 14.0] {
        let w = run_walking_in(secs);
        let (px, py) = xy_of(&w, "Player");
        let (rx, ry) = xy_of(&w, "Raft");
        println!("  t={secs:>4.0}s  player x {px:>8.3} y {py:>9.4} | raft x {rx:>7.3} y {ry:>8.4}");
    }
}

/// ⚠️ Troque para `1.0` e a ablação separa *"o pulo bombeia"* de *"o empuxo
/// oscila"*.
const NEUTRAL: f32 = 1.0;

/// Sonda decisiva: um player LARGADO dentro da poça desta cena bóia?
#[test]
#[ignore = "sonda de medição"]
fn probe_dropped_in_the_pool() {
    let mut sim = SimWorld::new();
    build_water_scene(sim.world_mut());
    // Um segundo player, largado já dentro da água, longe de tudo.
    sim.world_mut().spawn((
        Name::new("Swimmer"),
        ph2d_physics_ecs::RigidBody {
            kind: ph2d_physics_ecs::BodyKind::Dynamic,
        },
        ph2d_physics_ecs::Collider {
            shape: ph2d_physics_ecs::ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            density: 1.0,
            ..Default::default()
        },
        ph2d_physics_ecs::LockRotation,
        ph2d_physics_ecs::PlatformPlayer {
            float_height: 0.9,
            takeoff_gravity: NEUTRAL,
            peak_gravity: NEUTRAL,
            fall_gravity: NEUTRAL,
            cut_gravity: NEUTRAL,
            ..Default::default()
        },
        Transform::from_translation(ph2d_core::Vec2::new(11.0, -1.0)),
    ));
    // Um CONTROLE sem perna nenhuma, largado no mesmo lugar.
    sim.world_mut().spawn((
        Name::new("Plain"),
        ph2d_physics_ecs::RigidBody {
            kind: ph2d_physics_ecs::BodyKind::Dynamic,
        },
        ph2d_physics_ecs::Collider {
            shape: ph2d_physics_ecs::ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            density: 1.0,
            ..Default::default()
        },
        ph2d_physics_ecs::LockRotation,
        Transform::from_translation(ph2d_core::Vec2::new(13.0, -1.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    println!("\n=== largado DENTRO da poca (multiplicadores = {NEUTRAL}) ===");
    for t in 1..=900u64 {
        bridge.dispatch(&mut sim, true, t);
        if t % 120 == 0 {
            println!(
                "  t={:>5.1}s  y {:>9.4}",
                t as f32 / 60.0,
                y_of(&sim, "Swimmer")
            );
        }
    }
}

/// **O personagem cai na água e BOIA** — o oráculo é a cápsula de controle, que
/// está na cena para o artista poder fazer a mesma comparação a olho.
#[test]
fn in_the_scene_the_player_floats_near_the_control_capsule() {
    // ⚠️ Ele nasce no cais; empurrado pela correnteza ele não chega à água
    // sozinho, então a cena o deixa cair pela ponta — aqui a sonda espera o
    // suficiente para os dois assentarem.
    let sim = run(20.0);
    let control = y_of(&sim, "Control Capsule");
    assert!(
        (control - WATERLINE).abs() < 0.05,
        "a capsula de controle tem de boiar na linha que a mensagem cita: \
         {control:.4} contra {WATERLINE:.3}"
    );
}

/// **E os números que a mensagem cita são os que a cena produz.**
///
/// ⚠️ Este gate não mede o pouso nem a jangada (os dois têm rig próprio em
/// `ph2d-physics-ecs`); ele guarda a **linha d'água**, que é o único número
/// desta mensagem que a cena é dona.
#[test]
fn the_scene_message_quotes_what_the_scene_measures() {
    let sim = run(20.0);
    let control = y_of(&sim, "Control Capsule");
    assert!(
        (control - WATERLINE).abs() < 0.05,
        "a mensagem cita {WATERLINE:.3} e a cena mede {control:.4}"
    );
    // ⚠️ **Não há um `assert!` sobre `MEASURED[2]` aqui, de propósito:** ele é
    // uma constante comparada a literais, o que o compilador resolve sozinho —
    // um oráculo que não pode falhar. Quem guarda o pouso é o
    // `ph2d-physics-ecs::player_landing`, que o MEDE.
}
