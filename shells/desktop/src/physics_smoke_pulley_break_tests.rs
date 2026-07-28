//! A sonda da cena 60 + o gate que mantém a mensagem dela honesta.

use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

/// A sonda da cena 60 — a ruptura.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_60 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_60() {
    let mut sim = SimWorld::new();
    build_break(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Snap", "Axle", "Holds"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    let mut events = Vec::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
        for b in bridge.joint_breaks() {
            events.push((t, b.force));
        }
    }
    println!("\n=== CENA 60 — a ruptura (2 s) ===");
    println!("{:>8} | {:>12}", "rig", "andou (m)");
    for (i, tag) in ["Snap", "Axle", "Holds"].iter().enumerate() {
        println!(
            "{tag:>8} | {:>12.3}",
            y_of(&mut sim, &format!("{tag} Load")) - start[i]
        );
    }
    // ⚠️ UMA vez: a arena é de TODAS as cordas, não de um rig. Imprimi-la por
    // linha diria o mesmo número três vezes fingindo ser três medidas.
    println!(
        "  roldanas ainda na rota: {} (nasceram 3)",
        bridge.pulley_wheel_arena().len()
    );
    for (t, f) in &events {
        println!("  rompeu no tique {t} carregando {f:.2} N");
    }
}

/// **A cena 60 diz os números que a sim produz** — o terceiro irmão.
///
/// ⚠️ O que este gate protege é a AFIRMAÇÃO CENTRAL da cena: que o tranco é
/// ordens de grandeza acima do peso. Se ela deixasse de valer, a mensagem
/// estaria ensinando ao artista uma intuição errada sobre break force — e essa
/// é exatamente a classe de erro que uma cena de smoke não pode ter, porque
/// ninguém confere um número numa screenshot.
#[test]
fn the_break_scene_states_the_numbers_the_sim_produces() {
    let mut sim = SimWorld::new();
    build_break(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Holds", "Snap", "Axle"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    let mut loads = Vec::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
        loads.extend(bridge.joint_breaks().iter().map(|b| b.force));
    }
    let moved = |sim: &mut SimWorld, i: usize, tag: &str| {
        (y_of(sim, &format!("{tag} Load")) - start[i]).abs()
    };
    assert!(
        moved(&mut sim, 0, "Holds") < MEASURED_HOLDS_DRIFT + 0.02,
        "o rig VERDE tinha de segurar e andou {:.4} m",
        moved(&mut sim, 0, "Holds")
    );
    for (i, tag) in [(1, "Snap"), (2, "Axle")] {
        assert!(
            (moved(&mut sim, i, tag) - MEASURED_FALL).abs() < 0.1,
            "{tag} caiu {:.4} m e a mensagem diz {MEASURED_FALL:.2}",
            moved(&mut sim, i, tag)
        );
    }
    assert_eq!(
        loads.len(),
        2,
        "duas coisas tinham de ceder, não {}",
        loads.len()
    );
    // A afirmação central: o tranco é ordens de grandeza acima do peso parado.
    let jerk = loads.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        (jerk / MEASURED_JERK - 1.0).abs() < 0.1,
        "o tranco foi {jerk:.0} N e a mensagem diz {MEASURED_JERK:.0}"
    );
    assert!(
        jerk > 50.0 * 29.4,
        "a cena afirma que o tranco é 177x o peso; medido {:.0}x",
        jerk / 29.4
    );
    // E a carga do EIXO, que é a outra grandeza.
    let axle = loads.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        (axle - MEASURED_AXLE_LOAD).abs() < 1.0,
        "o eixo cedeu carregando {axle:.1} N e a mensagem diz {MEASURED_AXLE_LOAD:.1}"
    );
    // A roldana saiu da rota, e só ela.
    assert_eq!(bridge.pulley_wheel_arena().len(), 2, "nasceram 3 roldanas");
}
