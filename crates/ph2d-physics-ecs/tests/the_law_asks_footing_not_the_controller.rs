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

/// **O TEXTO da ponte** — a porta única do assunto, partilhada com o
/// `the_push_is_our_law` por `#[path]`.
#[path = "player_bridge_source.rs"]
mod src_of;
use src_of::{player_family, player_loop};

/// **O CONTROLE:** o scanner encontra a família e ela tem o que se espera.
///
/// ⚠️ Sem isto um caminho errado deixa as duas afirmações abaixo verdes por
/// vácuo (`0 ocorrências` satisfaz *"no máximo uma"*), que é a forma de gate que
/// não pode falhar pelo motivo que alega.
#[test]
fn the_scanner_reads_the_bridge_it_claims_to_scan() {
    let src = player_family();
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
    let src = player_family();

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

/// **E a LEI recebe a velocidade JÁ absorvida** — a correção de ORDEM de
/// 2026-08-08.
///
/// ⚠️ **Arch-gate porque a falha é de ORDEM, e ordem não é observável num teste
/// de unidade:** o `settle` deixa no estado a queda que o mundo bloqueou e o
/// `kinematic_advance` a apaga — mas entre os dois corre a lei, e enquanto ela
/// lia aquela queda o freio da caminhada a tratava como escorregão, empurrando
/// o personagem morro acima ao longo da NORMAL da rampa. Trocar a chamada por
/// `was.kin.velocity` compila, roda, e a única coisa que muda é para onde o
/// personagem anda ao pousar.
///
/// ⚠️ **E a absorção pergunta ao `footing`, NUNCA ao `was.kin.grounded`.** A 1ª
/// versão deste gate afirmava o CONTRÁRIO (`arm.contains("was.kin.grounded")`,
/// com a prosa a chamar-lhe *"a porta COMPARTILHADA com o integrador"*) — ele
/// **pinava o defeito**, num arquivo cujo próprio nome diz o oposto. As duas
/// perguntas parecem a mesma e não são: a do integrador é *"eu TOQUEI no
/// mundo?"*, respondida pelo tique ANTERIOR, que no tique do CONTATO ainda diz
/// *no ar*; a da lei é o `footing` (K4), e ele já respondeu quando a lei corre,
/// porque o cast lê a pose de DEPOIS do `settle`. Com a pergunta velha o tique
/// do contato entrava na lei com a queda inteira: medido, **22,8 mm** de
/// deslocamento lateral num pouso vertical sobre rampa estática.
///
/// A metade comportamental é o
/// `player_kinematic::the_kinematic_landing_does_not_slide_along_the_ramp_normal`;
/// esta é a que diz **onde** a resposta tem de ser tomada.
#[test]
fn the_law_is_handed_the_velocity_the_ground_already_holds() {
    let src = player_loop();
    let stand = src
        .find("let stand = footing(")
        .expect("a resposta da lei sobre chao tem de existir");
    let bind = src
        .find("let vel = if owner.writes_own_pose()")
        .expect("o slot da velocidade da lei tem de existir");
    let motor = src
        .find("let step = player_motor(")
        .expect("a chamada da lei tem de existir");
    // ⚠️ **A ordem é a correção**, e são DOIS degraus: o chão fresco existe
    // antes de a velocidade ser escolhida, e a velocidade antes de a lei correr.
    assert!(
        stand < bind,
        "o `footing` fresco e' resolvido ANTES de a velocidade ser absorvida \
         (senao a absorcao so' tem a resposta do tique anterior)"
    );
    assert!(
        bind < motor,
        "a velocidade e' escolhida ANTES de a lei correr"
    );
    let arm = &src[bind..motor];
    assert!(
        arm.contains("supported_velocity("),
        "o ramo Snap tem de passar pela porta da absorcao. Trecho:\n{arm}"
    );
    assert!(
        arm.contains("stand.is_some()"),
        "e com a resposta da LEI sobre chao (`footing`), que e' fresca. Trecho:\n{arm}"
    );
    assert!(
        !arm.contains("was.kin.grounded"),
        "e NUNCA com o `grounded` do integrador, que no tique do contato ainda \
         diz `no ar`. Trecho:\n{arm}"
    );
    // ⚠️ **E o PISO da §8.1 é a OUTRA pergunta, resolvida ANTES do braço.**
    //
    // Ele pergunta *"ele já vinha a andar no chão?"* e a resposta certa é
    // justamente a do integrador — a que a asserção acima proíbe para a
    // absorção. As duas moram em bindings separados de propósito: escritas num
    // `if` só dentro do braço, a asserção acima dispara (e disparou), porque o
    // texto daquele ramo é o que prova que a absorção não usa a resposta velha.
    let floor = src
        .find("let floor = if was.kin.grounded")
        .expect("o piso da §8.1 tem de ser resolvido por um binding proprio");
    assert!(
        stand < floor && floor < bind,
        "o piso nasce DEPOIS do chao fresco (precisa da normal) e ANTES da \
         absorcao que o consome"
    );
}
