//! A sonda da cena 68 + o gate que mantém a mensagem dela honesta (W-SoftWeld).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte), devolvendo o
/// mundo — os mesmos corpos, os mesmos joints e a mesma gravidade que o artista
/// vê, e não uma re-encenação em `ph2d-physics`.
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_soft_weld(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn entity(sim: &mut SimWorld, name: &str) -> ph2d_ecs::Entity {
    let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("corpo vivo")
}

/// Quanto o braço da faixa pendeu, em graus (positivo = para baixo).
fn droop(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity(sim, &format!("{name} Arm"));
    -ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .rotation
        .to_degrees()
}

/// Quanto a ponta SOLDADA se afastou da parede, em metros. Numa solda isto é
/// zero — é a metade que separa vergar de soltar.
fn separation(sim: &mut SimWorld, name: &str, lane: usize) -> f32 {
    let e = entity(sim, &format!("{name} Arm"));
    let t = ph2d_ecs::world_transform(sim.world(), e).expect("transform");
    let (s, c) = t.rotation.sin_cos();
    let tip = [
        t.translation.x - c * ARM_HALF[0],
        t.translation.y - s * ARM_HALF[0],
    ];
    (tip[0] - LANES[lane]).hypot(tip[1] - ARM_Y)
}

const LANE_NAMES: [&str; 4] = ["Rigid", "Soft", "Floppy", "Impact"];

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_68 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_68() {
    // O pico do impacto tem de ser amostrado ENQUANTO a bola está em cima, então
    // a varredura roda tick a tick em vez de medir só o fim.
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_soft_weld(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut peak = 0.0f32;
    for t in 0..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        peak = peak.max(droop(&mut sim, "Impact"));
    }

    println!("\n=== cena 68 (10 s) ===");
    for (i, name) in LANE_NAMES.iter().enumerate() {
        println!(
            "  {name:<8} k={:<6.0} droop {:>7.2}°   separacao {:>7.4} m",
            LANE_STIFFNESS[i],
            droop(&mut sim, name),
            separation(&mut sim, name, i)
        );
    }
    println!("  Impact: pico sob a bola {peak:>7.2}°\n");
}

/// **Todo número que a mensagem afirma sai da cena.**
///
/// ⚠️ Uma cena que descreve o que ela NÃO faz é pior que uma sem mensagem: o
/// artista aprova o que leu. As tolerâncias são folgadas o bastante para não
/// flakar e apertadas o bastante para pegar uma constante que envelheceu.
#[test]
fn the_scene_measures_what_its_message_claims() {
    let mut sim = run(10.0);
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let d = droop(&mut sim, name);
        assert!(
            (d - MEASURED_DROOP_DEG[i]).abs() < 1.0,
            "{name}: pendeu {d:.2}°, a mensagem diz {:.2}°",
            MEASURED_DROOP_DEG[i]
        );
        let sep = separation(&mut sim, name, i);
        assert!(
            sep < 1e-3,
            "{name}: a solda se abriu {sep:.4} m — a mensagem promete {MEASURED_SEPARATION_M:.4}"
        );
    }
}

/// **O CONTROLE não se move, e as três moles se movem em ordem.** É o que faz da
/// cena um A/B: sem a faixa rígida ela não distinguiria *"a solda mole funciona"*
/// de *"toda solda sempre pendeu"*, e sem a ordem ela não mostraria que a dureza
/// é um knob.
#[test]
fn the_rigid_lane_is_the_control_and_the_stiffness_orders_the_rest() {
    let mut sim = run(10.0);
    let rigid = droop(&mut sim, "Rigid");
    let soft = droop(&mut sim, "Soft");
    let floppy = droop(&mut sim, "Floppy");

    assert!(rigid.abs() < 0.05, "a faixa RÍGIDA pendeu {rigid:.3}°");
    assert!(soft > 2.0, "a faixa mole não cedeu: {soft:.3}°");
    assert!(
        floppy > soft * 2.0,
        "a faixa FROUXA ({floppy:.2}°) tinha de ceder bem mais que a default \
         ({soft:.2}°) — senão o knob não é um knob"
    );
}

/// **E ela VOLTA.** O pico sob a bola e o repouso depois dela são a palavra
/// inteira: uma solda rígida não teria vergado, uma dobradiça não teria voltado.
#[test]
fn the_impact_lane_bends_far_and_springs_back() {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_soft_weld(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut peak = 0.0f32;
    for t in 0..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        peak = peak.max(droop(&mut sim, "Impact"));
    }
    let rest = droop(&mut sim, "Impact");

    assert!(
        peak > MEASURED_IMPACT_PEAK_DEG - 6.0,
        "o pico sob a bola foi {peak:.2}°, a mensagem diz {MEASURED_IMPACT_PEAK_DEG:.2}°"
    );
    assert!(
        (rest - MEASURED_IMPACT_REST_DEG).abs() < 1.5,
        "a viga assentou em {rest:.2}° e a mensagem diz {MEASURED_IMPACT_REST_DEG:.2}° — \
         ou a bola não saiu, ou a mola não devolveu"
    );
    assert!(
        peak > rest * 2.0,
        "o pico ({peak:.2}°) mal difere do repouso ({rest:.2}°): a bola não chegou \
         a vergar a viga, e a 4a faixa não mostra nada"
    );
}
