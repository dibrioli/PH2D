//! A sonda da cena 62 + os gates que mantêm a mensagem dela honesta e o segundo
//! diâmetro autorável pela UI (W-Pulley W4).

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

/// A sonda da cena 62 — o tambor diferencial.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_62 -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_62() {
    let mut sim = SimWorld::new();
    build_differential(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Diff", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    println!("\n=== CENA 62 — o tambor diferencial (2 s) ===");
    println!(
        "{:>8} | {:>12} {:>10} | {:>12} {:>10}",
        "rig", "carga andou", "carga y", "contra andou", "contra y"
    );
    for (i, tag) in ["Diff", "Plain"].iter().enumerate() {
        let ly = y_of(&mut sim, &format!("{tag} Load"));
        let cy = y_of(&mut sim, &format!("{tag} Counterweight"));
        println!(
            "{tag:>8} | {:>12.3} {ly:>10.3} | {:>12.3} {cy:>10.3}",
            ly - start[i],
            cy - HANG_Y
        );
    }
}

/// **A cena 62 diz a verdade** — os dois números da mensagem, medidos pelo
/// caminho do produto.
///
/// O oráculo é a DIFERENÇA entre os dois rigs: a carga e o contrapeso são os
/// mesmos nos dois, então a única coisa que pode explicar um segurar e o outro
/// cair é o segundo diâmetro do tambor.
#[test]
fn the_differential_scene_says_what_happens() {
    let mut sim = SimWorld::new();
    build_differential(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let start: Vec<f32> = ["Diff", "Plain"]
        .iter()
        .map(|t| y_of(&mut sim, &format!("{t} Load")))
        .collect();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, false, t);
    }
    let geared = y_of(&mut sim, "Diff Load") - start[0];
    let plain = y_of(&mut sim, "Plain Load") - start[1];
    /// Folga sobre os números da mensagem — a sim é determinística, então ela
    /// existe só para o gate não ser um fingerprint que qualquer afinação de
    /// solver quebra.
    const SLACK: f32 = 0.3;
    // O oráculo é o SINAL antes da magnitude: as duas cargas são iguais, então
    // andarem para lados opostos é o que só o segundo diâmetro explica.
    assert!(
        geared > 0.0 && plain < 0.0,
        "as MESMAS massas tinham de andar para lados opostos; engrenada {geared:.3} m \
         e comum {plain:.3} m"
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
}

/// **O segundo diâmetro é autorável, e digitar nele MUDA a cena** — a quarta
/// condição da política de UI do plano: as outras três podem estar verdes com a
/// sequência não levando a lugar nenhum.
#[test]
fn typing_an_out_radius_turns_a_plain_wheel_into_a_drum() {
    let travel = |out: f32| {
        let mut sim = SimWorld::new();
        build_differential(sim.world_mut());
        let drum = entity_of(&mut sim, "Plain Rope Drum");
        // A porta pura que o painel alcança — a mesma que a row do Inspector usa.
        let current = *sim
            .world()
            .get::<ph2d_physics_ecs::PulleyWheel>(drum)
            .expect("o tambor existe");
        let next = crate::render_loop::inspector_joint_wheel::wheel_with_edit(
            current,
            ph2d_editor::WheelFieldEdit::RadiusOut(out),
        )
        .expect("editar o raio de saída é uma escrita de componente");
        *sim.world_mut()
            .get_mut::<ph2d_physics_ecs::PulleyWheel>(drum)
            .expect("o tambor existe") = next;
        let mut bridge = PhysicsBridge::new();
        let y0 = y_of(&mut sim, "Plain Load");
        for t in 1..=120 {
            bridge.dispatch(&mut sim, false, t);
        }
        y_of(&mut sim, "Plain Load") - y0
    };
    let plain = travel(0.0);
    assert!(
        plain < -1.0,
        "com 0 no raio de saída a roldana é COMUM e a carga tinha de cair; ela \
         andou {plain:.3} m — a fixture não contém o fenômeno"
    );
    let geared = travel(R_OUT);
    assert!(
        geared > plain + 1.0,
        "digitar {R_OUT} no raio de saída faz do MESMO tambor um diferencial de \
         {:.0}x, e a mesma carga tinha de parar de cair; ela andou {geared:.3} m \
         contra {plain:.3}",
        R_IN / R_OUT
    );
}

/// **A cena 62 cabe no quadro que ela define** — irmão do gate da cena 63, pela
/// MESMA porta.
///
/// Ela é o caso mais agudo da família: a mensagem manda *selecionar o tambor* e
/// *olhar o segundo anel aparecer*, e com a câmera padrão ele nascia 7 m acima do
/// topo da tela.
#[test]
fn the_scene_fits_the_frame_it_sets() {
    let mut sim = SimWorld::new();
    build_differential(sim.world_mut());
    let worst = crate::physics_smoke_pulley::outside_frame(
        sim.world_mut(),
        CAMERA_CENTRE,
        CAMERA_HEIGHT,
        &["Floor"],
    );
    assert!(
        worst.is_none(),
        "a cena spawna algo fora do quadro que ela define: {worst:?} — o artista o \
         seleciona na Hierarquia e as alças dele são desenhadas fora da tela"
    );
}
