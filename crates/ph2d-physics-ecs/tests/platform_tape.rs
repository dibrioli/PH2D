//! **A FITA** (W7) — o controlador vira função de `(tick, fita)`.
//!
//! A pergunta é a que o resto deste módulo já responde e que o player tinha
//! quebrado: *arrastar a régua para trás e para a frente devolve a MESMA
//! corrida?*

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_physics_ecs::{HeldInput, InputTape, PlayerInput, PlayerInputAtTick};
use scene_fixture::{pose, scene};

/// Uma fita roteirizada: anda para a direita, pula no meio, e para no fim.
///
/// ⚠️ **Determinística por construção** — é o que a torna utilizável no C9 e no
/// gate: uma fita gravada de um teclado descreveria uma corrida que ninguém
/// consegue repetir.
fn scripted(ticks: u64) -> InputTape {
    let mut tape = InputTape::new();
    for t in 1..=ticks {
        tape.record(
            t,
            PlayerInput {
                drive: if t < 90 { 1.0 } else { 0.0 },
                jump: (40..48).contains(&t),
            },
        );
    }
    tape
}

/// **O SCRUB replaya o player** — a correção de bug da wave.
///
/// ⚠️ O laço de replay do `rewind` dirigia as poses da cena e **nunca chamava
/// `drive_players`**: o personagem caía pelos ticks replayados e parava onde a
/// gravidade o deixasse.
///
/// ⚠️ **A fixture tem de VOLTAR PARA O MEIO, e a 1ª versão não voltava.** Ela
/// scrubbava para o tique **0**, onde o laço de replay roda **zero** passos —
/// então a mutação que apaga o `drive_players` do replay **sobrevivia**, sobre
/// um gate verde escrito exatamente para a pegar. Um alvo no meio replaya de
/// verdade.
///
/// As DUAS rotas do `rewind_to` são exercitadas, e não por simetria: com o ring
/// quente ele **semeia** de um tique âncora e replaya poucos passos (é aí que o
/// `seed_jump_states` importa); com o ring vazio ele **reconstrói do repouso** e
/// replaya o run inteiro.
#[test]
fn scrubbing_back_to_the_middle_replays_the_player() {
    // ⚠️ **58 é MEDIDO, não escolhido: o alvo tem de cair DENTRO do arco.**
    // Varrendo 42..75 com o seed de estado de pulo desarmado, a divergência
    // cresce até **0,151 no tique 58** e some a partir do 62 — o arco RE-CONVERGE
    // (o pouso é o mesmo com ou sem o erro), exatamente como a pilha amortecida
    // do W1.5 esquece a perturbação. A 1ª versão deste gate mirava o tique 75,
    // depois da convergência, e a mutação do seed **sobrevivia**.
    const MID: u64 = 58;
    let straight = {
        let (mut sim, mut bridge, _p) = scene(0.0, 0.0);
        let mut tape = scripted(200);
        for t in 1..=MID {
            bridge.dispatch_with_tape(&mut sim, true, t, &mut tape);
        }
        pose(&sim)
    };
    assert!(
        straight.0 > 2.0,
        "a corrida tem de ter ANDADO, senao o gate compara dois personagens parados: {straight:?}"
    );

    for drop_ring in [false, true] {
        let (mut sim, mut bridge, _p) = scene(0.0, 0.0);
        let mut tape = scripted(200);
        for t in 1..=120 {
            bridge.dispatch_with_tape(&mut sim, true, t, &mut tape);
        }
        if drop_ring {
            // A rota de FALLBACK: sem âncora em cache, o rewind reconstrói do
            // repouso e replaya o run inteiro.
            bridge.forget_checkpoints();
        }
        bridge.dispatch_with_tape(&mut sim, true, MID, &mut tape);
        let scrubbed = pose(&sim);
        eprintln!(
            "ring {}: direto {straight:?} · scrub {scrubbed:?}",
            if drop_ring { "VAZIO" } else { "quente" }
        );
        assert!(
            (straight.0 - scrubbed.0).abs() < 0.05 && (straight.1 - scrubbed.1).abs() < 0.05,
            "o scrub tem de reproduzir a corrida (ring vazio={drop_ring}): \
             {straight:?} contra {scrubbed:?}"
        );
    }
}

/// ⚠️ **Sem fita nada muda** — a regressão da wave.
///
/// `HeldInput` é o caminho de todo chamador que não grava, e ele tem de dar a
/// MESMA trajetória do `dispatch` de sempre, ao bit.
#[test]
fn without_a_tape_the_world_is_byte_identical() {
    fn run(with_tape: bool) -> (f32, f32) {
        let (mut sim, mut bridge, player) = scene(0.0, 0.0);
        for t in 1..=90 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive: 1.0,
                    jump: false,
                },
            );
            if with_tape {
                bridge.dispatch_with_tape(&mut sim, true, t, &mut HeldInput);
            } else {
                bridge.dispatch(&mut sim, true, t);
            }
        }
        pose(&sim)
    }
    let with = run(true);
    let without = run(false);
    // ⚠️ **A igualdade sozinha é uma RAZÃO ENTRE DOIS DOENTES, e a mutação
    // provou:** o `dispatch` de sempre delega ao mesmo laço passando `HeldInput`,
    // então uma mutação que faz a fita escrever `PlayerInput::default()` em vez
    // de não tocar em nada PARA OS DOIS lados — e o `assert_eq!` continua verde
    // sobre dois personagens imóveis. O oráculo tem de dizer que a corrida
    // ACONTECEU.
    assert!(
        with.0 > 2.0,
        "o caminho com fita tem de ANDAR, senao a igualdade compara dois parados: {with:?}"
    );
    assert_eq!(
        with, without,
        "o caminho com `HeldInput` tem de ser byte-identico ao `dispatch` de sempre"
    );
}

/// **A fita MANDA na entrada segurada** — senão ela seria decoração.
#[test]
fn the_tape_overrides_what_the_caller_is_holding() {
    let (mut sim, mut bridge, player) = scene(0.0, 0.0);
    // O chamador segura "para a DIREITA"...
    bridge.set_player_input(
        player,
        PlayerInput {
            drive: 1.0,
            jump: false,
        },
    );
    // ...e a fita diz "para a ESQUERDA".
    let mut tape = InputTape::new();
    for t in 1..=90 {
        tape.record(
            t,
            PlayerInput {
                drive: -1.0,
                jump: false,
            },
        );
    }
    for t in 1..=90 {
        bridge.dispatch_with_tape(&mut sim, true, t, &mut tape);
    }
    let (x, _) = pose(&sim);
    eprintln!("com a fita mandando para a esquerda: x={x:.3}");
    assert!(
        x < -1.0,
        "a fita tem de vencer a entrada segurada: x={x:.3}"
    );
}

/// A fita responde o que gravou, e `None` fora do alcance.
#[test]
fn a_tape_answers_only_for_the_ticks_it_holds() {
    let mut tape = InputTape::new();
    tape.record(
        10,
        PlayerInput {
            drive: 0.5,
            jump: true,
        },
    );
    assert_eq!(
        tape.input(9),
        None,
        "antes do comeco ela nao tem nada a dizer"
    );
    assert_eq!(
        tape.input(10),
        Some(PlayerInput {
            drive: 0.5,
            jump: true
        })
    );
    assert_eq!(tape.input(11), None, "e depois do fim tambem nao");

    // ⚠️ Gravar à FRENTE preenche o vão com a última entrada: o dedo não muda
    // de posição enquanto ninguém olha.
    tape.record(
        13,
        PlayerInput {
            drive: -1.0,
            jump: false,
        },
    );
    assert_eq!(
        tape.input(12).map(|i| i.drive),
        Some(0.5),
        "o vao herda a ultima entrada"
    );
    assert_eq!(tape.len(), 4);

    // E regravar SOBRESCREVE — o artista está autorando por cima.
    tape.record(
        10,
        PlayerInput {
            drive: -0.25,
            jump: false,
        },
    );
    assert_eq!(tape.input(10).map(|i| i.drive), Some(-0.25));
}
