//! A sonda da cena 58 + o gate que mantém a mensagem honesta.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

fn run(ticks: u64) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    for t in 1..=ticks {
        bridge.dispatch(&mut sim, false, t);
    }
    (sim, bridge)
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

/// A sonda: roda a cena e imprime o que ela de fato faz.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_58 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_58() {
    let (mut sim, _) = run(180);
    println!("\n=== CENA 58 — o elevador (3 s) ===");
    for tag in ["Simple", "Tackle"] {
        let load = START_Y - y_of(&mut sim, &format!("{tag} Load"));
        let cw = y_of(&mut sim, &format!("{tag} Counterweight")) - START_Y;
        println!("{tag:>8}: carga desceu {load:>7.4} m | contrapeso subiu {cw:>7.4} m");
    }
}

/// **A mensagem afirma os números que a simulação produz.**
///
/// O molde do irmão da cena 57: uma cena que diz *"a talha ergue a carga"* e uma
/// simulação que a deixa cair é uma demonstração que ensina o oposto do que a
/// wave construiu — e nada além deste gate reconferiria isso.
#[test]
fn the_scene_message_states_the_numbers_the_sim_produces() {
    let (mut sim, _) = run(180);
    let simple_drop = START_Y - y_of(&mut sim, "Simple Load");
    let simple_rise = y_of(&mut sim, "Simple Counterweight") - START_Y;
    let tackle_drop = START_Y - y_of(&mut sim, "Tackle Load");

    for (got, said, what) in [
        (
            simple_drop,
            MEASURED_SIMPLE_LOAD_DROP,
            "queda da carga simples",
        ),
        (simple_rise, MEASURED_SIMPLE_CW_RISE, "subida do contrapeso"),
        (
            tackle_drop,
            MEASURED_TACKLE_LOAD_DROP,
            "queda da carga na talha",
        ),
    ] {
        assert!(
            (got - said).abs() < 0.05,
            "{what}: a mensagem diz {said:.2} m e a sim faz {got:.4} m"
        );
    }

    // **A corda é inextensível**, e é isso que o par de números do rig simples
    // diz junto — um afirmando 2,5 m sem o outro descreveria uma queda livre.
    assert!(
        (simple_drop - simple_rise).abs() < 0.05,
        "razão 1: desceu {simple_drop:.4} e subiu {simple_rise:.4}"
    );

    // **E a TALHA inverte quem ganha.** Mesmo par de massas nos dois rigs: com
    // razão 0,25 o contrapeso mais leve ergue a carga mais pesada, que é a
    // vantagem mecânica e o motivo de a razão existir.
    assert!(
        simple_drop > 0.5 && tackle_drop < -0.5,
        "a razão tem de inverter o resultado: {simple_drop:.4} contra {tackle_drop:.4}"
    );

    // **E o PREÇO da vantagem é a distância**: o lado leve anda `1/r` = 4 vezes
    // o que a carga anda. Sem esta metade a cena venderia energia de graça.
    let tackle_cw_drop = START_Y - y_of(&mut sim, "Tackle Counterweight");
    assert!(
        (tackle_cw_drop - MEASURED_TACKLE_CW_DROP).abs() < 0.05,
        "queda do contrapeso da talha: a mensagem diz {MEASURED_TACKLE_CW_DROP:.2} e a sim faz {tackle_cw_drop:.4}"
    );
    let travel_ratio = tackle_cw_drop / -tackle_drop;
    assert!(
        (travel_ratio - 4.0).abs() < 0.15,
        "o lado leve tem de andar 1/0,25 = 4x: mediu {travel_ratio:.3}"
    );
}
