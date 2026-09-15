//! Os gates da **lei do raio** do [`crate::Verb::SceneProject`], cobrada por
//! FORMA FECHADA — `SPEC_unblocked_brushes.md` §6.3 e §6.4.
//!
//! ⚠️ **Aqui a lei está SOZINHA no numerador:** um plano a uma distância que se
//! escreve à mão, e a resposta certa é essa distância. A cadeia de peso, a
//! curva e a força entram na bancada do corpus, não aqui — *um traço inteiro
//! mistura seis aplicações da curva com a re-medição da §6.6, e um desvio nele
//! não diz qual dos dois falhou* (espec §8.4).

use super::{distancia, folga_simetrica};
use ph2d_mesh::{Face, Mesh, Pose};

/// Um quadrado grande em `z = alt`, virado para cima — o alvo mais simples que
/// existe, e aquele cuja distância se lê sem calcular nada.
fn plano(alt: f32) -> Mesh {
    let l = 8.0;
    Mesh::from_parts(
        vec![[-l, -l, alt], [l, -l, alt], [l, l, alt], [-l, l, alt]],
        vec![Face::tri(0, 1, 2), Face::tri(0, 2, 3)],
    )
    .expect("o plano do gate")
}

/// O de baixo do dab: para dentro do ecrã numa vista de topo.
const PARA_BAIXO: [f32; 3] = [0.0, 0.0, -1.0];
const ORIGEM: [f32; 3] = [0.0, 0.0, 0.0];

/// ⭐⭐ **A DISTÂNCIA É A DO ALVO MAIS PERTO, e a folga entra DEPOIS do
/// vencedor** (espec §6.3, regras 1 e 4).
///
/// ⛔⛔ **A ordem é load-bearing e a segunda metade deste gate é o que a prende:**
/// com a folga aplicada ANTES da competição, os dois candidatos encolhiam por
/// igual e o vencedor seria o mesmo — por acidente. O que separa as duas ordens
/// é o **valor**, e é por isso que ele é afirmado e não só a escolha.
#[test]
fn ganha_o_alvo_mais_perto_e_a_folga_entra_depois() {
    let alvos = vec![(plano(-0.8), Pose::IDENTITY), (plano(-0.3), Pose::IDENTITY)];
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &alvos, false, 0.0)
        .expect("o raio tem de acertar em alguma coisa");
    assert!(
        (d - 0.3).abs() < 1e-6,
        "ganhou o alvo errado: {d} (o perto está a 0,3 e o longe a 0,8)"
    );
    // ⭐ Com folga, o vencedor continua a ser o perto E o valor é o dele menos
    // a folga — `0,3 − 0,1`, nunca `0,8 − 0,1`.
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &alvos, false, 0.1).expect("acerto");
    assert!(
        (d - 0.2).abs() < 1e-6,
        "a folga entrou no sítio errado da ordem: {d}"
    );
}

/// ⛔⛔ **NENHUM ACERTO ⇒ O VÉRTICE NÃO SE MOVE** (espec §6.3.3), e as três
/// maneiras de lá chegar.
///
/// ⚠️ **`None` e `Some(0.0)` não são a mesma coisa**, e este gate é metade da
/// razão de o tipo os separar: um acerto a distância zero ainda subtrai a folga
/// e pode **afastar** a peça (§6.4), enquanto um não-acerto é inerte.
#[test]
fn sem_acerto_nao_ha_lei() {
    let vazio: Vec<(Mesh, Pose)> = Vec::new();
    assert_eq!(
        distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &vazio, false, 0.0),
        None,
        "sem outra peça na cena o pincel tem de ser inerte"
    );
    // O alvo está ATRÁS e os dois sentidos estão desligados.
    let acima = vec![(plano(0.5), Pose::IDENTITY)];
    assert_eq!(
        distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &acima, false, 0.0),
        None,
        "um alvo do lado errado sem os dois sentidos não pode ser alcançado"
    );
    // ⭐ O CONTROLO: com os dois sentidos ele é alcançado, e com sinal
    // NEGATIVO. Sem esta metade o gate acima ficaria verde sobre uma lei que
    // nunca acerta em nada.
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &acima, true, 0.0)
        .expect("com os dois sentidos ele acerta");
    assert!(
        (d + 0.5).abs() < 1e-6,
        "um acerto para trás tem de vir com `d` negativo, e veio {d}"
    );
}

/// ⛔⛔ **AS DUAS ARMADILHAS DA FOLGA, as duas MEDIDAS no alvo** (espec §6.4) —
/// e este gate existe para elas serem **reproduzidas de propósito**, não
/// descobertas por alguém a ler o rótulo.
///
/// | caso | o que o rótulo *«distância mínima»* sugere | o que acontece |
/// |---|---|---|
/// | folga `0,6` sobre um vão de `0,5` | o barro pára a `0,1` do alvo | ele **AFASTA-SE** `0,1` |
/// | folga `0,1` num acerto para TRÁS | a excursão encolhe `0,1` | ela **CRESCE** `0,1` |
///
/// ⚠️ **A alternativa simétrica está escrita e NÃO shipa** — a decisão é do
/// dono (espec §10.3), e este gate mede as duas leis lado a lado para a troca
/// ser **uma linha**. *Reproduzir o alvo é reproduzir um defeito; divergir sem
/// o dizer é pior.*
#[test]
fn a_folga_so_e_minima_no_sentido_de_avanco() {
    let abaixo = vec![(plano(-0.5), Pose::IDENTITY)];
    let acima = vec![(plano(0.5), Pose::IDENTITY)];

    // (a) A folga maior que o vão INVERTE o sentido.
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &abaixo, false, 0.6).expect("acerto");
    assert!(
        (d + 0.1).abs() < 1e-6,
        "a folga maior que o vão tinha de dar `-0,1` (o barro afasta-se), e deu {d}"
    );
    assert!(
        (folga_simetrica(0.5, 0.6) - 0.0).abs() < 1e-6,
        "a lei simétrica tem de PARAR em zero, nunca inverter"
    );

    // (b) Num acerto para trás a folga SOMA em magnitude.
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &acima, true, 0.1).expect("acerto");
    assert!(
        (d + 0.6).abs() < 1e-6,
        "a folga tinha de CRESCER a excursão para `-0,6`, e deu {d}"
    );
    assert!(
        (folga_simetrica(-0.5, 0.1) + 0.4).abs() < 1e-6,
        "a lei simétrica tem de ENCOLHER a excursão para `-0,4`"
    );

    // ⭐ E o CONTROLO de que as duas leis não são a mesma função com outro
    // nome: no sentido de avanço elas **concordam**, e é só nas duas
    // armadilhas que se separam.
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &abaixo, false, 0.1).expect("acerto");
    assert!(
        (d - folga_simetrica(0.5, 0.1)).abs() < 1e-6,
        "no sentido de avanço as duas leis têm de dar o mesmo número"
    );
}

/// ⭐⭐⭐ **A DISTÂNCIA ATRAVESSA A ESCALA DAS DUAS PEÇAS** — o gate que o
/// cabeçalho do [`crate::projectar`] nomeia.
///
/// ⚠️⚠️ **O `t` de um acerto vem em unidades da peça CONSULTADA** (o
/// [`ph2d_mesh::Ray`] normaliza a direcção, e a
/// [`ph2d_mesh::Pose::ray_to_local`] leva o raio ao espaço da malha) ⇒ dois
/// candidatos em peças com escalas diferentes **não são comparáveis** pelo `t`
/// cru, e a resposta tem de vir na régua do objecto ACTIVO. *Sem a conversão os
/// dois números continuam `f32` plausíveis, e o `min |d|` da espec escolhe pelo
/// número errado — em silêncio.*
///
/// A fixtura: um alvo **escalado 2×** cujo plano local está em `z = −0,25`, ou
/// seja em `z = −0,5` no mundo. A resposta certa é `0,5`; sem a conversão seria
/// `0,25`.
#[test]
fn a_distancia_atravessa_a_escala_das_duas_pecas() {
    let alvo = vec![(plano(-0.25), Pose::new([0.0, 0.0, 0.0], 2.0))];
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &alvo, false, 0.0).expect("acerto");
    assert!(
        (d - 0.5).abs() < 1e-6,
        "a escala do ALVO não atravessou: {d} (o `t` cru daria 0,25)"
    );

    // ⭐ E a do ACTIVO pelo outro lado: com o activo a `2×`, o mesmo vão de
    // `0,5` no mundo mede `0,25` na régua dele — que é a régua em que o dab
    // escreve.
    let alvo = vec![(plano(-0.5), Pose::IDENTITY)];
    let d = distancia(
        ORIGEM,
        PARA_BAIXO,
        Pose::new([0.0, 0.0, 0.0], 2.0),
        &alvo,
        false,
        0.0,
    )
    .expect("acerto");
    assert!(
        (d - 0.25).abs() < 1e-6,
        "a escala do ACTIVO não atravessou: {d}"
    );
}

/// ⭐ **A TRANSLAÇÃO DO ALVO também atravessa** — o irmão barato do gate acima,
/// e ele existe porque as duas metades de uma pose falham por razões
/// diferentes: trocar o ponto pelo vector (ou vice-versa) dá um `d` plausível e
/// errado, que é exactamente o que a espec §6.3 avisa por escrito.
#[test]
fn a_translacao_do_alvo_atravessa_e_a_direccao_nao_a_apanha() {
    // O plano local está em `z = 0`, e a peça está pousada em `z = −0,5`.
    let alvo = vec![(plano(0.0), Pose::new([0.0, 0.0, -0.5], 1.0))];
    let d = distancia(ORIGEM, PARA_BAIXO, Pose::IDENTITY, &alvo, false, 0.0).expect("acerto");
    assert!(
        (d - 0.5).abs() < 1e-6,
        "a translação do alvo não atravessou: {d}"
    );
    // ⛔ O CONTROLO: a partir de um ponto de partida deslocado, a distância
    // muda pelo mesmo tanto — se a translação entrasse na DIREÇÃO em vez de no
    // PONTO, este número não se mexia.
    let d = distancia(
        [0.0, 0.0, 0.25],
        PARA_BAIXO,
        Pose::IDENTITY,
        &alvo,
        false,
        0.0,
    )
    .expect("acerto");
    assert!(
        (d - 0.75).abs() < 1e-6,
        "a origem do raio não é tratada como PONTO: {d}"
    );
}
