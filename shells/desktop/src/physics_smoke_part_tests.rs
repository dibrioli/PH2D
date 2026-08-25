//! A sonda da cena 70 + os gates que mantêm a mensagem dela honesta (W-PartFace).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_keys(sim.world_mut());
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

fn world_y(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity(sim, name);
    ph2d_ecs::world_transform(sim.world(), e)
        .expect("transform")
        .translation
        .y
}

const LANE_NAMES: [&str; 2] = ["Wide", "Slim"];

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_70 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_70() {
    let mut sim = run(5.0);
    println!("\n=== cena 70 (5 s) ===");
    for name in LANE_NAMES {
        println!(
            "  {name:<5} cabo {:>7.3}   palhetao {:>7.3}",
            world_y(&mut sim, &format!("{name} Handle")),
            world_y(&mut sim, &format!("{name} Bit")),
        );
    }
    println!("  (topo do muro 1.20 · topo do chao -0.80)");
}

/// **Todo número que a mensagem afirma sai da cena.**
#[test]
fn the_scene_measures_what_its_message_claims() {
    let mut sim = run(5.0);
    for (i, name) in LANE_NAMES.iter().enumerate() {
        let y = world_y(&mut sim, &format!("{name} Handle"));
        assert!(
            (y - MEASURED_HANDLE_Y[i]).abs() < 0.2,
            "{name}: o cabo parou em {y:.3}, a mensagem diz {:.2}",
            MEASURED_HANDLE_Y[i]
        );
    }
}

/// **O oráculo é BINÁRIO, e é isso que faz a cena valer:** a chave larga fica
/// ACIMA do muro, a estreita fica ABAIXO dele.
///
/// ⚠️ A faixa `Slim` é o CONTROLE — sem ela a cena não distinguiria *"a peça
/// larga entala"* de *"nenhuma chave passa nesta fenda"*.
#[test]
fn the_wide_key_is_stopped_by_the_wall_and_the_slim_one_goes_through() {
    let mut sim = run(5.0);
    // Topo do muro: centro 1,0 + meia-altura 0,2.
    const WALL_TOP: f32 = 1.2;
    let wide = world_y(&mut sim, "Wide Handle");
    let slim = world_y(&mut sim, "Slim Handle");
    assert!(
        wide > WALL_TOP,
        "a chave LARGA passou: o cabo parou em {wide:.3}, abaixo do topo do muro \
         ({WALL_TOP}) — o palhetão de meia-largura {:.2} cabe numa fenda de {:.2}?",
        BIT_HALF_X[0],
        SLOT_HALF
    );
    assert!(
        slim < WALL_TOP,
        "a chave ESTREITA não passou: o cabo parou em {slim:.3}, acima do topo do \
         muro — o CONTROLE da cena falhou, então nada nela é atribuível"
    );
}

/// **Afinar a PEÇA é o que liberta a chave** — o gesto inteiro da wave, medido
/// pelo caminho do produto (o edit do §11 sobre a entidade da peça).
///
/// Sem este gate a cena afirmaria uma promessa que ninguém verificou: que o
/// número digitado no campo *Half Width* de uma peça de fato muda a simulação.
#[test]
fn narrowing_the_part_lets_the_wide_key_through() {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_keys(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let bit = entity(&mut sim, "Wide Bit");
    // O MESMO edit que o campo do painel emite.
    crate::render_loop::inspector_physics_tests::apply(
        &mut sim,
        bit,
        ph2d_editor::PhysicsFieldEdit::HalfX(BIT_HALF_X[1]),
    );
    let mut bridge = PhysicsBridge::new();
    for t in 0..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = world_y(&mut sim, "Wide Handle");
    assert!(
        y < 1.2,
        "a chave larga continuou entalada em {y:.3} depois de afinar o palhetão — \
         o valor digitado não chegou ao solver"
    );
}

/// **Remover a PEÇA é o segundo caminho**, e ele tem de dar o mesmo resultado.
#[test]
fn removing_the_part_also_lets_the_wide_key_through() {
    let mut sim = SimWorld::new();
    crate::physics_smoke::spawn_floor(sim.world_mut());
    build_keys(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let bit = entity(&mut sim, "Wide Bit");
    crate::render_loop::inspector_physics_tests::apply(
        &mut sim,
        bit,
        ph2d_editor::PhysicsFieldEdit::Remove,
    );
    let mut bridge = PhysicsBridge::new();
    for t in 0..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = world_y(&mut sim, "Wide Handle");
    assert!(
        y < 1.2,
        "a chave continuou entalada em {y:.3} depois de *Remove Shape* — a forma \
         removida ainda alcança o solver"
    );
}

/// **Cada cabo tem DUAS peças**, que é o que o readout do dono afirma.
#[test]
fn each_handle_owns_exactly_two_parts() {
    let mut sim = run(0.0);
    for name in LANE_NAMES {
        let handle = entity(&mut sim, &format!("{name} Handle"));
        let mut q = sim.world_mut().query_filtered::<Entity, (
            bevy_ecs::query::With<Collider>,
            bevy_ecs::query::Without<RigidBody>,
        )>();
        let candidates: Vec<Entity> = q.iter(sim.world()).collect();
        assert_eq!(
            ph2d_physics_ecs::count_parts(sim.world(), handle, candidates),
            2,
            "{name} Handle não tem as duas peças que a mensagem promete"
        );
    }
}
