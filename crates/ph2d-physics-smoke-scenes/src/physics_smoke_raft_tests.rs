//! A sonda da cena 72 + os gates que mantêm a mensagem dela honesta
//! (W-CompoundZone).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_rafts(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn entity(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

/// `(altura do centro, inclinação em graus)`.
fn pose(sim: &mut SimWorld, i: usize) -> (f32, f32) {
    let e = entity(sim, LANE_NAMES[i]);
    let t = ph2d_ecs::world_transform(sim.world(), e).expect("transform");
    (t.translation.y, t.rotation.to_degrees())
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_72 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_72() {
    let mut sim = run(40.0);
    println!("\n=== cena 72 (40 s) ===");
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let (y, tilt) = pose(&mut sim, i);
        println!("  {name:<12} centro y {y:>7.4}   inclinacao {tilt:>8.3}deg");
    }
    println!("  (superficie da poca em y = 0; Arquimedes preve' 0,1250)");
}

/// **Todo número que a mensagem afirma sai da cena.**
#[test]
fn the_scene_measures_what_its_message_claims() {
    let mut sim = run(40.0);
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let (y, _) = pose(&mut sim, i);
        assert!(
            (y - MEASURED_Y[i]).abs() < 0.03,
            "{name}: centro em {y:.4}, a mensagem diz {:.2}",
            MEASURED_Y[i]
        );
    }
}

/// **NENHUMA das três capota** — o oráculo da wave, e o que a composta fazia.
///
/// ⚠️ O `Single` é o CONTROLE: se ele inclinar, a poça ou a fixture estão
/// erradas e nada mais nesta cena significa coisa alguma.
#[test]
fn every_raft_floats_level() {
    let mut sim = run(40.0);
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let (_, tilt) = pose(&mut sim, i);
        assert!(
            tilt.abs() < 2.0,
            "{name} inclinou {tilt:.3}deg -- o empuxo nasceu descentrado \
             (antes da wave a composta media -90,007)"
        );
    }
}

/// **A composta boia na MESMA linha d'água do controle** — mesma silhueta, mesma
/// massa, mesmo calado. É a metade que o gate da inclinação não cobre: força
/// dobrada e meia-força deixam a jangada nivelada e na altura ERRADA.
#[test]
fn the_compound_raft_sits_at_the_control_waterline() {
    let mut sim = run(40.0);
    let (y_one, _) = pose(&mut sim, 0);
    let (y_two, _) = pose(&mut sim, 1);
    assert!(
        (y_two - y_one).abs() < 0.02,
        "controle {y_one:.4} contra composta {y_two:.4}"
    );
}

/// **A carga SENSOR afunda a jangada, e nivelada** — ela desloca a mesma água e
/// carrega peso a mais.
///
/// ⚠️ O limiar é `0,05`, não `0,1`: a diferença medida é **0,0625**, e um bar
/// copiado do gate irmão da crate de física (onde a fixture é outra) reprovou
/// sobre produto correto. *Um limiar emprestado mede a fixture de origem.*
#[test]
fn the_sensor_cargo_raft_rides_lower_and_level() {
    let mut sim = run(40.0);
    let (y_plain, _) = pose(&mut sim, 0);
    let (y_cargo, tilt) = pose(&mut sim, 2);
    assert!(
        y_cargo < y_plain - 0.05,
        "a carga sensor deu empuxo: sem carga {y_plain:.4}, com carga {y_cargo:.4}"
    );
    assert!(
        tilt.abs() < 2.0,
        "a jangada com carga sensor inclinou {tilt:.3}deg -- ela e' simetrica \
         de proposito, para a lei aparecer sem ambiguidade"
    );
}
