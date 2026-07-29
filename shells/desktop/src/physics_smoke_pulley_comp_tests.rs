//! A sonda da cena 63 + o gate que mantém a mensagem dela honesta (W-Pulley W5).

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

/// A sonda da cena 63 — o tambor e a cadernal na mesma corda.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_63 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_63() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Geared", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 63 — a composicao (2 s) ===");
    println!(
        "{:>8} | {:>12} {:>10} | {:>12} {:>10}",
        "rig", "carga andou", "carga y", "contra andou", "contra y"
    );
    for (i, tag) in ["Geared", "Plain"].iter().enumerate() {
        let ly = y_of(&mut sim, &format!("{tag} Load"));
        let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
        println!(
            "{tag:>8} | {:>12.3} {ly:>10.3} | {:>12.3} {cy:>10.3}",
            ly - start[i],
            cy - COUNTER_Y
        );
    }
}

/// **A cena 63 diz a verdade** — os dois números da mensagem, medidos pelo
/// caminho do produto.
///
/// O oráculo é a DIFERENÇA entre os dois rigs: a carga e o contrapeso são os
/// mesmos nos dois e os dois têm cadernal móvel, então a única coisa que pode
/// explicar um subir e o outro cair é o segundo diâmetro do tambor MULTIPLICANDO
/// a vantagem que a cadernal já dava.
#[test]
fn the_composition_scene_says_what_happens() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Geared", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let geared = y_of(&mut sim, "Geared Load") - start[0];
    let plain = y_of(&mut sim, "Plain Load") - start[1];
    /// Folga sobre os números da mensagem — a sim é determinística, então ela
    /// existe só para o gate não ser um fingerprint que qualquer afinação de
    /// solver quebra.
    const SLACK: f32 = 0.3;
    // O SINAL antes da magnitude: as duas cargas são iguais e as duas cadernais
    // também, então andarem para lados opostos é o que só a engrenagem explica.
    assert!(
        geared > 0.0 && plain < 0.0,
        "as MESMAS massas, nas MESMAS talhas, tinham de andar para lados opostos; \
         engrenada {geared:.3} m e comum {plain:.3} m"
    );
    assert!(
        (geared - MEASURED_GEARED_RISE).abs() < SLACK,
        "a mensagem diz que a carga engrenada SOBE {MEASURED_GEARED_RISE:.2} m; ela \
         andou {geared:.3} m"
    );
    assert!(
        (plain + MEASURED_PLAIN_DROP).abs() < SLACK,
        "a mensagem diz que o controle CAI {MEASURED_PLAIN_DROP:.2} m ate o chao; ele \
         andou {plain:.3} m"
    );
    // ⚠️ **A RAZÃO é o oráculo mais forte da cena**, e ela não depende de nenhuma
    // massa: a corda é inextensível, então o lado do contrapeso TEM de andar
    // `2·(R/r)` vezes o que a carga anda, seja qual for a força que a move. Uma
    // afinação de solver move os dois números juntos e deixa a razão de pé.
    let counter = y_of(&mut sim, "Geared Counterweight") - COUNTER_Y;
    let ratio = -counter / geared;
    let mech = 2.0 * R_IN / R_OUT;
    assert!(
        (ratio - mech).abs() < 0.5,
        "a corda e inextensivel: o contrapeso tinha de andar {mech:.1}x o que a \
         carga anda. Carga {geared:.3} m, contrapeso {counter:.3} m, razao \
         {ratio:.2}"
    );
}

/// **O contrapeso do rig engrenado NUNCA alcança o tambor** — o degenerado que o
/// W1 nomeou (um corpo que passa da própria roldana não tem tangente), e que já
/// derrubou a primeira versão da cena 62.
///
/// Ele é o gate que protege a cena de si mesma: a vantagem 8 faz o contrapeso
/// andar **oito vezes** o que a carga anda, então a pista dele é o recurso escasso
/// desta montagem, não uma folga qualquer.
#[test]
fn the_counterweight_never_reaches_its_drum() {
    let mut sim = SimWorld::new();
    build_composition(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut lowest = f32::INFINITY;
    let mut highest = f32::NEG_INFINITY;
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
        for tag in ["Geared", "Plain"] {
            let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
            lowest = lowest.min(cy);
            highest = highest.max(cy);
        }
    }
    // ⚠️ **A barra é o CÍRCULO do tambor, não o centro dele.** A rota tangencia
    // a circunferência, então a âncora entra em degeneração ao cruzar
    // `DRUM_Y − R_IN` — usar o raio do CORPO aqui mediria a coisa errada por
    // 0,25 m e deixaria passar exatamente a falha que este gate existe para
    // pegar.
    assert!(
        highest < DRUM_Y - R_IN,
        "algum contrapeso chegou a {highest:.2}, contra o circulo do tambor que \
         comeca em {:.2}: a rota degenera e a cena passa a mostrar outra coisa",
        DRUM_Y - R_IN
    );
    assert!(
        lowest > 0.0,
        "algum contrapeso desceu a {lowest:.2} e atravessou o chao — a cena tem de \
         caber na pista que ela mesma da"
    );
}
