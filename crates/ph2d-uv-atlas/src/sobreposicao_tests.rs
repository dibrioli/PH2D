//! Gates da régua da sobreposição.
//!
//! ⚠️ **A geometria prova-se em FORMA FECHADA e nunca contra outra implementação nossa:**
//! cada caso abaixo tem a área calculada à mão, e é isso que faz destes gates uma régua e
//! não um espelho.

use super::sobreposicao::{Classe, RUIDO_RELATIVO, area_de_interseccao, medir};

const A: [[f32; 2]; 3] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];

/// ⭐ Os quatro casos que a lei tem de acertar, com a conta ao lado.
#[test]
fn a_area_em_que_dois_triangulos_se_cruzam_e_a_da_conta_a_mao() {
    // (a) O mesmo triângulo: a área inteira, `1/2`.
    assert!((area_de_interseccao(A, A) - 0.5).abs() < 1.0e-12);
    // (b) Disjuntos: zero.
    let longe = [[9.0, 9.0], [10.0, 9.0], [9.0, 10.0]];
    assert!(area_de_interseccao(A, longe) < 1.0e-15);
    // (c) ⛔ **A PARTILHAR UMA ARESTA: zero.** É o caso que decide se esta régua serve —
    // toda a vizinhança de uma malha o exercita, e um teste ingénuo de intersecção
    // acusaria a malha inteira. O quadrado `[0,1]²` partido pela diagonal.
    let irmao = [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    assert!(
        area_de_interseccao(A, irmao) < 1.0e-15,
        "dois triangulos que partilham a diagonal nao se cruzam: {}",
        area_de_interseccao(A, irmao)
    );
    // (d) Meio a meio: o mesmo triângulo deslocado meia unidade em `x` recorta um
    // triângulo semelhante de razão `1/2` ⇒ área `1/4` da de `A`, ou seja `1/8`.
    let meio = [[0.5, 0.0], [1.5, 0.0], [0.5, 1.0]];
    assert!(
        (area_de_interseccao(A, meio) - 0.125).abs() < 1.0e-12,
        "meia sobreposicao vale 1/8: {}",
        area_de_interseccao(A, meio)
    );
}

/// ⛔⛔ **A ORIENTAÇÃO não pode mudar a resposta.** Um triângulo DOBRADO tem área com
/// sinal negativo, e o recorte «para dentro» leria o semiplano errado — *o que apagaria
/// exactamente a classe de defeito que esta régua existe para achar*.
#[test]
fn um_triangulo_do_avesso_cruza_na_mesma() {
    let avesso = [A[0], A[2], A[1]];
    assert!((area_de_interseccao(A, avesso) - 0.5).abs() < 1.0e-12);
    assert!((area_de_interseccao(avesso, A) - 0.5).abs() < 1.0e-12);
}

/// ⭐ **A régua é simétrica**, e o recorte não o é por construção (um clipa o outro).
#[test]
fn recortar_a_para_b_da_o_mesmo_que_b_para_a() {
    let outro = [[0.3, -0.2], [1.4, 0.1], [0.1, 0.9]];
    let (x, y) = (area_de_interseccao(A, outro), area_de_interseccao(outro, A));
    assert!(
        (x - y).abs() <= 1.0e-12 && x > 0.01,
        "a area tem de ser a mesma nos dois sentidos e nao nula: {x} contra {y}"
    );
}

/// ⛔ **O piso de ruído é relativo e não absoluto** — uma malha fina tem triângulos de
/// área `1e-6` do atlas, e um epsilon absoluto acusaria a vizinhança inteira.
#[test]
fn o_ruido_e_uma_fraccao_e_nao_um_numero() {
    // ⭐ Um `assert!` sobre uma constante é **dobrado pelo compilador**: escrito assim,
    // afinar o piso para fora da faixa deixa de compilar em vez de reprovar num teste.
    const { assert!(RUIDO_RELATIVO > 0.0 && RUIDO_RELATIVO < 1.0e-3) };
    // Os mesmos dois irmãos do caso (c), mil vezes mais pequenos: a resposta é a mesma.
    let peq = |t: [[f32; 2]; 3]| t.map(|z| [z[0] * 1.0e-3, z[1] * 1.0e-3]);
    let irmao = [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let cruz = area_de_interseccao(peq(A), peq(irmao));
    let menor = 0.5e-6;
    assert!(
        cruz <= menor * RUIDO_RELATIVO,
        "a diagonal partilhada tem de ficar abaixo do piso a qualquer escala: {cruz}"
    );
}

/// ⭐⭐⭐ **As quatro classes existem e o relatório indexa-as sem colisão.**
///
/// ⚠️ Sem isto, dois nomes com o mesmo índice somariam no mesmo balde e a tabela da
/// atribuição ficaria muda sobre uma das classes.
#[test]
fn cada_classe_tem_um_indice_so_e_um_nome_so() {
    let mut is: Vec<usize> = Classe::ALL.iter().map(|c| c.indice()).collect();
    is.sort_unstable();
    assert_eq!(is, vec![0, 1, 2, 3]);
    let mut ns: Vec<&str> = Classe::ALL.iter().map(|c| c.nome()).collect();
    ns.sort_unstable();
    ns.dedup();
    assert_eq!(ns.len(), 4);
}

/// ⛔⛔ **O PISO DE POPULAÇÃO da régua.** Um atlas sem área lê `0 %` de cruzamento, que é
/// o mesmo byte de um atlas perfeito — a coluna `triangulos_com_area` é o que os separa.
#[test]
fn um_atlas_vazio_nao_se_le_como_um_atlas_limpo() {
    let (mesh, cut, map, jumps) = super::lib_tests::fita(0, false);
    let a = super::build(&mesh, &cut, &map, &jumps);
    let s = medir(&mesh, &a);
    assert!(s.triangulos_com_area > 0, "a fixtura tem area");
    assert_eq!(s.triangulos, 6, "seis triangulos na fita");
    // ⭐ O CONTROLO: uma fita que não se dobra não cruza nada.
    assert_eq!(s.pares.len(), 0, "{:?}", s.pares);
    assert!(s.fraccao() < 1.0e-12);

    let vazio = super::Atlas::default();
    let z = medir(&mesh, &vazio);
    assert_eq!(
        z.triangulos_com_area, 0,
        "sem `(u, v)` nenhum triangulo tem area — e o piso di-lo"
    );
}
