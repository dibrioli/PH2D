//! **QUANTO o personagem de fato SOBE** — a sonda que responde ao report do
//! Enio (*"Max Slope na UI diz 45, mas o player sobe ate' aproximadamente
//! 60graus"*).
//!
//! ⚠️ Sonda, não gate: ela **imprime** a varredura e não afirma nada. O número
//! que ela produz é o que decide se existe defeito e de que tamanho — a regra
//! do §0 (medir antes de escrever qualquer limite, e antes de qualquer cura).
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --test measure_max_slope -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_physics_ecs::{PlatformPlayer, PlayerInput};
use scene_fixture::{pose, scene};

/// Sobe (ou desce) quantos metros em 3 s, empurrando rampa acima com o limite
/// de rampa em `limit_deg`.
fn climb(ramp_deg: f32, limit_deg: f32) -> f32 {
    climb_with_air(ramp_deg, limit_deg, None)
}

/// A mesma coisa, com o **controle aéreo** ablacionado — a ablação por ENTRADA
/// que atribui (ou não) a subida ao empurrão horizontal do modo-ar.
fn climb_with_air(ramp_deg: f32, limit_deg: f32, air: Option<f32>) -> f32 {
    let slope = ramp_deg.to_radians();
    let (mut sim, mut bridge, player) = scene(slope, 0.0);
    {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        p.max_slope_deg = limit_deg;
        if let Some(a) = air {
            p.air_acceleration = a;
        }
    }
    // Meio segundo parado para a perna assentar antes de empurrar.
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, y0) = pose(&sim);
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    for tick in 31..=210 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let (_, y1) = pose(&sim);
    y1 - y0
}

#[test]
#[ignore = "sonda de medicao"]
fn measure_what_the_player_actually_climbs() {
    eprintln!("=== O QUE O PLAYER DE FATO SOBE (limite autorado = 45 deg) ===");
    eprintln!("  rampa | subiu em 3 s | veredito");
    for deg in [20, 30, 40, 44, 46, 50, 55, 60, 63, 65, 70] {
        let d = climb(deg as f32, 45.0);
        let verdict = if d > 0.25 {
            "SOBE"
        } else if d < -0.25 {
            "escorrega"
        } else {
            "fica"
        };
        eprintln!("  {deg:>3}deg | {d:+8.3} m | {verdict}");
    }

    eprintln!();
    eprintln!("=== CONTROLE: o limite MOVE a fronteira? (rampa fixa em 55 deg) ===");
    for limit in [10, 30, 45, 54, 56, 70, 89] {
        let d = climb(55.0, limit as f32);
        eprintln!("  limite {limit:>3}deg | {d:+8.3} m");
    }

    eprintln!();
    eprintln!("=== ATRIBUICAO: ablacionar o CONTROLE AEREO (limite 45) ===");
    eprintln!("  rampa | air=20 (o default) | air=0 | air=5");
    for deg in [46, 48, 50, 52, 54] {
        let full = climb_with_air(deg as f32, 45.0, None);
        let none = climb_with_air(deg as f32, 45.0, Some(0.0));
        let some = climb_with_air(deg as f32, 45.0, Some(5.0));
        eprintln!("  {deg:>3}deg | {full:+8.3} m | {none:+8.3} m | {some:+8.3} m");
    }
}
