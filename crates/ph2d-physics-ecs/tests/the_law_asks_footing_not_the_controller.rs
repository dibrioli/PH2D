//! **K4 — a resposta da LEI sobre chão é a `footing`, nos DOIS modos.**
//!
//! O `move_shape` do rapier devolve um `grounded: bool`, e ele é útil: a wave
//! W-KinMove o usa para o INTEGRADOR (*"há algo a segurar-me AGORA?"*, o clamp da
//! gravidade — ver `KinematicState::grounded`). O que ele **não** pode virar é a
//! resposta que o pulo, o perdão do coyote, a caminhada e o agachar consomem:
//! essas leem a [`ph2d_platformer::footing`], e uma segunda resposta a *"estou no
//! chão?"* é a doença que este módulo curou quatro vezes.
//!
//! ⚠️ **Arch-gate, e não gate de unidade, porque a distinção mora numa LINHA de
//! fiação:** os dois valores são `bool`, então trocá-los compila, roda, e a
//! suíte inteira fica verde — o personagem só passa a pular meio segundo cedo
//! demais numa rampa, e ninguém liga isso a este arquivo.

use std::fs;

fn bridge_player() -> String {
    fs::read_to_string(format!(
        "{}/src/bridge/player.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("o arquivo da ponte do player tem de existir")
}

/// **O CONTROLE:** o scanner encontra o arquivo e ele tem o que se espera.
///
/// ⚠️ Sem isto um caminho errado deixa as duas afirmações abaixo verdes por
/// vácuo (`0 ocorrências` satisfaz *"no máximo uma"*), que é a forma de gate que
/// não pode falhar pelo motivo que alega.
#[test]
fn the_scanner_reads_the_bridge_it_claims_to_scan() {
    let src = bridge_player();
    assert!(
        src.contains("player_motor("),
        "o scanner tem de achar a chamada da lei"
    );
    assert!(
        src.contains("kinematic_settle("),
        "e a do assentamento cinematico"
    );
}

/// **A LEI recebe a `footing`, e o `grounded` do controlador não a alcança.**
#[test]
fn the_controllers_grounded_only_reaches_the_integrator() {
    let src = bridge_player();

    // Um só sítio lê o `grounded` do `CharacterMove`, e ele está DENTRO da
    // chamada do assentamento.
    let reads: Vec<_> = src.match_indices("got.grounded").collect();
    assert_eq!(
        reads.len(),
        1,
        "o `grounded` do controlador tem exatamente UM leitor; achei {}",
        reads.len()
    );
    let at = reads[0].0;
    let call = src
        .rfind("kinematic_settle(")
        .expect("o assentamento tem de existir");
    let close = src[call..]
        .find(");")
        .map(|i| call + i)
        .unwrap_or(src.len());
    assert!(
        call < at && at < close,
        "o unico leitor tem de ser o `kinematic_settle` — o INTEGRADOR, nao a lei"
    );

    // E a lei não vê nenhum dos dois: ela recebe a amostra do CAST, de que a
    // `footing` deriva.
    let motor = src
        .find("let step = player_motor(")
        .expect("a chamada da lei tem de existir");
    let motor_end = src[motor..]
        .find("\n            );")
        .map(|i| motor + i)
        .expect("a chamada da lei tem de fechar");
    let args = &src[motor..motor_end];
    assert!(
        !args.contains("grounded"),
        "a lei nao pode receber um `grounded`: ela pergunta a `footing`. Args:\n{args}"
    );
    assert!(
        args.contains("sample.as_ref()"),
        "a lei recebe a AMOSTRA do cast, que e' de onde a `footing` sai. Args:\n{args}"
    );
}
