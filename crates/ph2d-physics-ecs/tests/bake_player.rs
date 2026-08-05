//! **ASSAR UM PLAYER** (W16) — o bake reproduz a CORRIDA GRAVADA, não o dedo.
//!
//! O item estava no §4 do plano 06 desde o começo, com a data marcada
//! (*"desbloqueado desde a W7 — com a fita, assar passa a fazer sentido"*), e a
//! medição que abriu a wave está no `measure_bake_player`.

#[path = "platform_crouch_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::{FrozenScene, InputTape, PlayerInput, bake};
use rig_fixture::{pose, rig};

const TICKS: u64 = 90;
const DT: f64 = 1.0 / 60.0;

/// A corrida gravada: andar para a DIREITA, e pular no meio.
fn performance() -> InputTape {
    let mut t = InputTape::new();
    for k in 1..=TICKS {
        t.record(
            k,
            PlayerInput {
                drive: 1.0,
                jump: (30..38).contains(&k),
                ..PlayerInput::default()
            },
        );
    }
    t
}

/// Andar para a ESQUERDA — o dedo que segura o oposto da corrida.
fn hold_left() -> PlayerInput {
    PlayerInput {
        drive: -1.0,
        ..PlayerInput::default()
    }
}

fn bake_x(tape: &mut InputTape, held: PlayerInput) -> Vec<f32> {
    let mut r = rig(0.0, None);
    let player = r.player;
    r.bridge.set_player_input(player, held);
    let trajs = bake::bake_trajectories_with_scene_and_tape(
        &mut r.bridge,
        &mut r.sim,
        &[player],
        TICKS,
        DT,
        &mut FrozenScene,
        tape,
    );
    trajs[0].samples.iter().map(|s| s.1).collect()
}

/// **O bake replaya a FITA, não o dedo de agora** — a wave inteira num número.
///
/// ⚠️ **O dedo segura ESQUERDA de propósito.** Se ele segurasse direita, o
/// resultado sairia igual ao da corrida **por acidente da fixture**, e o gate
/// ficaria verde sobre o defeito — que é exactamente o modo de falha desta
/// classe: o caminho sem fita não faz *nada*, ele grava *o que quer que esteja
/// pressionado*, e às vezes isso parece certo.
///
/// ⚠️ **Mutação medida:** trocar o `dispatch_with_scene_and_tape` do bake pelo
/// `dispatch_with_scene` (o caminho de antes desta wave) grava **x = −8,765** —
/// o espelho exacto da corrida.
#[test]
fn a_player_bake_replays_the_tape_not_the_finger() {
    let xs = bake_x(&mut performance(), hold_left());
    let last = *xs.last().expect("o bake grava amostras");
    assert!(
        last > 5.0,
        "o bake seguiu o DEDO em vez da fita: x terminou em {last:.3}"
    );
}

/// **E o que ele grava É a corrida** — o endpoint bate com o da corrida ao vivo.
///
/// ⚠️ Este é o gate de FIDELIDADE, e ele é distinto do de cima: aquele prova que
/// a fita venceu o dedo, este prova que a fita foi reproduzida *inteira*. Uma
/// fita lida com um tique de atraso passaria no primeiro.
#[test]
fn a_player_bake_reproduces_the_recorded_run() {
    let mut live = rig(0.0, None);
    let mut tp = performance();
    for k in 1..=TICKS {
        live.bridge
            .dispatch_with_tape(&mut live.sim, true, k, &mut tp);
    }
    let (lx, _) = pose(&live.sim);

    let xs = bake_x(&mut performance(), PlayerInput::default());
    let last = *xs.last().expect("o bake grava amostras");
    assert!(
        (last - lx).abs() < 0.01,
        "o bake gravou {last:.3} para uma corrida que foi a {lx:.3}"
    );
    assert!(
        lx > 5.0,
        "a fixture tem de ter ANDADO, senao compara dois parados"
    );
}

/// **O canal X deixa de ser CONSTANTE** — a consequência que o artista vê.
///
/// ⚠️ Sem a fita, `channel(X)` devolvia `None` e **nenhuma track horizontal era
/// escrita**: o artista assava uma corrida de nove metros e recebia uma curva só
/// de Y. É um defeito que não parece um defeito — parece uma feature que não
/// gravou nada.
#[test]
fn the_horizontal_channel_of_a_baked_run_is_not_constant() {
    let mut r = rig(0.0, None);
    let player = r.player;
    let trajs = bake::bake_trajectories_with_scene_and_tape(
        &mut r.bridge,
        &mut r.sim,
        &[player],
        TICKS,
        DT,
        &mut FrozenScene,
        &mut performance(),
    );
    assert!(
        trajs[0].channel(bake::PoseChannel::X).is_some(),
        "o canal X saiu CONSTANTE: nenhuma track horizontal seria escrita"
    );
}

/// **PASSADO o fim da gravação o personagem PARA** — o adaptador `RecordedRun`.
///
/// ⚠️ **É a metade do defeito que sobreviveria a uma correção ingénua:** a fita
/// devolve `None` fora do alcance dela, e `None` quer dizer *"use a segurada"* —
/// certo AO VIVO (o artista ainda joga) e errado num BAKE (a corrida acabou).
///
/// ⚠️ **E a minha primeira versão deste gate afirmava a coisa ERRADA.** Ela dizia
/// que sem o adaptador a cauda *"segue o dedo"*, e a mutação **não sangrou** — o
/// `take_taped_input` sobrescreve a entrada retida no primeiro tique gravado e
/// **não a restaura** ao calar, então a cauda repete o **ÚLTIMO TIQUE DA
/// GRAVAÇÃO**, para sempre. O dedo só manda quando a fita está *vazia*, que é
/// outro caso (e é o que o primeiro gate deste arquivo mede). Duas causas, dois
/// números, e a minha previsão cobria a que não estava a acontecer.
///
/// ⚠️ **Mutação medida:** tirar o `RecordedRun` faz a gravação que acabou em
/// `x = 2,765` terminar o bake em **`8,765`** — seis metros que ninguém jogou.
/// Com ele, a cauda desacelera e para em **`2,935`**.
#[test]
fn past_the_recorded_run_the_bake_goes_idle() {
    /// A gravação cobre só o primeiro terço do alcance assado.
    const RECORDED: u64 = 30;
    /// Quanto a inércia da caminhada ainda leva o personagem depois de a
    /// gravação calar — MEDIDO em 0,17 m, com folga para o dobro.
    const COAST: f32 = 0.4;

    let mut short = InputTape::new();
    for k in 1..=RECORDED {
        short.record(
            k,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
    }
    let xs = bake_x(&mut short, hold_left());
    let at_end_of_tape = xs[RECORDED as usize];
    let last = *xs.last().expect("o bake grava amostras");
    assert!(
        at_end_of_tape > 1.0,
        "a fixture tem de ter ANDADO durante a gravacao: x={at_end_of_tape:.3}"
    );
    assert!(
        (last - at_end_of_tape) < COAST,
        "a cauda continuou a andar: x foi de {at_end_of_tape:.3} para {last:.3}          depois de a gravacao acabar"
    );
    // ⚠️ E a outra ponta: ele nao pode andar para TRAS. Sem esta metade, um
    // adaptador que travasse o corpo passaria — e travar nao e' o que "o artista
    // parou de jogar" significa.
    assert!(
        last > at_end_of_tape - 0.05,
        "a cauda andou para tras: {at_end_of_tape:.3} -> {last:.3}"
    );
}
