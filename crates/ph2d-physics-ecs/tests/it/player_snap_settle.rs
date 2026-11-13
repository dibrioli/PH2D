//! **O PLAYER ASSENTA NO TIQUE EM QUE A PERNA VÊ O CHÃO, E O PRIMEIRO PULO SOBE A JUMP HEIGHT**
//! (Bug #13, `docs/Physics/BUGS_physics.md`) — Snap (`PlayerMode::Kinematic`) e dinâmico.
//!
//! Medido antes da cura (H = 1,0, nascido a 0,9, Snap): queda livre de 22 tiques com a lei a dizer
//! `Ground` desde o 16; premir no 16..21 subia 0,357 m (o `v0` somado a uma velocidade a descer) e
//! assente ainda 0,956 (o `launch_speed` modelava 16 passos de Euler, o Snap integra um).

use crate::platform_water_scene as scene;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{BodyKind, PhysicsBridge, PlatformPlayer, RigidBody};
use ph2d_platformer::{FootingKind, PlayerInput};
use scene::{FLOAT, floor, subject_tuned};

fn snap(sim: &mut SimWorld, who: ph2d_ecs::Entity) {
    let mut e = sim.world_mut().entity_mut(who);
    e.insert(ph2d_physics_ecs::PlayerMode::Kinematic);
    if let Some(mut rb) = e.get_mut::<RigidBody>() {
        rb.kind = BodyKind::Kinematic;
    }
}

/// `(y por tique, chão por tique)` de uma corrida; `press` = o tique a partir do qual o pulo é preso.
fn run(kin: bool, spawn: f32, h: f32, press: Option<u64>, ticks: u64) -> (Vec<f32>, Vec<bool>) {
    let mut sim = SimWorld::new();
    floor(&mut sim, 0.0);
    let tune = PlatformPlayer {
        float_height: FLOAT,
        jump_height: h,
        ..PlatformPlayer::default()
    };
    let who = subject_tuned(&mut sim, true, spawn, Some(tune));
    if kin {
        snap(&mut sim, who);
    }
    let mut bridge = PhysicsBridge::new();
    let (mut ys, mut ground) = (vec![scene::y_of(&sim, "Subject")], vec![false]);
    for t in 1..=ticks {
        let jump = press.is_some_and(|p| t >= p);
        bridge.set_player_input(
            who,
            PlayerInput {
                jump,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        ys.push(scene::y_of(&sim, "Subject"));
        ground.push(
            bridge
                .player_view(who)
                .is_some_and(|v| v.footing == FootingKind::Ground),
        );
    }
    (ys, ground)
}

/// O primeiro tique depois do qual a altura fica a < 1 mm da de repouso (a do tique 240).
fn settle_tick(ys: &[f32]) -> usize {
    let rest = *ys.last().unwrap();
    let last_off = ys.iter().rposition(|y| (y - rest).abs() >= 1.0e-3);
    last_off.map_or(0, |i| i + 1)
}

/// **(a) Nascido acima do chão, dentro do alcance da perna, ele assenta**: Snap em 2 tiques
/// (vão 0,05–0,24 sobre a pele; a perna Snap = pé 0,5 + `cling_distance` 0,25) e o dinâmico na
/// mola (vão 0,05–0,24 sobre o `float_height` 0,9).
///
/// ⚠️ **Mutação:** `snap_down` devolver o `wanted` → Snap assenta no tique 39 (vão 0,15) e 72 (0,24).
#[test]
fn a_player_born_in_reach_of_its_leg_settles() {
    let mut bad = Vec::new();
    for gap in [0.05_f32, 0.15, 0.24] {
        let (ys, _) = run(true, 0.51 + gap, 1.0, None, 240);
        let n = settle_tick(&ys);
        if n > SNAP_SETTLE {
            bad.push(format!("snap vao {gap}: assentou no tique {n}"));
        }
        let (ys, _) = run(false, FLOAT + gap, 1.0, None, 240);
        let n = settle_tick(&ys);
        if n > DYNAMIC_SETTLE {
            bad.push(format!("dinamico vao {gap}: assentou no tique {n}"));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// Snap: medido 2 nos três vãos — o encaixe é pedido no 1.º `drive_players` e a pose cinemática
/// (`set_next_kinematic_pose`) chega ao corpo no passo.
const SNAP_SETTLE: usize = 2;
/// O dinâmico assenta na MOLA autorada (`ride_spring`, amortecida) — o controle de que a cura não
/// lhe toca; tecto verde antes e depois da cura.
const DYNAMIC_SETTLE: usize = 21;

/// **(b) Pulo preso no PRIMEIRO tique em que a lei diz `Ground` sobe a Jump Height ± 1 mm**, Snap e
/// dinâmico, H 0,5–2,5; Snap nascido à altura do dinâmico (cai até à faixa da perna) e dentro dela.
///
/// ⚠️ **Mutações:** `kin_in` devolver o `was` → Snap a 0,9 sobe 0,556 a H = 1; `world_steps` sob
/// Snap = o do mundo → todo Snap sai 0,956 a H = 1.
#[test]
fn the_first_jump_on_the_ground_rises_the_jump_height() {
    let mut bad = Vec::new();
    for h in [0.5_f32, 1.0, 1.5, 2.0, 2.5] {
        for (kin, spawn) in [(true, FLOAT), (true, 0.7), (false, FLOAT)] {
            let (_, ground) = run(kin, spawn, h, None, 60);
            let Some(first) = ground.iter().position(|g| *g) else {
                bad.push(format!("snap={kin} y0={spawn}: nunca no chao"));
                continue;
            };
            let first = first as u64;
            let (ys, _) = run(kin, spawn, h, Some(first), first + 140);
            let base = ys[first as usize - 1];
            let top = ys.iter().copied().fold(f32::MIN, f32::max);
            let rise = top - base;
            if (rise - h).abs() > 1.0e-3 {
                bad.push(format!(
                    "snap={kin} y0={spawn} h={h}: subiu {rise:.4} (premido no tique {first})"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// **(c) Largado de 3 m, ele assenta no tique em que a lei o vê de pé** — o encaixe do POUSO
/// (`KinematicState::stood`: ar → de pé). Medido: no mesmo tique; antes, ~75 tiques a descer
/// `g·dt²` por tique dentro da faixa da perna.
///
/// E **nascido a 0,9** (fora da perna Snap 0,75, dentro da perna AUTORADA `float_height + cling`)
/// assenta em ≤ 2 tiques: o encaixe do nascimento pergunta ao mundo no tique 1, antes do primeiro
/// passo, e o índice das consultas já tem o chão (`PhysicsWorld::refresh_query_index`).
///
/// ⚠️ **Mutação:** sem o refresh na porta dos controladores → assenta no tique 16 (a broad phase só
/// se construía no primeiro passo e o cast do encaixe não achava nada).
#[test]
fn a_snap_player_settles_where_it_lands() {
    let (ys, ground) = run(true, 3.0, 1.0, None, 240);
    let first = ground.iter().position(|g| *g).expect("nunca no chao");
    let n = settle_tick(&ys);
    assert!(
        n <= first,
        "largado de 3 m: a lei diz Ground no tique {first}, assentou no {n}"
    );
    let (ys, _) = run(true, FLOAT, 1.0, None, 240);
    let n = settle_tick(&ys);
    assert!(n <= SNAP_SETTLE, "nascido a {FLOAT}: assentou no tique {n}");
}
