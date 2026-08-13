//! Os gates da cena 114 (`W-Brake`) — a DERRAPADA, medida nesta geometria.
//!
//! ⚠️ **A cena inteira é um contraste de três**, então o gate corre os TRÊS: um
//! gate que só afirmasse *"o da direita pára curto"* passaria numa cena em que
//! os três parassem no mesmo sítio.
//!
//! ⚠️ **E a sonda vem ANTES do roteiro** — é a política do plano 00 §7.3, e esta
//! linha já teve duas cenas a afirmar coisas que a medição desmentiu.

use super::{BRAKES, DECK_END, MARK_X, RUN_ACCEL, RUN_SPEED, build_brake_scene, lane_x};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// A cena montada, com o relógio pronto a andar.
fn rig() -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    let _ = build_brake_scene(sim.world_mut());
    (sim, PhysicsBridge::new())
}

/// Onde está o personagem chamado `tag`.
fn at(sim: &SimWorld, tag: &str) -> (f32, f32) {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == tag {
            return (t.translation.x, t.translation.y);
        }
    }
    panic!("o personagem {tag} tem de existir");
}

/// O gesto do roteiro: correr para a direita até passar da marca, LARGAR, e
/// deixar assentar. Devolve, por raia, `(o x relativo em que largou, o x
/// relativo em que parou, caiu?)`.
///
/// ⚠️ **A largada é por POSIÇÃO e não por número de tiques**: os três aceleram
/// igual (a wave não toca o arranque), mas medir por tique deixaria a fixture
/// refém de um número que a próxima wave de arranque mudaria.
fn run_and_release() -> Vec<(f32, f32, bool)> {
    run_with([BRAKES[0].0, BRAKES[1].0, BRAKES[2].0])
}

/// O mesmo gesto, com os freios REESCRITOS — é o que os passos 5 e 6 do roteiro
/// pedem ao artista que faça pelo Inspector, e é assim que os números deles são
/// MEDIDOS em vez de estimados.
fn run_with(brakes: [f32; 3]) -> Vec<(f32, f32, bool)> {
    let (mut sim, mut bridge) = rig();
    for (i, (_, tag)) in BRAKES.iter().enumerate() {
        let mut q = sim
            .world_mut()
            .try_query::<(&Name, &mut ph2d_physics_ecs::PlatformPlayer)>()
            .unwrap();
        for (n, mut p) in q.iter_mut(sim.world_mut()) {
            if n.as_str() == *tag {
                p.brake_scale = brakes[i];
            }
        }
    }
    let mut released = vec![None::<f32>; BRAKES.len()];
    let mut tick = 0u64;

    for _ in 0..900 {
        tick += 1;
        // Quem ainda não passou da marca continua a correr; quem passou, larga.
        // ⚠️ A entrada é do MUNDO (a ponte a entrega a todo player), então o
        // gesto é o do artista: uma seta só. O `drive` cai a zero quando o
        // ÚLTIMO deles cruza — é o que a mão faz.
        let all_past = BRAKES.iter().enumerate().all(|(i, (_, tag))| {
            let x = at(&sim, tag).0 - lane_x(i);
            x >= MARK_X
        });
        let drive = if all_past { 0.0 } else { 1.0 };
        for (i, (_, tag)) in BRAKES.iter().enumerate() {
            if all_past && released[i].is_none() {
                released[i] = Some(at(&sim, tag).0 - lane_x(i));
            }
        }
        for e in sim
            .world()
            .try_query::<(bevy_ecs::entity::Entity, &ph2d_physics_ecs::PlatformPlayer)>()
            .unwrap()
            .iter(sim.world())
            .map(|(e, _)| e)
            .collect::<Vec<_>>()
        {
            bridge.set_player_input(
                e,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
    }

    BRAKES
        .iter()
        .enumerate()
        .map(|(i, (_, tag))| {
            let (x, y) = at(&sim, tag);
            (
                released[i].unwrap_or(f32::NAN),
                x - lane_x(i),
                y < -1.0, // caiu no poço
            )
        })
        .collect()
}

/// **A SONDA que numerou o roteiro** — a varredura que escolheu [`RUN_ACCEL`].
///
/// `cargo test -p ph2d-host-desktop --bins measure_the_scene_brake -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_scene_brake() {
    eprintln!("  cena: speed {RUN_SPEED} accel {RUN_ACCEL} marca {MARK_X} beira {DECK_END}");
    eprintln!("  raia        largou    parou   derrapou   caiu");
    for ((brake, tag), (rel, stop, fell)) in BRAKES.iter().zip(run_and_release()) {
        eprintln!(
            "  {tag:8} b={brake:4.2}  {rel:6.2}  {stop:7.2}   {:6.2}   {fell}",
            stop - rel
        );
    }
    // Os passos 5 e 6 do roteiro: o artista reescreve o freio da raia da
    // ESQUERDA pelo Inspector. A sonda mede o que ele vai ver.
    eprintln!("  -- a raia da ESQUERDA, com o freio reescrito no Inspector:");
    for b in [0.25_f32, 0.5, 1.0, 2.0, 4.0, 8.0, 40.0] {
        let r = run_with([b, BRAKES[1].0, BRAKES[2].0]);
        let (rel, stop, fell) = r[0];
        eprintln!(
            "  b={b:5.2}  derrapou {:6.2}   parou {stop:6.2}   caiu {fell}",
            stop - rel
        );
    }
}

/// **Os três param em sítios DIFERENTES, e o gelo não pára.**
///
/// ⚠️ **É o gate que torna a cena uma cena:** o contraste tem de estar dentro do
/// quadro, e a fixture larga o direcional no mesmo x para os três. Sem ele a
/// cena poderia autorar uma aceleração em que as três paragens colapsam no mesmo
/// pixel e o roteiro pediria ao Enio para ver uma diferença que não existe.
///
/// **Mutação que deve sangrar:** dar o mesmo `brake_scale` às três raias.
#[test]
fn the_three_lanes_stop_at_visibly_different_places() {
    let runs = run_and_release();
    let skid: Vec<f32> = runs.iter().map(|(rel, stop, _)| stop - rel).collect();

    assert!(
        runs[0].2,
        "o do GELO nao pode parar: ele desliza ate' a beira e cai ({runs:?})"
    );
    assert!(
        !runs[1].2 && !runs[2].2,
        "os outros dois PARAM na plataforma ({runs:?})"
    );
    assert!(
        skid[1] > skid[2],
        "o freio 2 tem de parar mais curto que o freio 1: {:.2} vs {:.2}",
        skid[1],
        skid[2]
    );
    // ⚠️ **A barra é o que o OLHO precisa**, não o que a lei consegue: meio metro
    // é mais que a largura de um personagem (0,40 m), então a diferença é
    // legível sem régua.
    assert!(
        skid[1] - skid[2] > 0.5,
        "a diferenca tem de ser VISIVEL (> 0,5 m): {:.2} vs {:.2}",
        skid[1],
        skid[2]
    );
}

/// **Quem pára, pára ANTES da beira** — a plataforma é longa o bastante.
///
/// ⚠️ Sem isto a cena mostraria os três a cair e não diria nada sobre nenhum: o
/// roteiro pede ao Enio para ver DOIS a parar, e uma beira curta demais tornaria
/// esse passo impossível de cumprir com o produto correcto.
#[test]
fn the_deck_is_long_enough_for_the_two_that_stop() {
    for (i, (rel, stop, fell)) in run_and_release().into_iter().enumerate() {
        if i == 0 {
            continue; // o gelo cai, e é essa a demonstração dele
        }
        assert!(!fell, "a raia {i} tem de parar na plataforma");
        assert!(
            stop < DECK_END - 0.5,
            "e com folga da beira: parou em {stop:.2}, a beira e' {DECK_END}"
        );
        assert!(rel.is_finite() && rel >= MARK_X, "largou depois da marca");
    }
}

/// **As raias não se alcançam** — medido na geometria MONTADA, não nas consts.
///
/// ⚠️ **A primeira versão comparava `LANE_SPAN > DECK_END + 1.0`** — duas
/// constantes, que o clippy nomeou como asserção de valor constante — e ela era
/// verde sobre uma cena ERRADA: o poço de cada raia vai até `DECK_END + 8`, então
/// as duas últimas unidades dele passavam por baixo do deck da raia seguinte, e o
/// do gelo aterrava lá. *Um oráculo que não olha para o produto não vê o produto.*
#[test]
fn the_lanes_never_touch() {
    let mut sim = SimWorld::new();
    let _ = build_brake_scene(sim.world_mut());

    // A extensão em x de tudo o que cada raia montou.
    let mut span = vec![(f32::MAX, f32::MIN); BRAKES.len()];
    let mut q = sim
        .world()
        .try_query::<(&Name, &Transform, &ph2d_physics_ecs::Collider)>()
        .unwrap();
    for (n, t, c) in q.iter(sim.world()) {
        let Some(i) = BRAKES
            .iter()
            .position(|(_, tag)| n.as_str() == *tag || n.as_str().starts_with(&format!("{tag} ")))
        else {
            continue;
        };
        let half_x = match c.shape {
            ph2d_physics_ecs::ColliderShape::Cuboid { half_x, .. } => half_x,
            ph2d_physics_ecs::ColliderShape::Capsule { radius, .. } => radius,
            ph2d_physics_ecs::ColliderShape::Ball { radius } => radius,
        };
        span[i].0 = span[i].0.min(t.translation.x - half_x);
        span[i].1 = span[i].1.max(t.translation.x + half_x);
    }

    for (i, (lo, hi)) in span.iter().enumerate() {
        assert!(
            lo.is_finite() && hi.is_finite(),
            "a raia {i} tem de ter montado alguma coisa"
        );
        if i + 1 < span.len() {
            assert!(
                *hi < span[i + 1].0,
                "a raia {i} acaba em {hi:.2} e a {} comeca em {:.2} — elas encostam",
                i + 1,
                span[i + 1].0
            );
        }
    }
}
