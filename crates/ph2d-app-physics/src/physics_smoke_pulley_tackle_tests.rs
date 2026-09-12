//! A sonda da cena 61 + os gates que mantêm a mensagem dela honesta e a
//! montagem autorável pela UI (W-Pulley W3).

use super::*;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

/// A sonda da cena 61 — a talha.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_61 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_61() {
    let mut sim = SimWorld::new();
    build_tackle(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Tackle", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Block")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 61 — a talha (2 s) ===");
    println!(
        "{:>8} | {:>12} | {:>12}",
        "rig", "bloco andou", "contrapeso"
    );
    for (i, tag) in ["Tackle", "Plain"].iter().enumerate() {
        println!(
            "{tag:>8} | {:>12.3} | {:>12.3}",
            y_of(&mut sim, &format!("{tag} Block")) - start[i],
            y_of(&mut sim, &format!("{tag} Counterweight")),
        );
    }
    // O eixo da roldana montada tem de estar EM CIMA do bloco, não onde ele
    // nasceu: é a metade geométrica da wave, e ela se vê no desenho.
    let block = y_of(&mut sim, "Tackle Block");
    let wheel = y_of(&mut sim, "Tackle Rope Wheel 1");
    println!(
        "  eixo montado: y={wheel:.3} · bloco y={block:.3} · delta {:.4}",
        wheel - block
    );
}

/// **A cena 61 diz a verdade** — os dois números da mensagem, medidos pelo
/// caminho do produto.
///
/// O oráculo é a DIFERENÇA entre os dois rigs, e não um limiar por rig: a carga e
/// o contrapeso são os mesmos nos dois, então a única coisa que pode explicar um
/// ficar e o outro cair é quantos ramos seguram o bloco.
#[test]
fn the_tackle_scene_says_what_happens() {
    let mut sim = SimWorld::new();
    build_tackle(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Tackle", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Block")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let tackle = y_of(&mut sim, "Tackle Block") - start[0];
    let plain = y_of(&mut sim, "Plain Block") - start[1];
    /// A folga sobre os números que a mensagem diz — a sim é determinística, então
    /// ela existe só para o gate não ser um fingerprint que qualquer afinação de
    /// solver quebra.
    const SLACK: f32 = 0.05;
    assert!(
        tackle.abs() < MEASURED_TACKLE_DRIFT + SLACK,
        "a mensagem diz que a talha SEGURA ({MEASURED_TACKLE_DRIFT:.2} m); o bloco \
         andou {tackle:.3} m"
    );
    assert!(
        plain < -(MEASURED_PLAIN_DROP - 0.5),
        "a mensagem diz que o controle CAI {MEASURED_PLAIN_DROP:.1} m; ele andou {plain:.3} m"
    );
    // E o eixo montado segue o bloco — a metade que o desenho mostra.
    let d = y_of(&mut sim, "Tackle Rope Wheel 1") - y_of(&mut sim, "Tackle Block");
    assert!(
        d.abs() < 0.05,
        "o eixo montado ficou {d:.3} m longe do bloco que o carrega"
    );
}

/// **A cena 61 oferece o ímã que a mensagem dela promete** (W6) — e o CONTROLE
/// da mesma cena continua sem nada a que colar.
///
/// ⚠️ **A cena é o único lugar onde o número da mensagem pode ser conferido.** Os
/// gates do kernel (`ph2d-physics-ecs::pulley_wheel_snap`) medem uma CAIXA, que
/// oferece nove; aqui o bloco é um DISCO e oferece cinco, então uma mensagem
/// escrita a partir do gate errado diria ao artista para procurar quatro pontos
/// que não existem — a classe de erro que esta linha já pagou duas vezes escrevendo
/// cena antes de medir.
///
/// A metade do CENÁRIO é a que impede a wave de ter *perdido* uma recusa: a
/// roldana de cima não pertence a corpo nenhum, e o ímã não pode abrir nela.
#[test]
fn the_tackle_scene_offers_the_magnet_it_promises_and_only_where_it_should() {
    let mut sim = SimWorld::new();
    build_tackle(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);

    let mounted = entity_of(&mut sim, "Tackle Rope Wheel 1");
    let mut out = [[0.0f32; 2]; ph2d_physics_ecs::ShapeDesc::MAX_SNAP_POINTS];
    let n = bridge.wheel_snap_targets(&sim, mounted, &mut out);
    assert_eq!(
        n, MEASURED_SNAP_TARGETS,
        "a mensagem diz {MEASURED_SNAP_TARGETS} pontos e a cena ofereceu {n} — o \
         bloco é um DISCO (centro + 4 cardinais), nunca uma caixa"
    );
    // O centro do disco é o primeiro candidato, e é o único ponto que um artista
    // não acerta a olho: ele não está no contorno.
    let block = y_of(&mut sim, "Tackle Block");
    assert!(
        (out[0][1] - block).abs() < 1.0e-4,
        "o 1º candidato tinha de ser o CENTRO do bloco (y={block:.3}); saiu {:?}",
        out[0]
    );

    // O CONTROLE: a roldana FIXA é do cenário — zero candidatos.
    let scenery = entity_of(&mut sim, "Tackle Rope Wheel 2");
    assert_eq!(
        bridge.wheel_snap_targets(&sim, scenery, &mut out),
        0,
        "a roldana do CENÁRIO não pertence a corpo nenhum: o ímã não pode abrir nela"
    );
}
