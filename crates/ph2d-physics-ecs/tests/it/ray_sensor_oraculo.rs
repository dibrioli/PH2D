//! ⭐⭐⭐ **O oráculo do Godot e o custo medido das melhorias do Ray Sensor** (GEF 07 RS) — irmão do
//! [`super::ray_sensor_melhorias`] pelo tecto de 700 linhas.

use crate::ray_sensor_melhorias::{Cena, bloco, cena, nome};
use crate::ray_sensor_versao_c::raio;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, RaySensor};

/// A linha `caso | …` da fixture do Godot.
fn caso<'a>(fixture: &'a str, chave: &str) -> &'a str {
    fixture
        .lines()
        .find(|l| l.split(" | ").next() == Some(chave))
        .unwrap_or_else(|| panic!("a fixture do Godot não tem o caso «{chave}»"))
}

/// O campo `nome=…` de uma linha.
fn campo<'a>(linha: &'a str, nome: &str) -> &'a str {
    linha
        .split(" | ")
        .find_map(|p| p.strip_prefix(&format!("{nome}=")))
        .unwrap_or_else(|| panic!("«{linha}» sem «{nome}»"))
}

fn par(s: &str) -> [f32; 2] {
    let v: Vec<f32> = s
        .trim_matches(|c| c == '(' || c == ')')
        .split(", ")
        .map(|x| x.parse().expect("número"))
        .collect();
    [v[0], v[1]]
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1.0e-3 && (a[1] - b[1]).abs() < 1.0e-3
}

/// ⭐⭐⭐ **O ORÁCULO, caso a caso** (§0.9): a cena do `oraculo.gd` no nosso motor dá o mesmo acerto que o
/// Godot 4.7.2 em cada passo — a linha, desligado (RS-6), a lista de ignorados (RS-12, o `add_exception`),
/// todos os acertos, o círculo varrido (RS-1, o `ShapeCast2D`: a distância exacta cai entre a `safe` e a
/// `unsafe` dele), o tiro por código (RS-8, o `intersect_ray`) e o alvo apagado (RS-11).
///
/// **Mutações que sangram:** o lance ignorar `thickness` (a linha passa ao lado do Side) · ignorar
/// `enabled` · o `also_ignore` não tirar o Floor · o `raycast` excluir sempre um corpo.
#[test]
fn oraculo_godot_das_melhorias() {
    let fixture = include_str!(
        "../../../../docs/GameEngineFuncional/componentes/oraculo_raio_rs/godot_rs.txt"
    );
    assert!(
        fixture.contains("Godot 4."),
        "a fixture tem o cabeçalho do Godot"
    );
    let um = |c: &mut Cena| {
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut c.sim, true, 1);
        let h = b.ray_sensor_hits().get(&c.raio).copied();
        (b, h)
    };

    let mut c = cena(RaySensor::default());
    let (_, h) = um(&mut c);
    let h = h.expect("line");
    let g = caso(fixture, "line");
    assert_eq!(nome(&c.sim, h.body), campo(g, "hit"));
    assert!(perto(h.point, par(campo(g, "point"))), "{:?}", h.point);
    assert!(perto(h.normal, par(campo(g, "normal"))), "{:?}", h.normal);

    let mut c = cena(RaySensor {
        enabled: false,
        ..RaySensor::default()
    });
    let (b, h) = um(&mut c);
    assert_eq!(campo(caso(fixture, "disabled"), "is_colliding"), "false");
    assert_eq!(h, None, "desligado não vê");
    assert!(b.ray_marks().is_empty(), "desligado não desenha");

    let mut c = cena(RaySensor {
        also_ignore: vec![11],
        ..RaySensor::default()
    });
    let (_, h) = um(&mut c);
    let g = caso(fixture, "also_ignore_floor");
    let h = h.expect("Floor2");
    assert_eq!(nome(&c.sim, h.body), campo(g, "hit"));
    assert!((h.distance - campo(g, "dist").parse::<f32>().expect("d")).abs() < 1.0e-3);

    let mut c = cena(RaySensor {
        all_hits: true,
        ..RaySensor::default()
    });
    let (b, _) = um(&mut c);
    let nossos: Vec<String> = b
        .ray_sensor_hits_all(c.raio)
        .iter()
        .map(|h| nome(&c.sim, h.body))
        .collect();
    assert_eq!(
        nossos.join(", "),
        caso(fixture, "all_hits")
            .split(" | ")
            .nth(1)
            .expect("lista")
    );

    let mut c = cena(RaySensor {
        thickness: 0.6,
        ..RaySensor::default()
    });
    let (_, h) = um(&mut c);
    let g = caso(fixture, "thick_0.6");
    let h = h.expect("o círculo acerta o Side");
    assert_eq!(
        nome(&c.sim, h.body),
        campo(g, "hit"),
        "a linha passaria ao lado"
    );
    assert_eq!(h.body, c.side);
    assert!(perto(h.point, par(campo(g, "point"))), "{:?}", h.point);
    let (safe, unsafe_): (f32, f32) = (
        campo(g, "safe").parse().expect("safe"),
        campo(g, "unsafe").parse().expect("unsafe"),
    );
    // A exacta: o centro desce até o círculo de 0,3 tocar o canto (0,2 ; −0,45) — 0,45 − √0,05.
    assert!(
        (h.distance - (0.45 - 0.05f32.sqrt())).abs() < 1.0e-3,
        "{}",
        h.distance
    );
    assert!(
        safe <= h.distance && h.distance <= unsafe_,
        "{safe} ≤ {} ≤ {unsafe_}",
        h.distance
    );

    let mut c = cena(RaySensor::default());
    let (b, _) = um(&mut c);
    let tiro = b.raycast(
        c.sim.world(),
        None,
        [0.0, 0.5],
        [0.0, -2.5],
        &RaySensor::default(),
    );
    let g = caso(fixture, "raycast_code");
    let h = tiro.first().expect("o tiro acerta");
    assert_eq!(nome(&c.sim, h.body), campo(g, "hit"));
    assert!(perto(h.point, par(campo(g, "point"))), "{:?}", h.point);
    assert!(perto(h.normal, par(campo(g, "normal"))));
    // Controlo: disparado PELO Owner, «Ignore my body» tira-o e o tiro vai ao chão.
    let do_dono = b.raycast(
        c.sim.world(),
        Some(c.dono),
        [0.0, 0.5],
        [0.0, -2.5],
        &RaySensor::default(),
    );
    assert_eq!(do_dono.first().map(|h| h.body), Some(c.floor));

    let mut c = cena(RaySensor::default());
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut c.sim, true, 1);
    c.sim.world_mut().despawn(c.floor);
    b.dispatch(&mut c.sim, true, 2);
    let g = caso(fixture, "target_deleted");
    let h = b.ray_sensor_hits().get(&c.raio).expect("o seguinte");
    assert_eq!(nome(&c.sim, h.body), campo(g, "hit"));
    assert_eq!(h.body, c.floor2);
}

/// ⭐ **RS-9 — o custo de «See all hits» MEDIDO** (§0.0: antes de qualquer limite). Uma fase de 200 peças
/// num chão, o raio a varrê-las todas; corre em `--profile smoke`:
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-physics-ecs --profile smoke --test it mede_o_custo_de_see_all_hits -- --ignored --nocapture`
///
/// A tabela vive no doc-comment do `cast_ray_query_all` (`ph2d-physics/src/world/cast_query.rs`).
#[test]
#[ignore = "sonda de custo: corre à mão, em --profile smoke"]
fn mede_o_custo_de_see_all_hits() {
    for (largura, todos, cone) in [
        (0.0, false, 0.0),
        (0.0, true, 0.0),
        (0.3, false, 0.0),
        (0.3, true, 0.0),
        (0.0, false, 60.0),
        (0.0, false, 360.0),
    ] {
        let mut sim = SimWorld::new();
        for i in 0..200u64 {
            #[allow(clippy::cast_precision_loss)]
            bloco(
                &mut sim,
                "Piece",
                (0.5 + i as f32 * 0.5, 0.0),
                (0.1, 0.1),
                100 + i,
            );
        }
        raio(
            &mut sim,
            Vec2::ZERO,
            None,
            RaySensor {
                angle: 0.0,
                reach: 120.0,
                thickness: largura,
                all_hits: todos,
                cone,
                ..RaySensor::default()
            },
        );
        let mut b = PhysicsBridge::new();
        b.dispatch(&mut sim, true, 1);
        let mut melhor = f64::INFINITY;
        for bloco_ in 0..7u64 {
            let t0 = std::time::Instant::now();
            for t in 0..50u64 {
                b.dispatch(&mut sim, true, 2 + bloco_ * 50 + t);
            }
            melhor = melhor.min(t0.elapsed().as_secs_f64() / 50.0);
        }
        let n = b
            .ray_sensor_hits_all(
                sim.world_mut()
                    .query::<(Entity, &RaySensor)>()
                    .iter(sim.world())
                    .next()
                    .expect("r")
                    .0,
            )
            .len();
        println!(
            "thickness {largura} all_hits {todos} cone {cone}: {n} acertos, {:.1} µs por tique (o mínimo de 7 blocos)",
            melhor * 1.0e6
        );
    }
}
