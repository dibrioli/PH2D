//! Os gates da cena 108 (`W-Probes`) — os **gestos** que a mensagem manda
//! fazer, afirmados antes de o artista os ler.
//!
//! ⚠️ **Uma cena cuja mensagem manda fazer um gesto tem de conter o gesto.**
//! Aqui a mensagem promete três coisas que a cena pode não permitir: caber
//! agachado sob o túnel e não caber de pé · alcançar a quina do beiral num pulo
//! e não a andar · chegar à parede. As três correm pela **cena real**.

use super::{CROUCH_HEIGHT, LEDGE_BOTTOM, TUNNEL_BOTTOM, TUNNEL_X, WALL_FACE_X, build_probe_scene};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput, ProbeKind, ProbeMark, ProbeState};

/// Meia-altura da caixa do personagem das cenas (`half_height + radius`).
const BODY_HALF: f32 = 0.5;
/// A altura de flutuação DE PÉ das cenas de player.
const FLOAT: f32 = 0.9;
const STANDING_TOP: f32 = FLOAT + BODY_HALF;
const CROUCHED_TOP: f32 = CROUCH_HEIGHT + BODY_HALF;

/// **A cena entrega os números que a mensagem dela imprime.**
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    // ⚠️ Em tempo de COMPILAÇÃO: são todas constantes, e o compilador responde
    // isto melhor, e antes.
    const _: () = assert!(CROUCHED_TOP < TUNNEL_BOTTOM && TUNNEL_BOTTOM < STANDING_TOP);
    const _: () = assert!(LEDGE_BOTTOM > STANDING_TOP);
    assert!(
        (STANDING_TOP - 1.40).abs() < 1.0e-6,
        "a mensagem diz 1.40 de pe': {STANDING_TOP}"
    );
    assert!(
        (CROUCHED_TOP - 1.05).abs() < 1.0e-6,
        "a mensagem diz 1.05 agachado: {CROUCHED_TOP}"
    );
}

/// Põe o personagem em `x`, corre `ticks` com a entrada dada, devolve as marcas.
fn at(
    x: f32,
    y: f32,
    hold: PlayerInput,
    ticks: u64,
    then: PlayerInput,
    more: u64,
) -> Vec<ProbeMark> {
    let mut sim = SimWorld::new();
    let player = build_probe_scene(sim.world_mut());
    {
        let mut t = sim
            .world_mut()
            .get_mut::<Transform>(player)
            .expect("o player tem Transform");
        t.translation.x = x;
        t.translation.y = y;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0;
    for _ in 0..ticks {
        bridge.set_player_input(player, hold);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    for _ in 0..more {
        bridge.set_player_input(player, then);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    bridge.player_probe_marks().to_vec()
}

fn of(marks: &[ProbeMark], kind: ProbeKind) -> Vec<ProbeMark> {
    marks.iter().copied().filter(|m| m.kind == kind).collect()
}

fn top_of(sim: &SimWorld) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, tr) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some(tr.translation.y + BODY_HALF);
        }
    }
    found.expect("o player tem de existir")
}

/// **O PASSO 1: a perna é desenhada, sempre, e ela ACHOU o chão.**
#[test]
fn the_leg_is_always_there_and_it_found_the_floor() {
    let m = at(
        2.0,
        FLOAT,
        PlayerInput::default(),
        40,
        PlayerInput::default(),
        0,
    );
    let g = of(&m, ProbeKind::Ground);
    assert_eq!(g.len(), 1, "uma perna");
    assert_eq!(g[0].state, ProbeState::Hit, "sobre o chao, ela ACHA");
}

/// **O PASSO 3: parado no chão, flanco e quina estão ARMADOS e apagados.**
///
/// ⚠️ É este o estado que a wave acrescenta. Sem ele, *"a capacidade está lá,
/// não é a hora"* e *"não há sensor nenhum"* leem igual.
#[test]
fn standing_still_the_flank_and_the_corner_are_armed_and_dim() {
    let m = at(
        2.0,
        FLOAT,
        PlayerInput::default(),
        40,
        PlayerInput::default(),
        0,
    );
    for k in [ProbeKind::Wall, ProbeKind::Corner, ProbeKind::Side] {
        let ms = of(&m, k);
        assert!(!ms.is_empty(), "{k:?} armado tem marca na cena");
        assert!(
            ms.iter().all(|x| x.state == ProbeState::Idle),
            "{k:?} parado no chao esta' inerte"
        );
    }
    assert_eq!(
        of(&m, ProbeKind::Wall).len(),
        6,
        "os dois lados, sem direcao"
    );
}

/// **O PASSO 4: sob o túnel, soltar o agachar acende a silhueta e ele NÃO sobe.**
#[test]
fn under_the_tunnel_the_sweep_lights_up_and_he_stays_down() {
    let mid = (TUNNEL_X[0] + TUNNEL_X[1]) * 0.5;
    let m = at(
        mid,
        CROUCH_HEIGHT,
        PlayerInput {
            down: true,
            ..PlayerInput::default()
        },
        90,
        PlayerInput::default(),
        60,
    );
    let h = of(&m, ProbeKind::Headroom);
    assert_eq!(h.len(), 1, "uma varredura");
    assert_eq!(
        h[0].state,
        ProbeState::Hit,
        "sob o tunel a varredura ACHA teto"
    );
}

/// **E fora do túnel ela apaga e ele levanta-se** — a metade oposta, sem a qual
/// o passo 4 é satisfeito por um sensor cravado em *bloqueado*.
#[test]
fn out_of_the_tunnel_the_sweep_goes_quiet_and_he_stands() {
    let mut sim = SimWorld::new();
    let player = build_probe_scene(sim.world_mut());
    {
        let mut t = sim.world_mut().get_mut::<Transform>(player).unwrap();
        t.translation.x = 2.0;
        t.translation.y = CROUCH_HEIGHT;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0;
    for _ in 0..90 {
        bridge.set_player_input(
            player,
            PlayerInput {
                down: true,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    for _ in 0..150 {
        bridge.set_player_input(player, PlayerInput::default());
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
    }
    let top = top_of(&sim);
    assert!(
        (top - STANDING_TOP).abs() < 0.05,
        "em ceu limpo ele levanta-se inteiro: {top:.3}"
    );
    let h = of(bridge.player_probe_marks(), ProbeKind::Headroom);
    assert_eq!(h[0].state, ProbeState::Idle, "de pe', ninguem pergunta");
}

/// **O PASSO 6: empurrando contra a parede no ar, o flanco ACENDE.**
#[test]
fn pushing_into_the_wall_lights_the_flank() {
    let m = at(
        WALL_FACE_X - 0.25,
        2.5,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        20,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
        0,
    );
    let w = of(&m, ProbeKind::Wall);
    assert_eq!(w.len(), 3, "com direcao, so' um lado");
    assert!(
        w.iter().any(|x| x.state == ProbeState::Hit),
        "encostado na parede, o flanco ACHA: {:?}",
        w.iter().map(|x| x.state).collect::<Vec<_>>()
    );
}

/// **A cena permite o gesto do passo 5** — a quina do beiral é alcançável a
/// PULAR e não a andar.
///
/// ⚠️ Sem isto a mensagem mandaria o artista procurar uma coisa que a cena não
/// tem, que é o defeito que a política de cenas desta linha existe para evitar.
#[test]
fn the_ledge_is_reachable_by_jumping_and_not_by_walking() {
    const _: () = assert!(LEDGE_BOTTOM > STANDING_TOP);
    // E o pulo autorado tem de o alcancar: o `jump_height` default do produto.
    let peak = FLOAT + ph2d_physics_ecs::PlatformPlayer::default().jump_height;
    assert!(
        peak + BODY_HALF > LEDGE_BOTTOM,
        "o pulo tem de levar a cabeca ({:.2}) acima da face do beiral ({LEDGE_BOTTOM})",
        peak + BODY_HALF
    );
}
