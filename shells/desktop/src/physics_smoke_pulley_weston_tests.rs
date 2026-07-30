//! A sonda da cena 64 + o gate que mantém a mensagem dela honesta (W-Weston).

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

/// A sonda da cena 64 — as MESMAS duas circunferências, um chip de diferença.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_64 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_64() {
    let mut sim = SimWorld::new();
    build_weston(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Weston", "Drum"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 64 — a talha de Weston (2 s) ===");
    println!(
        "R = {R_IN}, r = {R_RET} · weston 2R/(R-r) = {:.2} · tambor 2R/r = {:.2}",
        2.0 * R_IN / (R_IN - R_RET),
        2.0 * R_IN / R_RET
    );
    println!(
        "{:>8} | {:>12} {:>10} | {:>12} {:>10}",
        "rig", "carga andou", "carga y", "contra andou", "contra y"
    );
    for (i, tag) in ["Weston", "Drum"].iter().enumerate() {
        let ly = y_of(&mut sim, &format!("{tag} Load"));
        let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
        println!(
            "{tag:>8} | {:>12.3} {ly:>10.3} | {:>12.3} {cy:>10.3}",
            ly - start[i],
            cy - COUNTER_Y
        );
    }
}

/// **A mensagem da cena 64 não pode PROMETER o que a cena não faz.**
///
/// Os dois rigs têm de andar para lados OPOSTOS — é a única coisa que a mensagem
/// afirma e que um bug poderia derrubar em silêncio. Barras generosas de propósito: o
/// que se gateia é o SINAL (a wave), não a terceira decimal (que o smoke lê).
///
/// ⚠️ **Duas cenas desta linha já afirmaram números que a medição desmentiu**, e é
/// por isso que este gate existe ao lado da sonda em vez de a mensagem confiar nela.
#[test]
fn the_two_rigs_of_scene_64_move_to_opposite_sides() {
    let mut sim = SimWorld::new();
    build_weston(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let w0 = y_of(&mut sim, "Weston Load");
    let d0 = y_of(&mut sim, "Drum Load");
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let (w, d) = (
        y_of(&mut sim, "Weston Load") - w0,
        y_of(&mut sim, "Drum Load") - d0,
    );
    assert!(
        w > 0.05,
        "a carga da WESTON tinha de SUBIR (vantagem {:.1}); andou {w:.4}",
        2.0 * R_IN / (R_IN - R_RET)
    );
    assert!(
        d < -0.2,
        "a carga do TAMBOR tinha de CAIR (vantagem {:.2}); andou {d:.4}",
        2.0 * R_IN / R_RET
    );
    // E os números que a mensagem imprime batem com o que a cena faz.
    assert!(
        (w - MEASURED_WESTON_RISE).abs() < 0.05,
        "a mensagem promete +{MEASURED_WESTON_RISE:.2} m e a cena deu {w:.4}"
    );
    assert!(
        (-d - MEASURED_DRUM_DROP).abs() < 0.1,
        "a mensagem promete -{MEASURED_DRUM_DROP:.2} m e a cena deu {d:.4}"
    );
    let cd = COUNTER_Y - y_of(&mut sim, "Weston Counterweight");
    assert!(
        (cd - MEASURED_WESTON_COUNTER_DROP).abs() < 0.2,
        "a mensagem promete o contrapeso descendo {MEASURED_WESTON_COUNTER_DROP:.2} m; \
         deu {cd:.4}"
    );
}

/// **O chip é a ÚNICA diferença entre os dois rigs.**
///
/// A mensagem afirma *"as MESMAS duas circunferências"*, e essa é a frase que faz a
/// demonstração valer algo: se um dos rigs tivesse raios diferentes, a wave estaria
/// sendo mostrada por um número em vez de por uma topologia.
///
/// ⚠️ Mutação: dar ao rig do tambor um `radius_out` menor (o jeito "óbvio" de fazer
/// as duas cargas subirem) deixa os gates de movimento verdes e **destrói a
/// demonstração** — os dois rigs deixariam de ser comparáveis.
#[test]
fn the_chip_is_the_only_difference_between_the_two_rigs() {
    let mut sim = SimWorld::new();
    build_weston(sim.world_mut());
    let mut q = sim
        .world_mut()
        .query::<(&Name, &ph2d_physics_ecs::PulleyWheel)>();
    let axles: Vec<_> = q
        .iter(sim.world())
        .filter(|(n, _)| n.as_str().ends_with("Rope Axle"))
        .map(|(n, w)| (n.as_str().to_string(), w.radius, w.radius_out))
        .collect();
    assert_eq!(axles.len(), 2, "dois eixos, um por rig");
    assert_eq!(
        axles[0].1, axles[1].1,
        "o raio de ENTRADA é o mesmo nos dois"
    );
    assert_eq!(
        axles[0].2, axles[1].2,
        "e o de RETORNO também — o chip é a única diferença"
    );
    let mut wq = sim.world_mut().query::<&ph2d_physics_ecs::WestonAxle>();
    assert_eq!(
        wq.iter(sim.world()).count(),
        1,
        "e exatamente UM dos dois carrega o marcador"
    );
}
