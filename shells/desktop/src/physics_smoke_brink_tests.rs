//! Os gates da cena 119 (`W-Brink`) — **a cena tem de mostrar o que a mensagem
//! promete**, nesta geometria.
//!
//! ⚠️ Uma cena de smoke é código que ninguém compila mentalmente: sem estes
//! gates ela pode nascer com o personagem fora do patamar, a raia do controle
//! sem cair, ou a fenda larga demais para a perna — e o artista leria qualquer
//! um desses como *"a feature não funciona"*.

use super::physics_smoke_brink::{GAP, WALK_SPEED, build_brink_scene};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlatformPlayer, PlayerInput};

/// Anda as três raias para a DIREITA por `ticks` tiques e devolve a pose final.
fn run(ticks: u64) -> Vec<(String, [f32; 2])> {
    let mut sim = SimWorld::new();
    let players = build_brink_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    for i in 1..=ticks {
        for p in players {
            bridge.set_player_input(
                p,
                PlayerInput {
                    drive: 1.0,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, i);
    }
    players
        .iter()
        .map(|&p| {
            let t = sim.world().get::<Transform>(p).expect("transform");
            let n = sim
                .world()
                .get::<Name>(p)
                .expect("nome")
                .as_str()
                .to_string();
            (n, [t.translation.x, t.translation.y])
        })
        .collect()
}

/// **A cena mostra o contraste que a mensagem promete: um cai, o outro não.**
///
/// ⚠️ As duas metades no mesmo teste de propósito. Sem a primeira o gate ficaria
/// verde sobre uma cena em que ninguém anda; sem a segunda, sobre uma trava
/// inerte.
#[test]
fn the_control_lane_falls_and_the_armed_one_stops() {
    let end = run(240);
    let by = |name: &str| {
        end.iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| *p)
            .unwrap_or_else(|| panic!("a cena 119 monta a raia {name}"))
    };
    let control = by("Walks Off");
    let armed = by("Stops At Edge");
    assert!(
        control[1] < -1.0,
        "a raia de CONTROLE tem de cair, senao a cena nao mostra contraste \
         nenhum (y={:.4})",
        control[1]
    );
    assert!(
        armed[1] > -0.5,
        "a raia armada nao pode cair (y={:.4})",
        armed[1]
    );
}

/// **A raia da FENDA atravessa** — a metade que o primeiro desenho da wave não
/// conseguia, e a razão de o sensor perguntar à frente do corpo.
///
/// ⚠️ O oráculo é a posição RELATIVA à quina da própria raia, não um número
/// absoluto: um `x` cru quebraria em silêncio no dia em que o `LANE_SPAN` mudar.
#[test]
fn the_gap_lane_crosses_the_gap_and_stops_at_the_far_edge() {
    let end = run(240);
    let (_, gap_lane) = end
        .iter()
        .find(|(n, _)| n == "Crosses The Gap")
        .expect("a cena 119 monta a raia da fenda");
    // A raia 2 comeca em `2 * LANE_SPAN`; a quina PERTO fica em `+4`, a fenda
    // acaba em `+4 + GAP` e a quina FINAL em `+4 + GAP + 8`.
    let x0 = 2.0 * 16.0;
    let near_edge = x0 + 4.0;
    let far_edge = near_edge + GAP + 8.0;
    assert!(
        gap_lane[1] > -0.5,
        "ele nao pode cair na fenda (y={:.4})",
        gap_lane[1]
    );
    assert!(
        gap_lane[0] > near_edge + GAP,
        "ele tem de ATRAVESSAR a fenda (x={:.4}, a fenda acaba em {:.4})",
        gap_lane[0],
        near_edge + GAP
    );
    assert!(
        gap_lane[0] < far_edge + 0.4,
        "e tem de PARAR na quina final (x={:.4}, quina em {far_edge:.4})",
        gap_lane[0]
    );
}

/// **A fenda cabe na perna, e o gate diz o número** — uma fenda maior que o
/// alcance do leque faria a raia 3 medir o oposto do que a mensagem promete.
///
/// ⚠️ Ela é geometria da cena, então vive aqui e não numa nota: a perna de três
/// pés cobre `±(spread × meia-largura)` do centro, e é isso que decide se um vão
/// é *atravessável* ou *um patamar*.
#[test]
fn the_gap_is_narrower_than_the_leg_can_span() {
    let cfg = PlatformPlayer::default();
    let half_w = 0.2_f32; // a capsula da cena
    let span = 2.0 * half_w * cfg.foot_spread;
    assert!(
        GAP < span,
        "a fenda de {GAP} m tem de caber no alcance do leque ({span:.4} m), \
         senao a raia 3 mostra o contrario do que promete"
    );
}

/// **As três raias andam à MESMA velocidade** — o contraste é sobre a trava, e
/// uma raia mais rápida mudaria o alcance derivado junto.
#[test]
fn every_lane_walks_at_the_same_authored_speed() {
    let mut sim = SimWorld::new();
    let players = build_brink_scene(sim.world_mut());
    for p in players {
        let cfg = sim.world().get::<PlatformPlayer>(p).expect("player");
        assert_eq!(
            cfg.speed, WALK_SPEED,
            "a cena promete a MESMA velocidade nas tres raias"
        );
    }
    // E so' a raia 0 anda para fora — as outras duas travam.
    let allows: Vec<bool> = players
        .iter()
        .map(|&p| {
            sim.world()
                .get::<PlatformPlayer>(p)
                .expect("player")
                .walk_off_ledges
        })
        .collect();
    assert_eq!(
        allows,
        vec![true, false, false],
        "so' a raia de CONTROLE anda para fora do patamar"
    );
}
