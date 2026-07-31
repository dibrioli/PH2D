//! A sonda da cena 65 + o gate que mantém a mensagem dela honesta (W-JointWorld).

use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn pos_of(sim: &mut SimWorld, name: &str) -> [f32; 2] {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| [t.translation.x, t.translation.y])
        .expect("corpo vivo")
}

/// A sonda da cena 65 — os dois pêndulos, lado a lado.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_65 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_65() {
    let mut sim = SimWorld::new();
    build_world_pin(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start = [pos_of(&mut sim, "Old Bob"), pos_of(&mut sim, "New Bob")];
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 65 — o pino de mundo (2 s) ===");
    println!(
        "{:>10} | {:>10} {:>10} | {:>10}",
        "rig", "x", "y", "percurso"
    );
    for (i, name) in ["Old Bob", "New Bob"].iter().enumerate() {
        let p = pos_of(&mut sim, name);
        let d = ((p[0] - start[i][0]).powi(2) + (p[1] - start[i][1]).powi(2)).sqrt();
        println!("{name:>10} | {:>10.4} {:>10.4} | {d:>10.4}", p[0], p[1]);
    }
}

/// **Os dois pêndulos fazem a MESMA coisa** — é a afirmação inteira da cena.
///
/// ⚠️ O gate compara os dois **entre si**, e não cada um contra um número: o que
/// a wave promete não é *"o corpo fica em tal lugar"*, é *"o pino de mundo é um
/// pivô de verdade"*. Um oráculo absoluto ficaria verde se AMBOS congelassem.
/// Por isso a segunda metade exige que eles tenham **se movido**.
#[test]
fn the_two_pendulums_of_scene_65_swing_alike() {
    let mut sim = SimWorld::new();
    build_world_pin(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start = [pos_of(&mut sim, "Old Bob"), pos_of(&mut sim, "New Bob")];
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let old = pos_of(&mut sim, "Old Bob");
    let new = pos_of(&mut sim, "New Bob");
    // Mesma pose RELATIVA à própria âncora (os rigs estão a 6 m um do outro).
    let rel_old = [old[0] - LEFT_X, old[1] - ANCHOR_Y];
    let rel_new = [new[0] - RIGHT_X, new[1] - ANCHOR_Y];
    assert!(
        (rel_old[0] - rel_new[0]).abs() < 0.05 && (rel_old[1] - rel_new[1]).abs() < 0.05,
        "os dois pêndulos tinham de estar na MESMA pose relativa; \
         antigo {rel_old:?} contra novo {rel_new:?}"
    );
    // E os dois se MOVERAM — senão o gate acima seria verde sobre dois corpos
    // congelados, que é exatamente o modo de falha do pino de mundo.
    let travelled = ((new[0] - start[1][0]).powi(2) + (new[1] - start[1][1]).powi(2)).sqrt();
    assert!(
        travelled > 0.3,
        "o pêndulo do pino de mundo mal se moveu ({travelled:.4} m) — ele está \
         SOLDADO, não pendurado"
    );
    assert!(
        (travelled - MEASURED_SWING).abs() < 0.05,
        "a mensagem promete {MEASURED_SWING:.3} m de percurso e a cena deu {travelled:.4}"
    );
}

/// **O rig NOVO não tem objeto a mais, e o VELHO tem** — a wave inteira numa
/// contagem.
#[test]
fn the_new_rig_costs_one_fewer_object_than_the_old_one() {
    let mut sim = SimWorld::new();
    build_world_pin(sim.world_mut());
    let mut q = sim.world_mut().query::<&Name>();
    let names: Vec<String> = q
        .iter(sim.world())
        .map(|n| n.as_str().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n == "Invented Hook"),
        "o rig ANTIGO tem de carregar o gancho inventado — ele é o CONTROLE, e \
         sem ele a cena não mostra o que a wave remove"
    );
    let new_side = names
        .iter()
        .filter(|n| n.as_str() == "New Bob" || n.as_str() == "Wall Pin")
        .count();
    assert_eq!(
        new_side, 2,
        "o rig NOVO tem de ser exatamente o corpo e o pino — nada mais"
    );
}
