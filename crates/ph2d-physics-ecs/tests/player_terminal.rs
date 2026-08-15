//! **O TETO DE QUEDA, pela porta do produto** (`W-Fall`) — a lei, a ponte e o
//! solver juntos, nos DOIS modos.
//!
//! ⚠️ **O oráculo é a FORMA da sequência, nunca um campo de estado:** uma
//! velocidade terminal é uma queda que **PARA de acelerar**, e o que se afirma é
//! que a descida por segundo deixa de crescer — com o **CONTROLE** ao lado, a
//! mesma cena sem teto, a crescer sem parar. Um gate sobre um booleano ficaria
//! verde com o corpo a cair como sempre.
//!
//! ⚠️ **O modo é um PAR, não um componente.** A porta é o `pose_owner`, e ela
//! pergunta ao KIND do corpo que a ponte de facto construiu antes de olhar para
//! o `PlayerMode` — *é o corpo que existe que importa, não o que foi pedido*.
//! Inserir só o componente sobre um corpo `Dynamic` deixa as duas colunas a
//! medir o MESMO caminho, e foi o que o controlo positivo da sonda apanhou
//! (`tests/measure_terminal.rs`).

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, PhysicsBridge, PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};
use scene_fixture::{pose, scene};

/// De quantos metros se larga o personagem.
///
/// ⚠️ **ARITMÉTICA, não gosto:** uma queda livre percorre `½·g·t²`, então os
/// cinco segundos que estes gates medem custam **123 m**. Se o corpo POUSAR
/// dentro da janela, as últimas colunas descrevem o CHÃO e leem-se como
/// *"assentou"* — a fixture a mentir exactamente sobre a pergunta que ela existe
/// para responder. Com 400 m a janela inteira é queda, com folga.
const DROP: f32 = 400.0;

/// Quantos segundos cada gate observa.
const SECS: usize = 5;

/// Uma cena plana com o personagem no ar, o teto autorado (`0.0` = desligado) e
/// o planeio autorado (`0.0` = desligado).
fn dropped(mode: Option<PlayerMode>, cap: f32, glide: f32) -> (SimWorld, PhysicsBridge, Entity) {
    let (mut sim, bridge, player) = scene(0.0, 0.0);
    if let Some(m) = mode {
        sim.world_mut().entity_mut(player).insert((
            m,
            RigidBody {
                kind: BodyKind::Kinematic,
            },
        ));
    }
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.max_fall_speed = cap;
        p.glide_fall_speed = glide;
    }
    let y = pose(&sim).1;
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(player) {
        t.translation.y = y + DROP;
    }
    (sim, bridge, player)
}

/// Deixa cair e devolve quantos metros o corpo desceu em CADA segundo.
///
/// ⚠️ **A velocidade é DERIVADA da pose entre tiques, e não lida de um campo** —
/// os dois modos guardam o estado em sítios diferentes (o corpo do rapier num, o
/// `KinematicState` noutro), e uma sonda que perguntasse a um deles estaria a
/// medir modos diferentes com réguas diferentes.
fn descent_per_second(mode: Option<PlayerMode>, cap: f32, glide: f32, held: bool) -> Vec<f32> {
    let (mut sim, mut bridge, player) = dropped(mode, cap, glide);
    let mut out = Vec::with_capacity(SECS);
    let mut tick = 0_u64;
    for _ in 0..SECS {
        let before = pose(&sim).1;
        for _ in 0..60 {
            tick += 1;
            bridge.set_player_input(
                player,
                PlayerInput {
                    jump: held,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, tick);
        }
        out.push(before - pose(&sim).1);
    }
    out
}

/// Os dois modos, com o nome que o smoke usa.
const MODES: [(Option<PlayerMode>, &str); 2] =
    [(None, "Spring"), (Some(PlayerMode::Kinematic), "Snap")];

/// **A queda com teto ASSENTA, e a sem teto NÃO** — nos dois modos, e sem que o
/// jogador toque em nada.
///
/// ⚠️ **O CONTROLE está dentro do mesmo gate de propósito:** *"a descida parou de
/// crescer"* só quer dizer alguma coisa ao lado de uma queda que continua a
/// crescer. Sem ele, a mesma asserção passaria numa cena em que nada cai.
///
/// ⚠️ **E a barra não é um `==`:** o freio é aplicado no topo do tique e a
/// gravidade soma **dentro** dele, então a descida assenta uns 6% acima do
/// número autorado (ver o topo de `ph2d_platformer::descent`). Um gate de
/// igualdade exacta nasceria vermelho sobre produto correto.
#[test]
fn a_capped_fall_settles_and_an_uncapped_one_does_not() {
    const CAP: f32 = 10.0;
    for (mode, tag) in MODES {
        let capped = descent_per_second(mode, CAP, 0.0, false);
        let free = descent_per_second(mode, 0.0, 0.0, false);

        let last = capped[SECS - 1];
        assert!(
            (CAP..CAP * 1.15).contains(&last),
            "[{tag}] a queda capada tem de assentar no teto de {CAP}: {capped:?}"
        );
        assert!(
            capped[SECS - 1] - capped[SECS - 2] < 0.05,
            "[{tag}] a queda capada ainda esta' a acelerar: {capped:?}"
        );

        // O CONTROLE: sem teto, o ultimo segundo e' muito mais fundo que o
        // primeiro, e cada segundo cresce ~g.
        assert!(
            free[SECS - 1] > free[0] + 30.0,
            "[{tag}] o CONTROLE sem teto tinha de continuar a acelerar: {free:?}"
        );
    }
}

/// **O teto de queda não pergunta nada ao jogador** — é o discriminante entre as
/// duas leis, e o que faz dele uma velocidade terminal em vez de uma segunda
/// assistência: o planeio existe enquanto o dedo dura, este vale sempre.
#[test]
fn the_cap_holds_with_the_finger_up_and_down() {
    const CAP: f32 = 12.0;
    for (mode, tag) in MODES {
        for held in [false, true] {
            let d = descent_per_second(mode, CAP, 0.0, held);
            assert!(
                (CAP..CAP * 1.15).contains(&d[SECS - 1]),
                "[{tag}] dedo {held}: a queda tem de assentar em {CAP}: {d:?}"
            );
        }
    }
}

/// **Com as duas leis vivas, vence a MENOR — e é uma porta só** (ver
/// `ph2d_platformer::descent_ceiling`).
///
/// ⚠️ **Este é o gate de PRODUTO da composição**, e ele afirma os dois sentidos:
/// com o dedo em baixo manda o planeio (mais apertado), com o dedo em cima sobra
/// o teto — que continua vivo. Um `max` acidental passaria por metade de um gate
/// que só medisse um deles, e a consequência não seria cosmética: o planeio
/// ganharia o poder de **acelerar** uma queda que o teto já tinha limitado.
#[test]
fn the_glide_tightens_the_fall_and_letting_go_leaves_the_cap() {
    const CAP: f32 = 20.0;
    const GLIDE: f32 = 3.0;
    for (mode, tag) in MODES {
        let held = descent_per_second(mode, CAP, GLIDE, true);
        let free = descent_per_second(mode, CAP, GLIDE, false);
        assert!(
            (GLIDE..GLIDE * 1.35).contains(&held[SECS - 1]),
            "[{tag}] segurar o pulo tem de travar no planeio {GLIDE}: {held:?}"
        );
        assert!(
            (CAP..CAP * 1.15).contains(&free[SECS - 1]),
            "[{tag}] sem o dedo sobra o teto {CAP}, nunca o ilimitado: {free:?}"
        );
    }
}
