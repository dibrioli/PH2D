//! A sonda da cena 77 + os gates que mantêm a mensagem dela honesta (W-RailRope).

use super::*;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{IkOptions, PhysicsBridge};

/// A cabeça de uma pista — o elo mais à ESQUERDA, que é o que a mensagem manda
/// pegar.
fn head(sim: &mut SimWorld, lane: usize) -> Entity {
    let want = format!("{}0", LANE_NAMES[lane]);
    let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 77 nao montou '{want}'"))
}

/// Arrasta a cabeça da pista `lane` na direção `dir` e devolve o deslocamento de
/// cada um dos quatro elos, na ordem em que a corrente os alcança.
fn drag(lane: usize, dir: [f32; 2], d: f32) -> Vec<f32> {
    let mut sim = SimWorld::new();
    build_rail_rope_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    let h = head(&mut sim, lane);
    let names: Vec<String> = (0..4).map(|i| format!("{}{i}", LANE_NAMES[lane])).collect();
    let id = |sim: &mut SimWorld, n: &str| -> Entity {
        let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::Name)>();
        q.iter(sim.world())
            .find(|(_, m)| m.as_str() == n)
            .map(|(e, _)| e)
            .expect("elo existe")
    };
    let es: Vec<Entity> = names.iter().map(|n| id(&mut sim, n)).collect();
    let at = |sim: &SimWorld, e: Entity| {
        let t = sim.world().get::<ph2d_ecs::Transform>(e).expect("pose");
        [t.translation.x, t.translation.y]
    };
    let before: Vec<[f32; 2]> = es.iter().map(|&e| at(&sim, e)).collect();
    assert!(b.ik_begin(h), "pegar a cabeca tem de abrir gesto");
    for k in 1..=20i16 {
        let f = d * f32::from(k) / 20.0;
        let t = [before[0][0] + dir[0] * f, before[0][1] + dir[1] * f];
        for (e, tr, r) in b.ik_move(t, 0.0, IkOptions::default()) {
            if let Some(mut p) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
                p.translation = ph2d_core::Vec2::new(tr[0], tr[1]);
                p.rotation = r;
            }
        }
        b.dispatch(&mut sim, false, 0);
    }
    es.iter()
        .zip(&before)
        .map(|(&e, a)| {
            let c = at(&sim, e);
            (c[0] - a[0]).hypot(c[1] - a[1])
        })
        .collect()
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_77 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_77() {
    println!("\n=== cena 77 — o mastro telescopico (puxada de 2 m) ===");
    println!(
        "  {:<22} {:?}",
        "trilho, ao longo",
        drag(0, [1.0, 0.0], 2.0)
    );
    println!(
        "  {:<22} {:?}",
        "trilho, de traves",
        drag(1, [0.0, 1.0], 2.0)
    );
    println!(
        "  {:<22} {:?}",
        "soldado (controle)",
        drag(2, [1.0, 0.0], 2.0)
    );
}

/// **A pista de cima TELESCOPA** — cada elo come o próprio curso.
#[test]
fn the_rail_lane_telescopes_by_its_stroke() {
    let d = drag(0, [1.0, 0.0], 2.0);
    for i in 1..4 {
        let lag = d[i - 1] - d[i];
        assert!(
            (lag - STROKE).abs() < 0.05,
            "o elo {i} atrasou {lag:.3} m e o curso e' {STROKE}: {d:?}"
        );
    }
}

/// **A do meio vai INTEIRA** — de través um rail não tem liberdade.
#[test]
fn the_cross_lane_travels_whole() {
    let d = drag(1, [0.0, 1.0], 2.0);
    for (i, v) in d.iter().enumerate() {
        assert!(
            (v - 2.0).abs() < 1e-2,
            "o elo {i} da pista de traves ficou para tras: {d:?}"
        );
    }
}

/// **O CONTROLE: a corrente soldada vai inteira NA MESMA direção da primeira.**
///
/// ⚠️ Ela é arrastada ao longo de +X, igual à pista do trilho — é essa
/// coincidência que a torna um controle: as duas recebem o MESMO gesto e só o
/// tipo do elo difere. Sem ela a cena não distingue *o trilho desliza* de *tudo
/// é rígido*.
#[test]
fn the_welded_control_travels_whole_under_the_same_gesture() {
    let d = drag(2, [1.0, 0.0], 2.0);
    for (i, v) in d.iter().enumerate() {
        assert!(
            (v - 2.0).abs() < 1e-2,
            "o elo {i} da corrente SOLDADA ficou para tras: {d:?}"
        );
    }
}

/// **A cena monta as três pistas que ela nomeia.**
#[test]
fn the_scene_builds_the_three_lanes_it_names() {
    let mut sim = SimWorld::new();
    build_rail_rope_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    for lane in 0..3 {
        let _ = head(&mut sim, lane);
    }
}
