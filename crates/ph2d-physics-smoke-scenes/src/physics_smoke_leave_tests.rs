//! Os gates da cena 118 (`W-Leave`) — o ELEVADOR, medido nesta geometria.
//!
//! ⚠️ **A cena inteira é um contraste**, então os gates correm as TRÊS raias: um
//! gate que só afirmasse *"o da esquerda mal sobe"* passaria numa cena em que
//! ninguém pula.
//!
//! ⚠️ **E o número que a mensagem promete é MEDIDO aqui**, não escolhido: a
//! velocidade do elevador é uma velocidade terminal (`v = g/d`), e uma cena que
//! anunciasse 4 m/s com o arrasto errado ensinaria ao artista um número que ela
//! não entrega.

use super::{JUMP_HEIGHT, LANES, LIFT_SPEED, lane_x};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// A cena montada, com o relógio pronto a andar.
fn rig() -> (SimWorld, PhysicsBridge, Vec<(Entity, String)>) {
    let mut sim = SimWorld::new();
    let _ = super::build_leave_scene(sim.world_mut());
    let names: Vec<String> = LANES.iter().map(|(t, _)| format!("Rider {t}")).collect();
    let mut out: Vec<(Entity, String)> = {
        let mut q = sim.world_mut().try_query::<(Entity, &Name)>().unwrap();
        q.iter(sim.world())
            .filter(|(_, n)| names.iter().any(|w| w == n.as_str()))
            .map(|(e, n)| (e, n.as_str().to_string()))
            .collect()
    };
    // A ordem das RAIAS, e não a de spawn — as colunas do relatório têm de
    // corresponder ao que o artista vê da esquerda para a direita.
    out.sort_by_key(|(_, n)| names.iter().position(|w| w == n).unwrap_or(usize::MAX));
    (sim, PhysicsBridge::new(), out)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .map(|t| t.translation.y)
        .expect("o personagem tem de existir")
}

fn lift_y(sim: &SimWorld, lane: usize) -> f32 {
    let want = format!("Lift {}", LANES[lane].0);
    let mut q = sim
        .world()
        .try_query::<(&Name, &Transform)>()
        .expect("query");
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == want)
        .map(|(_, t)| t.translation.y)
        .expect("o elevador tem de existir")
}

/// Deixa a cena correr `settle` tiques, **pula** em todas as raias e devolve o
/// pico de cada uma acima do ponto de partida.
fn peaks(settle: u64, flight: u64) -> Vec<f32> {
    let (mut sim, mut bridge, ids) = rig();
    let mut tick = 0u64;
    for _ in 0..settle {
        tick += 1;
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, tick);
    }
    let y0: Vec<f32> = ids.iter().map(|(e, _)| y_of(&sim, *e)).collect();
    let mut best = vec![0.0f32; ids.len()];
    for _ in 0..flight {
        tick += 1;
        for (e, _) in &ids {
            bridge.set_player_input(
                *e,
                PlayerInput {
                    jump: true,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        for (i, (e, _)) in ids.iter().enumerate() {
            best[i] = best[i].max(y_of(&sim, *e) - y0[i]);
        }
    }
    best
}

/// **O ELEVADOR DESCE À VELOCIDADE QUE A CENA ANUNCIA.**
///
/// ⚠️ A mensagem promete `4,00 m/s` e o número é uma **velocidade terminal**
/// (`v = g/d`), não uma pose escrita à mão — se o arrasto se mover, a cena passa
/// a ensinar um número que ela não entrega, e é isto que o pega.
#[test]
fn the_lift_descends_at_the_speed_the_scene_claims() {
    let (mut sim, mut bridge, ids) = rig();
    for t in 1..=90u64 {
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let before = lift_y(&sim, 0);
    for t in 91..=120u64 {
        for (e, _) in &ids {
            bridge.set_player_input(*e, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, t);
    }
    let speed = (before - lift_y(&sim, 0)) * 2.0; // meio segundo
    assert!(
        (speed - LIFT_SPEED).abs() < 0.15,
        "o elevador tem de descer aos {LIFT_SPEED:.2} m/s que a mensagem promete: {speed:.4}"
    );
}

/// **O PASSO 1 DO ROTEIRO: a raia da esquerda quase não sai do elevador.**
///
/// ⚠️ **O oráculo é a comparação entre raias, não um limiar absoluto** — a mesma
/// cena, a mesma autoria, e só a política difere; um número solto não
/// distinguiria *"a política funciona"* de *"ninguém pulou"*.
#[test]
fn the_left_lane_barely_leaves_the_descending_lift() {
    let p = peaks(90, 30);
    let (full, up_only, nothing) = (p[0], p[1], p[2]);
    assert!(
        up_only > full * 3.0,
        "a raia Up Only tem de subir MUITO mais que a Full ({full:.4} contra {up_only:.4})"
    );
    assert!(
        nothing > full * 3.0,
        "a raia Nothing tambem ({full:.4} contra {nothing:.4})"
    );
    // ...e as duas entregam perto da altura AUTORADA, que e' a promessa inteira.
    for (tag, got) in [("Up Only", up_only), ("Nothing", nothing)] {
        assert!(
            got > JUMP_HEIGHT * 0.6,
            "{tag} tem de entregar perto dos {JUMP_HEIGHT:.2} m autorados em meio \
             segundo de voo: {got:.4}"
        );
    }
}

/// **O PASSO 3 DO ROTEIRO: no chão PARADO as três raias são a mesma.**
///
/// ⚠️ É este gate que prova que a política não alcança chão que não se move — e
/// é a metade que o roteiro manda o artista conferir com os olhos.
#[test]
fn on_the_still_floor_the_three_lanes_jump_alike() {
    // 12 m a 4 m/s = 3 s; 300 tiques dão folga para os três assentarem.
    let p = peaks(300, 30);
    for i in 1..LANES.len() {
        assert!(
            (p[i] - p[0]).abs() < 0.02,
            "no chao PARADO a raia {} tem de pular como a Full ({:.4} contra {:.4})",
            LANES[i].0,
            p[0],
            p[i]
        );
    }
    assert!(
        p[0] > JUMP_HEIGHT * 0.6,
        "e as tres tem de pular de facto (pico {:.4})",
        p[0]
    );
}

/// **As raias não se tocam** — a geometria de uma não pode alcançar a outra.
#[test]
fn the_lanes_do_not_reach_each_other() {
    assert!(lane_x(1) - lane_x(0) > 10.0);
}

/// **A mensagem descreve a cena que existe.**
#[test]
fn the_scene_prints_the_numbers_it_builds() {
    assert!(super::LEAVE_SMOKE_MESSAGE.contains("O ELEVADOR (W-Leave)"));
    assert!(super::LEAVE_SMOKE_MESSAGE.contains("4.00 m/s"));
    const _: () = assert!(LIFT_SPEED == 4.0);
}

/// **A SONDA que dá os números ao roteiro.**
///
/// Rode: `cargo test -p ph2d-host-desktop --release --bins
/// what_each_lane_does -- --ignored --nocapture`
#[test]
#[ignore = "sonda de dimensionamento"]
fn what_each_lane_does() {
    println!("\n== pico de cada raia (m), meio segundo de voo ==");
    for (label, settle) in [("a DESCER", 90u64), ("no CHAO parado", 300)] {
        let p = peaks(settle, 30);
        println!("  {label}:");
        for (i, (tag, _)) in LANES.iter().enumerate() {
            println!("    {tag:<9}  {:>7.4}", p[i]);
        }
    }
}
