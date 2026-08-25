//! A sonda da cena 71 + os gates que mantêm a mensagem dela honesta
//! (W-PartSensor).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build_characters(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    (sim, bridge)
}

fn entity(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn foot(sim: &mut SimWorld, i: usize) -> Entity {
    entity(sim, &format!("{} Foot", LANE_NAMES[i]))
}

fn torso_y(sim: &mut SimWorld, i: usize) -> f32 {
    let e = entity(sim, &format!("{} Torso", LANE_NAMES[i]));
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_71 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_71() {
    let (mut sim, bridge) = run(5.0);
    println!("\n=== cena 71 (5 s) ===");
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let f = foot(&mut sim, i);
        println!(
            "  {name:<9} tronco y {:>7.3}   pe' aceso {}   dentro {:?}",
            torso_y(&mut sim, i),
            bridge.is_triggered(f),
            bridge.bodies_inside(f).len(),
        );
    }
    println!("  triggered_sensors(): {:?}", bridge.triggered_sensors());
}

/// **Todo booleano que a mensagem afirma sai da cena.**
#[test]
fn the_scene_measures_what_its_message_claims() {
    let (mut sim, bridge) = run(5.0);
    for (i, (name, want)) in LANE_NAMES.iter().zip(MEASURED_LIT).enumerate() {
        let f = foot(&mut sim, i);
        assert_eq!(
            bridge.is_triggered(f),
            want,
            "{name}: o pe' aceso e' {}, a mensagem diz {want}",
            bridge.is_triggered(f),
        );
    }
}

/// **O CONTROLE é a faixa do meio** — sem ela a cena não distinguiria *"o sensor
/// dispara"* de *"todo pé fica aceso"*.
#[test]
fn the_hovering_character_is_the_control_and_never_lights() {
    let (mut sim, bridge) = run(5.0);
    let hovering = foot(&mut sim, 1);
    assert!(
        !bridge.is_triggered(hovering),
        "o CONTROLE acendeu: nada nesta cena e' atribuivel"
    );
    // E ele de fato paira — se caísse, o controle teria sido atropelado pelo
    // experimento (a quarta vez que isso acontece nesta linha).
    let y = torso_y(&mut sim, 1);
    assert!(
        y > 2.0,
        "o CONTROLE caiu ate' {y:.3}: ele devia pairar com GravityScale 0"
    );
}

/// **Um pé-sensor nunca acende o TRONCO** — marcar uma peça não transforma o
/// corpo inteiro num gatilho.
#[test]
fn marking_a_part_does_not_make_the_whole_body_a_trigger() {
    let (mut sim, bridge) = run(5.0);
    for name in LANE_NAMES {
        let torso = entity(&mut sim, &format!("{name} Torso"));
        assert!(!bridge.is_triggered(torso), "{name} Torso acendeu");
    }
    // E quem acende são exatamente os pés que a mensagem promete.
    let lit = bridge.triggered_sensors();
    let expect: Vec<Entity> = (0..LANES.len())
        .filter(|i| MEASURED_LIT[*i])
        .map(|i| foot(&mut sim, i))
        .collect();
    assert_eq!(
        lit.len(),
        expect.len(),
        "acesos: {lit:?}, esperados {expect:?}"
    );
    for e in expect {
        assert!(lit.contains(&e), "{e:?} devia estar aceso: {lit:?}");
    }
}

/// **O passo 3 da mensagem é verificável:** trocar o pé para Solid o torna
/// APOIO, e o tronco sobe.
///
/// ⚠️ É o oráculo que separa *"o sensor atravessa"* de *"o sensor não existe"* —
/// um pé que o solver ignorasse por completo daria o MESMO resultado do sensor
/// na altura do tronco, e só este braço os distingue.
#[test]
fn a_solid_foot_props_the_torso_up_by_its_own_height() {
    let (mut sim, _) = run(3.0);
    let sensor_y = torso_y(&mut sim, 0);

    let mut sim = SimWorld::new();
    build_characters(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let f = foot(&mut sim, 0);
    sim.world_mut()
        .get_mut::<Collider>(f)
        .expect("collider")
        .is_sensor = false;
    let mut bridge = PhysicsBridge::new();
    for t in 0..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let solid_y = torso_y(&mut sim, 0);

    let lift = solid_y - sensor_y;
    let expect = FOOT_HALF[1] * 2.0;
    assert!(
        (lift - expect).abs() < 0.05,
        "o pe' solido devia erguer o tronco {expect:.2} m; ergueu {lift:.3} \
         (sensor {sensor_y:.3}, solido {solid_y:.3})"
    );
}
