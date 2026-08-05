//! **O que um BAKE de player grava hoje** (W16, a medição que abriu a wave).
//!
//! `cargo test -p ph2d-physics-ecs --test measure_bake_player -- --ignored --nocapture`
//!
//! ⚠️ **Sonda, não gate.** Ela existe para o defeito ter um número antes de
//! qualquer linha ser escrita — e para a nota que ficar não ser uma suspeita.

#[path = "platform_crouch_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::{FrozenScene, InputTape, PlayerInput, bake};
use rig_fixture::{crouch_right, pose, rig, walk_right};

const TICKS: u64 = 90;
const DT: f64 = 1.0 / 60.0;

/// A corrida que o artista deu: andar para a direita, e pular no meio.
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

/// **O que a corrida FOI, contra o que o bake ESCREVE.**
#[test]
#[ignore]
fn measure_what_a_player_bake_records() {
    // ── (a) a corrida de verdade, dirigida pela FITA ─────────────────────────
    let mut live = rig(0.0, None);
    let mut tp = performance();
    for k in 1..=TICKS {
        live.bridge
            .dispatch_with_tape(&mut live.sim, true, k, &mut tp);
    }
    let (lx, ly) = pose(&live.sim);

    // ── (b) o que o bake grava, pela porta que o produto usa ────────────────
    let mut baked = rig(0.0, None);
    let player = baked.player;
    let trajs = bake::bake_trajectories_with_scene(
        &mut baked.bridge,
        &mut baked.sim,
        &[player],
        TICKS,
        DT,
        &mut FrozenScene,
    );
    let t = &trajs[0];
    let xs: Vec<f32> = t.samples.iter().map(|s| s.1).collect();
    let ys: Vec<f32> = t.samples.iter().map(|s| s.2).collect();
    let span = |v: &[f32]| {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for &n in v {
            lo = lo.min(n);
            hi = hi.max(n);
        }
        hi - lo
    };

    eprintln!("== a corrida contra o que o bake escreve ==");
    eprintln!("  a FITA leva o personagem a x={lx:6.3} y={ly:6.3} em {TICKS} tiques");
    eprintln!(
        "  o BAKE grava x de {:6.3} a {:6.3} (amplitude {:6.3})",
        xs.first().copied().unwrap_or_default(),
        xs.last().copied().unwrap_or_default(),
        span(&xs)
    );
    eprintln!(
        "  o BAKE grava y de {:6.3} a {:6.3} (amplitude {:6.3})",
        ys.first().copied().unwrap_or_default(),
        ys.last().copied().unwrap_or_default(),
        span(&ys)
    );
    eprintln!(
        "  canal X constante? {}   canal Y constante? {}",
        t.channel(bake::PoseChannel::X).is_none(),
        t.channel(bake::PoseChannel::Y).is_none()
    );
}

/// **E o que o DEDO DE AGORA faz ao bake** — a metade PIOR do defeito.
///
/// O caminho sem fita dirige os players pelo `player_input` retido, ou seja
/// *pelo que o artista está a segurar no instante do clique*. Então o modo de
/// falha não é *"nada acontece"* — é **"o bake grava o que quer que você esteja
/// a segurar"**, que às vezes parece certo e por isso é pior.
///
/// ⚠️ Esta célula segura **ESQUERDA** enquanto a fita diz DIREITA. Se ela
/// segurasse direita, o número sairia igual ao da corrida **por acidente da
/// fixture**, e a sonda mostraria um defeito a parecer correcto.
#[test]
#[ignore]
fn measure_what_the_finger_of_now_bakes() {
    let mut r = rig(0.0, None);
    let player = r.player;
    // O dedo de AGORA: para a ESQUERDA, contra a corrida que a fita descreve.
    r.bridge.set_player_input(
        player,
        PlayerInput {
            drive: -1.0,
            ..PlayerInput::default()
        },
    );
    let trajs = bake::bake_trajectories_with_scene(
        &mut r.bridge,
        &mut r.sim,
        &[player],
        TICKS,
        DT,
        &mut FrozenScene,
    );
    let xs: Vec<f32> = trajs[0].samples.iter().map(|s| s.1).collect();
    eprintln!("== o dedo de AGORA, segurando ESQUERDA durante o bake ==");
    eprintln!(
        "  o BAKE grava x de {:6.3} a {:6.3}",
        xs.first().copied().unwrap_or_default(),
        xs.last().copied().unwrap_or_default()
    );

    let mut q = rig(0.0, None);
    let qp = q.player;
    q.bridge.set_player_input(qp, PlayerInput::default());
    let quiet = bake::bake_trajectories_with_scene(
        &mut q.bridge,
        &mut q.sim,
        &[qp],
        TICKS,
        DT,
        &mut FrozenScene,
    );
    let qxs: Vec<f32> = quiet[0].samples.iter().map(|s| s.1).collect();
    eprintln!(
        "  com o dedo PARADO: x de {:6.3} a {:6.3}",
        qxs.first().copied().unwrap_or_default(),
        qxs.last().copied().unwrap_or_default()
    );
    let _ = (crouch_right, walk_right);
}

/// **O que a CAUDA de um bake faz** — passado o fim da gravação.
///
/// ⚠️ Esta célula existe porque a minha primeira hipótese sobre ela estava
/// ERRADA: eu escrevi um gate afirmando que a cauda *"segue o dedo"*, e a
/// mutação não sangrou. A causa é que o `take_taped_input` **não restaura** a
/// entrada segurada quando a fita cala — ela já foi sobrescrita pelo primeiro
/// tique gravado. Logo a cauda **repete o ÚLTIMO tique da gravação**, para
/// sempre.
#[test]
#[ignore]
fn measure_what_the_tail_of_a_bake_does() {
    const RECORDED: u64 = 30;
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
    let mut r = rig(0.0, None);
    let player = r.player;
    // O dedo de AGORA aponta para a ESQUERDA, o oposto da gravação.
    r.bridge.set_player_input(
        player,
        PlayerInput {
            drive: -1.0,
            ..PlayerInput::default()
        },
    );
    let trajs = bake::bake_trajectories_with_scene_and_tape(
        &mut r.bridge,
        &mut r.sim,
        &[player],
        TICKS,
        DT,
        &mut FrozenScene,
        &mut short,
    );
    let xs: Vec<f32> = trajs[0].samples.iter().map(|s| s.1).collect();
    eprintln!("== a cauda, com a gravacao a acabar no tique {RECORDED} ==");
    eprintln!(
        "  x no fim da fita = {:6.3}   x no fim do bake = {:6.3}",
        xs[RECORDED as usize],
        xs.last().copied().unwrap_or_default()
    );
}
