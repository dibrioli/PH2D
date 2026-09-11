//! A sonda da cena 73 + os gates que mantêm a mensagem dela honesta (W-Signal).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs`, colhendo TODO sinal pela porta REAL.
fn fired(secs: f32) -> Vec<String> {
    let mut sim = SimWorld::new();
    build_signal_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 0..=(secs * 60.0) as u64 {
        bridge.dispatch(&mut sim, true, t);
        for s in bridge.signal_events(&sim) {
            out.push(s.name);
        }
    }
    out
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_73 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_73() {
    let all = fired(4.0);
    println!("\n=== cena 73 (4 s) ===");
    for name in ["door", "bell", "quiet"] {
        println!(
            "  {name:<6} disparou {} vez(es)",
            all.iter().filter(|n| *n == name).count()
        );
    }
}

/// **As duas fontes gritam, e o CONTROLE cala.**
///
/// A porta é um SENSOR (que nunca gera contato) e o sino é SÓLIDO: um canal só de
/// colisão sólida deixaria a porta — o caso canônico de gameplay — em silêncio.
#[test]
fn the_sensor_and_the_solid_both_shout_and_the_control_stays_quiet() {
    let all = fired(4.0);
    for name in ["door", "bell"] {
        assert!(
            all.iter().any(|n| n == name),
            "'{name}' não disparou: {all:?}"
        );
    }
    assert!(
        !all.iter().any(|n| n == "quiet"),
        "o CONTROLE gritou -- a cena nao distingue o sinal de tudo disparar"
    );
}

/// **UMA vez, não uma por quadro** — a bola do sino fica encostada nele, e o
/// canal reporta a CHEGADA e não o estado.
#[test]
fn each_arrival_shouts_once_not_once_per_frame() {
    let all = fired(4.0);
    let n = all.iter().filter(|s| *s == "bell").count();
    assert_eq!(
        n, 1,
        "o sino gritou {n} vezes em 4 s -- um som por quadro nao e' um som"
    );
}
