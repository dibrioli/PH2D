//! **A cena 75, medida HEADLESS antes de a mensagem ser escrita.**
//!
//! A política do plano: *toda wave ganha cena com números MEDIDOS*. Este arquivo
//! dirige os dois guinchos pelas portas do produto e afirma o que a mensagem
//! promete ao artista — inclusive o CONTROLE, que é a metade que faz a outra
//! significar alguma coisa.

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::PhysicsBridge;

fn scene() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    build(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena tem de conter '{name}'"))
}

/// A folga de TANGENTE entre a carga e o eixo do guincho `x`.
fn gap(sim: &SimWorld, load: Entity, x: f32) -> f32 {
    let t = sim.world().get::<Transform>(load).expect("pose");
    let d = (t.translation.x - x).hypot(t.translation.y - BOOM_Y);
    (d * d - WHEEL_R * WHEEL_R).max(0.0).sqrt()
}

/// **O CONTROLE entra na roldana e o limitado PARA.**
///
/// É o par que a cena mostra lado a lado, e o controle é o que impede este gate
/// de ser verde sobre um limitador que não faz nada.
#[test]
fn the_free_winch_reaches_the_wheel_and_the_held_one_stops() {
    let (mut sim, mut bridge) = scene();
    let free = named(&mut sim, "Free Load");
    let held = named(&mut sim, "Held Load");
    let (mut lo_free, mut lo_held) = (f32::INFINITY, f32::INFINITY);
    for tick in 1..900u64 {
        bridge.dispatch(&mut sim, true, tick);
        lo_free = lo_free.min(gap(&sim, free, -4.0));
        lo_held = lo_held.min(gap(&sim, held, 4.0));
    }
    assert!(
        lo_free < 0.05,
        "o CONTROLE tem de encostar na roldana (folga mínima {lo_free:.4} m)"
    );
    assert!(
        lo_held > STOP_M - 0.15,
        "o limitador de {STOP_M} m não segurou (folga mínima {lo_held:.4} m)"
    );
}

/// **As duas marcas existem e ficam SOBRE a corda** — sem elas o gesto que a cena
/// manda fazer não tem alvo.
#[test]
fn both_ropes_offer_their_two_marks_on_the_rope() {
    let (mut sim, bridge) = scene();
    for (tag, stop) in [("Free Rope", 0.0f32), ("Held Rope", STOP_M)] {
        let rope = named(&mut sim, tag);
        let legs = bridge.rope_stop_legs(rope);
        let leg = legs[0].unwrap_or_else(|| panic!("{tag}: a ponta A tem roldana"));
        let mark = ph2d_physics_ecs::stop_mark(&leg, stop);
        // A marca está no segmento âncora→tangência: a projeção dela de volta dá
        // o mesmo número, e ela fica entre as duas pontas.
        let back = ph2d_physics_ecs::stop_at_point(&leg, mark);
        assert!(
            (back - stop).abs() < 1e-3,
            "{tag}: {stop} voltou como {back}"
        );
        assert!(
            stop <= leg.len,
            "{tag}: o limitador ({stop}) passou do trecho ({})",
            leg.len
        );
    }
}

/// **A sonda que alimenta a mensagem da cena.**
///
/// `cargo test -p ph2d-host-desktop --release --bins measure_the_stop_scene -- --ignored --nocapture`
#[test]
#[ignore = "sonda"]
fn measure_the_stop_scene() {
    let (mut sim, mut bridge) = scene();
    let free = named(&mut sim, "Free Load");
    let held = named(&mut sim, "Held Load");
    println!("\n=== CENA 75 (limitador {STOP_M} m no verde) ===");
    println!("   t (s) | folga LIVRE | folga LIMITADA");
    for tick in 1..=900u64 {
        bridge.dispatch(&mut sim, true, tick);
        if tick % 150 == 0 {
            println!(
                "  {:6.2} | {:11.4} | {:14.4}",
                tick as f32 / 60.0,
                gap(&sim, free, -4.0),
                gap(&sim, held, 4.0)
            );
        }
    }
}
