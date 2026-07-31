//! A sonda da cena 66 + o gate que mantém a mensagem dela honesta (W-JointCopy).

use super::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PhysicsJoint};

/// O ângulo do portão, **graus**, depois de `ticks` da sim.
fn angle_deg(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.rotation.to_degrees())
        .expect("portão vivo")
}

/// Roda a cena por 2 s e devolve, por portão, a **maior excursão** em graus.
///
/// ⚠️ **O máximo sobre a TRAJETÓRIA, nunca o ângulo do último tick.** Um portão
/// sem batente é um PÊNDULO: ele desce, sobe do outro lado e volta, então o
/// instante `t = 2 s` é um ponto arbitrário de um ciclo — a primeira versão
/// desta sonda mediu ali e reportou **38,1°** para os três livres, um número
/// que não descreve nada e que teria virado a constante da mensagem. É
/// exatamente a fixture que o W3 já pagou uma vez (*"o pêndulo é PERIÓDICO …
/// tudo virou TRAJETÓRIA"*), e ela reincidiu aqui.
fn run(paste_onto_the_rest: bool) -> [f32; 4] {
    let mut sim = SimWorld::new();
    build_joint_copy(sim.world_mut());
    if paste_onto_the_rest {
        // O paste, pela PORTA — a mesma que o botão da §12 usa. Um paste
        // simulado com um `limits_enabled = true` escrito à mão provaria que a
        // física respeita batentes, que ninguém duvidava.
        let source = *joint_named(&mut sim, "Pin A");
        for pin in ["Pin B", "Pin C", "Pin D"] {
            let e = entity_named(&mut sim, pin);
            let current = *sim.world().get::<PhysicsJoint>(e).expect("pino");
            let next = current.with_properties_of(&source).clamped();
            *sim.world_mut().get_mut::<PhysicsJoint>(e).expect("pino") = next;
        }
    }
    let mut bridge = PhysicsBridge::new();
    let mut out = [0.0f32; 4];
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
        for (i, name) in NAMES.iter().enumerate() {
            out[i] = out[i].max(angle_deg(&mut sim, name).abs());
        }
    }
    out
}

fn entity_named(sim: &mut SimWorld, name: &str) -> ph2d_ecs::Entity {
    let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn joint_named<'w>(sim: &'w mut SimWorld, name: &str) -> &'w PhysicsJoint {
    let e = entity_named(sim, name);
    sim.world().get::<PhysicsJoint>(e).expect("joint")
}

/// A sonda da cena 66 — os quatro portões, antes e depois da colagem.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_66 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_66() {
    let before = run(false);
    let after = run(true);
    println!("\n=== CENA 66 — copiar/colar propriedades de joint (2 s) ===");
    println!(
        "{:>8} | {:>18} | {:>18}",
        "portao", "sem colar (max deg)", "colado (max deg)"
    );
    for (i, name) in NAMES.iter().enumerate() {
        println!("{name:>8} | {:>18.1} | {:>18.1}", before[i], after[i]);
    }
    println!(
        "\nbatente autorado: +/-{TUNED_LIMIT_DEG:.0} deg — o afinado tem de parar nele, \
         e os tres tem de passar dele ate colarem"
    );
}

/// **A cena demonstra o que a mensagem afirma.**
///
/// ⚠️ O oráculo é a RELAÇÃO entre os portões, não um número absoluto: o que a
/// wave promete é *"colar iguala"*, e um bar absoluto ficaria verde numa cena em
/// que os quatro congelassem por outro motivo. Por isso três asserções — o
/// afinado obedece ao batente, os crus PASSAM dele, e depois da colagem os
/// quatro cabem na mesma faixa.
#[test]
fn scene_66_shows_the_paste_making_four_gates_alike() {
    let before = run(false);
    let after = run(true);
    // Folga sobre o batente: o solver o alcança por restrição, não por igualdade.
    let bar = TUNED_LIMIT_DEG + 3.0;

    assert!(
        before[0] < bar,
        "o portao AFINADO passou do proprio batente ({:.1} deg > {bar:.1}): a cena \
         nao demonstra nada se o controle nao obedece",
        before[0]
    );
    for i in 1..4 {
        assert!(
            before[i] > bar * 2.0,
            "o portao {} parou em {:.1} deg sem batente nenhum — a cena precisa \
             que os tres CAIAM, senao a colagem nao muda nada visivel",
            NAMES[i],
            before[i]
        );
    }
    for i in 0..4 {
        assert!(
            after[i] < bar,
            "depois da colagem o portao {} ainda gira {:.1} deg — o paste nao \
             levou os batentes",
            NAMES[i],
            after[i]
        );
    }
}

/// **E os números da mensagem são os que a sonda mede.**
///
/// Uma cena que afirma graus que ninguém mediu é a forma exata de mentir devagar
/// — esta linha já corrigiu dois palpites meus nesta wave.
#[test]
fn the_message_of_scene_66_quotes_measured_numbers() {
    let before = run(false);
    let after = run(true);
    let close = |a: f32, b: f32, what: &str| {
        assert!(
            (a - b).abs() < 1.5,
            "a mensagem diz {b:.1} deg para {what} e a sonda mede {a:.1}"
        );
    };
    close(before[0], MEASURED_TUNED_DEG, "o portao afinado");
    close(before[1], MEASURED_PLAIN_DEG, "um portao cru");
    close(after[1], MEASURED_PASTED_DEG, "um portao depois da colagem");
}
