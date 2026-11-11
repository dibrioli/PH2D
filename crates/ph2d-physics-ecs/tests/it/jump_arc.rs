//! **A curva do canvas é o pulo do jogo** (`ph2d_platformer::arc`, ficha GEF 04): o arco que o
//! editor desenha contra o player REAL na ponte (rapier, mola, cápsula, sub-passos) num chão plano.

use crate::platform_scene::{pose, scene};

use ph2d_ecs::{Entity, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};
use ph2d_platformer::arc::{ArcWorld, JumpArc, jump_arcs};
use ph2d_platformer::feel::Feel;

const DT: f32 = 1.0 / 60.0;

/// O que se compara de um pulo: o ápice (acima do arranque, para o lado), o tempo no ar e o alcance.
#[derive(Debug)]
struct Medido {
    apex_up: f32,
    apex_side: f32,
    air_ticks: usize,
    reach: f32,
}

/// O pouso, igual para os dois: o primeiro tique depois do ápice abaixo de [`POUSO`]. A perna do
/// jogo apara a queda (não desce à linha de partida); o desenho atravessa-a no mesmo tique.
const POUSO: f32 = 0.1;

fn pouso(ys: &[[f32; 2]]) -> usize {
    let top = ys
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map_or(0, |(i, _)| i);
    (top..ys.len())
        .find(|&i| ys[i][1] < POUSO)
        .unwrap_or(ys.len() - 1)
}

fn medir(pts: &[[f32; 2]]) -> Medido {
    let top = pts
        .iter()
        .copied()
        .max_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap();
    let i = pouso(pts);
    Medido {
        apex_up: top[1],
        apex_side: top[0],
        air_ticks: i,
        reach: pts[i][0],
    }
}

fn do_arco(a: &JumpArc) -> Medido {
    medir(&a.points)
}

#[derive(Copy, Clone)]
enum Como {
    Correndo,
    Parado,
    Curto,
}

fn do_jogo(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, como: Como) -> Medido {
    let drive = 1.0;
    let mut tick = 0_u64;
    // Assenta na perna; correndo, já chega à velocidade máxima antes do salto.
    for _ in 0..60 {
        let d = if matches!(como, Como::Parado) {
            0.0
        } else {
            drive
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: d,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    let (x0, y0) = pose(sim);
    let mut pts = vec![[0.0, 0.0]];
    for i in 0..200 {
        let jump = match como {
            Como::Curto => i == 0,
            _ => true,
        };
        bridge.set_player_input(
            player,
            PlayerInput {
                drive,
                jump,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
        let (x, y) = pose(sim);
        pts.push([x - x0, y - y0]);
    }
    medir(&pts)
}

fn caso(feel: Option<Feel>, como: Como) -> (Medido, Medido) {
    let (mut sim, mut bridge, player) = scene(0.0, -20.0);
    let w = ph2d_physics_ecs::arc_world(&bridge.settings(), 1.0);
    let cfg = {
        let mut e = sim.world_mut().entity_mut(player);
        let mut p = e.get_mut::<PlatformPlayer>().unwrap();
        if let Some(f) = feel {
            p.set_feel(f, w);
        }
        p.config()
    };
    let arcs = jump_arcs(&cfg, w);
    let arco = match como {
        Como::Correndo => &arcs.full,
        Como::Parado => &arcs.standing,
        Como::Curto => &arcs.short,
    };
    (do_arco(arco), do_jogo(&mut sim, &mut bridge, player, como))
}

#[test]
fn o_arco_do_desenho_e_o_do_jogo() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for (nome, como) in [
            ("correndo", Como::Correndo),
            ("parado", Como::Parado),
            ("curto", Como::Curto),
        ] {
            let (a, j) = caso(feel, como);
            eprintln!(
                "{feel:?} {nome}: desenho {a:?}\n{:>w$}jogo    {j:?}",
                "",
                w = 0
            );
            // Altura: 1,3 cm medido (o sub-passo do rapier e o nosso diferem na ordem). Lado do
            // ápice: um tique de corrida (6 m/s · 1/60), porque num topo achatado dois tiques empatam.
            let ok = (a.apex_up - j.apex_up).abs() < 0.02
                && (a.apex_side - j.apex_side).abs() < 0.11
                && a.air_ticks.abs_diff(j.air_ticks) <= 1
                && (a.reach - j.reach).abs() < 0.1;
            if !ok {
                falhas.push(format!("{feel:?} {nome}"));
            }
        }
    }
    assert!(falhas.is_empty(), "o desenho mente em: {falhas:?}");
}
