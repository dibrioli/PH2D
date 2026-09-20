//! ⭐⭐⭐ **OS GATES DO [`Surface::at_base_color`]** — a cor de um TEXEL entra como `base_color`.
//!
//! Ver o doc da porta para o mecanismo e a tabela medida. Estes gates afirmam as duas metades que
//! ela promete: que ela é **exacta** (não uma aproximação barata) e que ela **muda a imagem** onde
//! multiplicar-depois erra.

use super::*;

/// Uma lâmpada branca perto do espelho — é lá que o destaque vive, e é lá que as duas leis
/// discordam. ⚠️ Medir longe do destaque daria um gate verde sobre a diferença toda.
fn perto_do_espelho() -> ([f32; 3], [f32; 3], [f32; 3]) {
    let luz = {
        let v = [0.25f32, 0.0, 0.968_246_3];
        let q = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        [v[0] / q, v[1] / q, v[2] / q]
    };
    (luz, [0.13, 0.0, 0.991_5], [0.0, 0.0, 1.0])
}

/// ⭐⭐⭐ **SEM VERNIZ ela é o `prepare()` inteiro, AO BIT** — em todos os campos.
///
/// ⚠️ A comparação é do [`Surface`] **inteiro** e não só da resposta: um campo novo derivado do
/// `base_color` que alguém acrescente ao `prepare` amanhã reprova aqui, que é o que impede esta
/// porta de se tornar uma aproximação em silêncio.
#[test]
fn sem_verniz_a_cor_do_texel_e_o_prepare_inteiro_ao_bit() {
    for cor in [[0.8, 0.1, 0.1], [0.05, 0.05, 0.9], [0.0; 3], [1.0; 3]] {
        let base = OpenPbr::default();
        assert_eq!(base.coat_weight, 0.0, "controlo: a omissão não tem verniz");
        let rapido = base.prepare().at_base_color(cor);
        let inteiro = OpenPbr {
            base_color: cor,
            ..base
        }
        .prepare();
        assert_eq!(
            rapido, inteiro,
            "a troca de cor tem de dar o mesmo `Surface` que um `prepare` inteiro ({cor:?})"
        );
    }
}

/// ⭐⭐⭐ **COM VERNIZ ela também é, e é aqui que as quatro linhas re-derivadas se pagam.**
///
/// ⚠️ **Este é o CONTROLO do gate acima:** sem verniz o braço curto nem toca no
/// `modulated_base_darkening`, logo aquele gate ficaria verde sobre uma porta que nunca re-deriva
/// nada. Com `coat_weight × coat_darkening > 0` o campo MUDA com a cor, e a igualdade ao bit prova
/// que as quatro linhas daqui são as quatro linhas de lá.
#[test]
fn com_verniz_a_cor_do_texel_continua_a_ser_o_prepare_inteiro_ao_bit() {
    let base = OpenPbr {
        coat_weight: 0.7,
        coat_darkening: 0.9,
        ..OpenPbr::default()
    };
    let mut mexeu = 0;
    for cor in [[0.8, 0.1, 0.1], [0.05, 0.05, 0.9], [1.0; 3]] {
        let rapido = base.prepare().at_base_color(cor);
        let inteiro = OpenPbr {
            base_color: cor,
            ..base
        }
        .prepare();
        assert_eq!(rapido, inteiro, "com verniz, a cor {cor:?} divergiu");
        if rapido.modulated_base_darkening != base.prepare().modulated_base_darkening {
            mexeu += 1;
        }
    }
    // **O PISO**: se o escurecimento não se mexesse com a cor, o gate afirmaria uma igualdade
    // trivial — as duas quatro-linhas dariam o mesmo por não fazerem nada.
    assert!(
        mexeu >= 2,
        "controlo: o escurecimento tem de depender da cor ({mexeu})"
    );
}

/// ⛔⛔ **O DESTAQUE DE UM DIELÉCTRICO NÃO É TINGIDO PELO ALBEDO** — e multiplicar depois tinge-o.
///
/// É a razão de a porta existir, escrita como régua. Um plástico vermelho tem destaque **branco**;
/// só um metal o tinge. ⚠️ A barra é a RAZÃO entre canais e não um valor absoluto: uma barra de
/// magnitude mediria o brilho da lâmpada, que não é a pergunta.
#[test]
fn o_destaque_de_um_dielectrico_nao_e_tingido_pelo_albedo() {
    let (luz, n, v) = perto_do_espelho();
    let s = OpenPbr::default().prepare();
    let vermelho = [0.8f32, 0.1, 0.1];

    let lei = s.at_base_color(vermelho).direct(n, v, luz, [1.0; 3]);
    let razao_lei = lei[0] / lei[2];

    // O que multiplicar-depois daria, no MESMO ponto — a lei rejeitada, escrita aqui para a régua
    // poder discriminar. *Uma barra sem o lado errado ao lado não separa nada.*
    let d = s.direct(n, v, luz, [1.0; 3]);
    let razao_multiplicada = (vermelho[0] * d[0]) / (vermelho[2] * d[2]);

    assert!(
        razao_lei < 2.0,
        "o destaque saiu tingido: R/B {razao_lei:.2} (a lei entrega ~1,5)"
    );
    assert!(
        razao_multiplicada > 5.0,
        "controlo: multiplicar depois TEM de tingir, senão esta régua não discrimina ({razao_multiplicada:.2})"
    );
    // ⚠️ E o controlo do CONTROLO: as duas leis concordam num albedo CINZENTO, onde a razão entre
    // canais é `1` nas duas. Sem isto, um gate que medisse apenas «as duas diferem» passaria com
    // uma porta que devolvesse lixo.
    let cinzento = [0.5f32; 3];
    let g = s.at_base_color(cinzento).direct(n, v, luz, [1.0; 3]);
    assert!(
        (g[0] / g[2] - 1.0).abs() < 1e-6,
        "num cinzento a razão tem de ser 1"
    );
}
