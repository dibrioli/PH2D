//! ⭐⭐⭐⭐ **A PORTA QUE ESTOUROU** — os gates do veredito do
//! [`super::Topologia::payload`] e da lei que ele responde face a face, a
//! [`super::Topologia::descreve`].
//!
//! ⛔⛔ **A fixtura é o report do dono de 2026-09-21, reduzida:** um plano
//! nascido sobre uma malha de QUADS e a MESMA malha depois de triangulada. O
//! pânico dele foi `index out of bounds: the len is 196608 but the index is
//! 196608`, que é `4 × 49 152` — a conta `4 * f + s` com `f` a valer
//! exactamente a contagem de faces do plano; aqui a mesma reprodução lê
//! `len 8, index 8`, porque a fixtura tem `2` faces em vez de `49 152`.
//!
//! ⚠️ **E ela só reproduzia em RELEASE.** A linha que aqui estava era um
//! `debug_assert_eq!`, logo no perfil `smoke` — que é o que o dono corre — ela
//! não existia. *Uma promessa escrita num `debug_assert` é uma promessa que o
//! produto não faz*, e é por isso que estes gates medem um VALOR DE RETORNO.

use super::topo::{PAYLOAD_STRIDE, Topologia};

/// Dois quads a partilhar uma aresta — `6` vértices, `2` faces.
fn quads() -> Vec<Vec<u32>> {
    vec![vec![0, 1, 2, 3], vec![1, 4, 5, 2]]
}

/// Os MESMOS dois quads depois de `triangulate()` — `6` vértices, `4` faces.
fn triangulados() -> Vec<Vec<u32>> {
    vec![vec![0, 1, 2], vec![0, 2, 3], vec![1, 4, 5], vec![1, 5, 2]]
}

fn it(f: &[Vec<u32>]) -> impl Iterator<Item = &[u32]> {
    f.iter().map(std::vec::Vec::as_slice)
}

/// ⭐⭐⭐ **O CONTROLO, e sem ele os três gates de recusa aprovariam uma porta
/// que recusa TUDO.**
#[test]
fn as_faces_de_que_o_plano_nasceu_sao_aceites_e_o_registo_fica_cheio() {
    let q = quads();
    let topo = Topologia::nova(6, it(&q), 2);
    let mut pay = Vec::new();
    assert!(
        topo.payload(it(&q), &mut pay),
        "a porta recusou as faces dela"
    );
    assert_eq!(
        pay.len(),
        q.len() * PAYLOAD_STRIDE,
        "o registo tem de ter uma entrada por face"
    );
}

/// ⭐⭐⭐⭐ **O DAB QUE REFINA: mais faces, os MESMOS cantos.**
///
/// ⛔⛔ **Esta fixtura nasceu de uma MUTAÇÃO SOBREVIVENTE.** A primeira
/// redacção media o caso do dono — quads triangulados — e ali a recusa dispara
/// em `f = 0` pelos CANTOS (um triângulo onde o plano espera um quad), logo
/// apagar a cerca da CONTAGEM não mudava um bit: *duas cercas que se tapam uma
/// à outra leem-se como uma cerca a funcionar*.
///
/// ⚠️ **E o caso é REAL e não um exercício:** a triangulação do pen-down é uma
/// vez por traço, mas o `refine_for_dab` parte triângulos em triângulos a cada
/// dab — ali a contagem de faces sobe com **todos** os cantos a `3`, e esta é a
/// única cerca que fica entre o artista e o pânico.
///
/// ⚠️ **A segunda asserção é metade do gate:** uma porta que devolvesse `false`
/// deixando `out` meio escrito entregaria ao device o registo de duas faces de
/// uma malha e o resto de outra — *bytes válidos no sítio errado*, que é pior
/// do que o pânico porque é mudo. Aqui ela morde a sério: quando a recusa
/// chega, `out` já tem `2 × PAYLOAD_STRIDE` palavras dentro.
#[test]
fn uma_malha_com_mais_faces_e_os_mesmos_cantos_e_recusada_e_nao_estoura() {
    let t = triangulados();
    let base: Vec<Vec<u32>> = vec![t[0].clone(), t[1].clone()];
    let topo = Topologia::nova(6, it(&base), 2);
    assert_eq!(topo.faces(), 2, "o CONTROLO da fixtura");
    let mut pay = Vec::new();
    assert!(
        !topo.payload(it(&t), &mut pay),
        "o plano tem {} faces e a malha tem {} — tinha de recusar",
        topo.faces(),
        t.len()
    );
    assert!(
        pay.is_empty(),
        "uma recusa NUNCA deixa registo meio escrito"
    );
}

/// ⭐⭐⭐ **O PÂNICO DO DONO, reproduzido — e hoje ele é um `false`.**
///
/// ⚠️ **Qual das três cercas dispara aqui está MEDIDO e é a dos CANTOS**, não a
/// da contagem: triangular um quad põe um triângulo na face `0`, onde o plano
/// espera quatro cantos. *A cerca da contagem é a que cobre o dab, e tem
/// fixtura própria acima — escrever só este caso deixaria a outra sem régua.*
#[test]
fn o_panico_do_dono_reproduzido_uma_peca_de_quads_triangulada() {
    let (q, t) = (quads(), triangulados());
    let topo = Topologia::nova(6, it(&q), 2);
    let mut pay = vec![7u32; 3];
    assert!(
        !topo.payload(it(&t), &mut pay),
        "o plano descreve {} quads e a malha tem {} triângulos",
        topo.faces(),
        t.len()
    );
    assert!(
        pay.is_empty(),
        "uma recusa NUNCA deixa registo meio escrito"
    );
}

/// ⛔⛔ **A metade CURTA é a que nunca estourou, e é a pior.**
///
/// Com menos faces do que a topologia o laço acaba sozinho: nenhum índice sai
/// de alcance, o registo fica truncado, e o shader lê o bloco de interior de
/// uma face que já não está lá — **tinta válida no sítio errado, em silêncio**.
/// *É por isso que a régua é a IGUALDADE e não um tecto.*
#[test]
fn uma_malha_com_menos_faces_e_recusada_embora_nada_saia_de_alcance() {
    let t = triangulados();
    let topo = Topologia::nova(6, it(&t), 2);
    let curta = vec![t[0].clone(), t[1].clone()];
    let mut pay = vec![7u32; 3];
    assert!(
        !topo.payload(it(&curta), &mut pay),
        "o plano tem {} faces e a malha tem {} — tinha de recusar",
        topo.faces(),
        curta.len()
    );
    assert!(
        pay.is_empty(),
        "uma recusa NUNCA deixa registo meio escrito"
    );
}

/// ⭐⭐ **E a MESMA contagem de faces com outros cantos também é outra malha.**
///
/// ⚠️ Esta é a única das três que a [`Topologia::descreve`] **não** vê: as duas
/// contagens batem (`6` vértices, `2` faces) e o que difere é o número de
/// cantos de cada face. *A régua de contagens é necessária e não suficiente, e
/// é por isso que o veredito forte é o do payload, face a face.*
#[test]
fn a_mesma_contagem_de_faces_com_outros_cantos_e_recusada() {
    let q = quads();
    let topo = Topologia::nova(6, it(&q), 2);
    let dois_tris: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![1, 4, 5]];
    assert!(
        topo.descreve(6, dois_tris.len()),
        "o CONTROLO: as contagens batem, logo só o payload pode separar isto"
    );
    let mut pay = Vec::new();
    assert!(
        !topo.payload(it(&dois_tris), &mut pay),
        "um quad e um triângulo não são a mesma face"
    );
    assert!(
        pay.is_empty(),
        "uma recusa NUNCA deixa registo meio escrito"
    );
}

/// ⭐⭐⭐ **A LEI, e as DUAS metades medidas uma de cada vez.**
///
/// ⚠️ Cada asserção move **uma** contagem e deixa a outra parada — com as duas
/// a mexerem juntas, uma régua que só olhasse para os vértices passaria.
#[test]
fn a_lei_de_descrever_mede_os_vertices_e_as_faces() {
    let q = quads();
    let topo = Topologia::nova(6, it(&q), 2);
    assert!(topo.descreve(6, 2), "o CONTROLO: a malha de que ele nasceu");
    assert!(!topo.descreve(7, 2), "um vértice a mais é outra malha");
    assert!(!topo.descreve(6, 4), "duas faces a mais são outra malha");
    assert!(!topo.descreve(5, 2), "um vértice a menos é outra malha");
    assert!(!topo.descreve(6, 1), "uma face a menos é outra malha");
}

/// Uma grelha de quads `n × n` (vértices `(n+1)²`) — aberta, o que basta para
/// contar: a lei do índice não pergunta se a malha fecha.
fn grelha(n: u32) -> Vec<Vec<u32>> {
    let w = n + 1;
    let mut f = Vec::new();
    for y in 0..n {
        for x in 0..n {
            let a = y * w + x;
            f.push(vec![a, a + 1, a + w + 1, a + w]);
        }
    }
    f
}

/// ⭐ **A conta PREVISTA é a conta ALOCADA** — a [`Topologia::amostras_ao_nivel`]
/// responde *«cabe?»* antes de se construir, e só vale se disser o mesmo que
/// o [`crate::total`] depois. Quads e triângulos, porque o interior deles
/// conta-se por fórmulas diferentes.
#[test]
fn a_conta_prevista_e_a_conta_alocada() {
    for faces in [quads(), triangulados(), grelha(3)] {
        let v = faces.iter().flatten().map(|&i| i as usize + 1).max().unwrap_or(0);
        let base = Topologia::nova(v, it(&faces), 0);
        for k in 0..=5u8 {
            let alocada = crate::total(&Topologia::nova(v, it(&faces), k));
            assert_eq!(
                base.amostras_ao_nivel(k),
                alocada as u64,
                "nível {k}: a previsão não é o que se aloca"
            );
        }
    }
}

/// ⛔⛔ **Uma malha GRANDE desce ao nível que cabe no ÍNDICE dela** — os
/// prefixos são `u32`, e somá-los além dele dava a volta em silêncio (tinta
/// no sítio errado). `301²` vértices a `lado 256` pedem `5,9e9` amostras; a
/// `lado 128` são `1,5e9`, que cabem.
///
/// ⚠️ **O CONTROLO é uma malha pequena no mesmo pedido:** ela fica no `8`, senão
/// a guarda desceria toda a gente e o teste passaria a medir outra coisa.
#[test]
fn uma_malha_grande_desce_ao_nivel_que_cabe_no_indice() {
    let grande = grelha(300);
    let v = 301 * 301;
    let t = Topologia::nova(v, it(&grande), 8);
    assert!(t.amostras_ao_nivel(8) > u64::from(u32::MAX), "a fixtura estoura o índice a 256x");
    assert_eq!(t.nivel_uniforme(), Some(7), "o nível não desceu ao que cabe");
    assert_eq!(
        crate::total(&t) as u64,
        t.amostras_ao_nivel(7),
        "o plano construído não é o do nível que ficou"
    );

    let pequena = grelha(3);
    assert_eq!(
        Topologia::nova(16, it(&pequena), 8).nivel_uniforme(),
        Some(8),
        "o CONTROLO: uma malha pequena fica no nível pedido"
    );
}

/// ⛔ **Regraduar acima do índice é RECUSADO, nunca descido em silêncio** — a
/// lista de níveis é do chamador. O CONTROLO é a lista que cabe.
#[test]
fn regraduar_acima_do_indice_e_recusado() {
    let grande = grelha(300);
    let base = Topologia::nova(301 * 301, it(&grande), 0);
    let n = base.faces();
    assert!(base.regraduada(&vec![8; n]).is_none(), "estourou o índice e foi aceite");
    assert!(base.regraduada(&vec![7; n]).is_some(), "o CONTROLO: a lista que cabe");
}
