//! **Os gates do canal de VERBO BOOLEANO na pose** — irmão do [`super::tests`] por LOC (HR-18).
//!
//! O que só se pode afirmar aqui é a metade que esta crate possui: *de que verbo para que verbo, e
//! a que altura do caminho*. Quem cozinha a booleana é a shell, e os gates do desenho vivem lá.

use crate::{BoolMorph, Machine, ObjectPose, StateRole, Transition, UiState};
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_vec_scene::VecPathId;

const OPERAND: VecPathId = 7;

/// Uma pose de operando: o verbo próprio dela e o do grupo acima.
fn pose(op: Option<u8>, group_op: Option<u8>) -> ObjectPose {
    ObjectPose {
        bool_op: op,
        bool_group_op: group_op,
        ..ObjectPose::new(OPERAND)
    }
}

/// **O verbo SEGURA na partida, e a troca viaja por fora.**
///
/// ⚠️ As duas metades são um gate só de propósito: a que segura sem a que viaja seria um degrau
/// (o verbo salta na chegada e ninguém desenha o meio), e a que viaja sem a que segura seria um
/// número interpolado entre dois códigos — que dá a operação **errada**, porque o `2` entre
/// `Union` (0) e `Exclude` (3) é `Intersect`, que não está em nenhuma das duas pontas.
#[test]
fn the_verb_holds_at_the_start_and_the_change_travels_beside_it() {
    let tr = Transition::new(
        &[pose(Some(0), Some(0))],
        &[pose(Some(1), Some(0))], // Union -> Subtract, nesta forma
    );

    let mid = tr.at(0.5);
    assert_eq!(
        mid[0].bool_op,
        Some(0),
        "a pose do meio tem de SEGURAR o verbo de partida"
    );

    let morphs = tr.bool_morphs(0.5);
    assert_eq!(morphs.len(), 1, "a troca não foi publicada: {morphs:?}");
    assert_eq!(morphs[0].id, OPERAND);
    assert_eq!(morphs[0].op, Some(1), "o recado tem de nomear a CHEGADA");
    assert!((morphs[0].t - 0.5).abs() < 1e-12);
}

/// **A CHEGADA é exata**: em `t = 1` a pose já traz o verbo de destino, e não há recado nenhum.
///
/// ⚠️ **A segunda metade é a que paga o quadro:** um recado publicado na ponta faria o grupo
/// cozinhar DUAS vezes e casar um `Plan` para desenhar exactamente o que um cozimento desenha.
#[test]
fn the_ends_carry_the_verb_and_publish_no_morph() {
    let tr = Transition::new(&[pose(Some(0), Some(0))], &[pose(Some(1), Some(0))]);
    assert_eq!(tr.at(1.0)[0].bool_op, Some(1), "a chegada tem de ser exata");
    assert_eq!(tr.at(0.0)[0].bool_op, Some(0), "a partida tem de ser exata");
    for t in [0.0, 1.0] {
        assert!(
            tr.bool_morphs(t).is_empty(),
            "a ponta t={t} publicou recado: o quadro pagaria dois cozimentos para nada"
        );
    }
}

/// ⭐ **TROCAR A OPERAÇÃO DO GRUPO também anima** — e é este o gesto que o artista faz primeiro
/// (clicar `Subtract` com o grupo em mãos).
///
/// ⚠️ Ele é um gate próprio, e não uma variação do de cima: o grupo é uma entidade **sem
/// `VecPathId`**, então ele não tem pose. Se este canal não existisse, um `Trim` autorado no Hover
/// não animaria coisa nenhuma — e nada ficaria vermelho, porque o outro canal continuaria certo.
#[test]
fn changing_the_groups_operation_travels_even_when_no_shape_verb_moves() {
    let tr = Transition::new(
        &[pose(None, Some(0))], // o grupo em Union, a forma a herdar
        &[pose(None, Some(5))], // o grupo em Trim — uma RECEITA, sem verbo por forma nenhum
    );
    let morphs = tr.bool_morphs(0.25);
    assert_eq!(
        morphs.len(),
        1,
        "trocar a operação do GRUPO não foi publicada: {morphs:?}"
    );
    assert_eq!(morphs[0].group_op, Some(5));
    assert_eq!(
        morphs[0].op, None,
        "a forma continua a herdar, e o recado tem de dizer isso"
    );
}

/// **O CONTROLE: sem troca de verbo não há recado.**
///
/// ⚠️ Sem ele, um gate que apenas visse recados aparecerem não distinguiria *"a troca certa foi
/// publicada"* de *"tudo publica sempre"* — e o segundo caso faria toda transição de UI da casa
/// pagar dois cozimentos por quadro.
#[test]
fn a_pose_that_moves_without_changing_verb_publishes_nothing() {
    let tr = Transition::new(
        &[pose(Some(1), Some(0))],
        &[ObjectPose {
            translation: [40.0, 0.0],
            ..pose(Some(1), Some(0))
        }],
    );
    assert!(
        !tr.at(0.5).is_empty(),
        "a fixture não moveu nada: ela não prova o controlo"
    );
    assert!(
        tr.bool_morphs(0.5).is_empty(),
        "uma pose que só se move publicou uma troca de verbo"
    );
}

/// **A máquina publica o recado enquanto anda e o APAGA ao chegar.**
///
/// ⚠️ O apagamento é a metade que se esquece, e o modo de falha é caro: um recado que sobrevivesse
/// à chegada faria o grupo cozinhar duas pontas **para sempre**, e a conta só apareceria num
/// profiler.
#[test]
fn the_machine_publishes_while_it_walks_and_clears_on_arrival() {
    let mut default = UiState::new(StateRole::Default);
    default.objects = vec![pose(Some(0), Some(0))];
    let mut hover = UiState::new(StateRole::Hover);
    hover.objects = vec![pose(Some(1), Some(0))];

    let mut m = Machine::new(vec![default, hover]).expect("dois estados");
    assert!(
        m.bool_morphs().is_empty(),
        "uma máquina parada não pode publicar recado nenhum"
    );

    m.go_to(1, 1.0, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    m.advance(0.5);
    let mid: Vec<BoolMorph> = m.bool_morphs().to_vec();
    assert_eq!(mid.len(), 1, "a máquina não publicou a troca: {mid:?}");

    m.advance(1.0); // chega
    assert!(
        m.bool_morphs().is_empty(),
        "o recado sobreviveu à chegada: o grupo cozinharia duas pontas para sempre"
    );
    assert_eq!(
        m.pose()[0].bool_op,
        Some(1),
        "a chegada tem de trazer o verbo de destino"
    );
}

/// ⛔ **QUEM SÓ EXISTE DE UM DOS LADOS NÃO FALA PELO GRUPO** (auditoria de 2026-08-23).
///
/// ⚠️ A operação de uma booleana é um fato do GRUPO, e a pose carrega-o por REDUNDÂNCIA — cada
/// operando repete o mesmo número. Quem entra (ou sai) não tem com que concordar: deixá-lo falar
/// faz o `install`, que escreve pose a pose no MESMO quadro, decidir a receita pela **ordem de
/// iteração de um `Vec`**. O que se vê é uma booleana que muda de operação no primeiro quadro da
/// transição, e o culpado é a posição de um elemento numa lista.
///
/// ⚠️ **O verbo PRÓPRIO fica**, e a assimetria é a afirmação: ele é fato de UMA forma, e quem
/// entra já chega na pose de destino — como já chega na posição e na tinta de destino.
#[test]
fn whoever_exists_on_only_one_side_does_not_speak_for_the_group() {
    const ARRIVING: VecPathId = 9;
    let staying = pose(Some(0), Some(0));
    let newcomer = ObjectPose {
        bool_op: Some(1),
        bool_group_op: Some(1), // ...e o estado de chegada põe o grupo em Subtract
        ..ObjectPose::new(ARRIVING)
    };
    // ⚠️ Quem fica tem de MUDAR alguma coisa, senão ele não entra na transição de todo (objetos
    // idênticos nos dois lados não são interpolados — lei do `Transition::new`) e o CONTROLE
    // abaixo mediria a ausência dele em vez do canal.
    let stayed = ObjectPose {
        translation: [4.0, 0.0],
        ..pose(Some(0), Some(0))
    };
    let tr = Transition::new(
        std::slice::from_ref(&staying),
        &[stayed.clone(), newcomer.clone()],
    );

    let mid = tr.at(0.5);
    let entering = mid
        .iter()
        .find(|p| p.id == ARRIVING)
        .expect("quem entra tem de aparecer na pose do meio");
    assert_eq!(
        entering.bool_group_op, None,
        "quem entra falou pelo GRUPO: a receita passa a ser decidida pela ordem da lista"
    );
    assert_eq!(
        entering.bool_op,
        Some(1),
        "o verbo PROPRIO de quem entra tem de chegar com ele, como a posicao dele chega"
    );

    // ⚠️ **O CONTROLE, e ele é a outra metade:** quem fica nos dois lados continua a falar pelo
    // grupo — sem isto o gate ficaria verde sobre um canal que ninguém escreve mais.
    let stays = mid
        .iter()
        .find(|p| p.id == OPERAND)
        .expect("quem fica tem de aparecer");
    assert_eq!(
        stays.bool_group_op,
        Some(0),
        "quem existe nos DOIS lados tem de continuar a falar pelo grupo"
    );

    // E quem SAI também não fala: a transição inversa.
    let back = Transition::new(&[stayed, newcomer], &[staying]);
    let leaving = back.at(0.5);
    assert_eq!(
        leaving
            .iter()
            .find(|p| p.id == ARRIVING)
            .expect("quem sai tem de aparecer")
            .bool_group_op,
        None,
        "quem sai falou pelo GRUPO"
    );
}
