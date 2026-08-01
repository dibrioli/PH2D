//! A sonda da cena 69 + o gate que mantém a mensagem dela honesta (W-Compound).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_compound(sim.world_mut());
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
        .expect("entidade viva")
}

fn world_y(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity(sim, name);
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

/// A ponta de BAIXO de uma perna — o número que mostra o atravessamento.
fn leg_tip_y(sim: &mut SimWorld, name: &str) -> f32 {
    world_y(sim, name) - LEG_HALF[1]
}

const LANE_NAMES: [&str; 2] = ["Bare", "Solid"];

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_69 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_69() {
    let mut sim = run(5.0);
    println!("\n=== cena 69 (5 s) ===");
    for name in LANE_NAMES {
        println!(
            "  {name:<6} tampo {:>7.3}   ponta da perna {:>7.3}",
            world_y(&mut sim, &format!("{name} Top")),
            leg_tip_y(&mut sim, &format!("{name} Leg 1")),
        );
    }
    println!();
}

/// **Todo número que a mensagem afirma sai da cena.**
#[test]
fn the_scene_measures_what_its_message_claims() {
    let mut sim = run(5.0);
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let top = world_y(&mut sim, &format!("{name} Top"));
        assert!(
            (top - MEASURED_TOP_Y[i]).abs() < 0.2,
            "{name}: o tampo parou em {top:.3}, a mensagem diz {:.2}",
            MEASURED_TOP_Y[i]
        );
        let tip = leg_tip_y(&mut sim, &format!("{name} Leg 1"));
        assert!(
            (tip - MEASURED_LEG_TIP_Y[i]).abs() < 0.2,
            "{name}: a ponta da perna parou em {tip:.3}, a mensagem diz {:.2}",
            MEASURED_LEG_TIP_Y[i]
        );
    }
}

/// **A faixa SEM peças é o CONTROLE, e ela tem de estar errada.**
///
/// ⚠️ Sem esta metade a cena não distinguiria *"as peças funcionam"* de *"mesas
/// sempre pararam em cima das pernas"* — e o defeito que a wave fecha é
/// justamente o silencioso: as pernas desenhadas atravessam o chão.
#[test]
fn the_bare_table_sinks_and_the_one_with_parts_stands() {
    let mut sim = run(5.0);
    let bare = leg_tip_y(&mut sim, "Bare Leg 1");
    let solid = leg_tip_y(&mut sim, "Solid Leg 1");
    // ⚠️ **O oráculo é o CHÃO e não um limiar meu.** A 1ª versão deste gate
    // cravou `0.4` supondo o chão em `0,5`; o do smoke tem topo em **−0,80**
    // (`spawn_floor`: centro −1,0, meia-altura 0,2), e o gate falhou sobre um
    // produto correto.
    const FLOOR_TOP: f32 = -0.8;
    assert!(
        bare < FLOOR_TOP - 0.5,
        "o CONTROLE não afundou: a perna sem collider parou em {bare:.3}, e ela \
         devia atravessar o chão (topo em {FLOOR_TOP})"
    );
    assert!(
        solid > FLOOR_TOP - 0.05,
        "a perna com collider afundou até {solid:.3}, e o chão está em {FLOOR_TOP} — \
         ela não está segurando a mesa"
    );
    assert!(
        solid > bare + 1.0,
        "as duas mesas pararam quase na mesma altura ({bare:.3} contra {solid:.3}): \
         a cena não mostra diferença nenhuma"
    );
}
